"""Extended mixes of three Pip Malt tracks (CC BY), in the way of the
Tunnel Vision extended mix: new sections built from each track's own
patterns and samples, drums strong and in front, the outro left out and
a jump back into the first full section."""
import copy, os, sys, itmod
from ittime import order_times

CFG = {
  'elevator': dict(kick=0, snare=1, drums=list(range(10)), bass=[15], rhythm=[15], leads=[13, 14, 10, 11, 12],
                   riff=(1, 2), chorus=(11, 12), insert_after=24, end_at=28, loop_to=1, gv=64, bpb=16),
  'mistake':  dict(kick=3, snare=4, drums=[3, 4, 5, 6], bass=[2], rhythm=[0, 1], leads=[7, 8, 9, 10, 11, 12],
                   riff=(9, 10), chorus=(13, 14), insert_after=38, end_at=43, loop_to=1, gv=112, bpb=16),
  'spindly':  dict(kick=5, snare=6, drums=[5, 6, 7, 8], bass=[4, 9], rhythm=[0, 2], leads=[1, 3],
                   riff=(6, 7), chorus=(20, 21), insert_after=31, end_at=36, loop_to=6, gv=112, bpb=16),
}
PUNCH = 30            # a spare channel: the kick doubled for punch

def first(g, ch):
    for row in g:
        c = row[ch]
        if c and c[0] is not None and c[0] < 120 and c[1]: return c[0], c[1]
    return None

def build(name):
    S = '/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad'
    cfg = CFG[name]
    t = itmod.read(f'{S}/pip/{name}.it')
    P = t.patterns
    orig = [o for o in t.orders if o < 200]
    K, SN = cfg['kick'], cfg['snare']
    kick = first(P[cfg['riff'][0]], K) or first(P[orig[3]], K)
    snare = first(P[cfg['riff'][0]], SN) or first(P[orig[5]], SN)
    for g in P:
        for row in g:
            while len(row) < 64: row.append(None)

    def cp(i): return copy.deepcopy(P[i])
    def mute(g, chans, rows=None):
        for r, row in enumerate(g):
            if rows is None or r in rows:
                for c in chans: row[c] = None
        return g
    def transpose(g, chans, k):
        for row in g:
            for c in chans:
                cell = row[c]
                if cell and cell[0] is not None and cell[0] < 120: cell[0] = min(119, cell[0] + k)
        return g
    def drums_thin(g):
        """kick on the bar's first beat and the snare's backbeat only"""
        for r, row in enumerate(g):
            for c in cfg['drums']:
                if c in (K, SN): continue
                row[c] = None
            if row[K] and r % cfg['bpb'] not in (0, cfg['bpb'] // 2 + 2): row[K] = None
        return g
    def build_roll(g):
        """kick on every beat, a snare roll that speeds up into the next pattern"""
        n = len(g)
        for r, row in enumerate(g):
            for c in cfg['drums']: row[c] = None
            if r % 4 == 0 and r < n - 4: row[K] = [kick[0], kick[1], 64, None, None]
            half = n // 2
            step = 4 if r < half else (2 if r < half + n // 4 else 1)
            if r >= n // 4 and (r - n // 4) % step == 0:
                row[SN] = [snare[0], snare[1], int(20 + 44 * r / n), None, None]
        return g
    def half_time(g):
        for r, row in enumerate(g):
            if row[SN] and row[SN][1] == snare[1]: row[SN] = None
            if r % 16 == 8: row[SN] = [snare[0], snare[1], 64, None, None]
            if row[K] and r % 16 not in (0, 6, 10): row[K] = None
            if r % 16 in (0, 6, 10) and not row[K]: row[K] = [kick[0], kick[1], 60, None, None]
        return g

    new = []
    def add(g):
        P.append(g); new.append(len(P) - 1); return len(P) - 1
    r1, r2 = cfg['riff']; c1, c2 = cfg['chorus']
    lead = cfg['leads']; keep = set(cfg['drums'] + cfg['bass'] + cfg['rhythm'])
    others = [c for c in range(16) if c not in keep]
    sections = [
        # breakdown: thin drums, bass and rhythm guitar
        add(drums_thin(mute(cp(r1), lead))), add(drums_thin(mute(cp(r2), lead))),
        # build: kick on every beat, a speeding snare roll, rhythm only
        add(mute(build_roll(cp(r1)), lead)), add(mute(build_roll(cp(r2)), lead)),
        # drop: the chorus with the lead an octave up
        add(transpose(cp(c1), lead, 12)), add(transpose(cp(c2), lead, 12)),
        # half-time riff
        add(half_time(cp(r1))), add(half_time(cp(r2))),
        # and the riff in full, lead lifted, before the song goes on
        add(transpose(cp(r1), lead, 12)), add(transpose(cp(r2), lead, 12)),
    ]
    ia, ea = cfg['insert_after'], cfg['end_at']
    orders = orig[:ia + 1] + sections + orig[ia + 1:ea + 1]
    # jump back at the end (B = command 2) on the last pattern's last row, a free channel
    last = copy.deepcopy(P[orders[-1]]); P.append(last); orders[-1] = len(P) - 1
    last[len(last) - 1][31] = [None, None, None, 2, cfg['loop_to']]
    # drums in front: the kick doubled for punch, other channels a little lower
    for o in (set() if os.environ.get('NOPUNCH') else set(orders)):
        for row in P[o]:
            k = row[K]
            if k and k[0] is not None and k[0] < 120 and k[1] == kick[1]:
                row[PUNCH] = [k[0], k[1], 56 if k[2] is None else min(64, k[2]), None, None]
    chvol = {c: 64 for c in cfg['drums']}
    chvol.update({c: int(os.environ.get('OTHERS', '46')) for c in range(16) if c not in cfg['drums']})
    chvol[PUNCH] = 64; chvol[31] = 64
    if os.environ.get('NOCHVOL'): chvol = {PUNCH: 64, 31: 64}
    pan = {PUNCH: 32, 31: 32}
    out = f'{S}/pip/{name}-extended.it'
    itmod.write(t, out, P, orders, global_volume=int(os.environ.get('GV', cfg['gv'])), chvol=chvol, chpan=pan)
    ts = order_times(t, orders, P)
    print(name, 'orders', len(orders), 'new patterns', len(new), 'length before the loop %d:%02d' % (ts[-1] // 60, ts[-1] % 60),
          'loop to order', cfg['loop_to'], 'kick', kick, 'snare', snare)
    return out, ts

if __name__ == '__main__':
    for n in sys.argv[1:]: build(n)
