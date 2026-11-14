//! Everything around the game: the intro logo, the menu, the instructions,
//! the score and statistics, the Hall of Fame and the name entry
//! (FUN_0040bae0, 0040d0a6, 0040bc4a, 0040c096, 0040c572, 0040c9ae).
//! Laid out as on the original's 80x50 console when the terminal is that
//! tall, and fitted into fewer rows when it is not.

use crate::ascii::Screen;
use crate::game::{self, Kind};
use crate::gfx::{self, Renderer};
use crate::hiscore::Table;
use crate::input::Input;

/// A 5x7 block font for the titles the original wrote in 3D outline type.
fn glyph(c: char) -> [&'static str; 7] {
    match c {
        'S' => [
            " ####", "#    ", "#    ", " ### ", "    #", "    #", "#### ",
        ],
        'Y' => [
            "#   #", "#   #", " # # ", "  #  ", "  #  ", "  #  ", "  #  ",
        ],
        'h' => [
            "#    ", "#    ", "# ## ", "##  #", "#   #", "#   #", "#   #",
        ],
        'o' => [
            "     ", "     ", " ### ", "#   #", "#   #", "#   #", " ### ",
        ],
        't' => [
            " #   ", " #   ", "#### ", " #   ", " #   ", " #  #", "  ## ",
        ],
        '\'' => [
            "  #  ", "  #  ", " #   ", "     ", "     ", "     ", "     ",
        ],
        'e' => [
            "     ", "     ", " ### ", "#   #", "#####", "#    ", " ### ",
        ],
        'm' => [
            "     ", "     ", "## # ", "# # #", "# # #", "# # #", "# # #",
        ],
        'u' => [
            "     ", "     ", "#   #", "#   #", "#   #", "#  ##", " ## #",
        ],
        'r' => [
            "     ", "     ", "# ## ", "##  #", "#    ", "#    ", "#    ",
        ],
        'c' => [
            "     ", "     ", " ####", "#    ", "#    ", "#    ", " ####",
        ],
        _ => ["     "; 7],
    }
}

/// Extruded block text, centred on the origin, `px` units a pixel.
pub fn text3d(r: &mut Renderer, s: &str, px: f32, depth: f32) {
    let n = s.chars().count() as f32;
    let width = n * 6.0 * px;
    r.cull = true;
    for (i, c) in s.chars().enumerate() {
        let g = glyph(c);
        for (row, line) in g.iter().enumerate() {
            for (col, b) in line.bytes().enumerate() {
                if b != b'#' {
                    continue;
                }
                let x = -width / 2.0 + (i as f32 * 6.0 + col as f32 + 0.5) * px;
                let y = (3.0 - row as f32) * px;
                r.push();
                r.translate(x, y, 0.0);
                gfx::boxx(r, px * 0.5, px * 0.5, depth * 0.5);
                r.pop();
            }
        }
    }
    r.cull = false;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Intro,
    Menu,
    HighScores,
    Instructions,
    Game,
    Score,
    NameEntry,
    Quit,
}

pub fn button(s: &mut Screen, input: &Input, x: i32, y: i32, label: &str, on: u8, off: u8) -> bool {
    let w = label.len() as i32 + 4;
    let hover = input.mx >= x && input.mx < x + w && input.my >= y && input.my < y + 3;
    let a = if hover { on } else { off };
    let edge = format!("+{}+", "-".repeat(label.len() + 2));
    s.put(x, y, a, &edge);
    s.put(x, y + 1, a, &format!("| {label} |"));
    s.put(x, y + 2, a, &edge);
    hover && input.clicked
}

pub fn intro(r: &mut Renderer, t: u32) {
    let tf = t as f32;
    r.load_identity();
    r.look_at(
        [tf / 100.0 - 15.0, 3.0 - tf / 500.0, 5.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    );
    r.light([-10.0, 10.0, 20.0, 1.0]);
    r.rotate(tf * 0.25, 0.0, 1.0, 0.0);
    r.color4(0.0, 0.0, 1.0 - ((tf - 1500.0) / 1500.0).abs(), 1.0);
    gfx::prims(r, crate::logo::LOGO);
}

pub fn menu(r: &mut Renderer, s: &mut Screen, input: &Input, t: u32, clock: u32) -> Option<State> {
    let w = s.w as i32;
    let h = s.h as i32;
    let cx = (w - 80) / 2;
    r.load_identity();
    r.light([-20.0, 20.0, 20.0, 1.0]);
    r.color3(0.0, 1.0, 1.0);
    let tc = clock as f32;
    let lift = if h >= 50 { 0.2 } else { 0.55 };
    r.translate(0.0, lift, -3.2);
    r.rotate((tc / 2000.0).sin() * 45.0, 0.0, 1.0, 0.0);
    r.rotate((tc / 1000.0).sin() * 5.0, 0.0, 0.0, 1.0);
    text3d(r, "Shoot 'em", 0.07, 0.4);

    let (by, vy, cy) = if h >= 50 {
        let o = (h - 50) / 2;
        (o + 36, o + 30, o + 44)
    } else {
        (h - 12, h - 14, h - 5)
    };
    let mut go = None;
    if button(s, input, cx + 8, by, "PLAY GAME", 0x20, 0x02) {
        go = Some(State::Game);
    }
    if button(s, input, cx + 25, by, "INSTRUCTIONS", 0x80, 0x07) {
        go = Some(State::Instructions);
    }
    if button(s, input, cx + 45, by, "HIGH SCORES", 0x80, 0x07) {
        go = Some(State::HighScores);
    }
    if button(s, input, cx + 64, by, "EXIT", 0x40, 0x04) {
        go = Some(State::Quit);
    }
    // the buttons slide in from the left, as they did
    for x in 0..w.max(80) {
        if (t as i32) - 200 < x * 10 {
            s.put(x, by, 7, " ");
            s.put(x, by + 2, 7, " ");
        }
        if (t as i32) - 200 < 790 - x * 10 {
            s.put(x, by + 1, 7, " ");
        }
    }
    s.put(cx + 34, vy, 9, "Version 1.0");
    s.put(cx + 23, cy, 1, "Copyright (C) 2005, Jan Ringos");
    s.put(
        cx + 11,
        cy + 1,
        1,
        "Terminal edition for Asciiverse, 2026, by his permission.",
    );
    s.put(cx + 6, cy + 3, 1, "Freeware. Music: Crush, Digital Rendezvous, Mecanum Overdrive remixes");
    s.put(
        cx + 6,
        cy + 4,
        1,
        "after Drozerix; Level 1 to 3 and Title Screen by Juhani Junkala (CC0).",
    );
    if input.key_esc {
        return Some(State::Quit);
    }
    if input.key_enter {
        return Some(State::Game);
    }
    go
}

const INSTRUCTIONS: &[(i32, i32, u8, &str)] = &[
    (25, 2, 0xf, "INSTRUCTIONS"),
    (25, 4, 7, "    Your goal is to destroy the falling objects"),
    (
        25,
        5,
        7,
        "before they reach the ground and to survive as long",
    ),
    (
        25,
        6,
        7,
        "as possible. The longer you survive, the more points",
    ),
    (
        25,
        7,
        7,
        "you will get. You will also get extra points for each",
    ),
    (25, 8, 7, "object destroyed."),
    (
        25,
        10,
        7,
        "    Use your mouse to aim and fire. Primary button",
    ),
    (
        25,
        11,
        7,
        "controls the Machine Guns, secondary is Mine Emitor.",
    ),
    (
        25,
        12,
        7,
        "Ammunition informations are shown in the aiming box",
    ),
    (25, 13, 7, "and in lower corners of the screen."),
    (
        25,
        14,
        8,
        "    Keys: arrows aim, Space fires, M drops a mine.",
    ),
    (25, 15, 8, "    S shows or hides the sound scope."),
    (25, 16, 0xf, "HEALTH"),
    (
        25,
        18,
        7,
        "    When an object manages to get to the bottom grid",
    ),
    (
        25,
        19,
        7,
        "it will explode decreasing your health. Bonuses and",
    ),
    (
        25,
        20,
        7,
        "ammo crates do not explode. Your health will increase",
    ),
    (25, 21, 7, "every time you destroy green diamond."),
    (25, 22, 7, "    Watch your health bellow score indicator."),
    (3, 24, 0xf, "OBJECTS: "),
    (7, 26, 0xf, "Box & Sphere"),
    (7, 28, 0xd, "Small Sphere"),
    (7, 30, 10, "Armor"),
    (7, 32, 0xb, "Bonus"),
    (7, 34, 0xc, "Mine"),
    (7, 36, 0xe, "Ammo"),
    (7, 38, 0xc, "Rapid Ammo"),
    (9, 38, 0xd, "pid Am"),
    (10, 38, 9, "id A"),
    (20, 26, 7, "- Easy to destroy (25 pts)."),
    (20, 27, 8, "    Fall cause light damage."),
    (20, 28, 7, "- Average difficult to destroy (50 pts)."),
    (20, 29, 8, "    Cause medium damage."),
    (20, 30, 7, "- Difficult to destroy (200 pts)."),
    (20, 31, 8, "    Cause average damage."),
    (20, 32, 7, "- Easy to destroy (2000 pts)."),
    (20, 33, 8, "    Cause no damage."),
    (
        20,
        34,
        7,
        "- Very easy to detonate (1000 pts). Cause heavy damage.",
    ),
    (
        20,
        35,
        8,
        "    Detonation destroys nearby objects and collects ammo.",
    ),
    (20, 36, 7, "- Easy to destroy (1000 pts). Cause no damage."),
    (20, 37, 8, "    Carries 80 bullets, may contain extra mine."),
    (20, 38, 7, "- Easy to destroy (1000 pts). Cause no damage."),
    (
        20,
        39,
        8,
        "    Carries 160 ultra bullets for rapid fire mode.",
    ),
    (3, 41, 0xf, "MORE INFO: "),
    (
        10,
        43,
        7,
        "Press Escape or click BACK to return to the menu.",
    ),
];

const ICONS: &[(i32, Kind, bool, bool)] = &[
    (26, Kind::Box, false, false),
    (28, Kind::Sphere, true, false),
    (30, Kind::Armor, false, false),
    (32, Kind::Bonus, false, false),
    (34, Kind::Mine, false, false),
    (36, Kind::Ammo, false, false),
    (38, Kind::Ammo, false, true),
];

/// Places a model so it appears centred on a text cell.
fn at_cell(r: &mut Renderer, w: i32, h: i32, col: f32, row: f32, aspect: f32, rows_tall: f32) {
    let d = 6.0;
    let x = (2.0 * (col + 0.5) / w as f32 - 1.0) * d * 0.5 * aspect;
    let y = (1.0 - 2.0 * (row + 0.5) / h as f32) * d * 0.5;
    r.load_identity();
    r.translate(x, y, -d);
    let k = rows_tall * d / h as f32 / 1.5;
    r.mult(&gfx::Mat4::scale(k, k, k));
}

pub fn instructions(
    r: &mut Renderer,
    s: &mut Screen,
    input: &Input,
    t: u32,
    scroll: &mut i32,
) -> Option<State> {
    let w = s.w as i32;
    let h = s.h as i32;
    let cx = (w - 80) / 2;
    let aspect = w as f32 / (2.0 * h as f32);
    let view = h - 4; // rows above the BACK button
    let max_scroll = (44 - view).max(0);
    if input.wheel != 0 || input.up || input.down {
        *scroll += input.wheel * 3 + if input.down { 3 } else { 0 } - if input.up { 3 } else { 0 };
    }
    *scroll = (*scroll).clamp(0, max_scroll);
    let oy = if h >= 50 { (h - 50) / 2 } else { -*scroll };
    for &(x, y, a, text) in INSTRUCTIONS {
        let yy = y + oy;
        if yy < view || h >= 50 {
            s.put(cx + x, yy, a, text);
        }
    }
    r.light([-20.0, 20.0, 20.0, 1.0]);
    for &(row, kind, small, rapid) in ICONS {
        let yy = row + oy;
        if yy < 0 || (yy >= view && h < 50) {
            continue;
        }
        at_cell(r, w, h, (cx + 4) as f32, yy as f32, aspect, 2.0);
        game::draw_icon(r, kind, small, rapid, t);
    }
    if max_scroll > 0 && h < 50 {
        s.put(cx + 2, h - 2, 8, "Arrows or the wheel scroll");
    }
    let (bx, by) = if h >= 50 {
        (cx + 68, oy + 45)
    } else {
        (cx + 68, h - 3)
    };
    if button(s, input, bx, by, "BACK", 0x20, 0x02) || input.key_esc || input.key_enter {
        return Some(State::Menu);
    }
    None
}

fn comment(score: u32) -> (&'static str, i32) {
    match score {
        0..=9_999 => ("You wanted to loose, don't you?", 25),
        10_000..=19_999 => ("This is your firts game, am I right?", 25),
        20_000..=29_999 => ("Trainer monkey is able to get more points.", 20),
        30_000..=39_999 => ("Try again looser!", 30),
        40_000..=49_999 => ("Your grandmother would be proud.", 25),
        50_000..=74_999 => ("I am sure you are able to get more points.", 20),
        75_000..=99_999 => ("You are closing to 100,000 pal.", 25),
        100_000..=124_999 => ("That was nice game.", 30),
        125_000..=149_999 => ("Really interesting how you ride those guns.", 20),
        150_000..=174_999 => ("Wow! Don't you want to join army?", 25),
        175_000..=199_999 => ("You get almost over 200,000! Maybe next time.", 20),
        200_000..=249_999 => ("You are kidding me? Weren't you betatester?", 20),
        250_000..=299_999 => ("You are simply one of the best.", 25),
        300_000..=399_999 => ("Only few are so good to be best of the best.", 20),
        _ => ("Unbelievable! How did you do that???", 20),
    }
}

fn blink_arrows(s: &mut Screen, cx: i32, y: i32, t: u32) {
    let i = ((t / 100) % 8) as i32;
    if i < 4 {
        s.put(cx + 35 - i, y, 7, " ");
        s.put(cx + 44 + i, y, 7, " ");
    }
}

pub fn score(
    r: &mut Renderer,
    s: &mut Screen,
    input: &Input,
    t: u32,
    g: &game::Game,
) -> Option<State> {
    let w = s.w as i32;
    let h = s.h as i32;
    let cx = (w - 80) / 2;
    let oy = if h >= 50 { (h - 50) / 2 } else { h - 2 - 42 };
    r.load_identity();
    r.light([-20.0, 20.0, 20.0, 1.0]);
    r.color3(0.0, 0.0, 1.0);
    r.translate(0.0, if h >= 50 { 0.6 } else { 1.22 }, -3.0);
    r.rotate((t as f32 / 2000.0).sin() * 20.0, 0.0, 1.0, 0.0);
    r.rotate((t as f32 / 1000.0).sin() * 5.0, 0.0, 0.0, 1.0);
    text3d(r, "Your Score", if h >= 50 { 0.07 } else { 0.055 }, 0.2);

    s.put(cx + 32, oy + 20, 0xf, &format!(">>>> {:06} <<<<", g.score));
    blink_arrows(s, cx, oy + 20, t);
    let (c, x) = comment(g.score);
    s.put(cx + x, oy + 22, 7, c);
    let st = &g.stats;
    let pct = |a: u32, b: u32| if b == 0 { 0 } else { a * 100 / b };
    s.put(cx + 33, oy + 26, 0xf, "Game Statistics");
    s.put(
        cx + 11,
        oy + 28,
        7,
        &format!(
            "Bullets hit/fired: {} / {}",
            st.bullets_hit, st.bullets_fired
        ),
    );
    s.put(
        cx + 11,
        oy + 29,
        8,
        &format!(
            "    Fire accuracy: {}%",
            pct(st.bullets_hit, st.bullets_fired)
        ),
    );
    s.put(
        cx + 10,
        oy + 31,
        7,
        &format!(
            "Mines hit/released: {} / {}",
            st.mines_hit, st.mines_released
        ),
    );
    s.put(
        cx + 11,
        oy + 32,
        8,
        &format!(
            "    Mine accuracy: {}%",
            pct(st.mines_hit, st.mines_released)
        ),
    );
    s.put(
        cx + 10,
        oy + 34,
        7,
        &format!("Time points gained: {}", st.time_points),
    );
    s.put(
        cx + 11,
        oy + 35,
        7,
        &format!("     Bonus points: {}", st.bonuses.0 * 2000),
    );
    s.put(
        cx + 11,
        oy + 37,
        7,
        &format!("  Armor collected: {}%", st.armor.0 * 5),
    );
    s.put(
        cx + 45,
        oy + 28,
        7,
        &format!("Objects destroyed: {} / {}", st.destroyed, st.spawned),
    );
    s.put(
        cx + 45,
        oy + 30,
        7,
        &format!("          Spheres: {} / {}", st.spheres.0, st.spheres.1),
    );
    s.put(
        cx + 45,
        oy + 31,
        7,
        &format!("            Boxes: {} / {}", st.boxes.0, st.boxes.1),
    );
    s.put(
        cx + 45,
        oy + 32,
        7,
        &format!("            Mines: {} / {}", st.mines.0, st.mines.1),
    );
    s.put(
        cx + 45,
        oy + 34,
        7,
        &format!("   Armor diamonds: {} / {}", st.armor.0, st.armor.1),
    );
    s.put(
        cx + 45,
        oy + 35,
        7,
        &format!("          Bonuses: {} / {}", st.bonuses.0, st.bonuses.1),
    );
    s.put(
        cx + 45,
        oy + 36,
        7,
        &format!("       Ammo packs: {} / {}", st.ammo.0, st.ammo.1),
    );
    if button(s, input, cx + 36, oy + 40, " OK ", 0x20, 0x02) || input.key_esc || input.key_enter {
        return Some(State::HighScores);
    }
    None
}

fn stars(r: &mut Renderer, w: i32, h: i32, row: i32, t: u32, left: i32, right: i32) {
    let aspect = w as f32 / (2.0 * h as f32);
    r.light([-20.0, 20.0, 20.0, 1.0]);
    for (col, sign) in [(left, 1.0), (right, -1.0)] {
        at_cell(r, w, h, col as f32, row as f32, aspect, 3.0);
        r.color3(1.0, 1.0, 0.0);
        r.rotate(sign * (t as f32 * 0.1 + 90.0), 0.0, 1.0, 0.0);
        r.cull = true;
        gfx::sphere(r, 0.75, 5, 2, true);
        r.cull = false;
    }
}

pub fn high_scores(
    r: &mut Renderer,
    s: &mut Screen,
    input: &Input,
    t: u32,
    table: &Table,
    top: &mut i32,
) -> Option<State> {
    let w = s.w as i32;
    let h = s.h as i32;
    let cx = (w - 80) / 2;
    let oy = if h >= 50 { (h - 50) / 2 } else { 0 };
    let list_rows = if h >= 50 { 36 } else { h - 10 };
    let shown = (list_rows / 3).max(1);
    stars(r, w, h, oy + 3, t, cx + 26, cx + 53);
    s.put(cx + 34, oy + 2, 0xf, "HALL OF FAME");
    s.put(cx + 32, oy + 4, 8, "The best players");
    let max_top = table.entries.len() as i32 - shown;
    if input.up || input.wheel < 0 {
        *top -= 1;
    }
    if input.down || input.wheel > 0 {
        *top += 1;
    }
    *top = (*top).clamp(0, max_top.max(0));
    for k in 0..shown {
        let i = (*top + k) as usize;
        let Some(e) = table.entries.get(i) else { break };
        let y = oy + 7 + k * 3;
        s.put(
            cx + 24,
            y,
            0xf,
            &format!("{:3}. {} [{}]", i + 1, e.nick, e.score),
        );
        s.put(cx + 31, y + 1, 7, &e.msg1);
        s.put(cx + 33, y + 2, 8, &e.msg2);
    }
    let by = if h >= 50 { oy + 45 } else { h - 3 };
    if button(s, input, cx + 68, oy + 20.min(by - 8), "^", 0x80, 0x07) {
        *top -= shown;
    }
    if button(s, input, cx + 68, oy + 24.min(by - 4), "v", 0x80, 0x07) {
        *top += shown;
    }
    if button(s, input, cx + 46, by, "BACK", 0x20, 0x02) || input.key_esc || input.key_enter {
        return Some(State::Menu);
    }
    None
}

pub struct NameEntry {
    pub fields: [String; 3],
    pub focus: usize,
}

impl NameEntry {
    pub fn new() -> NameEntry {
        NameEntry {
            fields: [String::new(), String::new(), String::new()],
            focus: 0,
        }
    }
}

pub fn name_entry(
    r: &mut Renderer,
    s: &mut Screen,
    input: &Input,
    t: u32,
    g: &game::Game,
    ne: &mut NameEntry,
) -> Option<State> {
    let w = s.w as i32;
    let h = s.h as i32;
    let cx = (w - 80) / 2;
    // the original's rows 14 to 47, or 20 rows when there are fewer
    let row = |orig: i32, compact: i32| -> i32 {
        if h >= 50 {
            (h - 50) / 2 + orig
        } else {
            (h - 21) / 2 + compact
        }
    };
    stars(r, w, h, row(14, 0), t, cx + 21, cx + 58);
    s.put(cx + 25, row(14, 0), 0xf, "!!! YOU ARE IN HALL OF FAME !!!");
    s.put(
        cx + 32,
        row(20, 2),
        0xf,
        &format!(">>>> {:06} <<<<", g.score),
    );
    blink_arrows(s, cx, row(20, 2), t);
    s.put(
        cx + 22,
        row(27, 4),
        7,
        "Add your nickname to High scores table.",
    );
    s.put(
        cx + 15,
        row(28, 5),
        7,
        "You can provide short message to all, if you want to.",
    );
    s.put(cx + 18, row(34, 8), 0xb, "Nickname:");
    s.put(cx + 28, row(33, 7), 0xb, "+-------------------+");
    s.put(cx + 28, row(34, 8), 0xb, "|                   |");
    s.put(cx + 28, row(35, 9), 0xb, "+-------------------+");
    s.put(cx + 18, row(38, 11), 0xb, " Message:");
    s.put(
        cx + 28,
        row(37, 10),
        0xb,
        "+-------------------------------------+",
    );
    s.put(
        cx + 28,
        row(38, 11),
        0xb,
        "|                                   < |",
    );
    s.put(
        cx + 28,
        row(39, 12),
        0xb,
        "| >                                   |",
    );
    s.put(
        cx + 28,
        row(40, 13),
        0xb,
        "+-------------------------------------+",
    );
    s.put(cx + 30, row(34, 8), 7, &ne.fields[0]);
    s.put(cx + 30, row(38, 11), 7, &ne.fields[1]);
    s.put(cx + 32, row(39, 12), 7, &ne.fields[2]);
    let caret = if (t / 200) & 1 == 0 { 0x80 } else { 0x07 };
    let (fx, fy) = match ne.focus {
        0 => (cx + 30 + ne.fields[0].len() as i32, row(34, 8)),
        1 => (cx + 30 + ne.fields[1].len() as i32, row(38, 11)),
        _ => (cx + 32 + ne.fields[2].len() as i32, row(39, 12)),
    };
    s.attr(fx, fy, caret);
    if input.key_enter && ne.focus == 2 {
        return Some(State::Score);
    }
    if input.key_tab || input.key_enter {
        ne.focus = (ne.focus + 1) % 3;
    }
    let limit = if ne.focus == 0 { 16 } else { 32 };
    for &c in &input.typed {
        if (' '..='~').contains(&c) && c != '%' && c != '|' && ne.fields[ne.focus].len() < limit {
            ne.fields[ne.focus].push(c);
        }
    }
    if input.key_backspace {
        ne.fields[ne.focus].pop();
    }
    if button(s, input, cx + 36, row(45, 15), " OK ", 0x20, 0x02) {
        return Some(State::Score);
    }
    if input.key_esc {
        ne.fields[0].clear();
        return Some(State::Score);
    }
    None
}
