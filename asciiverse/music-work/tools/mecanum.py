"""Mecanum Overdrive (Asciiverse remix): Drozerix's track with our own drums
building over it (none, then kick, snare, hats, fills, a heavy full beat),
looping at the peak."""
import os
from common import *

SRC = '../shoot-em-music/modules/175349-drozerix_-_mecanum_overdrive.xm'

def build(out):
    b = Builder(SRC, 4)
    b.scale = float(os.environ.get('SCALE', '0.6'))
    x = b.x
    K, S, H, CR, RI = (b.ins[k] for k in ('kick', 'snare', 'hat', 'crash', 'ride'))
    TH, TM, TL = b.ins['tom hi'], b.ins['tom mid'], b.ins['tom lo']
    ck, cs, ch, cc = (b.base + i for i in range(4))

    def stage_of(o):
        if o < 9: return 0
        if o < 18: return 1
        if o < 27: return 2
        if o < 34: return 3
        if o < 42: return 4
        if o < 44: return 5     # his acceleration: a roll
        return 6

    def fill(p):
        n = len(p)
        if n < 16: return
        for k, (dr, t) in enumerate(((8, S), (6, S), (4, TH), (3, TM), (2, TL), (1, TL))):
            b.put(p, n - dr, cs, C4, t, 50)

    pats = [b.widen(p) for p in x.patterns]
    orders = []
    src_orders = x.orders[:52]                 # his outro (orders 52 to 55) left out
    for o, src in enumerate(src_orders):
        st = stage_of(o)
        p = b.widen(x.patterns[src])
        n = len(p)
        for r in range(n):
            bp = r % 32
            if st >= 1:
                if st == 6:
                    if r % 2 == 0: b.put(p, r, ck, C4, K, 56 if r % 8 == 0 else 44)
                elif st >= 4 and bp in (0, 16, 20):
                    b.put(p, r, ck, C4, K, 54)
                elif r % 8 == 0:
                    b.put(p, r, ck, C4, K, 34 + 6 * st)
            if st >= 2 and st != 5 and bp in (8, 24):
                b.put(p, r, cs, C4, S, 40 + 4 * st)
            if st in (3, 4) and r % 4 == 0 and r % 32:
                b.put(p, r, ch, C4, H, 26 if r % 8 else 32)
            if st == 4 and r % 4 == 2:
                b.put(p, r, ch, C4, H, 14)
            if st == 6 and r % 4 == 0 and r % 32:
                b.put(p, r, cc, C4, RI, 26)
        if st == 5:                             # roll through the acceleration
            for r in range(n):
                step = 4 if r < n // 2 else (2 if r < 3 * n // 4 else 1)
                if r % step == 0: b.put(p, r, cs, C4, S, 24 + int(36 * r / n))
        if st >= 3 and o % 2 == 0: b.put(p, 0, cc, C4, CR, 40)
        if st == 6: b.put(p, 0, cc, C4, CR, 46)
        if st in (3, 4) and o % 4 == 3: fill(p)
        if o == 26 or o == 33: fill(p)
        pats.append(p); orders.append(len(pats) - 1)
    b.fx(pats[orders[0]], 0, cc, 0x10, int(os.environ.get('GV', '48')))
    restart = 44
    b.write(out, pats, orders, restart, x.bpm, x.tempo, 'Mecanum Overdrive rmx')
    print(out, 'orders', len(orders), 'restart', restart)

if __name__ == '__main__':
    build('mecanum-remix.xm')
