# How the original Shoot 'em works

Notes from taking apart Jan Ringoš' Shoot 'em 1.0 (Windows XP console,
2005), done with his permission ("Feel free to port the game in any way
you like; even decompile it if necessary", see PERMISSION.md). Its source
is lost, so everything here comes from the program itself.

## The files

`original/ShootEm.zip` (sha256
`a693a01f799240c201f30b53eb7fb7b1a7880736196066c3060faa63414439d0`,
downloaded from tringi.trimcore.cz/Shoot_em_ASCII on 2026-11-13) holds:

- `ShootEm.dat`: the game, a 32-bit console PE built with MinGW GCC on
  2005-02-20 19:48:53 UTC. It carries all its sounds and music as
  resources.
- `ShootEm.exe`: only a full-screen loader (his page says to delete it and
  rename the .dat on Windows 7 and later).
- `fmod.dll`: FMOD 3.x, UPX-packed.
- `doc/options.txt`, `res.txt`, `tips.txt`, `support.txt`.

Resources inside `ShootEm.dat`:

- `FMODSOUND` 100 to 127: 28 Ogg Vorbis samples, shipped here unchanged in
  `assets/sfx/`.
- `MOD` 100: "boombass" by The Kid (menu music); `MOD` 101: "BadLands" by
  Matt Furniss (game music). Both 4-channel Protracker modules, in
  `assets/music/`.
- icons and a manifest.

Imports tell the story: OPENGL32/GLU32 to draw, GDI32 for an off-screen
bitmap, KERNEL32 console calls to read the mouse and write cells, FMOD for
sound, WININET for the online score list.

## Tools

Ghidra 12.2 headless decompiled every function (`asciiverse/tools/`
holds the script that dumps them); pefile pulled the resources and read
constants; capstone settled argument order where the decompiler blurred
it (the gluLookAt calls). The original was run under Wine 10 in a
throwaway container with no network, in an 80x50 terminal, for ground
truth frames.

## The renderer (the clever part)

1. At start it creates a memory DC and a bitmap of 80x50 cells times the
   antialiasing factor: 80x50 (`n`), 160x100 (`b`, default) or 320x200
   (`a`), and an OpenGL context on it (FUN_00402510).
2. Each frame is drawn with ordinary OpenGL 1.1: glFrustum with a 45 degree
   vertical field, near 0.1, far 32, aspect 8:5 (FUN_00402688); one
   positional light; colour material; depth test; cull faces on solid
   models; alpha blending for debris, blasts and the crates' glass.
3. glReadPixels reads the red, green and blue planes, and each 2x2 or
   4x4 block is averaged into one byte per channel per cell.
4. FUN_00402852 turns each cell into a character and a console attribute:
   - brightness = (r + g + b) / 255, capped at 1, picks from the ramp
     ` .,:;=+tIYVRBM` (14 steps; the first step it is below);
   - colour: red if r > 0.1, green if g > 0.1, blue if b > 0.1, bright if
     any channel > 0.5; a blank is light grey.
5. A text layer (FUN_00402a46) is laid over the picture, and the whole
   80x50 buffer goes out with WriteConsoleOutputCharacter/Attribute.

This edition does the same in `src/gfx.rs` (a small software rasteriser
standing in for OpenGL) and `src/ascii.rs` (the conversion, unchanged).

## Main loop and screens

FUN_00401486 parses the command line letters (`a b n` antialiasing, `f`
FPS box, `m` male narrator, `c` no debris, `p` double debris), loads the
high scores, checks for the 80x50 console, loads samples and music, sets
up OpenGL and runs a state machine with the frame time in ms and the time
in the state:

| state | function | screen |
|---|---|---|
| 0 | FUN_0040bae0 | intro: the blue logo (FUN_004098d0) flying past, 3 s |
| 1 | FUN_0040d0a6 | menu: 3D "Shoot 'em" in Verdana outlines, four buttons |
| 2 | FUN_0040c572 | Hall of Fame list, scrolls, synchronise and back |
| 3 | FUN_0040bc4a | instructions |
| 4 | FUN_00402ba0 | the game |
| 5 | FUN_0040c096 | your score, a comment, statistics |
| 6 | FUN_0040c9ae | name entry when the score makes the table |
| 9 | | exit |

## The game (FUN_00402ba0), with every number

Camera: gluLookAt eye (0, 2, 8), looking at (aimX, aimY + 1, 0), up y.
Aim from the mouse cell: aimX = col / 20 - 2, aimY = 2.5 - row / 20 (row
capped at 49, col at 79). Light at (-10, 20, 20).

Guns: two frames at (+-0.5, -0.3, -0.5) in eye space pointing -z, turned
by aimX * -29 degrees about y, then (aimY - 1.1) * 28 about x. Drawn in eye
space with 12 degrees of toe-in, recoiling by cooldown * 0.5.

Machine guns (left button):
- 200 rounds at the start (shown doubled, 400: each shot fires both guns).
- A shot every 0.1 s, every 0.04 s while rapid rounds last.
- Bullets start 2 units along the gun, moved into world space by
  (0, 2, 8), fly 0.02 units/ms, vanish beyond 32 units.
- They hit a falling object within 1.2 x its size of the stretch flown this
  frame and half as much again.
- Beeps at 40, 30, 20 and 10 rounds; a voice at 0; a click when empty.

Mines (right button):
- 4 at the start; one every 2 s.
- Launched from the right gun's frame, drift 0.005 units/ms.
- Go off when shot (+1000) or when they touch an object (+1000).

Blasts: radius 4, fading out over 1 s. For their first 0.1 s they destroy
everything within reach for no points, but armour heals (+5) and crates
give 25 rounds (the tips say "halved").

Spawning:
- One object every (1 - t / 360000) s, at least 0.1 s.
- x and z are a random whole number from -4 to 4.
- Every random choice is "ROUND(rand * n)" with the FPU set to truncate,
  so it gives 0 to n - 1.
- Kinds, by rand * 6:

| roll | kind | size | hits | notes |
|---|---|---|---|---|
| 0 | sphere | 0.75 | 2 | grey; 1 in 5 small: 0.5, 4 hits, magenta |
| 1 | box | 0.5 | 2 | grey, tumbling |
| 2 | armour diamond | 0.75 | 6 | green, falls twice as fast |
| 3 | bonus | 0.5 | 3 | cyan, falls twice as fast |
| 4 | mine | 0.75 | 1 | red; rerolled 7 times in 8 |
| 5 | ammo crate | 0.75 | 2 | rerolled 2 in 3; 1 in 6 is rapid (blue) |

Falling: 0.001 units/ms times (0.5 + t / 800000), from height 10.

At the grid (fallen 10):
- Spheres, boxes, armour and mines explode.
- Health drops by 2 x size x hits left; a mine takes 2 more.
- Bonuses and crates fall on through and are gone at 15.
- Voices at health below 50 and below 25.

Shot down:

| kind | points | other |
|---|---|---|
| sphere | 25 (small 50) | |
| box | 25 | |
| armour | 200 | health +5, up to 100 |
| bonus | 2000 | |
| mine | 1000 | a blast |
| crate | 100 | +40 rounds, or +80 and 80 rapid; a mine half the time |

Score:
- +1 every 100 ms survived (the tips' "10 points a second").
- The game ends when health reaches 0.

Debris: 8 triangles per explosion (16 for spheres and crates, 6 for
armour, 4 for bonuses, 12 for blasts), doubled by `p`, none with `c`.
They tumble about all three axes and fade over 3 s.

HUD (the text layer):
- the compass along row 3;
- the aiming box round the mouse with its scale, the ammunition bars and
  the counts;
- "!!! GET READY !!!" and a random tip in the first seconds;
- the boxes along the bottom: mines with their cooldown; score and health
  (flashing below 25); ammunition with the rate bar and a stack of
  rounds.

## What was not carried over

- `ShootEm.exe`, the full-screen loader: terminals are not full-screen
  consoles.
- The online score synchronisation (WININET to his old Mx-3.cz site, long
  gone).
- One quirk: the spawn switch has a seventh case for an invisible,
  harmless object kind that truncation makes unreachable, so nothing is
  lost.
