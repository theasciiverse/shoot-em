//! From pixels to characters, as the original did it (FUN_00402852), and
//! the text layer drawn over the picture (FUN_00402a46, FUN_00402ae8).

use std::io::Write;

/// The original's brightness ramp, darkest first.
pub const RAMP: &[u8] = b" .,:;=+tIYVRBM";

/// The Windows XP console palette the game was made for.
pub const PALETTE: [[u8; 3]; 16] = [
    [0, 0, 0],
    [0, 0, 128],
    [0, 128, 0],
    [0, 128, 128],
    [128, 0, 0],
    [128, 0, 128],
    [128, 128, 0],
    [192, 192, 192],
    [128, 128, 128],
    [0, 0, 255],
    [0, 255, 0],
    [0, 255, 255],
    [255, 0, 0],
    [255, 0, 255],
    [255, 255, 0],
    [255, 255, 255],
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    pub ch: char,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
}

impl Cell {
    pub const BLANK: Cell = Cell {
        ch: ' ',
        fg: [192, 192, 192],
        bg: [0, 0, 0],
    };
}

/// One cell of the picture: the character and console attribute the
/// original chose for an averaged (r, g, b) in 0..=1 per channel.
pub fn classify(r: f32, g: f32, b: f32) -> (u8, u8) {
    let sum = (r + g + b).min(1.0);
    let n = RAMP.len();
    let step = 1.0 / (n - 1) as f32;
    let mut ch = RAMP[n - 1];
    for (u, &c) in RAMP.iter().enumerate() {
        if sum < (u + 1) as f32 * step {
            ch = c;
            break;
        }
    }
    let mut attr = 0u8;
    if ch == RAMP[0] {
        attr = 7;
    }
    if r > 0.1 {
        attr |= 4;
    }
    if g > 0.1 {
        attr |= 2;
    }
    if b > 0.1 {
        attr |= 1;
    }
    if r > 0.5 || g > 0.5 || b > 0.5 {
        attr |= 8;
    }
    (ch, attr)
}

pub struct Screen {
    pub w: usize,
    pub h: usize,
    pub cells: Vec<Cell>,
    /// the text layer, drawn over the picture: Some(char, attribute)
    pub text: Vec<Option<(char, u8)>>,
    shown: Vec<Option<Cell>>,
}

impl Screen {
    pub fn new(w: usize, h: usize) -> Self {
        Screen {
            w,
            h,
            cells: vec![Cell::BLANK; w * h],
            text: vec![None; w * h],
            shown: vec![None; w * h],
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            *self = Screen::new(w, h);
        }
    }

    pub fn invalidate(&mut self) {
        self.shown.fill(None);
    }

    /// Turns the framebuffer (`ss` x `2*ss` samples a cell) into cells.
    pub fn fill_from_pixels(&mut self, px: &[[f32; 3]], pw: usize, ss: usize, truecolor: bool) {
        let sy = ss * 2;
        let n = (ss * sy) as f32;
        for cy in 0..self.h {
            for cx in 0..self.w {
                let mut acc = [0.0f32; 3];
                for y in 0..sy {
                    let row = (cy * sy + y) * pw;
                    for x in 0..ss {
                        let p = px[row + cx * ss + x];
                        acc[0] += p[0];
                        acc[1] += p[1];
                        acc[2] += p[2];
                    }
                }
                // the original averaged bytes, so quantise like it did
                let (r, g, b) = (
                    (acc[0] / n * 255.0).floor() / 255.0,
                    (acc[1] / n * 255.0).floor() / 255.0,
                    (acc[2] / n * 255.0).floor() / 255.0,
                );
                let (ch, attr) = classify(r, g, b);
                let fg = if truecolor && ch != b' ' {
                    let m = r.max(g).max(b).max(1e-6);
                    let k = 1.0 / m;
                    let lift = |v: f32| ((v * k).min(1.0) * 255.0) as u8;
                    [lift(r), lift(g), lift(b)]
                } else {
                    PALETTE[(attr & 15) as usize]
                };
                self.cells[cy * self.w + cx] = Cell {
                    ch: ch as char,
                    fg,
                    bg: [0, 0, 0],
                };
            }
        }
    }

    pub fn clear_cells(&mut self) {
        self.cells.fill(Cell::BLANK);
    }

    /// FUN_00402a46: text at (x, y) with a console attribute (low nibble
    /// foreground, high nibble background). Off-screen parts are dropped.
    pub fn put(&mut self, x: i32, y: i32, attr: u8, s: &str) {
        if y < 0 || y >= self.h as i32 {
            return;
        }
        for (i, c) in s.chars().enumerate() {
            let cx = x + i as i32;
            if cx < 0 || cx >= self.w as i32 {
                continue;
            }
            self.text[y as usize * self.w + cx as usize] = Some((c, attr));
        }
    }

    /// FUN_00402ac0: recolour one text cell.
    pub fn attr(&mut self, x: i32, y: i32, attr: u8) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return;
        }
        let i = y as usize * self.w + x as usize;
        let c = match self.text[i] {
            Some((c, _)) => c,
            None => self.cells[i].ch,
        };
        self.text[i] = Some((c, attr));
    }

    /// FUN_00402ae8: the text layer over the picture, then out to the
    /// terminal, writing only the cells that changed since last time.
    pub fn present(&mut self, out: &mut impl Write) -> std::io::Result<()> {
        let mut buf = String::with_capacity(self.w * self.h * 4);
        let mut last_fg = None;
        let mut last_bg = None;
        let mut cursor: Option<(usize, usize)> = None;
        for y in 0..self.h {
            for x in 0..self.w {
                let i = y * self.w + x;
                let mut c = self.cells[i];
                if let Some((ch, a)) = self.text[i] {
                    c = Cell {
                        ch,
                        fg: PALETTE[(a & 15) as usize],
                        bg: PALETTE[((a >> 4) & 15) as usize],
                    };
                }
                if self.shown[i] == Some(c) {
                    continue;
                }
                self.shown[i] = Some(c);
                if cursor != Some((x, y)) {
                    buf.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));
                }
                if last_fg != Some(c.fg) {
                    buf.push_str(&format!("\x1b[38;2;{};{};{}m", c.fg[0], c.fg[1], c.fg[2]));
                    last_fg = Some(c.fg);
                }
                if last_bg != Some(c.bg) {
                    buf.push_str(&format!("\x1b[48;2;{};{};{}m", c.bg[0], c.bg[1], c.bg[2]));
                    last_bg = Some(c.bg);
                }
                buf.push(c.ch);
                cursor = Some((x + 1, y));
            }
        }
        self.text.fill(None);
        if !buf.is_empty() {
            buf.push_str("\x1b[0m");
            out.write_all(buf.as_bytes())?;
            out.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_is_a_blank_in_grey() {
        assert_eq!(classify(0.0, 0.0, 0.0), (b' ', 7));
    }

    #[test]
    fn white_is_the_densest_character_in_bright_white() {
        assert_eq!(classify(1.0, 1.0, 1.0), (b'M', 15));
    }

    #[test]
    fn dim_red_is_a_dark_red_dot() {
        let (ch, attr) = classify(0.12, 0.0, 0.0);
        assert_eq!(ch, b'.');
        assert_eq!(attr, 4);
    }
}
