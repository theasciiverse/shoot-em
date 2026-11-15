"""Girl From Mars (Asciiverse remix, heavy): Drozerix's intro unchanged, then
a slower, heavy, steady groove to shoot over: a deep kick on every beat, a
hard backbeat, low power chords on the beat, no hats or double kick, and
his busiest high parts left out."""
import os, collections
from common import *

SRC = '../shoot-em-music/modules/173935-girl_from_mars.xm'
DRUMS = {3, 4, 5}
MELODIC = {1, 2, 6, 7}
MEL = float(os.environ.get('MEL', '0.62'))

def build(out, bpm):
    b = Builder(SRC, 6)
    b.scale = float(os.environ.get('SCALE', '1.0'))
    x = b.x
    K, S, CR = b.ins['kick heavy'], b.ins['snare heavy'], b.ins['crash']
    TM, TL = b.ins['tom mid'], b.ins['tom lo']
    GO, GM, BS = b.ins['guitar open'], b.ins['guitar mute'], b.ins['bass']
    ck, cs, cc, gl, gr, cb = (b.base + i for i in range(6))

    pats = [b.widen(p) for p in x.patterns]
    orders = list(x.orders[:5])                      # his intro

    # the last intro pattern with a slow, heavy tom fill; the tempo eases down
    fill = b.widen(x.patterns[6])
    for r, t in ((48, TM), (52, TM), (56, TL), (58, TL), (60, TL), (62, S)):
        b.put(fill, r, cs, C4, t, 60)
    for r in range(32, 64, 8):
        b.put(fill, r, ck, C4, K, 50)
    steps = list(range(48, 64, 4))
    for k, r in enumerate(steps):
        b.fx(fill, r, cc, 0xF, round(128 + (bpm - 128) * (k + 1) / len(steps)))
    pats.append(fill); orders.append(len(pats) - 1)

    def busy_channels(p):
        """channels that run fast (more than 24 notes in 64 rows) or very high"""
        out = set()
        for c in range(x.chans):
            notes = [p[r][c][0] for r in range(len(p)) if 0 < p[r][c][0] < 97]
            if len(notes) > 24 or any(n >= note('C', 7) for n in notes):
                out.add(c)
        return out

    def heavy(src, first=False):
        p = b.strip(b.widen(x.patterns[src]), DRUMS)
        for c in busy_channels(x.patterns[src]):
            for r in range(len(p)):
                if p[r][c][1] not in DRUMS: p[r][c] = [0, 0, 0, 0, 0]
        # his melody a step back and lower: channels that sit high go down an octave
        for c in range(x.chans):
            ns = sorted(p[r][c][0] for r in range(len(p)) if 0 < p[r][c][0] < 97)
            high = ns and ns[len(ns) // 2] >= note('C', 6)
            for r in range(len(p)):
                cell = p[r][c]
                if high and 0 < cell[0] < 97: cell[0] -= 12
                if cell[0] and cell[0] < 97:
                    v = cell[2] - 0x10 if 0x10 <= cell[2] <= 0x50 else 64
                    cell[2] = V(round(v * MEL))
        roots = roots16(x, x.patterns[src], MELODIC)
        if first: b.fx(p, 0, cc, 0xF, bpm)
        for r in range(len(p)):
            bp = r % 32
            if bp in GROOVE_KICK: b.put(p, r, ck, C4, K, 64 if bp in (0, 16) else 56)   # the reference groove
            if bp in GROOVE_SNARE: b.put(p, r, cs, C4, S, 56)        # backbeat
        if src in (4, 9, 7, 15, 19, 2): b.put(p, 0, cc, C4, CR, 30)
        for blk, root in enumerate(roots):
            n = LOW[root]; r0 = blk * 16
            b.put(p, r0, gl, n, GO, 26, 0x8, 0x50)
            b.put(p, r0, gr, n, GO, 26, 0x8, 0xB0)
            b.put(p, r0, cb, n, BS, 40)
            for h in (8,):                                           # one chug on the next beat
                b.put(p, r0 + h, gl, n, GM, 34, 0x8, 0x50)
                b.put(p, r0 + h, gr, n, GM, 34, 0x8, 0xB0)
            b.put(p, r0 + 15, cb, OFF, 0)
        return p

    seq = [4, 5, 9, 10, 7, 8, 13, 14, 15, 16, 17, 18, 19, 20, 21, 2, 6]
    restart = len(orders)
    for k, src in enumerate(seq):
        pats.append(heavy(src, first=(k == 0))); orders.append(len(pats) - 1)
    b.fx(pats[0], 0, cc, 0x10, int(os.environ.get('GV', '48')))
    b.write(out, pats, orders, restart, 128, 3, 'Girl From Mars heavy')
    print(out, 'orders', len(orders), 'restart', restart)

if __name__ == '__main__':
    build('gfm-heavy.xm', int(os.environ.get('BPM', '112')))
