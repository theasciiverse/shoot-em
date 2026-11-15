"""Tunnel Vision (Asciiverse extended mix, heavy drums): fuzion_mixer's Tunnel Vision
(CC BY 4.0), lengthened with new sections built from its own patterns and
samples (half time, dub, a fill and build, lifted leads, an arpeggio
spotlight, a final peak), skipping the outro and looping back into the
first drop."""
import copy, itmod

SRC = '../shoot-em-music/modules/205660-fuzion_mixer_-_tunnel_vision.it'
t = itmod.read(SRC)
P = t.patterns
BASS, SUB, KICK, CRASH, HATSN, RIDE, S909 = 2, 3, 4, 5, 6, 7, 8
ARPS = (13, 14, 15); LEADS = (19, 20, 21, 22, 23, 24); SAWS = (16, 17, 18)
I_SNARE, I_RIDE, I_909 = 13, 18, 22

def cp(i): return copy.deepcopy(P[i])
def is_note(n): return n is not None and n < 120

def mute(g, chans, rows=None):
    for r, row in enumerate(g):
        if rows and r not in rows: continue
        for c in chans: row[c] = None
    return g

def transpose(g, chans, k):
    for row in g:
        for c in chans:
            cell = row[c]
            if cell and is_note(cell[0]): cell[0] = max(0, min(119, cell[0] + k))
    return g

def octave_pop(g):
    for r, row in enumerate(g):
        cell = row[BASS]
        if cell and is_note(cell[0]) and r % 8 == 6: cell[0] = min(119, cell[0] + 12)
    return g

def half_time(g):
    for r, row in enumerate(g):
        if r % 16 == 8 and row[KICK]: row[KICK] = None
        c = row[HATSN]
        if c and c[1] == I_SNARE: row[HATSN] = None
        if r % 32 == 16: row[HATSN] = [60, I_SNARE, 60, None, None]
    return g

def ride_double(g):
    return g
def _ride_double_unused(g):
    for r, row in enumerate(g):
        if r % 8 == 4 and not row[RIDE]: row[RIDE] = [60, I_RIDE, 36, None, None]
    return g

def ride_sixteenths(g, start=0):
    return g
def _ride_sixteenths_unused(g, start=0):
    for r, row in enumerate(g):
        if r >= start and r % 2 == 0 and not row[RIDE]:
            row[RIDE] = [60, I_RIDE, 24 if r % 4 else 34, None, None]
    return g

def snare_build(g, start=64):
    """a roll that speeds up into the next downbeat, kick and bass dropping out at the end"""
    for r in range(start, len(g)):
        step = 4 if r < start + 32 else (2 if r < start + 48 else 1)
        if (r - start) % step == 0:
            vol = int(18 + 46 * (r - start) / (len(g) - start))
            g[r][S909] = [60, I_SNARE, vol, None, None]
    n = len(g)
    mute(g, [KICK], range(n - 8, n))
    mute(g, [BASS, SUB], range(n - 4, n))
    return g

new = []
def add(g):
    P.append(g); new.append(len(P) - 1); return len(P) - 1

# S1 dub: drums, bass and arpeggios, leads out; bass octave pops, ride on the off-beats
s1a = add(ride_double(octave_pop(mute(cp(13), LEADS + SAWS))))
s1b = add(ride_double(octave_pop(mute(cp(14), LEADS + SAWS))))
# S2 half time under the first drop's leads, lifted an octave
s2a = add(transpose(half_time(cp(9)), LEADS, 12))
s2b = add(transpose(half_time(cp(10)), LEADS, 12))
# S3 build: the stripped kick-and-saw section with a snare roll into the drop
s3a = add(cp(17))
s3b = add(snare_build(cp(18), 32))
# S4 the second drop with its lead lifted, ride doubled, bass popping
s4a = add(octave_pop(ride_double(transpose(cp(25), LEADS[:3], 12))))
s4b = add(octave_pop(ride_double(transpose(cp(26), LEADS[:3], 12))))
# S5 arpeggio spotlight in the third drop: leads out, arpeggios up an octave
s5a = add(transpose(mute(cp(54), LEADS), ARPS, 12))
s5b = add(snare_build(transpose(mute(cp(55), LEADS), ARPS, 12), 64))
# S6 final peak: everything, ride in sixteenths, lead lifted, then back to the first drop
s6a = add(ride_sixteenths(transpose(cp(59), LEADS[:3], 12)))
s6b = add(ride_sixteenths(transpose(cp(60), LEADS[:3], 12)))

orig = [o for o in t.orders if o != 255]
orders = (orig[:37]
          + [s1a, s1b, s2a, s2b, s3a, s3b, s4a, s4b]
          + orig[37:50]
          + [s5a, s5b]
          + orig[50:52]
          + [s6a, s6b])
loop_to = orig.index(9)            # the first drop
# jump back at the end of the last pattern (B = command 2), on the 909 channel
last = P[s6b]
last[len(last) - 1][S909] = [None, None, None, 2, loop_to]

import os
SUB, CLAP, PUNCH = 25, 26, 27          # spare channels: a sub-kick under every kick, a 909 under every backbeat snare
I_KICK = 2
# the reference groove (Junkala's Title Screen, 16ths at 8 rows a beat):
# kick on rows 0, 4, 12, 16, 28 of every 32-row bar; snare stays on 8 and 24.
GROOVE = {0, 4, 12, 16, 28}
for g in P:
    kicks = [r for r, row in enumerate(g) if row[KICK] and row[KICK][1] == I_KICK]
    if len(kicks) < 4: continue                     # intros, breaks, builds: as they are
    note_ = g[kicks[0]][KICK][0]
    for r, row in enumerate(g):
        near = any(abs(r - k) <= 8 for k in kicks)   # keep his dropouts
        if row[KICK] and row[KICK][1] == I_KICK: row[KICK] = None
        if r % 32 in GROOVE and near:
            row[KICK] = [note_, I_KICK, None, None, None]
for g in P:
    for row in g:
        while len(row) < 64: row.append(None)
        k = row[KICK]
        if k and k[1] == I_KICK and k[0] is not None and k[0] < 120:
            row[SUB] = [k[0] - int(os.environ.get('DROP', '12')), I_KICK, int(os.environ.get('SUBV', '64')), None, None]
            if os.environ.get('PUNCHV', '56') != '0':
                row[PUNCH] = [k[0], I_KICK, int(os.environ.get('PUNCHV', '56')), None, None]
        s_ = row[HATSN]
        if s_ and s_[1] == I_SNARE and s_[0] is not None and s_[0] < 120:
            row[CLAP] = [60, I_909, int(os.environ.get('CLAPV', '56')), None, None]
others = int(os.environ.get('OTHERS', '42'))
chvol = {c: others for c in range(25) if c not in (KICK, HATSN, S909)}
chvol.update({KICK: 64, HATSN: 64, S909: 64, SUB: 64, CLAP: 64, PUNCH: 64})
itmod.write(t, os.environ.get('OUT', 'tunnel-vision-extended-heavy.it'), P, orders,
            mix_volume=int(os.environ.get('MV', '64')), chvol=chvol, chpan={SUB: 32, CLAP: 32, PUNCH: 32})
secs = sum(len(P[o]) for o in orders) * 0.05
print('orders', len(orders), 'new patterns', new, 'loop to order', loop_to,
      'length before the loop %.0f s (%d:%02d)' % (secs, secs // 60, secs % 60))
