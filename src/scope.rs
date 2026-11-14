//! The Braille oscilloscope (after Bellbeat's): the last samples of the
//! game's own mix as a trace of Braille dots, 2x4 to a character cell,
//! steadied on a rising zero crossing so the wave stands still.

use crate::ascii::Screen;

/// Draws `samples` (oldest first) into `wc` x `hc` cells at (x, y) in the
/// console attribute `attr`. Cells with no dot are left alone.
#[allow(clippy::too_many_arguments)]
pub fn draw(s: &mut Screen, x: i32, y: i32, wc: i32, hc: i32, samples: &[f32], gain: f32, attr: u8) {
    if wc < 2 || hc < 1 || samples.len() < 8 {
        return;
    }
    let (wc, hc) = (wc as usize, hc as usize);
    let n = samples.len();
    let (w, h) = (wc * 2, hc * 4);
    let span = (w * 2).min(n / 2);
    // the latest rising zero crossing that leaves room for a whole trace
    let mut start = n - span;
    for i in (1..n - span).rev() {
        if samples[i - 1] < 0.0 && samples[i] >= 0.0 {
            start = i;
            break;
        }
    }
    let mut dots = vec![0u8; wc * hc];
    let mut set = |px: usize, py: usize| {
        if px < w && py < h {
            const BIT: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
            dots[(py / 4) * wc + px / 2] |= BIT[px % 2][py % 4];
        }
    };
    let ypix = |v: f32| (((1.0 - (v * gain).clamp(-1.0, 1.0)) * 0.5) * (h - 1) as f32).round() as usize;
    let mut prev = ypix(samples[start]);
    for px in 0..w {
        let yy = ypix(samples[start + px * span / w]);
        let (a, b) = if yy < prev { (yy, prev) } else { (prev, yy) };
        for py in a..=b {
            set(px, py);
        }
        prev = yy;
    }
    for cy in 0..hc {
        for cx in 0..wc {
            let d = dots[cy * wc + cx];
            if d != 0 {
                let ch = char::from_u32(0x2800 + d as u32).unwrap_or(' ');
                let mut buf = [0u8; 4];
                s.put(x + cx as i32, y + cy as i32, attr, ch.encode_utf8(&mut buf));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sine_draws_braille_and_silence_a_flat_line() {
        let sine: Vec<f32> = (0..2048).map(|i| (i as f32 * 0.05).sin() * 0.6).collect();
        let mut s = Screen::new(40, 4);
        draw(&mut s, 0, 0, 40, 3, &sine, 1.4, 0xb);
        let rows: Vec<usize> = (0..3)
            .map(|r| (0..40).filter(|&c| s.text[r * 40 + c].is_some()).count())
            .collect();
        assert!(rows.iter().all(|&n| n > 0), "the wave reaches every row: {rows:?}");
        let mut f = Screen::new(40, 4);
        draw(&mut f, 0, 0, 40, 3, &vec![0.0; 2048], 1.4, 0xb);
        let lit: Vec<usize> = (0..3)
            .map(|r| (0..40).filter(|&c| f.text[r * 40 + c].is_some()).count())
            .collect();
        assert_eq!(lit.iter().filter(|&&n| n > 0).count(), 1, "silence is one flat line: {lit:?}");
    }
}
