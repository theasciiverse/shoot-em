# This edition and the original

Shoot 'em 1.0 (Jan Ringoš, 2005) ran only in a Windows XP console of
80x50 cells. Its source is lost, so this edition is a new program,
written from what the original does (REVERSE.md), by the author's
permission (PERMISSION.md). The original release is kept, unchanged, in
`original/ShootEm.zip`.

## The same

- The rendering approach: a lit, depth-tested 3D scene drawn at several
  samples a cell, averaged, and turned into the characters
  ` .,:;=+tIYVRBM` and the console's 16 colours by the original's own
  rule.
- The game, rule for rule and number for number:
  - spawning, falling and speeding up;
  - the guns, rapid rounds, mines and blasts;
  - points, health, pickups and debris;
  - the camera and the aim;
  - the HUD and its layout.
- Its 28 sound samples (unchanged files from inside the program) and its
  two music modules. Sounds are panned and placed by where things are,
  as FMOD did.
- The intro logo, its geometry taken from the program.
- The screens: intro, menu, instructions, score and statistics, Hall of
  Fame with the original's starting table, and name entry.
- The command line letters: `a` (4x4 samples, the "4xMSAA" mode), `b`,
  `n`, `f`, `m` (male narrator), `c`, `p`.

## Different

- **Any terminal size** (80x24 at least), not a fixed 80x50 console.
  Terminal cells are twice as tall as wide, so each cell takes twice as
  many samples down as across, and the view is as wide as the window.
  The HUD keeps its place: centred, with its bottom boxes at the bottom.
  The instructions scroll when there are fewer than 50 rows.
- **Keys as well as the mouse:** arrows aim, Space fires, M drops a mine.
  Held keys are true holds where the terminal reports key releases
  (kitty protocol); elsewhere a key counts as held for 160 ms after each
  repeat.
- **`--truecolor`:** colours each character with its averaged colour
  instead of the 16 console colours. The default stays the original look.
- **Titles in 3D block letters,** extruded from a 5x7 font: the original
  used Windows' Verdana and Arial Black outlines.
- **The Hall of Fame stars** are spinning yellow solids, not Wingdings
  glyphs.
- **High scores** are saved as a small text file in the state folder:
  `$XDG_STATE_HOME/shoot-em`, else `~/.local/state/shoot-em`
  (Application Support on macOS, LocalAppData on Windows).
- **No score synchronisation:** the old server is gone. The menu credits
  name the music's authors in place of the engine line.
- **`--demo`:** the computer plays, for the catalog preview.
  `--mute` turns sound off.
- **Session logging:** `SHOOT_EM_LOG=debug` writes `shoot-em.log` in the
  state folder: every key and click, every state change, and a snapshot
  of the game each second.

## Built with

Rust; crossterm for the terminal; rodio for sound; xmrs and xmrsplayer to
play the Protracker music.
