"""Girl From Mars (Asciiverse remix): Drozerix's intro unchanged, a fill that
speeds up into a heavy metal section over his melody, looping there."""
import sys, copy
from common import *

SRC = '../shoot-em-music/modules/173935-girl_from_mars.xm'
DRUMS = {3, 4, 5}
MELODIC = {1, 2, 6, 7}

def build(out, bpm, groove):
    b = Builder(SRC, 6)
    b.scale = float(__import__('os').environ.get('SCALE', '0.62'))
    x = b.x
    K, S, H, CR, RI = (b.ins[k] for k in ('kick', 'snare', 'hat', 'crash', 'ride'))
    TH, TM, TL = b.ins['tom hi'], b.ins['tom mid'], b.ins['tom lo']
    GO, GM, BS = b.ins['guitar open'], b.ins['guitar mute'], b.ins['bass']
    ck, cs, cc, gl, gr, cb = (b.base + i for i in range(6))
    PL, PR = 0x8, 0x8

    pats = [b.widen(p) for p in x.patterns]          # originals, untouched
    orders = list(x.orders[:5])                      # 0,1,2,3,2: the intro

    # order 5 (pattern 6) with a fill that ramps the tempo up to the metal beat
    fill = b.widen(x.patterns[6])
    for r in range(32, 64, 4):
        b.put(fill, r, ck, C4, K, 44)
    for r in range(48, 58, 2):
        b.put(fill, r, cs, C4, S, 30 + (r - 48) * 3)
    for r, t in ((58, TH), (59, TH), (60, TM), (61, TM), (62, TL), (63, TL)):
        b.put(fill, r, cs, C4, t, 56)
    steps = list(range(48, 64, 2))
    for k, r in enumerate(steps):
        b.fx(fill, r, cc, 0xF, round(128 + (bpm - 128) * (k + 1) / len(steps)))
    pats.append(fill)
    orders.append(len(pats) - 1)

    def metal(src_index, half_time=False, first=False):
        p = b.strip(b.widen(x.patterns[src_index]), DRUMS)
        roots = roots16(x, x.patterns[src_index], MELODIC)
        rows = len(p)
        if first:
            b.fx(p, 0, cc, 0xF, bpm)
        b.put(p, 0, cc, C4, CR, 48)
        for r in range(rows):
            beat_pos = r % 8          # 8 rows a beat
            bar_pos = r % 32
            # kick
            if half_time:
                if r % 4 == 0: b.put(p, r, ck, C4, K, 60)
            elif groove == 'chug':
                if r % 2 == 0: b.put(p, r, ck, C4, K, 58 if beat_pos == 0 else 50)
            else:  # gallop: 8th and two 16ths
                if beat_pos in (0, 4, 6): b.put(p, r, ck, C4, K, 60 if beat_pos == 0 else 52)
            # snare
            if half_time:
                if bar_pos == 16: b.put(p, r, cs, C4, S, 62)
            elif bar_pos in (8, 24):
                b.put(p, r, cs, C4, S, 60)
            # ride on 8ths, a crash every bar on the downbeat
            if r % 32 == 0 and r: b.put(p, r, cc, C4, CR, 40)
            elif r % 4 == 0 and r: b.put(p, r, cc, C4, RI, 22)
        # guitars and bass, per 16-row block
        for blk, root in enumerate(roots):
            n = LOW[root]
            r0 = blk * 16
            b.put(p, r0, gl, n, GO, 34, 0x8, 0x40)
            b.put(p, r0, gr, n, GO, 34, 0x8, 0xC0)
            b.put(p, r0, cb, n, BS, 46)
            if half_time:
                hits = [8, 12]
            elif groove == 'chug':
                hits = list(range(4, 16, 2))
            else:
                hits = [4, 6, 8, 12, 14]
            b.put(p, r0 + hits[0], gl, OFF, 0); b.put(p, r0 + hits[0], gr, OFF, 0)
            for h in hits:
                b.put(p, r0 + h, gl, n, GM, 46, 0x8, 0x40)
                b.put(p, r0 + h, gr, n, GM, 46, 0x8, 0xC0)
            b.put(p, r0 + 15, cb, OFF, 0)
        return p

    b.fx(pats[0], 0, cc, 0x10, int(__import__('os').environ.get('GV', '44')))
    seq = [(4, False), (5, False), (9, False), (10, False), (7, False), (8, False),
           (13, False), (14, False), (15, True), (16, True), (17, True), (18, True),
           (19, False), (20, False), (21, False), (2, False), (6, False)]
    restart = len(orders)
    for k, (src, half) in enumerate(seq):
        pats.append(metal(src, half, first=(k == 0)))
        orders.append(len(pats) - 1)
    # the last pattern of the loop puts the tempo back for a clean loop point
    b.write(out, pats, orders, restart, 128, 3, 'Girl From Mars (rmx)')
    return orders, restart

if __name__ == '__main__':
    out, bpm, groove = sys.argv[1], int(sys.argv[2]), sys.argv[3]
    o, r = build(out, bpm, groove)
    print(out, 'orders', len(o), 'restart', r)
