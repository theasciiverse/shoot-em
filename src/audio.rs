//! Sound and music, in place of FMOD: the 28 samples the original carried
//! as resources (Ogg Vorbis, unchanged) and its two Protracker modules.
//! Without a sound device, or with --mute, everything is silent.

use rodio::buffer::SamplesBuffer;
use rodio::Source;
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

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

const MENU_SONG: &[u8] = include_bytes!("../assets/music/boombass.mod");
const GAME_SONG: &[u8] = include_bytes!("../assets/music/badlands.mod");

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
}

struct Pcm {
    channels: u16,
    rate: u32,
    data: Vec<f32>,
}

pub struct Audio {
    sink: Option<rodio::MixerDeviceSink>,
    pcm: Vec<Pcm>,
    song: Option<(Song, Arc<AtomicBool>)>,
    pub status: String,
}

impl Audio {
    pub fn new(mute: bool) -> Audio {
        let mut a = Audio {
            sink: None,
            pcm: Vec::new(),
            song: None,
            status: String::new(),
        };
        if mute {
            a.status = "muted".into();
            return a;
        }
        match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(mut s) => {
                s.log_on_drop(false);
                a.sink = Some(s);
            }
            Err(e) => {
                a.status = format!("no sound ({e})");
                return a;
            }
        }
        for bytes in SFX {
            a.pcm.push(decode(bytes));
        }
        a.status = "sound on".into();
        a
    }

    /// FSOUND_PlaySound with FSOUND_SetPan / FSOUND_SetVolume: pan and
    /// volume 0..=255, 128 the centre.
    pub fn play(&self, i: usize, pan: i32, vol: i32) {
        let Some(sink) = &self.sink else { return };
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
        sink.mixer().add(SamplesBuffer::new(
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
        let Some(sink) = &self.sink else { return };
        let bytes = match song {
            Song::Menu => MENU_SONG,
            Song::Game => GAME_SONG,
        };
        let Ok(module) = xmrs::prelude::Module::load_mod(bytes) else {
            return;
        };
        let module: &'static xmrs::prelude::Module = Box::leak(Box::new(module));
        let stop = Arc::new(AtomicBool::new(false));
        let player = xmrsplayer::prelude::XmrsPlayer::new(module, 44_100, 0);
        sink.mixer().add(
            Tracker {
                player,
                stop: stop.clone(),
            }
            .amplify(0.6),
        );
        self.song = Some((song, stop));
    }

    pub fn stop_song(&mut self) {
        if let Some((_, stop)) = self.song.take() {
            stop.store(true, Ordering::Relaxed);
        }
    }
}

struct Tracker {
    player: xmrsplayer::prelude::XmrsPlayer<'static>,
    stop: Arc<AtomicBool>,
}

impl Iterator for Tracker {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        if self.stop.load(Ordering::Relaxed) {
            return None;
        }
        self.player.next().map(|s| s as f32 / 32768.0)
    }
}

impl Source for Tracker {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> rodio::ChannelCount {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> rodio::SampleRate {
        NonZero::new(44_100).unwrap()
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
    fn both_modules_load() {
        assert!(xmrs::prelude::Module::load_mod(super::MENU_SONG).is_ok());
        assert!(xmrs::prelude::Module::load_mod(super::GAME_SONG).is_ok());
    }
}
