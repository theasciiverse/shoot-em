//! Sound and music, in place of FMOD: the 28 samples the original carried
//! as resources (Ogg Vorbis, unchanged) and our soundtrack (see README):
//! Crush (Asciiverse remix) in the menu, Juhani Junkala's Level 1 to 3 in
//! turn during play, his Title Screen after a game.
//!
//! Everything plays through one mixer of our own, whose output is copied
//! into a small ring for the Braille oscilloscope. With a sound device the
//! device pulls the mix; with --mute (or none) the game pulls it itself in
//! step with the clock, so the scope moves and nothing is heard.

use rodio::buffer::SamplesBuffer;
use rodio::Source;
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

const RATE: u32 = 44_100;

/// The samples, indexed as the original loaded them (resource 100 + i).
const SFX: [&[u8]; 28] = [
    include_bytes!("../assets/sfx/100.ogg"),
    include_bytes!("../assets/sfx/101.ogg"),
    include_bytes!("../assets/sfx/102.ogg"),
    include_bytes!("../assets/sfx/103.ogg"),
    include_bytes!("../assets/sfx/104.ogg"),
    include_bytes!("../assets/sfx/105.ogg"),
    include_bytes!("../assets/sfx/106.ogg"),
    include_bytes!("../assets/sfx/107.ogg"),
    include_bytes!("../assets/sfx/108.ogg"),
    include_bytes!("../assets/sfx/109.ogg"),
    include_bytes!("../assets/sfx/110.ogg"),
    include_bytes!("../assets/sfx/111.ogg"),
    include_bytes!("../assets/sfx/112.ogg"),
    include_bytes!("../assets/sfx/113.ogg"),
    include_bytes!("../assets/sfx/114.ogg"),
    include_bytes!("../assets/sfx/115.ogg"),
    include_bytes!("../assets/sfx/116.ogg"),
    include_bytes!("../assets/sfx/117.ogg"),
    include_bytes!("../assets/sfx/118.ogg"),
    include_bytes!("../assets/sfx/119.ogg"),
    include_bytes!("../assets/sfx/120.ogg"),
    include_bytes!("../assets/sfx/121.ogg"),
    include_bytes!("../assets/sfx/122.ogg"),
    include_bytes!("../assets/sfx/123.ogg"),
    include_bytes!("../assets/sfx/124.ogg"),
    include_bytes!("../assets/sfx/125.ogg"),
    include_bytes!("../assets/sfx/126.ogg"),
    include_bytes!("../assets/sfx/127.ogg"),
];

/// Crush (Asciiverse remix), after Drozerix's Crush (public domain).
const MENU_SONG: &[u8] = include_bytes!("../assets/music/intro/crush-remix.xm");
/// A song in the in-game playlist: Ogg Vorbis or a tracker module.
#[derive(Clone, Copy)]
enum Track {
    Ogg(&'static [u8]),
    /// a module, and the trim that matches it to Junkala's level
    Module(&'static [u8], f32),
}

/// In turn during play: Juhani Junkala's Level 1 to 3 from "5 Chiptunes
/// (Action)" (CC0); Digital Rendezvous, Mecanum Overdrive and Girl From
/// Mars (Asciiverse remixes), after Drozerix's originals (public domain);
/// and Tunnel Vision by fuzion_mixer, CC BY 4.0, remixed and extended by
/// Asciiverse.
const LEVEL_SONGS: [Track; 7] = [
    Track::Ogg(include_bytes!("../assets/music/levels/junkala-level-1.ogg")),
    Track::Module(include_bytes!("../assets/music/levels/digital-rendezvous-remix-heavy.xm"), 2.13),
    Track::Ogg(include_bytes!("../assets/music/levels/junkala-level-2.ogg")),
    Track::Ogg(include_bytes!("../assets/music/levels/junkala-level-3.ogg")),
    // builds its drums over about 2:40 and then loops at its peak; the
    // playlist plays it once through from the start, then moves on
    Track::Module(include_bytes!("../assets/music/levels/mecanum-remix-heavy.xm"), 2.41),
    // Tunnel Vision by fuzion_mixer, CC BY 4.0, remixed and extended by
    // Asciiverse: 6:50 once through
    Track::Module(include_bytes!("../assets/music/levels/tunnel-vision-extended.it"), 2.17),
    // Girl From Mars (Asciiverse extended mix), after Drozerix (public domain)
    Track::Module(include_bytes!("../assets/music/levels/girl_from_mars-extended.xm"), 2.11),
];
/// Juhani Junkala, Title Screen (CC0): after a game and in the Hall of Fame.
const END_SONG: &[u8] = include_bytes!("../assets/music/end/junkala-title-screen.ogg");
/// The menu song plays through, rests this long, and starts again.
const MENU_REST: Duration = Duration::from_secs(2);
/// Music gains, measured as integrated loudness (EBU R128, ffmpeg's
/// ebur128) through the game's own playback: the menu and the end music
/// both about -16 LUFS; every in-game song about -19 LUFS, under the guns
/// (-14.5) and the blasts (-11.6). Module trims below match each module
/// to the Ogg songs.
const MENU_GAIN: f32 = 0.75;
const LEVEL_GAIN: f32 = 0.25;
const END_GAIN: f32 = 0.413;
/// Samples kept for the oscilloscope (mono).
pub const SCOPE: usize = 2048;

/// What each sample is for, by the original's use of it.
pub mod sfx {
    pub const GUNS: usize = 0;
    pub const MINE_LAUNCH: usize = 1;
    pub const HIT: usize = 2; // 2..=7, picked at random
    pub const BLAST: usize = 7; // 7..=11, picked at random
    pub const AMMO_LOW: usize = 11;
    pub const AMMO_PICKUP: usize = 12;
    pub const GUNS_EMPTY: usize = 13;
    pub const MINES_EMPTY: usize = 14;
    pub const PICKUP: usize = 15;
    pub const MALE_ARMOR_50: usize = 16;
    pub const MALE_ARMOR_25: usize = 17;
    pub const MALE_NO_AMMO: usize = 18;
    pub const MALE_NO_MINES: usize = 19;
    pub const FEMALE_ARMOR_50: usize = 20;
    pub const FEMALE_ARMOR_25: usize = 21;
    pub const FEMALE_NO_AMMO: usize = 22;
    pub const FEMALE_NO_MINES: usize = 23;
    pub const INTRO: usize = 24;
    pub const CLICK: usize = 25;
    pub const GAME_OVER: usize = 27;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Song {
    Menu,
    Game,
    End,
}

struct Pcm {
    channels: u16,
    rate: u32,
    data: Vec<f32>,
}

/// The last mixed samples (mono), written by the audio side, read by the
/// screen: plain atomics, so neither side ever waits for the other.
pub struct ScopeRing {
    buf: Vec<AtomicU32>,
    at: AtomicUsize,
}

impl ScopeRing {
    fn new() -> ScopeRing {
        ScopeRing {
            buf: (0..SCOPE).map(|_| AtomicU32::new(0)).collect(),
            at: AtomicUsize::new(0),
        }
    }
    fn push(&self, v: f32) {
        let i = self.at.load(Ordering::Relaxed);
        self.buf[i].store(v.to_bits(), Ordering::Relaxed);
        self.at.store((i + 1) % SCOPE, Ordering::Relaxed);
    }
    /// Oldest first.
    pub fn snapshot(&self) -> Vec<f32> {
        let at = self.at.load(Ordering::Relaxed);
        (0..SCOPE)
            .map(|i| f32::from_bits(self.buf[(at + i) % SCOPE].load(Ordering::Relaxed)))
            .collect()
    }
}

/// Passes the mix through and copies it, folded to mono, into the ring.
struct Tap {
    inner: rodio::mixer::MixerSource,
    ring: Arc<ScopeRing>,
    left: f32,
    odd: bool,
}

impl Iterator for Tap {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let v = self.inner.next()?;
        if self.odd {
            self.ring.push((self.left + v) * 0.5);
        } else {
            self.left = v;
        }
        self.odd = !self.odd;
        Some(v)
    }
}

impl Source for Tap {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> rodio::ChannelCount {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        NonZero::new(RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub struct Audio {
    sink: Option<rodio::MixerDeviceSink>,
    mix: rodio::mixer::Mixer,
    /// muted: the game pulls the mix itself (see `tick`)
    pull: Option<Tap>,
    pub scope: Arc<ScopeRing>,
    pcm: Vec<Pcm>,
    song: Option<(Song, Arc<AtomicBool>)>,
    /// the level track to play next, kept across games
    next_level: Arc<AtomicUsize>,
    pub status: String,
}

impl Audio {
    pub fn new(mute: bool) -> Audio {
        let (mix, source) = rodio::mixer::mixer(NonZero::new(2).unwrap(), NonZero::new(RATE).unwrap());
        // an empty mixer counts as finished; a silent source keeps it going
        mix.add(rodio::source::Zero::new(NonZero::new(2).unwrap(), NonZero::new(RATE).unwrap()));
        let scope = Arc::new(ScopeRing::new());
        let tap = Tap {
            inner: source,
            ring: scope.clone(),
            left: 0.0,
            odd: false,
        };
        let mut a = Audio {
            sink: None,
            mix,
            pull: None,
            scope,
            pcm: Vec::new(),
            song: None,
            next_level: Arc::new(AtomicUsize::new(0)),
            status: String::new(),
        };
        let device = if mute {
            None
        } else {
            match rodio::DeviceSinkBuilder::open_default_sink() {
                Ok(mut s) => {
                    s.log_on_drop(false);
                    Some(s)
                }
                Err(e) => {
                    a.status = format!("no sound ({e})");
                    None
                }
            }
        };
        match device {
            Some(s) => {
                s.mixer().add(tap);
                a.sink = Some(s);
                a.status = "sound on".into();
            }
            None => {
                a.pull = Some(tap);
                if mute {
                    a.status = "muted".into();
                }
            }
        }
        for bytes in SFX {
            a.pcm.push(decode(bytes));
        }
        a
    }

    /// Muted: pull `ms` milliseconds of the mix, so the music moves on and
    /// the oscilloscope has something to show. Playing aloud: nothing to do.
    pub fn tick(&mut self, ms: u32) {
        if let Some(t) = &mut self.pull {
            for _ in 0..(RATE as u64 * ms as u64 / 1000 * 2) {
                t.next();
            }
        }
    }

    /// FSOUND_PlaySound with FSOUND_SetPan / FSOUND_SetVolume: pan and
    /// volume 0..=255, 128 the centre.
    pub fn play(&self, i: usize, pan: i32, vol: i32) {
        let Some(p) = self.pcm.get(i) else { return };
        let pan = (pan.clamp(0, 255) as f32) / 255.0;
        let vol = vol.clamp(0, 255) as f32 / 255.0;
        // FMOD's pan: full in both ears at the centre
        let (gl, gr) = (
            (2.0 * (1.0 - pan)).min(1.0) * vol,
            (2.0 * pan).min(1.0) * vol,
        );
        let mut out = Vec::with_capacity(p.data.len() / p.channels as usize * 2);
        match p.channels {
            1 => {
                for &s in &p.data {
                    out.push(s * gl);
                    out.push(s * gr);
                }
            }
            _ => {
                for f in p.data.chunks(p.channels as usize) {
                    out.push(f[0] * gl);
                    out.push(f[1] * gr);
                }
            }
        }
        self.mix.add(SamplesBuffer::new(
            NonZero::new(2).unwrap(),
            NonZero::new(p.rate).unwrap(),
            out,
        ));
    }

    pub fn play_centre(&self, i: usize) {
        self.play(i, 128, 255);
    }

    pub fn is_playing(&self, song: Song) -> bool {
        matches!(&self.song, Some((s, _)) if *s == song)
    }

    pub fn play_song(&mut self, song: Song) {
        self.stop_song();
        let stop = Arc::new(AtomicBool::new(false));
        match song {
            Song::Menu => {
                let Ok(module) = xmrs::prelude::Module::load(MENU_SONG) else {
                    return;
                };
                let module: &'static xmrs::prelude::Module = Box::leak(Box::new(module));
                self.mix.add(
                    MenuLoop {
                        module,
                        player: menu_player(module),
                        rest: 0,
                        stop: stop.clone(),
                    }
                    .amplify(MENU_GAIN),
                );
            }
            Song::Game => {
                self.mix.add(
                    Playlist {
                        next: self.next_level.clone(),
                        current: None,
                        stop: stop.clone(),
                    }
                    .amplify(LEVEL_GAIN),
                );
            }
            Song::End => {
                let p = decode(END_SONG);
                if p.data.is_empty() {
                    return;
                }
                self.mix.add(
                    Stoppable {
                        inner: SamplesBuffer::new(
                            NonZero::new(p.channels).unwrap(),
                            NonZero::new(p.rate).unwrap(),
                            p.data,
                        )
                        .repeat_infinite(),
                        stop: stop.clone(),
                    }
                    .amplify(END_GAIN),
                );
            }
        }
        self.song = Some((song, stop));
    }

    pub fn stop_song(&mut self) {
        if let Some((_, stop)) = self.song.take() {
            stop.store(true, Ordering::Relaxed);
        }
    }
}

fn menu_player(module: &'static xmrs::prelude::Module) -> xmrsplayer::prelude::XmrsPlayer<'static> {
    let mut p = xmrsplayer::prelude::XmrsPlayer::new(module, RATE, 0);
    p.set_max_loop_count(1);
    p
}

/// The menu song: once through, `MENU_REST` of silence, again.
struct MenuLoop {
    module: &'static xmrs::prelude::Module,
    player: xmrsplayer::prelude::XmrsPlayer<'static>,
    /// samples of silence still to give
    rest: usize,
    stop: Arc<AtomicBool>,
}

impl Iterator for MenuLoop {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.stop.load(Ordering::Relaxed) {
            return None;
        }
        if self.rest > 0 {
            self.rest -= 1;
            if self.rest == 0 {
                self.player = menu_player(self.module);
            }
            return Some(0.0);
        }
        match self.player.next() {
            Some(s) => Some(s as f32 / 32768.0),
            None => {
                self.rest = (RATE as f32 * MENU_REST.as_secs_f32()) as usize * 2;
                Some(0.0)
            }
        }
    }
}

impl Source for MenuLoop {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> rodio::ChannelCount {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        NonZero::new(RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// One level song while it plays, as stereo 44.1 kHz samples.
enum Playing {
    Ogg(rodio::Decoder<std::io::Cursor<&'static [u8]>>),
    Module(Box<xmrsplayer::prelude::XmrsPlayer<'static>>, f32),
}

impl Playing {
    fn start(t: Track) -> Option<Playing> {
        match t {
            Track::Ogg(b) => rodio::Decoder::try_from(std::io::Cursor::new(b)).ok().map(Playing::Ogg),
            Track::Module(b, trim) => {
                let m = xmrs::prelude::Module::load(b).ok()?;
                let m: &'static xmrs::prelude::Module = Box::leak(Box::new(m));
                let mut p = xmrsplayer::prelude::XmrsPlayer::new(m, RATE, 0);
                // once through, then the next song
                p.set_max_loop_count(1);
                Some(Playing::Module(Box::new(p), trim))
            }
        }
    }
    fn next(&mut self) -> Option<f32> {
        match self {
            Playing::Ogg(d) => d.next(),
            Playing::Module(p, trim) => p.next().map(|v| v as f32 / 32768.0 * *trim),
        }
    }
}

/// The level songs one after another, carrying on across games.
struct Playlist {
    next: Arc<AtomicUsize>,
    current: Option<Playing>,
    stop: Arc<AtomicBool>,
}

impl Iterator for Playlist {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.stop.load(Ordering::Relaxed) {
            return None;
        }
        for _ in 0..LEVEL_SONGS.len() + 1 {
            if let Some(d) = &mut self.current {
                if let Some(v) = d.next() {
                    return Some(v);
                }
            }
            let i = self.next.fetch_add(1, Ordering::Relaxed) % LEVEL_SONGS.len();
            self.current = Playing::start(LEVEL_SONGS[i]);
        }
        Some(0.0)
    }
}

impl Source for Playlist {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    // every level song is 44.1 kHz stereo (checked by a test)
    fn channels(&self) -> rodio::ChannelCount {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        NonZero::new(RATE).unwrap()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

struct Stoppable<S> {
    inner: S,
    stop: Arc<AtomicBool>,
}

impl<S: Source> Iterator for Stoppable<S> {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.stop.load(Ordering::Relaxed) {
            return None;
        }
        self.inner.next()
    }
}

impl<S: Source> Source for Stoppable<S> {
    fn current_span_len(&self) -> Option<usize> {
        self.inner.current_span_len()
    }
    fn channels(&self) -> rodio::ChannelCount {
        self.inner.channels()
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        self.inner.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn decode(bytes: &'static [u8]) -> Pcm {
    match rodio::Decoder::try_from(std::io::Cursor::new(bytes)) {
        Ok(d) => {
            let channels = d.channels().get();
            let rate = d.sample_rate().get();
            Pcm {
                channels,
                rate,
                data: d.collect(),
            }
        }
        Err(_) => Pcm {
            channels: 1,
            rate: 44_100,
            data: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_sample_decodes() {
        for (i, b) in super::SFX.iter().enumerate() {
            let p = super::decode(b);
            assert!(!p.data.is_empty(), "sample {i}");
        }
    }

    #[test]
    fn the_menu_song_loads_and_ends_after_one_pass() {
        let m = xmrs::prelude::Module::load(super::MENU_SONG).expect("crush-remix.xm");
        let m: &'static _ = Box::leak(Box::new(m));
        let mut p = super::menu_player(m);
        // it must end (so the rest and restart happen) within ten minutes
        let limit = 44_100 * 2 * 600;
        let n = (&mut p).take(limit).count();
        assert!(n < limit, "the menu song never ended");
        assert!(n > 44_100 * 2 * 10, "the menu song is suspiciously short");
    }

    #[test]
    fn every_song_decodes_at_44k_stereo() {
        let oggs = super::LEVEL_SONGS.iter().filter_map(|t| match t {
            super::Track::Ogg(b) => Some(*b),
            super::Track::Module(..) => None,
        });
        for b in oggs.chain([super::END_SONG]) {
            let d = rodio::Decoder::try_from(std::io::Cursor::new(b)).expect("ogg");
            use rodio::Source;
            assert_eq!(d.channels().get(), 2);
            assert_eq!(d.sample_rate().get(), 44_100);
        }
    }

    #[test]
    fn every_level_song_starts_and_the_module_ends() {
        for t in super::LEVEL_SONGS {
            let mut p = super::Playing::start(t).expect("level song");
            assert!(p.next().is_some());
            if let super::Playing::Module(..) = p {
                let limit = 44_100 * 2 * 900;
                let mut n = 0;
                while n < limit && p.next().is_some() {
                    n += 1;
                }
                assert!(n < limit, "the module never ended, so the playlist would stall");
            }
        }
    }

    #[test]
    fn the_playlist_moves_on_and_remembers() {
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        use std::sync::Arc;
        let next = Arc::new(AtomicUsize::new(0));
        let mut p = super::Playlist { next: next.clone(), current: None, stop: Arc::new(AtomicBool::new(false)) };
        assert!(p.next().is_some());
        assert_eq!(next.load(Ordering::Relaxed), 1, "the first song started");
        // a new game starts with the next song
        let mut q = super::Playlist { next: next.clone(), current: None, stop: Arc::new(AtomicBool::new(false)) };
        q.next();
        assert_eq!(next.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn muted_audio_still_feeds_the_scope() {
        let mut a = super::Audio::new(true);
        a.play_song(super::Song::Game);
        a.tick(500);
        let s = a.scope.snapshot();
        assert!(s.iter().any(|v| v.abs() > 0.01), "the scope saw only silence");
    }
}
