"""Crush (Asciiverse remix): Drozerix's opening unchanged, a walk into a
solid drum beat (ours, synthesised), looping on the beat section for the
menu."""
import os
from common import *

SRC = '../shoot-em-music/modules/179581-drozerix_-_crush.xm'
CHIP = {9, 10, 12}

def build(out):
    b = Builder(SRC, 4)
    b.scale = float(os.environ.get('SCALE', '0.6'))
    x = b.x
    K, S, H, CR = (b.ins[k] for k in ('kick', 'snare', 'hat', 'crash'))
    TH, TM, TL = b.ins['tom hi'], b.ins['tom mid'], b.ins['tom lo']
    ck, cs, ch, cc = (b.base + i for i in range(4))

    def strip(p):
        return b.strip(p, CHIP, chans={10, 11})

    def beat(p, crash=False, fill=False):
        for r in range(len(p)):
            bp = r % 32
            if bp in (0, 16, 22): b.put(p, r, ck, C4, K, 60 if bp != 22 else 50)
            if bp in (8, 24): b.put(p, r, cs, C4, S, 58)
            if r % 4 == 0: b.put(p, r, ch, C4, H, 34 if r % 8 == 0 else 26)
            elif r % 2 == 0: b.put(p, r, ch, C4, H, 14)
        if crash: b.put(p, 0, cc, C4, CR, 44)
        if fill:
            for r, t in ((56, S), (58, S), (59, S), (60, TH), (61, TM), (62, TL), (63, TL)):
                b.put(p, r, cs, C4, t, 54)
        return p

    pats = [b.widen(p) for p in x.patterns]
    orders = list(x.orders[:6])                       # the opening, as is

    # walk-in 1 (his pattern 2): the kick fades in on the beat, hats join halfway
    w1 = strip(b.widen(x.patterns[2]))
    for r in range(0, 64, 8):
        b.put(w1, r, ck, C4, K, 28 + r // 2)
    for r in range(32, 64, 4):
        b.put(w1, r, ch, C4, H, 14 + (r - 32) // 2)
    pats.append(w1); orders.append(len(pats) - 1)
    # walk-in 2 (his pattern 3): the full beat with a fill into the beat section
    w2 = beat(strip(b.widen(x.patterns[3])), fill=True)
    pats.append(w2); orders.append(len(pats) - 1)

    # global volume for headroom, set at the very start (G, applies to the whole song)
    b.fx(pats[orders[0]], 0, cc, 0x10, int(os.environ.get('GV', '44')))
    restart = len(orders)
    rest = x.orders[8:]
    for k, src in enumerate(rest):
        p = beat(strip(b.widen(x.patterns[src])), crash=(k % 2 == 0), fill=(k % 4 == 3))
        pats.append(p); orders.append(len(pats) - 1)
    b.write(out, pats, orders, restart, x.bpm, x.tempo, 'Crush (rmx)')
    rows = sum(len(pats[o]) for o in orders)
    print(out, 'orders', len(orders), 'restart', restart, 'about %.0f s' % (rows * 2.5 * 2 / 120))

if __name__ == '__main__':
    build('crush-remix.xm')
