//! The game proper: FUN_00402ba0 of the original, rule for rule. Times are
//! in milliseconds as there; REVERSE.md lists every number and where it
//! came from.

use crate::ascii::Screen;
use crate::audio::{sfx, Audio};
use crate::gfx::{self, Mat4, Renderer, V3};
use crate::Options;

/// The original's random numbers: uniform in [0, 1), and every
/// "ROUND(r * n)" it did ran with the FPU set to truncate.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15 | 1)
    }
    pub fn f(&mut self) -> f32 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        let v = self.0.wrapping_mul(0x2545_F491_4F6C_DD1D);
        (v >> 40) as f32 / (1u64 << 24) as f32
    }
    /// trunc(r * n): 0..n-1
    pub fn below(&mut self, n: u32) -> u32 {
        ((self.f() * n as f32) as u32).min(n.saturating_sub(1))
    }
    /// 1 - 2r, in (-1, 1]
    pub fn signed(&mut self) -> f32 {
        1.0 - 2.0 * self.f()
    }
}

/// A position with a direction and an up vector (the original's 0x30-byte
/// frames): guns, bullets, mines and particle fragments all use one.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub pos: V3,
    pub dir: V3,
    pub up: V3,
}

impl Frame {
    /// FUN_0040ae2e on dir and up: about x, in degrees.
    pub fn pitch(&mut self, deg: f32) {
        let (s, c) = deg.to_radians().sin_cos();
        for v in [&mut self.dir, &mut self.up] {
            let (y, z) = (v[1], v[2]);
            v[1] = y * c - s * z;
            v[2] = y * s + z * c;
        }
    }
    /// FUN_0040ae8c: about y.
    pub fn yaw(&mut self, deg: f32) {
        let (s, c) = deg.to_radians().sin_cos();
        for v in [&mut self.dir, &mut self.up] {
            let (x, z) = (v[0], v[2]);
            v[0] = x * c + s * z;
            v[2] = z * c - x * s;
        }
    }
    /// FUN_0040aeea: about z.
    pub fn roll(&mut self, deg: f32) {
        let (s, c) = deg.to_radians().sin_cos();
        for v in [&mut self.dir, &mut self.up] {
            let (x, y) = (v[0], v[1]);
            v[0] = x * c - s * y;
            v[1] = x * s + y * c;
        }
    }
    /// FUN_0040b20e: the frame as a matrix, -z along dir.
    pub fn matrix(&self) -> Mat4 {
        let d = gfx::norm(self.dir);
        let side = gfx::norm(gfx::cross(d, self.up));
        let up = gfx::norm(gfx::cross(side, d));
        Mat4([
            side[0],
            side[1],
            side[2],
            0.0,
            up[0],
            up[1],
            up[2],
            0.0,
            -d[0],
            -d[1],
            -d[2],
            0.0,
            self.pos[0],
            self.pos[1],
            self.pos[2],
            1.0,
        ])
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Sphere, // 0: a sphere, big and grey or small and magenta
    Box,    // 1
    Armor,  // 2: the green diamond
    Bonus,  // 3
    Mine,   // 4
    Ammo,   // 5: a crate, yellow, or blue with rapid ammunition
}

#[derive(Clone, Debug)]
pub struct Obj {
    pub x: f32,
    pub z: f32,
    pub size: f32,
    pub fall: f32,
    pub hp: f32,
    pub kind: Kind,
    pub rapid: bool,
}

impl Obj {
    pub fn pos(&self) -> V3 {
        [self.x, 10.0 - self.fall, self.z]
    }
    fn pan(&self) -> i32 {
        (self.x * 16.0 + 128.0) as i32
    }
    fn vol(&self) -> i32 {
        (self.z * 16.0 + 128.0) as i32
    }
    fn explodes_on_ground(&self) -> bool {
        matches!(
            self.kind,
            Kind::Sphere | Kind::Box | Kind::Armor | Kind::Mine
        )
    }
}

pub struct Particles {
    origin: V3,
    t: f32,
    color: V3,
    frags: Vec<(Frame, V3)>,
}

pub struct Blast {
    pos: V3,
    r: f32,
}

#[derive(Default, Clone, Debug)]
pub struct Stats {
    pub spawned: u32,
    pub spheres: (u32, u32),
    pub boxes: (u32, u32),
    pub mines: (u32, u32),
    pub armor: (u32, u32),
    pub bonuses: (u32, u32),
    pub ammo: (u32, u32),
    pub destroyed: u32,
    pub bullets_fired: u32,
    pub bullets_hit: u32,
    pub mines_released: u32,
    pub mines_hit: u32,
    pub time_points: u32,
}

pub const TIPS: [&str; 12] = [
    "Tip: Also your mines will explode if you hit them.",
    "Tip: The longer you survive the more points you'll get.",
    "Tip: Conserve your ammo in the beginning of the game.",
    "Tip: Later in game, focus to diamonds and bonuses only.",
    "Tip: Do not completely rely on aiming, better watch bullets.",
    "Tip: Shooting particles of waste is pointless.",
    "Tip: Try to catch the objects as soon as they appear.",
    "Tip: Collect ammo first before detonating nearby mines.",
    "Tip: Try to collect bonuses first before detonating mines.",
    "Tip: Do not let ultra ammo to be destroyed by mines.",
    "Tip: Ladies, switch to male narrator by \"m\" command line option.",
    "Tip: Visit http://Tringi.Mx-3.cz when you exit the game ;-)",
];

/// What the player does this frame, in the original's terms: the mouse
/// cell on an 80x50 console and the two buttons.
#[derive(Clone, Copy, Debug, Default)]
pub struct Controls {
    pub vx: f32,
    pub vy: f32,
    pub guns: bool,
    pub mine: bool,
}

pub struct Game {
    pub score: u32,
    pub health: f32,
    pub ammo: i32,
    pub rapid: i32,
    pub mines_left: i32,
    gun_cd: f32,
    mine_cd: f32,
    spawn_timer: f32,
    spawn_every: f32,
    pub tip: usize,
    pub objs: Vec<Obj>,
    bullets: Vec<Frame>,
    mines: Vec<Frame>,
    particles: Vec<Particles>,
    blasts: Vec<Blast>,
    pub stats: Stats,
    guns: [Frame; 2],
    aim: (f32, f32),
    pub over: bool,
    /// groove mode: what kills are worth, x1 to x5 with the combo
    pub groove_mult: u32,
    /// groove mode: spawns owed to the beat so far
    owed: f32,
}

impl Game {
    pub fn new(rng: &mut Rng) -> Game {
        Game {
            score: 0,
            health: 100.0,
            ammo: 200,
            rapid: 0,
            mines_left: 4,
            gun_cd: 0.0,
            mine_cd: 0.0,
            spawn_timer: 0.0,
            spawn_every: 1.0,
            tip: rng.below(TIPS.len() as u32) as usize,
            objs: Vec::new(),
            bullets: Vec::new(),
            mines: Vec::new(),
            particles: Vec::new(),
            blasts: Vec::new(),
            stats: Stats::default(),
            guns: [gun_base(0.5), gun_base(-0.5)],
            aim: (0.0, 1.1),
            over: false,
            groove_mult: 1,
            owed: 0.0,
        }
    }

    pub fn aim(&self) -> (f32, f32) {
        self.aim
    }

    /// Points for a kill or a catch, times the groove multiplier (x1 out of
    /// groove mode).
    fn points(&mut self, n: u32) {
        self.score += n * self.groove_mult.max(1);
    }

    fn voice(&self, opts: &Options, male: usize, female: usize) -> usize {
        if opts.male {
            male
        } else {
            female
        }
    }

    /// One frame: `dt` ms since the last one, `t` ms since the game began.
    /// In groove mode `beat` says what the music's beats did this frame,
    /// and objects fall in on them instead of on the timer.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        dt: u32,
        t: u32,
        c: Controls,
        opts: &Options,
        au: &Audio,
        rng: &mut Rng,
        beat: Option<crate::groove::BeatTick>,
    ) {
        let dtf = dt as f32;
        let vy = c.vy.clamp(0.0, 49.0);
        let vx = c.vx.clamp(0.0, 79.0);
        let aim_y = 2.5 - vy / 20.0;
        let aim_x = vx / 20.0 - 2.0;
        self.aim = (aim_x, aim_y);
        self.guns = [gun_base(0.5), gun_base(-0.5)];
        self.guns[0].yaw(aim_x * -29.0);
        self.guns[1].yaw(aim_x * -29.0);
        self.guns[0].pitch((aim_y - 1.1) * 28.0);
        self.guns[1].pitch((aim_y - 1.1) * 28.0);

        // bullets fly until they are 32 units out
        self.bullets.retain_mut(|b| {
            if gfx::len(b.pos) <= 32.0 {
                b.pos = gfx::add(b.pos, gfx::scale(b.dir, dtf * 0.02));
                true
            } else {
                false
            }
        });

        // the machine guns
        if self.gun_cd > 0.0 {
            self.gun_cd -= dtf * 0.001;
            if self.gun_cd < 0.0 {
                self.gun_cd = 0.0;
            }
        } else if c.guns {
            if self.ammo == 0 {
                au.play_centre(sfx::GUNS_EMPTY);
            } else {
                for g in self.guns {
                    let mut b = g;
                    b.pos = [
                        b.pos[0] + 2.0 * b.dir[0],
                        b.pos[1] + 2.0 * b.dir[1] + 2.0,
                        b.pos[2] + 2.0 * b.dir[2] + 8.0,
                    ];
                    self.bullets.push(b);
                }
                self.ammo -= 1;
                self.stats.bullets_fired += 2;
                if self.rapid > 0 {
                    self.rapid -= 1;
                }
                au.play(sfx::GUNS, (vx as i32) + 88, 255);
                if [40, 30, 20, 10].contains(&self.ammo) {
                    au.play_centre(sfx::AMMO_LOW);
                }
                if self.ammo == 0 {
                    au.play_centre(self.voice(opts, sfx::MALE_NO_AMMO, sfx::FEMALE_NO_AMMO));
                }
            }
            let rate = if self.rapid < 1 { 0.1 } else { 0.04 };
            self.gun_cd += rate - dtf * 0.001;
        }

        // mines drift slowly away
        self.mines.retain_mut(|m| {
            if gfx::len(m.pos) <= 32.0 {
                m.pos = gfx::add(m.pos, gfx::scale(m.dir, dtf * 0.005));
                true
            } else {
                false
            }
        });

        // the mine emitter
        if self.mine_cd > 0.0 {
            self.mine_cd -= dtf * 0.001;
        } else if c.mine {
            if self.mines_left == 0 {
                au.play_centre(sfx::MINES_EMPTY);
            } else {
                let mut m = self.guns[0];
                m.pos = [
                    m.dir[0],
                    m.pos[1] + m.dir[1] + 1.0,
                    m.pos[2] + m.dir[2] + 8.0,
                ];
                self.mines.push(m);
                self.mines_left -= 1;
                self.stats.mines_released += 1;
                au.play(sfx::MINE_LAUNCH, (vx as i32) + 88, 255);
                if self.mines_left == 0 {
                    au.play_centre(self.voice(opts, sfx::MALE_NO_MINES, sfx::FEMALE_NO_MINES));
                }
            }
            self.mine_cd = 2.0 - dtf * 0.001;
        }

        // the game speeds up as it goes on
        let speed = t as f32 / 800_000.0 + 0.5;
        self.spawn_every = (1.0 - t as f32 / 360_000.0).max(0.1);
        if (t % 100) as i64 - (dt as i64) < 1 {
            self.score += 1;
            self.stats.time_points += 1;
        }
        for o in &mut self.objs {
            let rate = if matches!(o.kind, Kind::Armor | Kind::Bonus) {
                0.002
            } else {
                0.001
            };
            o.fall += dtf * rate * speed;
        }

        self.blasts.retain_mut(|b| {
            b.r += dtf * 0.001;
            b.r < 5.0
        });
        for p in &mut self.particles {
            p.t += dtf * 0.001;
            for (f, v) in &mut p.frags {
                f.pos = gfx::add(f.pos, gfx::scale(*v, dtf * 0.001));
                f.pitch(dtf * 0.177_777_78);
                f.roll(dtf * 0.233_333_33);
                f.yaw(dtf * -0.111_111_11);
            }
        }
        self.particles.retain(|p| p.t < 3.0);

        // something new falls: on the timer, or in groove mode on the beat,
        // as many as the timer would have sent over that beat, and a wave
        // more when a loud bar begins
        match beat {
            None => {
                if self.spawn_timer <= 0.0 {
                    self.spawn(rng);
                    self.spawn_timer = self.spawn_every;
                } else {
                    self.spawn_timer -= dtf * 0.001;
                }
            }
            Some(b) => {
                if b.beats > 0 {
                    self.owed += b.beats as f32 * b.period as f32 / self.spawn_every;
                    let mut n = self.owed.floor() as u32;
                    self.owed -= n as f32;
                    if b.downbeat {
                        if b.energy > 1.35 {
                            n += 2;
                        } else if b.energy > 1.15 {
                            n += 1;
                        }
                    }
                    for _ in 0..n.min(6) {
                        self.spawn(rng);
                    }
                }
            }
        }

        self.ground(opts, au, rng);
        self.shoot_down(dtf, opts, au, rng);
        self.mines_hit_objects(au, rng);
        self.blasts_hit_objects(opts, au, rng);

        if self.health <= 0.0 && !self.over {
            self.over = true;
            au.play_centre(sfx::GAME_OVER);
        }
    }

    fn spawn(&mut self, rng: &mut Rng) {
        let x = rng.below(9) as f32 - 4.0;
        let z = rng.below(9) as f32 - 4.0;
        loop {
            let (kind, size, hp, rapid) = match rng.below(6) {
                0 => {
                    if rng.below(5) != 0 {
                        (Kind::Sphere, 0.75, 2.0, false)
                    } else {
                        (Kind::Sphere, 0.5, 4.0, false)
                    }
                }
                1 => (Kind::Box, 0.5, 2.0, false),
                2 => (Kind::Armor, 0.75, 6.0, false),
                3 => (Kind::Bonus, 0.5, 3.0, false),
                4 => {
                    if rng.below(8) != 0 {
                        continue;
                    }
                    (Kind::Mine, 0.75, 1.0, false)
                }
                _ => {
                    if rng.below(3) != 0 {
                        continue;
                    }
                    (Kind::Ammo, 0.75, 2.0, rng.below(6) == 0)
                }
            };
            match kind {
                Kind::Sphere => self.stats.spheres.1 += 1,
                Kind::Box => self.stats.boxes.1 += 1,
                Kind::Armor => self.stats.armor.1 += 1,
                Kind::Bonus => self.stats.bonuses.1 += 1,
                Kind::Mine => self.stats.mines.1 += 1,
                Kind::Ammo => self.stats.ammo.1 += 1,
            }
            self.stats.spawned += 1;
            self.objs.push(Obj {
                x,
                z,
                size,
                fall: 0.0,
                hp,
                kind,
                rapid,
            });
            return;
        }
    }

    fn burst(&mut self, at: V3, color: V3, count: usize, opts: &Options, rng: &mut Rng) {
        if opts.clear {
            return;
        }
        let n = if opts.more_particles {
            count * 2
        } else {
            count
        };
        let mut frags = Vec::with_capacity(n);
        for _ in 0..n {
            let dir = [rng.signed(), rng.signed(), rng.signed()];
            let vel = [rng.signed(), rng.signed(), rng.signed()];
            let f = Frame {
                pos: gfx::scale(vel, 0.5),
                dir,
                up: [0.0, 1.0, 0.0],
            };
            frags.push((f, vel));
        }
        self.particles.push(Particles {
            origin: at,
            t: 0.0,
            color,
            frags,
        });
    }

    fn colour_of(o: &Obj) -> V3 {
        match o.kind {
            Kind::Sphere if o.size < 0.6 => [1.0, 0.0, 1.0],
            Kind::Sphere | Kind::Box => [1.0, 1.0, 1.0],
            Kind::Armor => [0.0, 1.0, 0.0],
            Kind::Bonus => [0.0, 1.0, 1.0],
            Kind::Mine => [1.0, 0.0, 0.0],
            Kind::Ammo if o.rapid => [0.0, 0.0, 1.0],
            Kind::Ammo => [1.0, 1.0, 0.0],
        }
    }

    /// Objects that reach the grid: the dangerous ones explode and cost
    /// health; bonuses and crates fall on through and are gone.
    fn ground(&mut self, opts: &Options, au: &Audio, rng: &mut Rng) {
        let mut i = self.objs.len();
        while i > 0 {
            i -= 1;
            let o = self.objs[i].clone();
            if o.explodes_on_ground() {
                if o.fall <= 10.0 {
                    continue;
                }
                let n = match o.kind {
                    Kind::Sphere => 16,
                    Kind::Armor => 6,
                    _ => 8,
                };
                self.burst(o.pos(), Self::colour_of(&o), n, opts, rng);
                au.play(sfx::BLAST + rng.below(4) as usize, o.pan(), o.vol());
                let before = self.health;
                self.health -= 2.0 * o.size * o.hp;
                if o.kind == Kind::Mine {
                    self.health -= 2.0;
                }
                if before >= 50.0 && self.health < 50.0 {
                    au.play_centre(self.voice(opts, sfx::MALE_ARMOR_50, sfx::FEMALE_ARMOR_50));
                }
                if before >= 25.0 && self.health < 25.0 {
                    au.play_centre(self.voice(opts, sfx::MALE_ARMOR_25, sfx::FEMALE_ARMOR_25));
                }
                self.objs.remove(i);
            } else if o.fall > 15.0 {
                self.objs.remove(i);
            }
        }
    }

    /// Bullets against falling objects, then against flying mines.
    fn shoot_down(&mut self, dtf: f32, opts: &Options, au: &Audio, rng: &mut Rng) {
        let mut bi = self.bullets.len();
        while bi > 0 {
            bi -= 1;
            let b = self.bullets[bi];
            // the stretch the bullet covered and half as much again, as the
            // original tested it
            let prev = gfx::sub(b.pos, gfx::scale(b.dir, dtf * 0.03));
            let mut hit = false;
            let mut oi = self.objs.len();
            while oi > 0 {
                oi -= 1;
                let o = self.objs[oi].clone();
                if !segment_hits(prev, b.pos, o.pos(), 1.2 * o.size) {
                    continue;
                }
                hit = true;
                self.objs[oi].hp -= 1.0;
                if self.objs[oi].hp > 0.0 {
                    au.play(sfx::HIT + rng.below(5) as usize, o.pan(), o.vol());
                } else {
                    self.destroy(oi, opts, au, rng);
                }
                break;
            }
            if hit {
                self.bullets.remove(bi);
                self.stats.bullets_hit += 1;
                continue;
            }
            let mut mi = self.mines.len();
            while mi > 0 {
                mi -= 1;
                let m = self.mines[mi];
                if segment_hits(prev, b.pos, m.pos, 0.75) {
                    self.blasts.push(Blast { pos: m.pos, r: 4.0 });
                    self.points(1000);
                    au.play(
                        sfx::BLAST + rng.below(4) as usize,
                        (m.pos[0] * 16.0 + 128.0) as i32,
                        255,
                    );
                    self.mines.remove(mi);
                    self.bullets.remove(bi);
                    self.stats.bullets_hit += 1;
                    break;
                }
            }
        }
    }

    /// A falling object shot down by the guns: points, pickups, debris.
    fn destroy(&mut self, oi: usize, opts: &Options, au: &Audio, rng: &mut Rng) {
        let o = self.objs[oi].clone();
        let at = o.pos();
        let col = Self::colour_of(&o);
        match o.kind {
            Kind::Sphere => {
                self.points(if o.size < 0.6 { 50 } else { 25 });
                self.stats.spheres.0 += 1;
                au.play(sfx::BLAST + rng.below(4) as usize, o.pan(), o.vol());
                self.burst(at, col, 16, opts, rng);
            }
            Kind::Box => {
                self.points(25);
                self.stats.boxes.0 += 1;
                au.play(sfx::BLAST + rng.below(4) as usize, o.pan(), o.vol());
                self.burst(at, col, 8, opts, rng);
            }
            Kind::Armor => {
                au.play(sfx::PICKUP, 128, 255);
                self.health = (self.health + 5.0).min(100.0);
                self.points(200);
                self.stats.armor.0 += 1;
                self.burst(at, col, 6, opts, rng);
            }
            Kind::Bonus => {
                au.play(sfx::PICKUP, 128, 255);
                self.points(2000);
                self.stats.bonuses.0 += 1;
                self.burst(at, col, 4, opts, rng);
            }
            Kind::Mine => {
                self.blasts.push(Blast { pos: at, r: 4.0 });
                au.play(sfx::BLAST + rng.below(4) as usize, o.pan(), 255);
                self.points(1000);
                self.stats.mines.0 += 1;
            }
            Kind::Ammo => {
                au.play(sfx::AMMO_PICKUP, o.pan(), o.vol());
                if o.rapid {
                    self.rapid += 80;
                    self.ammo += 80;
                } else {
                    self.ammo += 40;
                }
                self.stats.ammo.0 += 1;
                if rng.f() >= 0.5 {
                    self.mines_left += 1;
                }
                self.points(100);
                self.burst(at, col, 16, opts, rng);
            }
        }
        self.stats.destroyed += 1;
        self.objs.remove(oi);
    }

    /// A flying mine touching a falling object goes off.
    fn mines_hit_objects(&mut self, au: &Audio, rng: &mut Rng) {
        let mut mi = self.mines.len();
        while mi > 0 {
            mi -= 1;
            let m = self.mines[mi];
            if self
                .objs
                .iter()
                .any(|o| gfx::dist(m.pos, o.pos()) <= 0.75 + o.size)
            {
                self.blasts.push(Blast { pos: m.pos, r: 4.0 });
                self.points(1000);
                self.stats.mines_hit += 1;
                au.play(
                    sfx::BLAST + rng.below(4) as usize,
                    (m.pos[0] * 16.0 + 128.0) as i32,
                    255,
                );
                self.mines.remove(mi);
            }
        }
    }

    /// For their first tenth of a second blasts take everything in reach,
    /// collecting armor and (halved) ammunition but no points.
    fn blasts_hit_objects(&mut self, opts: &Options, au: &Audio, rng: &mut Rng) {
        for bi in (0..self.blasts.len()).rev() {
            let (bp, br) = (self.blasts[bi].pos, self.blasts[bi].r);
            if br > 4.1 {
                continue;
            }
            let mut oi = self.objs.len();
            while oi > 0 {
                oi -= 1;
                let o = self.objs[oi].clone();
                if gfx::dist(bp, o.pos()) > br + o.size {
                    continue;
                }
                self.burst(o.pos(), Self::colour_of(&o), 12, opts, rng);
                match o.kind {
                    Kind::Sphere => self.stats.spheres.0 += 1,
                    Kind::Box => self.stats.boxes.0 += 1,
                    Kind::Armor => {
                        self.health = (self.health + 5.0).min(100.0);
                        self.stats.armor.0 += 1;
                    }
                    Kind::Mine => self.stats.mines.0 += 1,
                    Kind::Ammo => {
                        self.ammo += 25;
                        self.stats.ammo.0 += 1;
                    }
                    Kind::Bonus => {}
                }
                au.play(sfx::BLAST + rng.below(4) as usize, o.pan(), o.vol());
                self.stats.destroyed += 1;
                self.objs.remove(oi);
            }
        }
    }

    /// The scene, as the original drew it after the update.
    pub fn draw(&self, r: &mut Renderer, t: u32) {
        let (ax, ay) = self.aim;
        r.lighting = true;
        r.load_identity();
        r.look_at([0.0, 2.0, 8.0], [ax, ay + 1.0, 0.0], [0.0, 1.0, 0.0]);
        r.light([-10.0, 20.0, 20.0, 1.0]);
        draw_ground(r);

        // bullets: yellow, red while the rapid ammunition lasts
        r.color3(1.0, if self.rapid == 0 { 1.0 } else { 0.0 }, 0.0);
        for b in &self.bullets {
            r.push();
            r.mult(&b.matrix());
            draw_bullet(r);
            r.pop();
        }
        r.color3(1.0, 0.0, 0.0);
        for m in &self.mines {
            r.push();
            r.mult(&m.matrix());
            r.rotate(t as f32 * 0.1, 1.0, 0.0, 1.0);
            draw_mine(r, 0.75);
            r.pop();
        }

        r.blend = true;
        for p in &self.particles {
            r.push();
            r.translate(p.origin[0], p.origin[1], p.origin[2]);
            r.color4(p.color[0], p.color[1], p.color[2], 1.0 - p.t / 3.0);
            for (f, _) in &p.frags {
                r.push();
                r.mult(&f.matrix());
                let a = [-0.2, -0.2, 0.0];
                let b = [0.2, -0.2, 0.0];
                let c = [0.0, 0.3, 0.0];
                r.tri_flat([a, b, c], [0.0, 0.0, -1.0]);
                r.tri_flat([a, c, b], [0.0, 0.0, 1.0]);
                r.pop();
            }
            r.pop();
        }
        r.blend = false;

        // the two guns, in eye space, recoiling while they cool down
        for (i, g) in self.guns.iter().enumerate() {
            r.load_identity();
            r.mult(&g.matrix());
            if self.gun_cd > 0.0 {
                r.translate(0.0, 0.0, self.gun_cd * 0.5);
            }
            let s = if i == 0 { 1.0 } else { -1.0 };
            r.rotate(12.0 * s, 0.0, 1.0, 0.0);
            r.translate(-0.1 * s, 0.0, 0.0);
            r.rotate(8.0, 1.0, 0.0, 0.0);
            draw_gun(r);
        }

        r.load_identity();
        r.look_at([0.0, 2.0, 8.0], [ax, ay + 1.0, 0.0], [0.0, 1.0, 0.0]);
        for o in self.objs.iter().filter(|o| o.kind != Kind::Ammo) {
            r.push();
            r.translate(o.x, 10.0 - o.fall, o.z);
            r.color3(0.6, 0.6, 0.6);
            match o.kind {
                Kind::Sphere => {
                    if o.size < 0.6 {
                        r.color3(1.0, 0.0, 1.0);
                    }
                    r.cull = true;
                    gfx::sphere(r, o.size, 8, 8, false);
                    r.cull = false;
                }
                Kind::Box => {
                    r.rotate(36.0 * o.fall, 1.0, 1.0, 0.0);
                    r.rotate(t as f32 * 0.01, 0.0, 0.0, 1.0);
                    r.cull = true;
                    gfx::boxx(r, o.size, o.size, o.size);
                    r.cull = false;
                }
                Kind::Armor => {
                    r.color3(0.0, 1.0, 0.0);
                    r.rotate(t as f32 * 0.1, 0.0, 1.0, 0.0);
                    r.cull = true;
                    gfx::sphere(r, o.size, 4, 3, true);
                    r.cull = false;
                }
                Kind::Bonus => {
                    r.color3(0.0, 1.0, 1.0);
                    r.rotate(180.0 * o.fall, 1.0, 1.0, 0.0);
                    r.rotate(t as f32 * 0.1, 0.0, 0.0, 1.0);
                    r.cull = true;
                    gfx::sphere(r, o.size, 3, 2, true);
                    r.cull = false;
                }
                Kind::Mine => {
                    r.color3(1.0, 0.0, 0.0);
                    r.rotate(360.0 * o.fall, 1.0, 1.0, 0.0);
                    r.rotate(t as f32 * 0.2, 0.0, 0.0, 1.0);
                    draw_mine(r, o.size);
                }
                Kind::Ammo => {}
            }
            r.pop();
        }
        for o in self.objs.iter().filter(|o| o.kind == Kind::Ammo) {
            r.push();
            r.translate(o.x, 10.0 - o.fall, o.z);
            if o.rapid {
                r.color4(0.0, 0.0, 1.0, 1.0);
            } else {
                r.color4(1.0, 1.0, 0.0, 1.0);
            }
            r.rotate(o.fall * 180.0, 1.0, 1.0, 0.0);
            r.rotate(t as f32 * 0.1, 0.0, 0.0, 1.0);
            let s = o.size / 3.0;
            r.cull = true;
            gfx::boxx(r, s, s, s);
            r.color4(1.0, if o.rapid { 0.0 } else { 1.0 }, 0.0, 0.2);
            r.blend = true;
            gfx::sphere(r, o.size, 8, 8, false);
            r.blend = false;
            r.cull = false;
            r.pop();
        }
        r.blend = true;
        for b in &self.blasts {
            r.push();
            r.translate(b.pos[0], b.pos[1], b.pos[2]);
            r.color4(0.5, 0.0, 0.0, (5.0 - b.r).min(1.0));
            r.cull = true;
            gfx::sphere(r, b.r, 24, 24, false);
            r.cull = false;
            r.pop();
        }
        r.blend = false;
    }

    /// The text layer: the aiming box round the mouse, the compass, the
    /// score and health, ammunition and mines, and the start messages.
    pub fn hud(&self, s: &mut Screen, mx: i32, my: i32, vx: f32, vy: f32, t: u32) {
        let w = s.w as i32;
        let h = s.h as i32;
        let cx = (w - 80) / 2; // the original's 80 columns, centred
        let by = h - 50; // its bottom rows, kept at the bottom
        let vxi = vx as i32;
        let vyi = vy as i32;

        // the compass along the top
        for i in 0..21 {
            let mut ch = "-";
            if (vxi / 2 + 2 + i) & 3 == 0 {
                ch = "|";
                if (vxi / 2 + 2 + i) & 15 == 0 {
                    if vxi / 2 + i == 14 {
                        s.put(cx + i + 29, 4, 0xb, "-8");
                    }
                    if vxi / 2 + i == 30 {
                        s.put(cx + i + 30, 4, 0xb, "0");
                    }
                    if vxi / 2 + i == 46 {
                        s.put(cx + i + 29, 4, 0xb, "+8");
                    }
                }
            }
            s.put(cx + i + 30, 3, 0xb, ch);
        }

        // the aiming box
        s.put(mx - 8, my - 4, 0xb, "64 ");
        s.put(mx - 8, my + 4, 0xb, " 4 ");
        for i in 0..9 {
            let mut ch = "|";
            if (vyi + i) % 3 == 0 {
                ch = "+-";
            }
            if (vyi + i) % 12 == 0 {
                ch = "+--";
            }
            if i != 8 && i != 0 && (vyi + i) % 6 == 0 {
                let col = if (vyi + i) % 12 == 0 { 0xb } else { 3 };
                s.put(mx - 8, my - 4 + i, col, &format!("{:02} ", 60 - (vyi + i)));
            }
            s.put(mx - 5, my - 4 + i, 0xb, ch);
        }
        let bars = (self.ammo / 10).min(27);
        for i in 0..bars {
            let col = if i < 4 {
                0xc
            } else if i < 9 {
                0xe
            } else {
                0xa
            };
            s.put(mx + 5 + i / 9, my - i % 9 + 4, col, "#");
        }
        s.put(mx + 9, my - 4, 0xb, &format!("{}", self.mines_left));
        if self.rapid == 0 {
            s.put(mx + 9, my + 4, 0xb, &format!("{:03}", self.ammo * 2));
        } else {
            s.put(
                mx + 9,
                my + 4,
                0xb,
                &format!("{:03}", (self.ammo - self.rapid) * 2),
            );
            s.put(mx + 9, my + 5, 0xc, &format!("{:03}", self.rapid * 2));
        }

        // !!! GET READY !!! and a tip, at the start
        let mid = (h - 50) / 2;
        if (1001..3000).contains(&t) {
            let a = if (1251..2751).contains(&t) {
                if (1501..2501).contains(&t) {
                    0xf
                } else {
                    7
                }
            } else {
                8
            };
            s.put(cx + 32, mid + 12, a, "!!! GET READY !!!");
        }
        if (1001..4000).contains(&t) {
            let a = if (1251..3751).contains(&t) {
                if (1501..3501).contains(&t) {
                    0xf
                } else {
                    7
                }
            } else {
                8
            };
            let tip = TIPS[self.tip];
            s.put(cx + 40 - tip.len() as i32 / 2, mid + 14, a, tip);
        }

        // the score box with health, flashing when health is low
        let frame = if self.health <= 25.0 {
            if (t / 250) & 1 == 0 {
                0xc
            } else {
                0xe
            }
        } else {
            0xb
        };
        s.put(cx + 35, by + 44, frame, "+--------+");
        s.put(cx + 29, by + 46, frame, "+-----+--------+-----+");
        s.put(cx + 29, by + 48, frame, "+--------------------+");
        s.put(cx + 29, by + 47, frame, "|");
        s.put(cx + 50, by + 47, frame, "|");
        s.put(cx + 35, by + 45, frame, "|");
        s.put(cx + 44, by + 45, frame, "|");
        s.put(cx + 36, by + 45, 9, &format!(" {:06} ", self.score));
        for i in 0..20 {
            let ch = if self.health <= (i * 5) as f32 {
                " "
            } else {
                "|"
            };
            let col = if self.health > 50.0 {
                10
            } else if self.health > 25.0 {
                0xe
            } else {
                0xc
            };
            s.put(cx + 30 + i, by + 47, col, ch);
        }

        // ammunition, right
        s.put(cx + 61, by + 44, frame, "   +------------+");
        s.put(cx + 61, by + 45, frame, "   | Ammunition |");
        s.put(cx + 61, by + 46, frame, "   |            |");
        s.put(cx + 61, by + 47, frame, "   |            |");
        s.put(cx + 61, by + 48, frame, "   +------------+");
        if self.rapid == 0 {
            s.put(cx + 66, by + 46, 9, &format!("   {:04}   ", self.ammo * 2));
        } else {
            s.put(
                cx + 66,
                by + 46,
                9,
                &format!("{:04}", (self.ammo - self.rapid) * 2),
            );
            s.put(cx + 72, by + 46, 0xc, &format!("{:04}", self.rapid * 2));
        }
        for i in 0..10 {
            let f = if self.rapid == 0 { 100.0 } else { 250.0 };
            let (ch, col) = if (i as f32) / f < self.gun_cd {
                (" ", 9)
            } else {
                ("|", if self.rapid == 0 { 9 } else { 0xc })
            };
            s.put(cx + 66 + i, by + 47, col, ch);
        }
        let stack = (self.ammo - 1).rem_euclid(10) + 1;
        if self.ammo > 0 {
            for i in 0..stack {
                s.put(cx + 62 - i / 5, by + 48 - i % 5, frame, "#");
            }
        }

        // mines, left
        s.put(cx + 2, by + 44, frame, "+-------+");
        s.put(cx + 2, by + 45, frame, "| Mines |");
        s.put(cx + 2, by + 46, frame, "|       |");
        s.put(cx + 2, by + 47, frame, "|       |");
        s.put(cx + 2, by + 48, frame, "+-------+");
        s.put(cx + 4, by + 46, 9, &format!(" {:2}", self.mines_left));
        for i in 0..5 {
            let ch = if (4 - i) as f32 <= self.mine_cd * 2.5 {
                " "
            } else {
                "|"
            };
            s.put(cx + 4 + i, by + 47, 9, ch);
        }
    }

    /// --demo: the computer aims and fires like a fair player would.
    pub fn autopilot(&self, t: u32) -> Controls {
        let gun = [0.0, 1.7, 7.5];
        let target = self
            .objs
            .iter()
            .filter(|o| o.fall > 0.3 && o.fall < 9.6)
            .max_by(|a, b| {
                let want = |o: &Obj| {
                    o.fall
                        + match o.kind {
                            Kind::Bonus | Kind::Armor => 3.0,
                            Kind::Ammo if self.ammo < 60 => 6.0,
                            _ => 0.0,
                        }
                };
                want(a).partial_cmp(&want(b)).unwrap()
            });
        let Some(o) = target else {
            let wander = (t as f32 / 1500.0).sin();
            return Controls {
                vx: 40.0 + wander * 10.0,
                vy: 22.0,
                guns: false,
                mine: false,
            };
        };
        let mut p = o.pos();
        let d0 = gfx::dist(p, gun);
        let fall_speed = if matches!(o.kind, Kind::Armor | Kind::Bonus) {
            2.0
        } else {
            1.0
        } * (t as f32 / 800_000.0 + 0.5);
        p[1] -= fall_speed * (d0 / 20.0);
        let d = gfx::norm(gfx::sub(p, gun));
        let a = (-d[0]).asin().to_degrees();
        let pitch = (d[1] / a.to_radians().cos())
            .clamp(-1.0, 1.0)
            .asin()
            .to_degrees();
        let aim_x = a / -29.0;
        let aim_y = pitch / 28.0 + 1.1;
        let vx = ((aim_x + 2.0) * 20.0).clamp(0.0, 79.0);
        let vy = ((2.5 - aim_y) * 20.0).clamp(0.0, 49.0);
        let close = (vx - (self.aim.0 + 2.0) * 20.0).abs() < 3.0
            && (vy - (2.5 - self.aim.1) * 20.0).abs() < 3.0;
        let crowd = self
            .objs
            .iter()
            .filter(|q| gfx::dist(q.pos(), o.pos()) < 3.5)
            .count();
        // short bursts: no more bullets in the air than the target needs
        let enough = self.bullets.len() as f32 >= 2.0 * o.hp + 2.0;
        Controls {
            vx,
            vy,
            guns: close && !enough,
            mine: close && crowd >= 3 && self.mines_left > 1,
        }
    }
}

fn gun_base(x: f32) -> Frame {
    Frame {
        pos: [x, -0.3, -0.5],
        dir: [0.0, 0.0, -1.0],
        up: [0.0, 1.0, 0.0],
    }
}

/// FUN_0040cfee: does the segment a-b pass within `r` of `c`?
pub fn segment_hits(a: V3, b: V3, c: V3, r: f32) -> bool {
    let ab = gfx::sub(b, a);
    let l = gfx::len(ab);
    let closest = if l == 0.0 {
        a
    } else {
        let u = gfx::scale(ab, 1.0 / l);
        let t = gfx::dot(gfx::sub(c, a), u);
        if t <= 0.0 {
            a
        } else if t >= l {
            b
        } else {
            gfx::add(a, gfx::scale(u, t))
        }
    };
    gfx::dist(closest, c) <= r
}

/// FUN_0040a340: the dark yellow grid the objects must not reach, and the
/// red ring round it.
pub fn draw_ground(r: &mut Renderer) {
    r.color3(0.4, 0.4, 0.0);
    let n = [0.0, 1.0, 0.0];
    for i in -4..5 {
        let a = 0.25 + i as f32;
        let b = i as f32 - 0.25;
        // the original's grid lines are strips half a unit wide
        r.quad(
            [[a, 0.0, -4.0], [a, 0.0, 4.0], [b, 0.0, 4.0], [b, 0.0, -4.0]],
            n,
        );
        r.quad(
            [[-4.0, 0.0, a], [4.0, 0.0, a], [4.0, 0.0, b], [-4.0, 0.0, b]],
            n,
        );
    }
    r.color3(1.0, 0.0, 0.0);
    r.normal = n;
    let pts: Vec<V3> = (0..36)
        .map(|i| {
            let a = i as f32 * std::f32::consts::PI / 18.0;
            [6.0 * a.sin(), 0.0, 6.0 * a.cos()]
        })
        .collect();
    r.line_loop(&pts);
}

/// FUN_0040a91a: a bullet, a thin double pyramid along -z.
fn draw_bullet(r: &mut Renderer) {
    let q = 0.08;
    let tip = [0.0, 0.0, -0.4];
    let tail = [0.0, 0.0, 1.0];
    let ring = [[q, q, 0.0], [q, -q, 0.0], [-q, -q, 0.0], [-q, q, 0.0]];
    for i in 0..4 {
        let a = ring[i];
        let b = ring[(i + 1) % 4];
        let n1 = gfx::norm(gfx::cross(gfx::sub(a, tip), gfx::sub(b, tip)));
        r.tri_flat([tip, b, a], gfx::scale(n1, -1.0));
        r.tri_flat(
            [tail, a, b],
            gfx::norm(gfx::cross(gfx::sub(a, tail), gfx::sub(b, tail))),
        );
    }
}

/// FUN_0040a75a: a mine, a ball inside a band.
fn draw_mine(r: &mut Renderer, s: f32) {
    r.cull = true;
    gfx::sphere(r, s * 0.8, 8, 8, false);
    r.cull = false;
    r.push();
    r.translate(0.0, 0.0, s * -0.5 + s * 0.25);
    gfx::cylinder(r, s, s, s * 0.5, 16);
    r.pop();
}

/// FUN_0040a7ec: one machine gun, built of boxes.
fn draw_gun(r: &mut Renderer) {
    r.color3(0.15, 0.15, 0.15);
    gfx::boxx(r, 0.1, 0.04, 0.25);
    r.color3(1.0, 1.0, 0.0);
    gfx::boxx(r, 0.12, 0.01, 0.2);
    r.color3(0.0, 0.0, 1.0);
    gfx::boxx(r, 0.05, 0.08, 0.15);
    gfx::boxx(r, 0.01, 0.08, 0.25);
    r.push();
    r.translate(0.0, 0.0, -0.25);
    r.color3(0.3, 0.3, 0.3);
    gfx::boxx(r, 0.025, 0.025, 0.25);
    r.color3(1.0, 1.0, 1.0);
    gfx::boxx(r, 0.05, 0.06, 0.05);
    r.push();
    r.translate(0.05, 0.0, 0.0);
    gfx::boxx(r, 0.005, 0.005, 0.2);
    r.translate(-0.1, 0.0, 0.0);
    gfx::boxx(r, 0.005, 0.005, 0.2);
    r.pop();
    r.pop();
}

/// Models for the instructions page.
pub fn draw_icon(r: &mut Renderer, kind: Kind, small: bool, rapid: bool, t: u32) {
    r.rotate(t as f32 * 0.1, 0.0, 1.0, 0.0);
    r.cull = true;
    match kind {
        Kind::Sphere => {
            if small {
                r.color3(1.0, 0.0, 1.0);
                gfx::sphere(r, 0.5, 8, 8, false);
            } else {
                r.color3(0.6, 0.6, 0.6);
                gfx::sphere(r, 0.75, 8, 8, false);
            }
        }
        Kind::Box => {
            r.color3(0.6, 0.6, 0.6);
            r.rotate(30.0, 1.0, 1.0, 0.0);
            gfx::boxx(r, 0.5, 0.5, 0.5);
        }
        Kind::Armor => {
            r.color3(0.0, 1.0, 0.0);
            gfx::sphere(r, 0.75, 4, 3, true);
        }
        Kind::Bonus => {
            r.color3(0.0, 1.0, 1.0);
            r.rotate(90.0, 1.0, 0.0, 0.0);
            gfx::sphere(r, 0.6, 3, 2, true);
        }
        Kind::Mine => {
            r.color3(1.0, 0.0, 0.0);
            r.rotate(60.0, 1.0, 0.0, 0.0);
            r.cull = false;
            draw_mine(r, 0.75);
        }
        Kind::Ammo => {
            if rapid {
                r.color3(0.0, 0.0, 1.0);
            } else {
                r.color3(1.0, 1.0, 0.0);
            }
            r.rotate(30.0, 1.0, 1.0, 0.0);
            gfx::boxx(r, 0.25, 0.25, 0.25);
        }
    }
    r.cull = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> Options {
        Options::default()
    }

    #[test]
    fn a_fresh_game_has_the_original_stores() {
        let mut rng = Rng::new(1);
        let g = Game::new(&mut rng);
        assert_eq!((g.ammo * 2, g.mines_left, g.health), (400, 4, 100.0));
    }

    #[test]
    fn objects_spawn_over_the_grid() {
        let mut rng = Rng::new(7);
        let mut g = Game::new(&mut rng);
        for _ in 0..500 {
            g.spawn(&mut rng);
        }
        assert!(g
            .objs
            .iter()
            .all(|o| o.x >= -4.0 && o.x <= 4.0 && o.z >= -4.0 && o.z <= 4.0));
        assert!(g.objs.iter().any(|o| o.kind == Kind::Mine));
        assert!(g.objs.iter().any(|o| o.kind == Kind::Ammo && o.rapid));
    }

    #[test]
    fn a_sphere_on_the_ground_costs_health() {
        let mut rng = Rng::new(3);
        let mut g = Game::new(&mut rng);
        g.objs.push(Obj {
            x: 0.0,
            z: 0.0,
            size: 0.75,
            fall: 10.5,
            hp: 2.0,
            kind: Kind::Sphere,
            rapid: false,
        });
        let au = Audio::new(true);
        g.ground(&opts(), &au, &mut rng);
        assert!(g.objs.is_empty());
        assert!((g.health - 97.0).abs() < 1e-4);
    }

    #[test]
    fn the_demo_pilot_shoots_things_down() {
        let mut rng = Rng::new(11);
        let mut g = Game::new(&mut rng);
        let au = Audio::new(true);
        let mut t = 0;
        while t < 60_000 && !g.over {
            let c = g.autopilot(t);
            g.update(33, t, c, &opts(), &au, &mut rng, None);
            t += 33;
        }
        assert!(g.stats.destroyed > 20, "destroyed {}", g.stats.destroyed);
        assert!(g.score > 2000);
    }
}
