//! Shoot 'em: Jan Ringos' 3D shooting game (2005), whose whole 3D world is
//! drawn in coloured text, rebuilt from his Windows XP console program for
//! today's terminals. See REVERSE.md for how the original works and
//! PATCHES.md for what this edition does differently.

mod ascii;
mod audio;
mod game;
mod gfx;
mod hiscore;
mod input;
mod logo;
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
  --demo        the computer plays (straight into a game)
  --seed N      repeatable games
  --help        this text

Mouse: aim, left button fires the machine guns, right button drops a mine.
Keys: arrows aim, Space fires, M drops a mine, Escape returns to the menu.
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

    let code = run(&opts, &mut au, &mut table, &mut log);
    au.stop_song();
    drop(term);
    log.line("quit");
    std::process::exit(code);
}

fn run(opts: &Options, au: &mut Audio, table: &mut hiscore::Table, log: &mut Log) -> i32 {
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
        while event::poll(Duration::ZERO).unwrap_or(false) {
            if let Ok(ev) = event::read() {
                if let Some(l) = input.handle(&ev) {
                    log.line(&l);
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
        if w < 60 || h < 20 {
            scr.resize(w, h);
            scr.clear_cells();
            scr.put(0, 0, 0xc, "Shoot 'em needs at least 60x20");
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
                    input.mx = w as i32 / 2;
                    input.my = h as i32 / 2;
                }
                State::NameEntry => {
                    au.play_song(Song::Menu);
                    ne = screens::NameEntry::new();
                }
                State::HighScores => hof_top = 0,
                State::Instructions => scroll = 0,
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
                next = screens::menu(&mut ren, &mut scr, &input, t, clock);
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
                next = screens::high_scores(&mut ren, &mut scr, &input, t, table, &mut hof_top);
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
                game.update(dt, t, c, opts, au, &mut rng);
                game.draw(&mut ren, t);
                game.hud(
                    &mut scr,
                    input.mx,
                    input.my,
                    c.vx.clamp(0.0, 79.0),
                    c.vy.clamp(0.0, 49.0),
                    t,
                );
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
                    } else if table.qualifies(game.score) {
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
                    table.insert(hiscore::Entry {
                        nick,
                        msg1: ne.fields[1].clone(),
                        msg2: ne.fields[2].clone(),
                        score: game.score,
                    });
                    table.save();
                }
            }
            State::Score => {
                next = screens::score(&mut ren, &mut scr, &input, t, &game);
            }
            State::Quit => return 0,
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
