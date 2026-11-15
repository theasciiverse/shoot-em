//! Groove mode: the in-game music sets the pulse. Shots fired on the beat
//! score more and build a combo, objects fall in on the beats, bigger
//! waves come in on loud bars, and the sound scope flashes with each beat.
//! Jan's game without it is the default.
//!
//! The clock is the music itself: the playlist counts the frames it has
//! played and notes each beat's frame (audio.rs, `Beats`). Between the
//! audio side's updates the time is carried on by the wall clock, and the
//! player's calibration offset (output latency plus their own timing) is
//! taken off, as in Bellbeat.

use crate::audio::{BeatState, FRAMES_PER_SECOND};
use std::time::Instant;

/// A press this close to a beat is on it: perfect, then good. Generous on
/// purpose: aiming and timing at once is harder than pressing a lane key.
pub const PERFECT: f64 = 0.070;
pub const GOOD: f64 = 0.130;
/// The combo falls apart after this many beats without a press on one.
const COMBO_KEEP_BEATS: u64 = 4;
/// Points for a press on the beat, before the multiplier.
const PERFECT_POINTS: u32 = 50;
const GOOD_POINTS: u32 = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Judge {
    Perfect,
    Good,
    Off,
}

pub fn judge(d: f64) -> Judge {
    let a = d.abs();
    if a <= PERFECT {
        Judge::Perfect
    } else if a <= GOOD {
        Judge::Good
    } else {
        Judge::Off
    }
}

/// x1 for a combo under 4, then one more every 4 presses on the beat, to x5.
pub fn multiplier(combo: u32) -> u32 {
    1 + combo.min(16) / 4
}

/// Signed seconds from `t` to the nearest beat of a grid with a beat at
/// `beat` and period `period` (all seconds): negative when early.
pub fn to_nearest_beat(t: f64, beat: f64, period: f64) -> f64 {
    if period <= 0.0 {
        return f64::INFINITY;
    }
    let d = t - beat;
    d - (d / period).round() * period
}

/// What the beats did since the last frame, for the spawner.
#[derive(Clone, Copy, Debug, Default)]
pub struct BeatTick {
    /// beats that started since the last frame
    pub beats: u32,
    /// one of them started a bar
    pub downbeat: bool,
    /// the last bar's loudness over its running average (1 is usual)
    pub energy: f32,
    /// seconds between beats
    pub period: f64,
}

pub struct Groove {
    pub combo: u32,
    pub best_combo: u32,
    pub perfect: u32,
    pub good: u32,
    pub off: u32,
    /// the last judged press and when, for the HUD
    pub last: Option<(Judge, f64, Instant)>,
    /// calibration: milliseconds taken off the music's time
    pub offset_ms: f64,
    /// the beat count last seen, and the beat of the last press on one
    seen: u64,
    seen_down: u64,
    last_on_beat: u64,
    /// the frame count last read and when it changed
    clock: (u64, Instant),
}

impl Groove {
    pub fn new(offset_ms: f64) -> Groove {
        Groove {
            combo: 0,
            best_combo: 0,
            perfect: 0,
            good: 0,
            off: 0,
            last: None,
            offset_ms,
            seen: 0,
            seen_down: 0,
            last_on_beat: 0,
            clock: (0, Instant::now()),
        }
    }

    /// The music's time in seconds at `now`, as the player hears it.
    pub fn time(&mut self, b: &BeatState, now: Instant) -> f64 {
        if b.frames != self.clock.0 {
            self.clock = (b.frames, now);
        }
        let since = now.saturating_duration_since(self.clock.1).as_secs_f64().min(0.1);
        b.frames as f64 / FRAMES_PER_SECOND + since - self.offset_ms / 1000.0
    }

    /// Seconds from `t` to the nearest beat.
    pub fn to_beat(&self, b: &BeatState, t: f64) -> f64 {
        to_nearest_beat(t, b.last_beat as f64 / FRAMES_PER_SECOND, b.period / FRAMES_PER_SECOND)
    }

    /// How fresh the last beat is, 1 on it and falling to 0 a fifth of a
    /// beat later: for flashing the scope and the HUD.
    pub fn pulse(&self, b: &BeatState, t: f64) -> f32 {
        let p = (b.period / FRAMES_PER_SECOND).max(0.1);
        let d = to_nearest_beat(t, b.last_beat as f64 / FRAMES_PER_SECOND, p);
        if d < 0.0 {
            return 0.0;
        }
        (1.0 - d / (p * 0.2)).clamp(0.0, 1.0) as f32
    }

    /// The fire button went down at music time `t`: judge it, keep the
    /// combo, and return the points it earns (0 off the beat).
    pub fn press(&mut self, b: &BeatState, t: f64, now: Instant) -> u32 {
        let d = self.to_beat(b, t);
        let j = judge(d);
        self.last = Some((j, d, now));
        match j {
            Judge::Off => {
                self.off += 1;
                self.combo = 0;
                0
            }
            Judge::Perfect | Judge::Good => {
                if j == Judge::Perfect {
                    self.perfect += 1;
                } else {
                    self.good += 1;
                }
                self.combo += 1;
                self.best_combo = self.best_combo.max(self.combo);
                self.last_on_beat = b.count;
                let base = if j == Judge::Perfect { PERFECT_POINTS } else { GOOD_POINTS };
                base * multiplier(self.combo)
            }
        }
    }

    pub fn multiplier(&self) -> u32 {
        multiplier(self.combo)
    }

    /// Once a frame: what the beats did since the last frame; the combo
    /// lapses after a few beats without a press on one.
    pub fn tick(&mut self, b: &BeatState) -> BeatTick {
        if !b.playing {
            return BeatTick { beats: 0, downbeat: false, energy: 1.0, period: b.period / FRAMES_PER_SECOND };
        }
        let beats = b.count.saturating_sub(self.seen) as u32;
        let downbeat = b.last_downbeat != u64::MAX && b.last_downbeat > self.seen_down && b.last_downbeat <= b.count;
        if downbeat {
            self.seen_down = b.last_downbeat;
        }
        self.seen = b.count;
        if self.combo > 0 && b.count > self.last_on_beat + COMBO_KEEP_BEATS {
            self.combo = 0;
        }
        let energy = if b.avg_energy > 0.0 { b.bar_energy / b.avg_energy } else { 1.0 };
        BeatTick {
            beats,
            downbeat,
            energy,
            period: b.period / FRAMES_PER_SECOND,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn state(frames: u64, last_beat: u64, period: f64, count: u64) -> BeatState {
        BeatState {
            frames,
            last_beat,
            period,
            count,
            last_downbeat: u64::MAX,
            bar_energy: 0.0,
            avg_energy: 0.0,
            playing: true,
        }
    }

    #[test]
    fn the_nearest_beat_is_signed_and_wraps() {
        assert!((to_nearest_beat(10.05, 10.0, 0.5) - 0.05).abs() < 1e-9);
        assert!((to_nearest_beat(10.45, 10.0, 0.5) + 0.05).abs() < 1e-9, "early for the next beat");
        assert!((to_nearest_beat(12.0, 10.0, 0.5)).abs() < 1e-9, "four beats on");
    }

    #[test]
    fn judging_windows() {
        assert_eq!(judge(0.0), Judge::Perfect);
        assert_eq!(judge(-PERFECT), Judge::Perfect);
        assert_eq!(judge(PERFECT + 0.001), Judge::Good);
        assert_eq!(judge(-GOOD), Judge::Good);
        assert_eq!(judge(GOOD + 0.001), Judge::Off);
    }

    #[test]
    fn the_multiplier_climbs_to_five() {
        assert_eq!(multiplier(0), 1);
        assert_eq!(multiplier(3), 1);
        assert_eq!(multiplier(4), 2);
        assert_eq!(multiplier(12), 4);
        assert_eq!(multiplier(40), 5);
    }

    #[test]
    fn the_clock_follows_the_frames_and_the_offset() {
        let mut g = Groove::new(50.0);
        let t0 = Instant::now();
        let b = state(44_100, 0, 22_050.0, 2);
        let t = g.time(&b, t0);
        assert!((t - 0.95).abs() < 1e-6, "one second, less 50 ms: {t}");
        // the wall clock carries it on between updates, but not past 0.1 s
        let t = g.time(&b, t0 + Duration::from_millis(40));
        assert!((t - 0.99).abs() < 1e-3);
        let t = g.time(&b, t0 + Duration::from_secs(5));
        assert!((t - 1.05).abs() < 1e-3);
    }

    #[test]
    fn presses_on_the_beat_build_a_combo_and_off_ones_break_it() {
        let mut g = Groove::new(0.0);
        let now = Instant::now();
        let b = state(0, 44_100, 22_050.0, 10);
        assert_eq!(g.press(&b, 1.0, now), PERFECT_POINTS);
        assert_eq!(g.press(&b, 1.5 + 0.1, now), GOOD_POINTS);
        assert_eq!(g.combo, 2);
        assert_eq!(g.press(&b, 1.25, now), 0, "half way between beats");
        assert_eq!(g.combo, 0);
        for _ in 0..8 {
            g.press(&b, 2.0, now);
        }
        assert_eq!(g.multiplier(), 3);
        assert_eq!(g.best_combo, 8);
    }

    #[test]
    fn the_combo_lapses_without_beats_pressed() {
        let mut g = Groove::new(0.0);
        let now = Instant::now();
        let b = state(0, 0, 22_050.0, 1);
        g.press(&b, 0.0, now);
        assert_eq!(g.combo, 1);
        g.tick(&state(0, 0, 22_050.0, 4));
        assert_eq!(g.combo, 1);
        g.tick(&state(0, 0, 22_050.0, 6));
        assert_eq!(g.combo, 0);
    }

    #[test]
    fn ticks_count_beats_and_bars() {
        let mut g = Groove::new(0.0);
        let mut b = state(0, 0, 22_050.0, 3);
        b.last_downbeat = 1;
        let t = g.tick(&b);
        assert_eq!(t.beats, 3);
        assert!(t.downbeat);
        let t = g.tick(&b);
        assert_eq!(t.beats, 0);
        assert!(!t.downbeat, "the same bar is not counted twice");
    }
}
