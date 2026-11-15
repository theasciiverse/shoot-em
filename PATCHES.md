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
- Its 28 sound samples (unchanged files from inside the program), panned
  and placed by where things are, as FMOD did. The intro's sound plays as
  the logo flies past, as it did.
- The intro logo, its geometry taken from the program.
- The screens: intro, menu, instructions, score and statistics, Hall of
  Fame with the original's starting table, and name entry.
- The command line letters: `a` (4x4 samples, the "4xMSAA" mode), `b`,
  `n`, `f`, `m` (male narrator), `c`, `p`.

## Different

- **Groove mode** (1.1.0, David's idea, 2026-11-15; off by default,
  never with `--classic`): G in the menu or `--groove`, remembered in
  `settings.txt`. The in-game playlist counts the frames it plays and
  notes each beat (`audio.rs`, `Beats`): for the modules, every 4 rows at
  the song's speed (doubled until the pulse is 0.38 to 0.9 s), bars
  every 16; for Junkala's three Ogg songs, a grid measured once from the
  audio by onset autocorrelation weighted to the kick (`OGG_GRID`: Level
  1 at 181.09 BPM, played as its 90.5 BPM half-time pulse on the kicks,
  Levels 2 and 3 at 105.5; each loop a whole number of bars). The game's
  clock is those frames, carried on by the wall clock between updates,
  less the calibration offset (`groove.rs`, after Bellbeat). A fire
  press is timed when it is read and judged: PERFECT within 70 ms, GOOD
  within 130 ms; each press on a beat adds to a combo (x2 at 4, up to x5
  at 16) that multiplies kill points, and earns 50 or 20 points times
  the multiplier; four beats without one end the combo. Objects fall on
  the beats, as many as the timer would have sent over that time, plus
  one on a bar whose loudness is 15% over the running average and two
  at 35%. The sound scope swells and turns white on each beat. C in the
  menu calibrates: clicks at 100 BPM, the median of ten taps (after two
  warm-ups) is the offset. Groove scores go to `hiscores-groove.txt`.
  The demo pilot fires short bursts starting on the beat.
- **The sound scope sits with each screen's content** (1.1.0-av.2): on
  the menu, score, name entry and Hall of Fame it spans that screen's own
  block and is centred on its button (`screens::*_SPAN`), not on the
  terminal; the Hall of Fame's list and BACK sit right of centre, so its
  scope does too. `--screen menu|score|name|hiscores` opens a screen
  directly, for checking layouts.
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
- **New music (v1.0.0-av.2).** The original carried two Protracker modules
  that were not Jan's and could not be cleared: "BadLands" by Matt Furniss
  is the soundtrack of Domark's 1990 Amiga game Badlands (its own sample
  texts say "Downloaded From.. Music Channel BBS"), and "boombass" by The
  Kid has no licence. Both are gone from the build. In their place:
  - menu: Crush (Asciiverse remix), after Drozerix's Crush (public
    domain), played through, two seconds of rest, again;
  - in game, one after another, carrying on from game to game: Juhani
    Junkala's Level 1, 2 and 3 from "5 Chiptunes (Action)" (CC0), and
    Digital Rendezvous and Mecanum Overdrive (Asciiverse remixes), after
    Drozerix's originals (public domain), Girl From Mars (Asciiverse
    extended mix, v1.0.0-av.4) after Drozerix's (public domain), and
    (v1.0.0-av.3) Tunnel Vision
    by fuzion_mixer, CC BY 4.0, remixed and extended by Asciiverse
    (6:50), each played once through; the full credits are at the end
    of the instructions screen;
  - after a game and in the Hall of Fame: Junkala's Title Screen (CC0).
  Levels (v1.0.0-av.4) are measured as integrated loudness (EBU R128)
  through the game's own playback: menu and end music both -16 LUFS;
  every in-game song -19 LUFS, under the guns (-14.5) and blasts (-11.6).
- **The sound scope (v1.0.0-av.2):** the game's own mix, music and effects
  together, drawn as a Braille oscilloscope (after Bellbeat's): in play,
  a row of its own directly above the HUD's boxes; on the menu, score,
  name entry and Hall of Fame screens, up to three rows just under the
  content (v1.0.0-av.4, which also made it bright cyan). S shows
  or hides it, remembered in `settings.txt` in the state folder;
  `--classic` shows Jan's 2005 screen exactly, without it. With `--mute`
  the game still runs the mix silently, so the scope moves and nothing is
  heard.
- **`--demo`:** the computer plays, for the catalog preview.
  `--mute` turns sound off.
- **Session logging:** `SHOOT_EM_LOG=debug` writes `shoot-em.log` in the
  state folder: every key and click, every state change, and a snapshot
  of the game each second.

## Built with

Rust; crossterm for the terminal; rodio for sound and the Vorbis music;
xmrs and xmrsplayer to play the menu's tracker module.
