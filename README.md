# Shoot 'em

Jan Ringoš' 3D shooting game from 2005, whose whole 3D world is drawn in
coloured text, rebuilt for today's terminals on Linux, macOS and Windows.

Things fall from the sky over a little grid: spheres, boxes, green armour
diamonds, cyan bonuses, red mines, ammunition crates. Two machine guns
and a mine emitter are yours. Destroy everything before it reaches the
grid, or it explodes and takes your health with it. Survive as long as
you can.

## Play

With [Asciiverse](https://asciiverse.sh): `av install shoot-em`, then
start it from `av`.

Mouse: aim; the left button fires the machine guns, the right button
drops a mine. Keys: arrows aim, Space fires, M drops a mine, Escape
returns to the menu.

```
shoot-em [letters] [--truecolor] [--mute] [--demo] [--seed N]
```

The original's letters still work: `a` best antialiasing, `n` none, `f`
frames per second, `m` male narrator, `c` no debris, `p` twice the
debris. `--truecolor` colours each character with its real colour.

## About this edition

The original's source is lost; Jan Ringoš gave his permission to rebuild
it from the program itself (PERMISSION.md). How the original works is in
REVERSE.md, and what this edition does differently is in PATCHES.md. His
original release is kept in `original/`.
