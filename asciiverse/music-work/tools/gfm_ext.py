"""Girl From Mars (Asciiverse extended mix): Drozerix's track (public domain)
lengthened from its own material in the way of the Tunnel Vision extended
mix: his intro as is, a Tunnel Vision style beat of our own (synthesised:
kick on every beat, open hat on the off-beat, snare on 2 and 4, ride on the
beat) in place of his chip drums, and new sections (dub, half time, builds
with snare rolls, drops with the lead lifted, a final peak), looping back
into the first full section."""
import os, copy
from common import *

SRC = '../shoot-em-music/modules/173935-girl_from_mars.xm'
DRUMS = {3, 4, 5}
MELODIC = {1, 2, 6, 7}

def build(out):
    b = Builder(SRC, 4)
    b.scale = float(os.environ.get('SCALE', '0.8'))
    x = b.x
    K, S, H, CR, RI = (b.ins[k] for k in ('kick', 'snare', 'hat', 'crash', 'ride'))
    TH, TM, TL = b.ins['tom hi'], b.ins['tom mid'], b.ins['tom lo']
    ck, cs, ch, cc = (b.base + i for i in range(4))
    N = x.chans

    def busy(src):
        """his fast arpeggio and sweep channels (more than 24 notes in 64 rows)"""
        p = x.patterns[src]; out = set()
        for c in range(N):
            notes = [p[r][c][0] for r in range(len(p)) if 0 < p[r][c][0] < 97]
            if len(notes) > 24: out.add(c)
        return out
    def leadch(src):
        """the melody channels: not drums, not the bass, median note highest"""
        p = x.patterns[src]; med = {}
        for c in range(N):
            notes = sorted(p[r][c][0] for r in range(len(p)) if 0 < p[r][c][0] < 97 and p[r][c][1] not in DRUMS)
            if notes: med[c] = notes[len(notes) // 2]
        return sorted(med, key=med.get)[-3:] if med else []
    def base(src):
        return b.strip(b.widen(x.patterns[src]), DRUMS)
    def mute(p, chans):
        for row in p:
            for c in chans: row[c] = [0, 0, 0, 0, 0]
        return p
    def transpose(p, chans, k):
        for row in p:
            for c in chans:
                if 0 < row[c][0] < 97: row[c][0] = max(1, min(96, row[c][0] + k))
        return p
    def beat(p, crash=False, ride=True, half=False, kickvol=60):
        for r in range(len(p)):
            bp = r % 32
            if half:
                if bp in (0, 20): b.put(p, r, ck, C4, K, kickvol)
                if bp == 16: b.put(p, r, cs, C4, S, 58)
            else:
                if r % 8 == 0: b.put(p, r, ck, C4, K, kickvol)
                if bp in (8, 24): b.put(p, r, cs, C4, S, 56)
                if r % 8 == 4: b.put(p, r, ch, C4, H, 30)
                elif ride and r % 8 == 0 and r: b.put(p, r, cc, C4, RI, 18)
        if crash: b.put(p, 0, cc, C4, CR, 40)
        return p
    def roll(p, start=32):
        n = len(p)
        for r in range(start, n):
            step = 4 if r < start + 16 else (2 if r < start + 24 else 1)
            if (r - start) % step == 0: b.put(p, r, cs, C4, S, int(20 + 40 * (r - start) / (n - start)))
        for r in range(n - 4, n):
            p[r][ck] = [0, 0, 0, 0, 0]
        return p
    def fill(p):
        n = len(p)
        for dr, t in ((6, TH), (5, TH), (4, TM), (3, TM), (2, TL), (1, TL)):
            b.put(p, n - dr, cs, C4, t, 52)
        return p

    pats = [b.widen(p) for p in x.patterns]
    orders = list(x.orders[:6])                     # his intro, as is (0, 1, 2, 3, 2, 6)
    # the last intro pattern leads in with a fill
    intro_last = fill(b.widen(x.patterns[6]))
    pats.append(intro_last); orders[5] = len(pats) - 1
    restart = len(orders)
    def add(p): pats.append(p); orders.append(len(pats) - 1)

    # A: his main section with the beat
    for k, src in enumerate((4, 5, 9, 10, 7, 8, 13, 14)):
        add(beat(base(src), crash=(k % 2 == 0)))
    # B: dub, drums and his bass and pads, the busy and lead channels out
    for src in (7, 8):
        add(beat(mute(base(src), busy(src) | set(leadch(src))), ride=False))
    # C: half time under his melody lifted an octave
    for src in (9, 10):
        add(beat(transpose(base(src), leadch(src), 12), half=True, crash=(src == 9)))
    for src in (13, 14):
        add(beat(transpose(base(src), leadch(src), 12), half=True))
    # D: build, his arpeggio section with the kick only and a snare roll
    add(mute(base(15), set()))
    for r in range(0, 64, 8): b.put(pats[-1], r, ck, C4, K, 50)
    add(roll(beat(base(16), ride=False, kickvol=56), 16))
    # E: the drop, his main section with the lead an octave up
    for k, src in enumerate((7, 8, 9, 10)):
        add(beat(transpose(base(src), leadch(src), 12), crash=(k % 2 == 0)))
    # F: his arpeggio section with the beat
    for k, src in enumerate((15, 16, 17, 18, 19, 20, 21)):
        add(beat(base(src), crash=(k % 4 == 0)))
    # A again: his main section, ride doubled
    for k, src in enumerate((4, 5, 9, 10)):
        p = beat(base(src), crash=(k % 2 == 0))
        for r in range(4, 64, 8): b.put(p, r, cc, C4, RI, 20)
        add(p)
    # a second dub, on his pads
    for src in (13, 14):
        add(beat(mute(base(src), busy(src) | set(leadch(src))), ride=False))
    # G: a breakdown on his intro chords, kick only on 1 and 3, then a build
    for src in (22, 23):
        p = base(src)
        for r in range(0, 64, 16): b.put(p, r, ck, C4, K, 46)
        add(p)
    add(roll(beat(base(24), ride=False), 32))
    # his intro melody over the beat, then another build
    for src in (2, 6):
        add(beat(base(src), crash=(src == 2)))
    add(roll(beat(base(3), ride=False), 32))
    # H: final peak, the main section with everything and the lead lifted
    for k, src in enumerate((4, 5, 9, 10, 7, 8)):
        add(beat(transpose(base(src), leadch(src), 12) if k >= 2 else base(src), crash=(k % 2 == 0)))
    for src in (13, 14):
        add(beat(transpose(base(src), leadch(src), 12)))
    add(fill(beat(base(15))))
    add(beat(base(16)))
    b.fx(pats[orders[0]], 0, cc, 0x10, int(os.environ.get('GV', '48')))
    b.write(out, pats, orders, restart, x.bpm, x.tempo, 'Girl From Mars ext')
    rows = sum(len(pats[o]) for o in orders)
    secs = rows * 2.5 * 3 / 128
    print(out, 'orders', len(orders), 'restart', restart, 'length %d:%02d' % (secs // 60, secs % 60))

if __name__ == '__main__':
    build('gfm-extended.xm')
