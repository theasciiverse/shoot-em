"""Mecanum Overdrive (Asciiverse remix, heavy): Drozerix's track with our
drums building over it, modelled on Junkala's Title Screen groove: a deep
kick and a hard backbeat, steady, no hats or fast fills on top. His busy
chip kick and snare come out once our backbeat arrives. Loops at the peak."""
import os
from common import *

SRC = '../shoot-em-music/modules/175349-drozerix_-_mecanum_overdrive.xm'
CHIPDRUMS = {3, 4}            # his chip kick and snare on channel 2

def build(out):
    b = Builder(SRC, 3)
    b.scale = float(os.environ.get('SCALE', '1.0'))
    x = b.x
    K, S, CR = b.ins['kick heavy'], b.ins['snare heavy'], b.ins['crash']
    TL = b.ins['tom lo']
    ck, cs, cc = (b.base + i for i in range(3))

    def stage_of(o):
        if o < 9: return 0      # his intro and first build: as is
        if o < 18: return 1     # kick on the felt beats
        if o < 27: return 2     # + the backbeat snare
        if o < 44: return 3     # the full groove
        return 4                # the peak

    pats = [b.widen(p) for p in x.patterns]
    orders = []
    for o, src in enumerate(x.orders[:52]):          # his outro left out
        st = stage_of(o)
        p = b.widen(x.patterns[src])
        if st >= 2:
            p = b.strip(p, CHIPDRUMS, chans={2})
        for r in range(len(p)):
            bp = r % 32
            if st == 1 and bp in (0, 16): b.put(p, r, ck, C4, K, 54)
            if st >= 2 and bp in GROOVE_KICK and (st >= 3 or bp in (0, 16)):
                b.put(p, r, ck, C4, K, (64 if bp in (0, 16) else 54) if st >= 3 else 58)
            if st >= 2 and bp in GROOVE_SNARE: b.put(p, r, cs, C4, S, 50 if st == 2 else 58)
        if st >= 3 and o % 4 == 0: b.put(p, 0, cc, C4, CR, 34)
        if st == 4 and o % 2 == 0: b.put(p, 0, cc, C4, CR, 38)
        # one slow, heavy fill into each new stage
        if o in (17, 26, 43) and len(p) >= 32:
            for r in (len(p) - 8, len(p) - 4):
                b.put(p, r, cs, C4, TL, 60)
        pats.append(p); orders.append(len(pats) - 1)
    b.fx(pats[orders[0]], 0, cc, 0x10, int(os.environ.get('GV', '48')))
    b.write(out, pats, orders, 44, x.bpm, x.tempo, 'Mecanum Overdrive hvy')
    print(out, 'orders', len(orders), 'restart 44')

if __name__ == '__main__':
    build('mecanum-heavy.xm')
