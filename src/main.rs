//! Shoot 'em: Jan Ringos' 3D shooting game (2005), whose whole 3D world is
//! drawn in coloured text, rebuilt from his Windows XP console program for
//! today's terminals. See REVERSE.md for how the original works and
//! PATCHES.md for what this edition does differently.

mod ascii;
mod audio;
mod game;
mod gfx;
mod groove;
mod hiscore;
mod input;
mod logo;
mod scope;
mod screens;

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
    PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use audio::{sfx, Audio, Song};
use screens::State;

/// The original's command line letters ("shoot-em a m"), plus our flags.
#[derive(Clone, Debug)]
pub struct Options {
    /// anti-aliasing: 0 none, 1 normal (2x2), 2 best (4x4)
    pub aa: u32,
    pub fps: bool,
    pub male: bool,
    pub clear: bool,
    pub more_particles: bool,
    pub truecolor: bool,
    pub mute: bool,
    pub demo: bool,
    pub seed: Option<u64>,
    pub quit_after: Option<u32>,
    /// Jan's 2005 screen exactly: no oscilloscope, no groove mode
    pub classic: bool,
    /// start in groove mode (also a menu switch, remembered)
    pub groove: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            aa: 1,
            fps: false,
            male: false,
            clear: false,
            more_particles: false,
            truecolor: false,
            mute: false,
            demo: false,
            seed: None,
            quit_after: None,
            classic: false,
            groove: false,
        }
    }
}

const HELP: &str = "\
Shoot 'em: shoot down what falls before it hits the grid.

Usage: shoot-em [letters] [options]

The original's letters (any combination, e.g. \"shoot-em am\"):
  a   best antialiasing (4x4 samples a cell; slower)
  b   normal antialiasing (2x2, the default)
  n   no antialiasing (sharpest)
  f   show frames per second
  m   male narrator instead of female
  c   clear: no debris from destroyed objects
  p   twice as much debris

Options:
  --truecolor   colour each character with its real colour, not the
                original's 16 console colours
  --mute        no sound or music
  --classic     Jan's 2005 screen exactly as it was (no sound scope, no
                groove mode)
  --groove      groove mode: fire on the music's beat for a combo, objects
                fall in on the beat (also a switch in the menu)
  --demo        the computer plays (straight into a game)
  --seed N      repeatable games
  --help        this text

Mouse: aim, left button fires the machine guns, right button drops a mine.
Keys: arrows aim, Space fires, M drops a mine, Escape returns to the menu,
S shows or hides the sound scope, G switches groove mode (both remembered),
C in the menu calibrates groove timing (tap Space on the clicks).
";

fn parse_args() -> Options {
    let mut o = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--help" | "-h" => {
                print!("{HELP}");
                std::process::exit(0);
            }
            "--version" | "-V" => {
                println!("shoot-em {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--truecolor" => o.truecolor = true,
            "--mute" | "--silent" => o.mute = true,
            "--demo" => o.demo = true,
            "--classic" => o.classic = true,
            "--groove" => o.groove = true,
            "--seed" => o.seed = args.next().and_then(|s| s.parse().ok()),
            "--quit-after" => o.quit_after = args.next().and_then(|s| s.parse().ok()),
            s if !s.starts_with('-') => {
                for c in s.chars() {
                    match c {
                        'a' => o.aa = 2,
                        'b' => o.aa = 1,
                        'n' => o.aa = 0,
                        'f' => o.fps = true,
                        'm' => o.male = true,
                        'c' => o.clear = true,
                        'p' => o.more_particles = true,
                        _ => {} // unknown letters are ignored, as they were
                    }
                }
            }
            _ => {}
        }
    }
    o
}

/// $XDG_STATE_HOME/shoot-em, ~/.local/state/shoot-em, or the macOS and
/// Windows equivalents.
fn state_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("XDG_STATE_HOME").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d).join("shoot-em"));
    }
    if cfg!(windows) {
        return std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("shoot-em"));
    }
    let home = PathBuf::from(std::env::var_os("HOME")?);
    if cfg!(target_os = "macos") {
        Some(home.join("Library/Application Support/shoot-em"))
    } else {
        Some(home.join(".local/state/shoot-em"))
    }
}

/// settings.txt in the state folder, one `key=value` a line: `scope=on`
/// or `off`, `groove=on` or `off`, `offset=<ms>` (groove timing).
#[derive(Clone, Copy, Debug)]
struct Settings {
    scope: bool,
    groove: bool,
    offset_ms: f64,
}

fn load_settings(dir: &Option<PathBuf>) -> Settings {
    let mut st = Settings { scope: true, groove: false, offset_ms: 0.0 };
    if let Some(t) = dir.as_ref().and_then(|d| std::fs::read_to_string(d.join("settings.txt")).ok()) {
        for l in t.lines() {
            match l.trim().split_once('=') {
                Some(("scope", v)) => st.scope = v != "off",
                Some(("groove", v)) => st.groove = v == "on",
                Some(("offset", v)) => st.offset_ms = v.parse().unwrap_or(0.0),
                _ => {}
            }
        }
    }
    st
}

fn save_settings(dir: &Option<PathBuf>, st: &Settings) {
    if let Some(d) = dir {
        let _ = std::fs::create_dir_all(d);
        let _ = std::fs::write(
            d.join("settings.txt"),
            format!(
                "scope={}\ngroove={}\noffset={:.0}\n",
                if st.scope { "on" } else { "off" },
                if st.groove { "on" } else { "off" },
                st.offset_ms
            ),
        );
    }
}

/// Session log: SHOOT_EM_LOG=debug writes every key, click and state
/// change, and a snapshot each second, to shoot-em.log in the state folder.
struct Log(Option<std::fs::File>, Instant);

impl Log {
    fn open(dir: &Option<PathBuf>) -> Log {
        let on = std::env::var("SHOOT_EM_LOG")
            .map(|v| v == "debug")
            .unwrap_or(false);
        let f = if on {
            dir.as_ref().and_then(|d| {
                let _ = std::fs::create_dir_all(d);
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(d.join("shoot-em.log"))
                    .ok()
            })
        } else {
            None
        };
        Log(f, Instant::now())
    }
    fn line(&mut self, s: &str) {
        if let Some(f) = &mut self.0 {
            let _ = writeln!(f, "{:>9.3} {s}", self.1.elapsed().as_secs_f32());
        }
    }
}

struct Term;

impl Term {
    fn start() -> std::io::Result<Term> {
        terminal::enable_raw_mode()?;
        let mut out = std::io::stdout();
        execute!(out, EnterAlternateScreen, cursor::Hide, EnableMouseCapture)?;
        let _ = execute!(
            out,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        );
        write!(out, "\x1b]0;Shoot 'em\x07")?;
        out.flush()?;
        Ok(Term)
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        let mut out = std::io::stdout();
        let _ = execute!(out, PopKeyboardEnhancementFlags);
        let _ = execute!(out, DisableMouseCapture, cursor::Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn main() {
    let opts = parse_args();
    let dir = state_dir();
    let mut log = Log::open(&dir);
    log.line(&format!("start {opts:?}"));
    let mut au = Audio::new(opts.mute);
    log.line(&format!("audio: {}", au.status));
    let mut table = hiscore::Table::load(dir.as_ref().map(|d| d.join("hiscores.txt")));
    // groove scores are kept apart: the combo makes them a different game
    let mut gtable = hiscore::Table::load(dir.as_ref().map(|d| d.join("hiscores-groove.txt")));

    let term = match Term::start() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("shoot-em: cannot use this terminal: {e}");
            std::process::exit(1);
        }
    };
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut out = std::io::stdout();
        let _ = execute!(
            out,
            PopKeyboardEnhancementFlags,
            DisableMouseCapture,
            cursor::Show,
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
        prev_hook(info);
    }));

    let code = run(&opts, &mut au, &mut table, &mut gtable, &mut log, &dir);
    au.stop_song();
    drop(term);
    log.line("quit");
    std::process::exit(code);
}

fn run(
    opts: &Options,
    au: &mut Audio,
    table: &mut hiscore::Table,
    gtable: &mut hiscore::Table,
    log: &mut Log,
    dir: &Option<PathBuf>,
) -> i32 {
    let mut settings = load_settings(dir);
    let mut scope_on = !opts.classic && settings.scope;
    let mut groove_on = !opts.classic && (opts.groove || settings.groove);
    let mut groove = groove::Groove::new(settings.offset_ms);
    // fire presses this frame, at music time
    let mut presses: Vec<f64> = Vec::new();
    // calibration: when it began, the clicks given, the taps' offsets
    let mut cal_start = Instant::now();
    let mut cal_clicks = 0u32;
    let mut cal_taps: Vec<f64> = Vec::new();
    // groove mode in the demo: the pilot's fire, gated to the beat
    let mut demo_was_firing = false;
    let seed = opts.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1)
    });
    let mut rng = game::Rng::new(seed);
    let (w, h) = terminal::size().unwrap_or((100, 30));
    let (mut w, mut h) = (w as usize, h as usize);
    let ss = match opts.aa {
        0 => 1,
        2 => 4,
        _ => 2,
    };
    let mut scr = ascii::Screen::new(w, h);
    let mut ren = gfx::Renderer::new(w * ss, h * ss * 2);
    let mut input = input::Input {
        mx: w as i32 / 2,
        my: h as i32 / 2,
        ..Default::default()
    };
    let mut state = if opts.demo { State::Game } else { State::Intro };
    let mut entered = true;
    let mut t: u32 = 0;
    let mut game = game::Game::new(&mut rng);
    let mut ne = screens::NameEntry::new();
    let mut scroll = 0;
    let mut hof_top = 0;
    let mut clock: u32 = 0;
    let mut total: u32 = 0;
    let mut fps = [0.0f32; 32];
    let mut fps_i = 0;
    let mut last = Instant::now();
    let mut last_snapshot = 0u32;
    let mut out = std::io::BufWriter::with_capacity(1 << 16, std::io::stdout());
    let frame_time = Duration::from_millis(16);

    loop {
        input.new_frame();
        presses.clear();
        while event::poll(Duration::ZERO).unwrap_or(false) {
            if let Ok(ev) = event::read() {
                let was = input.guns_held();
                let left_was = input.left;
                if let Some(l) = input.handle(&ev) {
                    log.line(&l);
                }
                // a press of the fire button (or a new click), timed when it
                // is read: groove mode judges it against the music
                let pressed = (!was && input.guns_held()) || (!left_was && input.left);
                if pressed && groove_on && matches!(state, State::Game | State::Calibrate) {
                    if state == State::Calibrate {
                        presses.push(cal_start.elapsed().as_secs_f64());
                    } else {
                        let b = au.beats.snapshot();
                        presses.push(groove.time(&b, Instant::now()));
                    }
                }
            }
        }
        if input.quit {
            return 0;
        }
        if input.resized {
            if let Ok((nw, nh)) = terminal::size() {
                w = nw as usize;
                h = nh as usize;
            }
            scr.resize(w, h);
            scr.invalidate();
            let _ = write!(out, "\x1b[0m\x1b[2J");
        }
        if w < 80 || h < 24 {
            scr.resize(w, h);
            scr.clear_cells();
            scr.put(0, 0, 0xc, "Shoot 'em needs at least 80x24");
            let _ = scr.present(&mut out);
            std::thread::sleep(Duration::from_millis(100));
            continue;
        }
        ren.resize(w * ss, h * ss * 2);
        ren.set_projection(w as f32 / (2.0 * h as f32));

        let now = Instant::now();
        let dt = (now - last).as_millis().clamp(1, 100) as u32;
        last = now;
        clock = clock.wrapping_add(dt);
        total = total.wrapping_add(dt);
        au.tick(dt);

        // keyboard aiming moves the mouse cell
        if input.moved_by_keys != (0, 0) && state == State::Game {
            input.mx = (input.mx + input.moved_by_keys.0 * 2).clamp(0, w as i32 - 1);
            input.my = (input.my + input.moved_by_keys.1).clamp(0, h as i32 - 1);
        }

        ren.clear();
        ren.lighting = true;
        if entered {
            log.line(&format!("state {state:?}"));
            match state {
                State::Intro => au.play_centre(sfx::INTRO),
                State::Menu => {
                    if !au.is_playing(Song::Menu) {
                        au.play_song(Song::Menu);
                    }
                }
                State::Game => {
                    au.play_song(Song::Game);
                    game = game::Game::new(&mut rng);
                    groove = groove::Groove::new(settings.offset_ms);
                    log.line(&format!("groove {}", if groove_on { "on" } else { "off" }));
                    input.mx = w as i32 / 2;
                    input.my = h as i32 / 2;
                }
                State::NameEntry => {
                    au.play_song(Song::End);
                    ne = screens::NameEntry::new();
                }
                State::Score => {
                    if !au.is_playing(Song::End) {
                        au.play_song(Song::End);
                    }
                }
                State::HighScores => {
                    hof_top = 0;
                    if !au.is_playing(Song::End) {
                        au.play_song(Song::End);
                    }
                }
                State::Instructions => scroll = 0,
                State::Calibrate => {
                    au.stop_song();
                    cal_start = Instant::now();
                    cal_clicks = 0;
                    cal_taps.clear();
                }
                _ => {}
            }
        }
        let was = state;
        let mut next: Option<State> = None;
        match state {
            State::Intro => {
                screens::intro(&mut ren, t);
                if input.key_esc || input.key_enter || input.typed.contains(&' ') || t > 3000 {
                    next = Some(State::Menu);
                }
            }
            State::Menu => {
                let before = groove_on;
                next = screens::menu(
                    &mut ren,
                    &mut scr,
                    &input,
                    t,
                    clock,
                    if opts.classic { None } else { Some(&mut groove_on) },
                );
                if groove_on != before {
                    settings.groove = groove_on;
                    save_settings(dir, &settings);
                    au.play_centre(sfx::CLICK);
                    log.line(&format!("groove {}", if groove_on { "on" } else { "off" }));
                }
                if next == Some(State::Calibrate) {
                    groove_on = true;
                }
                if let Some(n) = next {
                    au.play_centre(sfx::CLICK);
                    if n == State::Game || n == State::Quit {
                        au.stop_song();
                    }
                }
            }
            State::Instructions => {
                next = screens::instructions(&mut ren, &mut scr, &input, t, &mut scroll);
            }
            State::HighScores => {
                let (tb, sub) = if groove_on { (&*gtable, "Groove mode") } else { (&*table, "The best players") };
                next = screens::high_scores(&mut ren, &mut scr, &input, t, tb, &mut hof_top, sub);
            }
            State::Game => {
                let c = if opts.demo {
                    let c = game.autopilot(t);
                    input.mx = (c.vx * w as f32 / 80.0) as i32;
                    input.my = (c.vy * h as f32 / 50.0) as i32;
                    c
                } else {
                    game::Controls {
                        vx: input.mx as f32 * 80.0 / w as f32,
                        vy: input.my as f32 * 50.0 / h as f32,
                        guns: input.guns_held(),
                        mine: input.mine_held(),
                    }
                };
                let mut c = c;
                let tick = if groove_on {
                    let b = au.beats.snapshot();
                    let now = Instant::now();
                    let gt = groove.time(&b, now);
                    if opts.demo {
                        // the pilot fires in short bursts that start on the beat
                        let near = groove.to_beat(&b, gt);
                        let firing = c.guns && (0.0..0.12).contains(&near);
                        if firing && !demo_was_firing {
                            presses.push(gt - near);
                        }
                        demo_was_firing = firing;
                        c.guns = firing;
                    }
                    let tick = groove.tick(&b);
                    for &p in &presses {
                        let pts = groove.press(&b, p, now);
                        game.score += pts;
                        if let Some((j, d, _)) = groove.last {
                            log.line(&format!("groove press {:?} {:+.0} ms combo {} +{}", j, d * 1000.0, groove.combo, pts));
                        }
                    }
                    game.groove_mult = groove.multiplier();
                    if tick.beats > 0 {
                        log.line(&format!(
                            "groove beat count={} period={:.3}s down={} energy={:.2} combo={} x{}",
                            b.count, tick.period, tick.downbeat, tick.energy, groove.combo, groove.multiplier()
                        ));
                    }
                    Some(tick)
                } else {
                    None
                };
                game.update(dt, t, c, opts, au, &mut rng, tick);
                game.draw(&mut ren, t);
                game.hud(
                    &mut scr,
                    input.mx,
                    input.my,
                    c.vx.clamp(0.0, 79.0),
                    c.vy.clamp(0.0, 49.0),
                    t,
                );
                if groove_on {
                    let b = au.beats.snapshot();
                    let gt = groove.time(&b, Instant::now());
                    let pulse = groove.pulse(&b, gt);
                    let cx = (w as i32 - 80) / 2;
                    let a = if pulse > 0.3 { 0xf } else { 0xd };
                    scr.put(cx + 2, 1, a, &format!(" GROOVE x{} ", groove.multiplier()));
                    scr.put(cx + 13, 1, 0xd, &format!("  combo {:<3} ", groove.combo));
                    if let Some((j, d, at)) = groove.last {
                        if at.elapsed() < Duration::from_millis(700) {
                            let (txt, col) = match j {
                                groove::Judge::Perfect => ("PERFECT", 0xe),
                                groove::Judge::Good => ("GOOD   ", 0xa),
                                groove::Judge::Off => ("OFF    ", 0x8),
                            };
                            scr.put(cx + 62, 1, col, &format!("{txt} {:+4.0} ms", d * 1000.0));
                        }
                    }
                }
                if t >= last_snapshot + 1000 {
                    last_snapshot = t;
                    log.line(&format!(
                        "snapshot t={} score={} health={:.1} ammo={} rapid={} mines={} objects={} aim={:?}",
                        t, game.score, game.health, game.ammo * 2, game.rapid * 2, game.mines_left, game.objs.len(), game.aim()
                    ));
                }
                if game.over {
                    au.stop_song();
                    log.line(&format!("game over, score {}", game.score));
                    next = Some(if opts.demo {
                        State::Game
                    } else if (if groove_on { &*gtable } else { &*table }).qualifies(game.score) {
                        State::NameEntry
                    } else {
                        State::Score
                    });
                } else if input.key_esc {
                    au.stop_song();
                    next = Some(if opts.demo { State::Quit } else { State::Menu });
                }
            }
            State::NameEntry => {
                next = screens::name_entry(&mut ren, &mut scr, &input, t, &game, &mut ne);
                if next == Some(State::Score) && !input.key_esc {
                    au.play_centre(sfx::CLICK);
                    let nick = if ne.fields[0].trim().is_empty() {
                        "Player".to_string()
                    } else {
                        ne.fields[0].clone()
                    };
                    let tb = if groove_on { &mut *gtable } else { &mut *table };
                    tb.insert(hiscore::Entry {
                        nick,
                        msg1: ne.fields[1].clone(),
                        msg2: ne.fields[2].clone(),
                        score: game.score,
                    });
                    tb.save();
                }
            }
            State::Score => {
                next = screens::score(&mut ren, &mut scr, &input, t, &game);
            }
            State::Calibrate => {
                // clicks every 0.6 s (100 BPM), starting a second in
                let el = cal_start.elapsed().as_secs_f64();
                let due = ((el - 1.0) / 0.6).floor() as i64 + 1;
                while (cal_clicks as i64) < due.max(0) {
                    au.play_centre(sfx::CLICK);
                    cal_clicks += 1;
                }
                for &p in &presses {
                    // the nearest click, and how far from it the tap was
                    let d = groove::to_nearest_beat(p, 1.0, 0.6);
                    if p > 0.8 && d.abs() < 0.25 {
                        cal_taps.push(d);
                    }
                }
                let used: Vec<f64> = cal_taps.iter().skip(2).copied().collect();
                if used.len() >= screens::CALIBRATE_TAPS && (settings.offset_ms - median(&used) * 1000.0).abs() > 0.5 {
                    settings.offset_ms = (median(&used) * 1000.0).round();
                    groove.offset_ms = settings.offset_ms;
                    save_settings(dir, &settings);
                    log.line(&format!("calibrated offset {:+.0} ms from {} taps", settings.offset_ms, used.len()));
                }
                let flash = el > 1.0 && groove::to_nearest_beat(el, 1.0, 0.6).abs() < 0.08;
                next = screens::calibrate(&mut scr, &input, &used, settings.offset_ms, flash);
            }
            State::Quit => return 0,
        }

        // S toggles the sound scope (not while typing a name)
        if !opts.classic
            && state != State::NameEntry
            && input.typed.iter().any(|&c| c == 's' || c == 'S')
        {
            scope_on = !scope_on;
            settings.scope = scope_on;
            save_settings(dir, &settings);
            log.line(&format!("scope {}", if scope_on { "on" } else { "off" }));
        }
        if scope_on {
            let (wi, hi) = (w as i32, h as i32);
            let cx = (wi - 80) / 2;
            let samples = au.scope.snapshot();
            // just under a screen's content (its last button ends at
            // `bottom`): up to three rows where the terminal allows, else
            // the last row
            let under = |bottom: i32| -> (i32, i32) {
                let y = bottom + 2;
                if y < hi {
                    (y, (hi - y).min(3))
                } else {
                    (hi - 1, 1)
                }
            };
            match state {
                // its own row, directly above the HUD's boxes, as wide as
                // the HUD (the boxes are kept at the bottom: rows h-6 to h-2)
                State::Game => {
                    // in groove mode it flashes white and swells on each beat
                    let (gain, attr) = if groove_on {
                        let b = au.beats.snapshot();
                        let gt = groove.time(&b, Instant::now());
                        let p = groove.pulse(&b, gt);
                        (1.6 * (1.0 + 0.8 * p), if p > 0.3 { 0xf } else { 0xb })
                    } else {
                        (1.6, 0xb)
                    };
                    scope::draw(&mut scr, cx + 2, hi - 7, 77, 1, &samples, gain, attr)
                }
                State::Menu => {
                    // clear of the buttons (two rows of them now) and the
                    // credits: under the credits when the terminal has room,
                    // else the free row between the copyright and the credits
                    let (y, rows) = if hi >= 50 {
                        let o = (hi - 50) / 2;
                        let below = o + 50;
                        if hi - below >= 2 {
                            (below, (hi - below).min(3))
                        } else {
                            (o + 46, 1)
                        }
                    } else {
                        (hi - 3, 1)
                    };
                    scope::draw(&mut scr, cx + 8, y, 64, rows, &samples, 1.6, 0xb);
                }
                State::Score => {
                    // the OK button ends at oy + 42
                    let oy = if hi >= 50 { (hi - 50) / 2 } else { hi - 44 };
                    let (y, rows) = under(oy + 42);
                    scope::draw(&mut scr, cx + 8, y, 64, rows, &samples, 1.6, 0xb);
                }
                State::HighScores => {
                    // the BACK button ends two rows below its top
                    let by = if hi >= 50 { (hi - 50) / 2 + 45 } else { hi - 3 };
                    let (y, rows) = under(by + 2);
                    scope::draw(&mut scr, cx + 8, y, 64, rows, &samples, 1.6, 0xb);
                }
                State::NameEntry => {
                    // the OK button: the original's rows 45 to 47
                    let bottom = if hi >= 50 { (hi - 50) / 2 + 47 } else { (hi - 21) / 2 + 17 };
                    let (y, rows) = under(bottom);
                    scope::draw(&mut scr, cx + 8, y, 64, rows, &samples, 1.6, 0xb);
                }
                _ => {}
            }
        }

        let pw = w * ss;
        scr.fill_from_pixels(&ren.color, pw, ss, opts.truecolor);
        if opts.fps {
            let avg: f32 = fps.iter().sum::<f32>() / 32.0;
            scr.put(2, 2, 7, "+----------+");
            scr.put(2, 3, 7, &format!("| FPS: {avg:03.0} |"));
            scr.put(2, 4, 7, "+----------+");
        }
        let _ = scr.present(&mut out);

        if let Some(n) = next {
            state = n;
            entered = true;
            t = 0;
            last_snapshot = 0;
            scr.invalidate();
            if n == State::Quit {
                return 0;
            }
        } else {
            entered = false;
            t = t.wrapping_add(dt);
        }
        let _ = was;
        if let Some(q) = opts.quit_after {
            if total >= q {
                return 0;
            }
        }
        let spent = now.elapsed();
        fps[fps_i] = 1000.0 / (dt as f32);
        fps_i = (fps_i + 1) % 32;
        if spent < frame_time {
            std::thread::sleep(frame_time - spent);
        }
    }
}

fn median(v: &[f64]) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if s.is_empty() {
        0.0
    } else {
        s[s.len() / 2]
    }
}
