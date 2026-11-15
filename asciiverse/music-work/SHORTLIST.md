# Shoot 'em music: origins and free replacements

Found 2026-11-14. Nothing in the game or its release has been changed; David picks.

## The two modules Shoot 'em ships now

- **BadLands** (`assets/music/badlands.mod`, in game): **commercial game music.**
  Its sample texts read "by matt furniss" and "Downloaded From.. Music Channel
  BBS (203)646-3058", with ST-01 samples. Its order table and first pattern
  are identical to Modland's `Protracker/Matt Furniss/badingame1.mod`, the
  in-game music of Domark's 1990 Amiga game *Badlands* (ExoticA:
  exotica.org.uk/wiki/Badlands). A BBS-era rip of a commercial game's score:
  neither Jan nor we can clear it. Replace it.
- **boombass** (`assets/music/boombass.mod`, menu): sample texts "by the kid",
  "thanx to mantronix for nize instruments", "another short tune". The same
  module (identical sample texts and first pattern, a slightly different
  order table, 7,270 bytes against our 6,246) is on Modland as
  `Protracker/Nugget/boombass.mod`, filed under the composer **Nugget** (47
  modules there, several with Finnish titles). An amateur demoscene tune with
  no licence; "The Kid" looks like Nugget's own credit. Clearing it would
  mean finding Nugget; not found on Demozoo's search (its API returned an
  error). Ask Jan, but plan to replace it too.

## Shortlist (all free to ship and to bundle in a game)

The game's player is `xmrs` 0.15 (default import features: MOD, XM, S3M,
IT), so every module below plays as it is. Junkala's pack is WAV; the game
already links rodio's Vorbis decoder, so converted OGG files would play with
a small change to `src/audio.rs`.

"Public Domain" modules on The Mod Archive carry CC's Public Domain
Dedication; CC0 is the same in effect; CC BY 4.0 needs credit (on the store
page and the game's credits screen). The Mod Archive FAQ: modules with
licence deeds may be used as the deed says; others need the composer's
permission.

"Feel" below is from the genre, tempo and ratings: nobody has listened yet.
Previews are the first 20 s of each, 44.1 kHz stereo WAV.

| Use | Title | Artist | Licence | Format | Length | Genre, tempo | Preview |
|---|---|---|---|---|---|---|---|
| menu | Cabin Fever! | Drozerix | [Public Domain](https://creativecommons.org/licenses/publicdomain/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=174833)) | XM | 2:00 | Chiptune, 132 BPM, member rating 10/10 | `intro/174833-cabin_fever.wav` |
| menu | Crush | Drozerix | [Public Domain](https://creativecommons.org/licenses/publicdomain/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=179581)) | XM | 2:05 | Chiptune, 120 BPM, member rating 10/10 | `intro/179581-drozerix_-_crush.wav` |
| menu | Girl From Mars | Drozerix | [Public Domain](https://creativecommons.org/licenses/publicdomain/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=173935)) | XM | 1:52 | Chiptune, 128 BPM, member rating 9/10 | `levels/173935-girl_from_mars.wav` |
| menu | Keygen 9000 | Drozerix | [Public Domain](https://creativecommons.org/licenses/publicdomain/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=173938)) | XM | 3:20 | Chiptune, 120 BPM, member rating 9/10 | `173938-keygen_9000.wav` (rejected, deleted) |
| game | Bassline Techno | Katie Cadet | [Public Domain](https://creativecommons.org/licenses/publicdomain/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=175335)) | XM | 1:42 | Electronic - Techno, 125 BPM, member rating 9/10 | `175335-kc-basslinetech.wav` (rejected, deleted) |
| game | Blade of fire | Zilly Mike | [Attribution](https://creativecommons.org/licenses/by/4.0/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=37197)) | XM | 3:07 | Rock - Hard, 160 BPM, member rating 10/10 | `37197-blade_of_fire.wav` (rejected, deleted) |
| game | Digital Rendezvous | Drozerix | [Public Domain](https://creativecommons.org/licenses/publicdomain/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=180821)) | XM | 2:44 | Chiptune, 138 BPM, member rating 10/10 | `levels/180821-drozerix_-_digital_rendezvous.wav` |
| game | It's Time | K. Jose | [CC0](https://creativecommons.org/publicdomain/zero/1.0/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=182142)) | IT | 1:55 | Electronic - Other, 150 BPM, member rating 10/10 | `182142-k_jose_-_its_time.wav` (rejected, deleted) |
| game | Mecanum Overdrive | Drozerix | [Public Domain](https://creativecommons.org/licenses/publicdomain/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=175349)) | XM | 3:18 | Video Game, 144 BPM, member rating 10/10 | `levels/175349-drozerix_-_mecanum_overdrive.wav` |
| game | Outbreak! [TSEC] | Andreas Viklund | [Attribution](https://creativecommons.org/licenses/by/4.0/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=34325)) | XM | 6:47 | Trance - Goa, 150 BPM, member rating 10/10 | `34325-av_out.wav` (rejected, deleted) |
| game | Suspicion | K. Jose | [CC0](https://creativecommons.org/publicdomain/zero/1.0/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=191220)) | S3M | 2:34 | Chiptune, 150 BPM, member rating 10/10 | `191220-k_jose_-_suspicion.wav` (rejected, deleted) |
| game | Tunnel Vision       [tqc] | fuzion_mixer | [Attribution](https://creativecommons.org/licenses/by/4.0/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=205660)) | IT | 6:26 | Trance - Goa, 150 BPM, member rating 10/10 | `levels/205660-fuzion_mixer_-_tunnel_vision.wav` |
| game | shutdown | Ven Sopeña | [Attribution](https://creativecommons.org/licenses/by/4.0/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=195328)) | XM | 3:08 | Electronic - Techno, 130 BPM, member rating 9/10 | `195328-ven_sopena_-_shutdown.wav` (rejected, deleted) |
| game | the-red-one | m0d | [Public Domain](https://creativecommons.org/licenses/publicdomain/) ([module page](https://modarchive.org/index.php?request=view_by_moduleid&query=53326)) | MOD | 4:29 | Demo Style, 125 BPM, member rating 10/10 | `levels/maybe-53326-musix-the-red-one.wav` |
| menu | Title Screen | Juhani Junkala | [CC0](https://opengameart.org/content/5-chiptunes-action) (pack's INFO.txt: "released under CC0 creative commons license. You can do anything you want with these tunes.") | WAV (convert to OGG) | 0:11 (loop) | chiptune, Retro Game Music Pack | `end/junkala-title-screen.wav` |
| game | Level 1 | Juhani Junkala | [CC0](https://opengameart.org/content/5-chiptunes-action) (pack's INFO.txt: "released under CC0 creative commons license. You can do anything you want with these tunes.") | WAV (convert to OGG) | 1:14 | chiptune, Retro Game Music Pack | `levels/junkala-level-1.wav` |
| game | Level 2 | Juhani Junkala | [CC0](https://opengameart.org/content/5-chiptunes-action) (pack's INFO.txt: "released under CC0 creative commons license. You can do anything you want with these tunes.") | WAV (convert to OGG) | 1:13 | chiptune, Retro Game Music Pack | `levels/junkala-level-2.wav` |
| game | Level 3 | Juhani Junkala | [CC0](https://opengameart.org/content/5-chiptunes-action) (pack's INFO.txt: "released under CC0 creative commons license. You can do anything you want with these tunes.") | WAV (convert to OGG) | 1:22 | chiptune, Retro Game Music Pack | `levels/junkala-level-3.wav` |
| (score screen) | Ending | Juhani Junkala | [CC0](https://opengameart.org/content/5-chiptunes-action) (pack's INFO.txt: "released under CC0 creative commons license. You can do anything you want with these tunes.") | WAV (convert to OGG) | 0:45 | chiptune, Retro Game Music Pack | `junkala-ending.wav` (rejected, deleted) |

Picks worth hearing first: **Mecanum Overdrive** (Drozerix, public domain,
144 BPM, filed as video-game music) or Junkala's **Level 1** for the game;
**Cabin Fever!** (Drozerix, public domain, 4 channels, 24 KB, Amiga-like)
or Junkala's **Title Screen** for the menu. Drozerix has five tunes here, all
public domain, so a menu and game pair by one composer is easy.

## Files

- Previews (never played aloud on beast) are sorted by use: `intro/` (menu), `end/` (score and Hall of Fame), `levels/` (in game). Rejected ones are deleted.
- `remixes/`: the Asciiverse remix modules (see below).
- `modules/`: the downloaded module files, as The Mod Archive serves them.
- The Junkala pack is in `../sem-dl/junkala/` (`action.zip`, CC0, with INFO.txt).

## David's decisions (2026-11-14)

- Menu: Crush (Asciiverse remix), APPROVED. In game: play it through, then a short silence (about 2 s), then start again (loop with a pause, done in the game's audio code, not in the module).
- Score / Hall of Fame: Junkala Title Screen as it is, the 11 s original looping (David: already really good, just short; extension parked for later. A filter/echo/stutter rework of the mix was tried and rejected: it lost the music. If revisited: transcribe to a module and compose new sections, or Audacity's AI continuation after checking MusicGen's terms).
- In game: Junkala Level 1, 2, 3 approved; Tunnel Vision approved, extended mix to be heard.

## Asciiverse remixes (2026-11-14)

Made on beast from the original modules; nothing in the game changed yet.
Every added instrument is synthesised by us in code (kick, snare, hats,
crash, ride, three toms, a distorted power-chord guitar open and muted, a
bass; `gfm/synth.py` in the scratchpad), so no new sample needs credit.
The modules are in `remixes/`. Previews are rendered with **xmrs 0.15,
the player Shoot 'em uses**, so they sound as the game will; all have 0
clipped samples (the originals clip in xmrs: Crush 5,123 samples in 90 s,
Mecanum 1,485, Digital Rendezvous 1,547). Each remix sets a global volume at
its start for that headroom. libopenmpt and xmrs agree on all of them
(envelope correlation 0.99 and up where compared).

**For the game:** Shoot 'em loads its songs with `Module::load_mod`, which
only reads MOD. These are XM and IT: use `xmrs::prelude::Module::load`
(detects the format; checked here, it loads both).

### Tunnel Vision (Asciiverse extended mix): in game

- **Credit (required, CC BY 4.0):** "Tunnel Vision by fuzion_mixer, CC BY
  4.0 (https://creativecommons.org/licenses/by/4.0/), remixed and extended
  by Asciiverse." Original: https://modarchive.org/index.php?request=view_by_moduleid&query=205660.
  Its own message says most samples are from MAZ's "no.1 instruments" CD.
- **What it is:** the original is already 6:26 (the 20 s preview showed only
  its start). The extended mix keeps all of it up to its third drop and
  adds 12 new patterns built from its own material, then skips its outro
  and loops back into its first drop: 6:50 before the loop, then the loop
  from the first drop on (about 5:14 a pass).
  - after the second drop (3:57): a **dub** section (drums, bass and
    arpeggios, leads out, bass octave pops, ride on the off-beats), a
    **half-time** section under the first drop's leads lifted an octave,
    a stripped **build** with a snare roll that speeds up into the drop
    (kick and bass drop out for the last beats), and the second drop again
    with its **lead an octave up** and the ride doubled
  - in the third drop: an **arpeggio spotlight** (leads out, arpeggios up an
    octave) ending in another roll
  - at the end: a **final peak** (everything, ride in sixteenths, lead
    lifted), then the jump back to the first drop
  - global mix volume set to 64 for headroom in xmrs (libopenmpt ignores
    it for this module because of its OpenMPT mix settings)
- Module: `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/tunnel-vision-extended.it`
- First 2:45: `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/tunnel-vision-extended.wav`
- 60 s from the new sections (from 3:57): `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/tunnel-vision-extended-middle.wav`

### Crush (Asciiverse remix): menu

- Drozerix, public domain. His opening as is (16 s), then a walk into the
  beat: the kick fades in on the beat with hats joining halfway, then the
  full beat with a fill. From there a solid beat of ours (kick on 1, 3 and
  the "and" of 3, snare on 2 and 4, eighth hats with sixteenth ghosts, a
  crash every 8 bars, a fill every 16) replaces his thin chip drums under
  all his remaining music. About 2:08; the module loops back to the beat
  section (order 8), not the opening. (David's plan to play it through
  and pause 2 s before starting again is for the game's audio code: stop
  at the song's end instead of letting xmrs loop.)
- Module: `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/crush-remix.xm`
- Preview (first 90 s): `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/intro/crush-remix.wav`

### Girl From Mars (Asciiverse remix): in game, two variants

- Drozerix, public domain. His intro unchanged (22 s), a fill (snare roll
  and toms) that speeds the tempo up over the last bar into a heavy metal
  section: double-kick drums, distorted power chords double-tracked left and
  right (an open chord on each half bar, then palm-muted chugs), a bass,
  over his melody and arpeggios (his chip drums taken out), with a
  half-time breakdown and his intro melody returning heavy at the end.
  Loops to the start of the metal section. The metal enters on the
  downbeat by construction (the fill ends on the pattern's last row, the
  crash is on the next pattern's first).
- **chug, 150 BPM:** `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/girl_from_mars-remix-chug150.wav` (module `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/girl_from_mars-remix-chug150.xm`)
- **gallop, 165 BPM** (kick and guitars in a gallop rhythm): `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/girl_from_mars-remix-gallop165.wav` (module `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/girl_from_mars-remix-gallop165.xm`)
- Both previews are the whole song once (about 80 s).

### Mecanum Overdrive (Asciiverse remix): in game

- Drozerix, public domain. His track unchanged with our drums building over
  it: none for the first 29 s, then a kick on the beat, then the snare on 2
  and 4 (0:56), hats and crashes with fills (1:27), sixteenth hats and more
  fills (1:55), a snare roll through his own acceleration (2:31), and a
  heavy full beat at the peak (2:38: double kick in sixteenths, ride,
  crashes). His outro is left out; the song loops at the peak (order 44).
- Module: `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/mecanum-remix.xm`
- The build, 0:22 to 1:52: `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/mecanum-remix.wav`
- The peak, 30 s from 2:38: `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/mecanum-remix-peak.wav`

### Digital Rendezvous (Asciiverse remix): in game

- Drozerix, public domain. His intro as is, then a driving beat of ours in
  place of his thin chip kit (kick on every beat, off-beat ride, snare on 2
  and 4, sixteenth hats, crashes at section starts, fills into each new
  section), lighter in his breakdowns (kick on 1 and 3, eighth hats). Loops
  back to the first beat section.
- Module: `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/digital-rendezvous-remix.xm`
- Preview (first 90 s): `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/digital-rendezvous-remix.wav`
- Keep: the opening sound effect (Jan's own sample, assets/sfx/124.ogg, sfx::INTRO, played as the blue logo flies past) stays exactly as it is, before the Crush remix starts (David, 2026-11-14).

## Heavy versions: drums to carry the gun (2026-11-14, later)

After David's notes ("a stronger but slower bass drum beat; too
jittery/high; something to accompany the gun"; Tunnel Vision's drums "too
weak"), every in-game remix has a **heavy** version. These supersede the
busy ones above (chug150, gallop165, the first Mecanum and Digital
Rendezvous remixes, the first Tunnel Vision extended mix); those stay only
for comparison.

**The groove they all share** is modelled on the drums of Juhani Junkala's
*Title Screen* (CC0, 8 bars at 170 BPM, felt as 85), which David chose as
the reference. Averaging its 8 bars sample by sample (the drums repeat,
the melody blurs out) shows, per bar in sixteenths:

```
kick   X . X . . . X . X . . . . . X .
snare  . . . . X . . . . . . . X . . .
hat    x . x . x . x . x . x . x . x .   (light)
```

Two quick kicks, the backbeat, two more and one pushed before the next
bar. In our 8-rows-a-beat patterns that is the kick on rows 0, 4, 12, 16
and 28 of every 32-row bar and the snare on 8 and 24. The heavy versions
use it with a new deep kick (a low sine body you feel, a punchy attack, a
long tail) and a hard snare (low body, long crack), both synthesised by
us, put in front, and leave the hats out (David: not busy or high).

### Tunnel Vision (Asciiverse extended mix, heavy drums): in game

- Same arrangement and build as the extended mix (6:50 before the loop
  back into the first drop). Its kick plays the reference groove instead
  of four on the floor wherever it played (its dropouts and builds kept),
  doubled at the same pitch for punch and an octave down for a sub you
  feel; its 909 snare layered under every backbeat snare; every other
  channel turned down (channel volume 34 of 64) so the drums sit in front,
  rather than squashing the mix; the ride clutter of the first extended
  mix left out. Mix volume 50: 0 clipped samples over the whole 6:50 in
  xmrs. Uses only the module's own samples (credit as above: "Tunnel
  Vision by fuzion_mixer, CC BY 4.0, remixed and extended by Asciiverse").
- Module: `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/tunnel-vision-extended-heavy.it`
- First 2:45: `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/tunnel-vision-extended-heavy.wav`
- 60 s from the new sections (from 3:57): `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/tunnel-vision-extended-heavy-middle.wav`

### Girl From Mars (Asciiverse remix, heavy): in game

- His intro as is (22 s), then a slow tom fill that eases the tempo down
  to 112 BPM into the groove: the deep kick in the reference pattern, the
  hard backbeat, one low power chord each half bar with a single chug on
  the next beat, a bass; no hats, no double kick. His busiest high parts
  (the every-row arpeggios and the sweeps up to the 7th octave) are out,
  his other melody channels that sit high play an octave lower, and his
  melody is about 4 dB down. The top end is about 4 times calmer than
  chug150 (1.2 high-band hits a second against 4.2). Loops at the start
  of the heavy section.
- Module: `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/girl_from_mars-remix-heavy.xm`
- Preview (whole song once, 85 s): `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/girl_from_mars-remix-heavy.wav`

### Mecanum Overdrive (Asciiverse remix, heavy): in game

- The drums still build: none for the first 29 s, then the deep kick on
  the felt beats (0:29), then the hard backbeat (0:56, his busy chip kick
  and snare taken out from here), then the full reference groove (1:27),
  louder at the peak (2:38) with crashes; one slow, heavy tom fill into
  each new stage, no fast fills, no hats. Loops at the peak.
- Module: `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/mecanum-remix-heavy.xm`
- The build, 0:22 to 1:52: `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/mecanum-remix-heavy.wav`
- The peak, 30 s from 2:38: `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/mecanum-remix-heavy-peak.wav`

### Digital Rendezvous (Asciiverse remix, heavy): in game

- His intro as is, then the reference groove with the deep kick and hard
  snare in place of his chip kit, no hats or off-beat ride; in his
  breakdowns only the kick on 1 and 3; slow two-hit tom fills into new
  sections. Loops back to the first beat section.
- Module: `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/digital-rendezvous-remix-heavy.xm`
- Preview (first 90 s): `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/digital-rendezvous-remix-heavy.wav`

All heavy previews: rendered with xmrs, 0 clipped samples.

## Metal finds (Mod Archive, 2026-11-14)

Searched The Mod Archive's Metal (general), Metal - Extreme, Rock - Hard
and Electronic - Industrial genres: 529 modules. 352 are under the Mod
Archive Distribution licence (not free to ship), 7 are non-commercial, and
161 module pages returned errors so their licence could not be read. These
8 are free to ship (Blade of fire, rejected, left out). 20 s previews,
rendered with xmrs; modules in `modules/metal-<id>.*`.

| Title | Artist | Licence | Format | Genre | Preview |
|---|---|---|---|---|---|
| Mega Destruction | Zilly Mike | [Public Domain](https://modarchive.org/index.php?request=view_by_moduleid&query=50252) | XM | Rock - Hard | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/metal-mega-destruction.wav` |
| beefeater | Zilly Mike | [CC BY](https://modarchive.org/index.php?request=view_by_moduleid&query=170004) | MOD | Rock - Hard | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/metal-beefeater.wav` |
| I Made a Mistake | Pip Malt | [CC BY](https://modarchive.org/index.php?request=view_by_moduleid&query=167122) | IT | Rock - Hard | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/metal-i-made-a-mistake.wav` |
| Monk | Pip Malt | [CC BY](https://modarchive.org/index.php?request=view_by_moduleid&query=163070) | IT | Rock - Hard | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/metal-monk.wav` |
| Pretty Big Stench | Pip Malt | [CC BY](https://modarchive.org/index.php?request=view_by_moduleid&query=166919) | IT | Rock - Hard | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/metal-pretty-big-stench.wav` |
| Spindly Deals | Pip Malt | [CC BY](https://modarchive.org/index.php?request=view_by_moduleid&query=165325) | IT | Rock - Hard | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/metal-spindly-deals.wav` |
| Down the Elevator Shaft | Pip Malt | [CC BY](https://modarchive.org/index.php?request=view_by_moduleid&query=163230) | IT | Electronic - Industrial | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/metal-down-the-elevator-shaft.wav` |
| Out with a vengeance... | BLuRry | [CC BY-SA](https://modarchive.org/index.php?request=view_by_moduleid&query=73281) | S3M | Electronic - Industrial | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/metal-out-with-a-vengeance.wav` (from 1:15; quiet track) |

Caution: xmrs and libopenmpt disagree on Pip Malt's IT modules (Monk:
xmrs 2.5 times louder, envelope correlation 0.59), so hear those in the
game itself before choosing one.
- In game: Digital Rendezvous (Asciiverse remix, heavy) APPROVED as is (David, 2026-11-14): 'this one definitely works ... good job'; copied to games/shoot-em/assets/music/levels/digital-rendezvous-remix-heavy.xm and added to the release playlist.
- In game: Mecanum Overdrive (Asciiverse remix, heavy) APPROVED as is (David, 2026-11-14); copied to games/shoot-em/assets/music/levels/.
- In game: Tunnel Vision (Asciiverse extended mix), the FIRST extended version (remixes/tunnel-vision-extended.it, 21:33), NOT the heavy one, APPROVED (David, 2026-11-14); copied to games/shoot-em/assets/music/levels/tunnel-vision-extended.it, replacing the original tunnel-vision.it. CC BY 4.0: 'Tunnel Vision by fuzion_mixer, CC BY 4.0, remixed and extended by Asciiverse'.

## Pip Malt extended mixes (2026-11-14)

David: "Extended versions of these have potential." All three are by Pip
Malt under **CC BY 4.0** (https://creativecommons.org/licenses/by/4.0/,
the licence each module page links). Credit lines:

- "Down the Elevator Shaft by Pip Malt, CC BY 4.0, remixed and extended by Asciiverse." (https://modarchive.org/index.php?request=view_by_moduleid&query=163230)
- "I Made a Mistake by Pip Malt, CC BY 4.0, remixed and extended by Asciiverse." (https://modarchive.org/index.php?request=view_by_moduleid&query=167122)
- "Spindly Deals by Pip Malt, CC BY 4.0, remixed and extended by Asciiverse." (https://modarchive.org/index.php?request=view_by_moduleid&query=165325)

The modules credit their samples to others: Elevator Shaft's kick and clap
"from Zanoma", piano and strings "from the MIDI library", drum stix "from
Tito", "the rest from anywhere and everywhere"; I Made a Mistake's guitars
and bass by Zack Ohren, drums by Claymore, kick from Betrayer, clean
guitar by Quasian; Spindly Deals' guitars by Malekith, drums by Claymore,
cymbals from Betrayer. Pip Malt licensed the modules as a whole under CC
BY 4.0.

**How they were made** (like the approved Tunnel Vision extended mix):
the song as it is up to where it would come round again, then ten new
patterns from its own main riff and chorus: a **breakdown** (only the
kick on the bar and the backbeat, bass and rhythm guitar, leads out), a
**build** (kick on every beat, a snare roll that speeds up, rhythm guitar
only), a **drop** (the chorus with the lead an octave up), a **half-time**
riff, and the riff in full with the lead lifted. Then the rest of the
song, its outro left out, and a jump back into its first full section.
Drums in front for shooting along to: the kick doubled on a spare channel
for punch, the drum channels at full volume and everything else turned
down a little. Their own drum feel is kept (no Junkala groove imposed:
these are metal).

**xmrs, the game's player:** these IT modules were the problem. At their
full global volume, xmrs overflows inside its mixer on Down the Elevator
Shaft (the full mix correlated only 0.29 with libopenmpt, while each
channel alone matched at 0.8 to 0.99), so its global volume is halved
(64), and every preview below is rendered with xmrs: 0 clipped samples.
What still differs is that Elevator Shaft's piano, strings and "New Age
Thingy" use resonant filters and filter or pitch envelopes, which xmrs
renders differently from libopenmpt; Spindly Deals matches libopenmpt
closely (0.99), I Made a Mistake in between. The previews are what the
game will play; hear them and judge by ear.

| Track | Before the loop | First 2:30 | 60 s from the new sections |
|---|---|---|---|
| Down the Elevator Shaft | 4:32 (loops to the first full section) | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/down-the-elevator-shaft-extended.wav` | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/down-the-elevator-shaft-extended-middle.wav` (from 2:48) |
| I Made a Mistake | 3:55 | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/i-made-a-mistake-extended.wav` | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/i-made-a-mistake-extended-middle.wav` (from 2:45) |
| Spindly Deals | 3:45 | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/spindly-deals-extended.wav` | `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/spindly-deals-extended-middle.wav` (from 2:28) |

Modules: `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/down-the-elevator-shaft-extended.it`, `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/i-made-a-mistake-extended.it`, `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/spindly-deals-extended.it`.

## Girl From Mars (Asciiverse extended mix), retake (2026-11-14)

David: "do a retake from the beginning of extending this. You can borrow
some of the beats from Tunnel Vision style." The metal remixes (chug150,
gallop165, heavy) are **dropped**; their previews are deleted from
`levels/` (their modules stay in `remixes/` for the record).

- Drozerix's original (public domain), made the way the approved Tunnel
  Vision extended mix was made. His intro is kept as is, with a short tom
  fill into the first full section. His thin chip drums are replaced by a
  **Tunnel Vision style beat** re-created with our own synthesised drums,
  so nothing from fuzion_mixer's module is used and no credit is needed:
  a kick on every beat, an open hat on the off-beat, a snare on 2 and 4,
  a ride on the beat, and crashes at section starts.
- Sections, all from his own patterns:
  - his main section with the beat
  - a **dub** (drums with his bass and pads, the busy arpeggios and the
    lead out)
  - **half time** under his melody lifted an octave
  - a **build** (his arpeggio section with only the kick, then a snare
    roll that speeds up and drops out before the beat)
  - the **drop**, his main section with the lead an octave up
  - his arpeggio section with the beat, then his main section again with
    the ride doubled
  - a second dub, a **breakdown** on his intro chords (the kick only on
    beats 1 and 3) with its build
  - his **intro melody over the beat** with another build
  - a **final peak** with the lead lifted
- 3:26 before the loop, then back to his first full section. Rendered
  with xmrs: 0 clipped samples.
- Module: `/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/remixes/girl_from_mars-extended.xm`
- First 2:30: `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/girl_from_mars-extended.wav`
- 60 s from 2:10 (the main section again, the second dub, the
  breakdown): `mpv /tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/shoot-em-music/levels/girl_from_mars-extended-middle.wav`
- In game: Girl From Mars (Asciiverse extended mix) APPROVED (David, 2026-11-15); assets/music/levels/girl_from_mars-extended.xm.
