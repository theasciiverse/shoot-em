//! Keys and mouse from the terminal, gathered once a frame the way the
//! original gathered console input (FUN_0040a198): the mouse cell, the
//! buttons held, and the keys pressed since the last frame.

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use std::time::{Duration, Instant};

/// Without key-release reports a held key arrives as repeats; it counts as
/// held this long after each one.
const HOLD: Duration = Duration::from_millis(160);

#[derive(Default)]
pub struct Input {
    pub mx: i32,
    pub my: i32,
    pub left: bool,
    pub right: bool,
    pub clicked: bool,
    pub wheel: i32,
    pub key_esc: bool,
    pub key_enter: bool,
    pub key_tab: bool,
    pub key_backspace: bool,
    pub up: bool,
    pub down: bool,
    pub left_key: bool,
    pub right_key: bool,
    pub typed: Vec<char>,
    pub quit: bool,
    pub resized: bool,
    pub space_until: Option<Instant>,
    pub mine_until: Option<Instant>,
    pub space_down: bool,
    pub mine_down: bool,
    pub releases: bool,
    pub moved_by_keys: (i32, i32),
}

impl Input {
    /// Clears what only lasts one frame.
    pub fn new_frame(&mut self) {
        self.clicked = false;
        self.wheel = 0;
        self.key_esc = false;
        self.key_enter = false;
        self.key_tab = false;
        self.key_backspace = false;
        self.up = false;
        self.down = false;
        self.left_key = false;
        self.right_key = false;
        self.typed.clear();
        self.resized = false;
        self.moved_by_keys = (0, 0);
    }

    pub fn guns_held(&self) -> bool {
        self.left || self.space_down || self.space_until.is_some_and(|t| Instant::now() < t)
    }

    pub fn mine_held(&self) -> bool {
        self.right || self.mine_down || self.mine_until.is_some_and(|t| Instant::now() < t)
    }

    /// Returns a line for the session log.
    pub fn handle(&mut self, ev: &Event) -> Option<String> {
        match ev {
            Event::Key(k) => {
                let release = k.kind == KeyEventKind::Release;
                if release {
                    self.releases = true;
                }
                match k.code {
                    KeyCode::Char(' ') => {
                        if release {
                            self.space_down = false;
                            self.space_until = None;
                        } else if self.releases {
                            self.space_down = true;
                        } else {
                            self.space_until = Some(Instant::now() + HOLD);
                        }
                    }
                    KeyCode::Char('m')
                    | KeyCode::Char('M')
                    | KeyCode::Char('x')
                    | KeyCode::Char('X')
                        if !self.typing_mode() =>
                    {
                        if release {
                            self.mine_down = false;
                            self.mine_until = None;
                        } else if self.releases {
                            self.mine_down = true;
                        } else {
                            self.mine_until = Some(Instant::now() + HOLD);
                        }
                    }
                    _ => {}
                }
                if release {
                    return Some(format!("key up {:?}", k.code));
                }
                if k.modifiers.contains(KeyModifiers::CONTROL)
                    && matches!(k.code, KeyCode::Char('c'))
                {
                    self.quit = true;
                }
                match k.code {
                    KeyCode::Esc => self.key_esc = true,
                    KeyCode::Enter => self.key_enter = true,
                    KeyCode::Tab => self.key_tab = true,
                    KeyCode::Backspace => self.key_backspace = true,
                    KeyCode::Up => {
                        self.up = true;
                        self.moved_by_keys.1 -= 1;
                    }
                    KeyCode::Down => {
                        self.down = true;
                        self.moved_by_keys.1 += 1;
                    }
                    KeyCode::Left => {
                        self.left_key = true;
                        self.moved_by_keys.0 -= 1;
                    }
                    KeyCode::Right => {
                        self.right_key = true;
                        self.moved_by_keys.0 += 1;
                    }
                    KeyCode::Char(c) => self.typed.push(c),
                    _ => {}
                }
                Some(format!("key {:?} {:?}", k.code, k.modifiers))
            }
            Event::Mouse(m) => {
                self.mx = m.column as i32;
                self.my = m.row as i32;
                match m.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        self.left = true;
                        self.clicked = true;
                    }
                    MouseEventKind::Up(MouseButton::Left) => self.left = false,
                    MouseEventKind::Down(MouseButton::Right) => self.right = true,
                    MouseEventKind::Up(MouseButton::Right) => self.right = false,
                    MouseEventKind::ScrollDown => self.wheel += 1,
                    MouseEventKind::ScrollUp => self.wheel -= 1,
                    _ => {}
                }
                if matches!(m.kind, MouseEventKind::Moved | MouseEventKind::Drag(_)) {
                    None
                } else {
                    Some(format!("mouse {:?} at {},{}", m.kind, m.column, m.row))
                }
            }
            Event::Resize(w, h) => {
                self.resized = true;
                Some(format!("resize {w}x{h}"))
            }
            _ => None,
        }
    }

    fn typing_mode(&self) -> bool {
        false
    }
}
