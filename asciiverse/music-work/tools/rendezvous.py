"""Digital Rendezvous (Asciiverse remix): Drozerix's track with a driving
drumbeat of our own in place of the thin chip kit, lighter in his
breakdowns, looping back to the first beat section."""
import os
from common import *

SRC = '../shoot-em-music/modules/180821-drozerix_-_digital_rendezvous.xm'
CHIP = {6, 7, 8}
INTRO = 3                          # orders 0 to 2: his intro, as is
BREAKDOWN = {12, 13, 14, 15, 21, 22, 23, 24, 25, 37}   # patterns without his kick

def build(out):
    b = Builder(SRC, 4)
    b.scale = float(os.environ.get('SCALE', '0.6'))
    x = b.x
    K, S, H, CR, RI = (b.ins[k] for k in ('kick', 'snare', 'hat', 'crash', 'ride'))
    TH, TM, TL = b.ins['tom hi'], b.ins['tom mid'], b.ins['tom lo']
    ck, cs, ch, cc = (b.base + i for i in range(4))
    pats = [b.widen(p) for p in x.patterns]
    orders = list(x.orders[:INTRO])
    src = x.orders[INTRO:]
    for k, pi in enumerate(src):
        p = b.strip(b.widen(x.patterns[pi]), CHIP, chans={0, 1, 15})
        n = len(p)
        light = pi in BREAKDOWN
        nxt = src[k + 1] if k + 1 < len(src) else src[0]
        for r in range(n):
            bp = r % 32
            if light:
                if bp in (0, 16): b.put(p, r, ck, C4, K, 44)
                if r % 4 == 0: b.put(p, r, ch, C4, H, 22 if r % 8 else 28)
            else:
                if r % 8 == 0: b.put(p, r, ck, C4, K, 58)
                if bp in (8, 24): b.put(p, r, cs, C4, S, 56)
                if r % 8 == 4: b.put(p, r, cc, C4, RI, 30)          # off-beat
                if r % 2 == 0 and r % 8 != 4: b.put(p, r, ch, C4, H, 26 if r % 4 == 0 else 14)
        if not light and (k == 0 or x.patterns[src[k - 1]] is not None and src[k - 1] in BREAKDOWN):
            b.put(p, 0, cc, C4, CR, 46)
        elif not light and k % 4 == 0:
            b.put(p, 0, cc, C4, CR, 40)
        # a fill when the next pattern changes section
        if n >= 16 and ((nxt in BREAKDOWN) != light or k % 8 == 7):
            for dr, t in ((8, S), (6, S), (4, TH), (3, TM), (2, TL), (1, TL)):
                b.put(p, n - dr, cs, C4, t, 52)
        pats.append(p); orders.append(len(pats) - 1)
    b.fx(pats[orders[0]], 0, cc, 0x10, int(os.environ.get('GV', '48')))
    b.write(out, pats, orders, INTRO, x.bpm, x.tempo, 'Digital Rendezvous rmx')
    print(out, 'orders', len(orders), 'restart', INTRO)

if __name__ == '__main__':
    build('digital-rendezvous-remix.xm')
