import copy, xm, synth

NAMES = ['C','C#','D','D#','E','F','F#','G','G#','A','A#','B']
def note(name, octave):
    return 1 + 12 * octave + NAMES.index(name)
C4 = note('C', 4)
OFF = 97
def V(v): return 0x10 + max(0, min(64, v))     # volume column

# instrument numbers are filled in by Builder
class Builder:
    def __init__(self, src, add_chans):
        self.x = xm.read(src)
        self.base = self.x.chans
        self.chans = self.x.chans + add_chans
        if self.chans % 2: self.chans += 1
        self.bank = synth.bank()
        self.ins = {}
        self.scale = 1.0
        for k, (name, _) in enumerate(self.bank):
            key = name.split(' (')[0]
            self.ins[key] = self.x.nins + 1 + k

    def empty(self, rows=64):
        return [[[0, 0, 0, 0, 0] for _ in range(self.chans)] for _ in range(rows)]

    def widen(self, cells):
        out = copy.deepcopy(cells)
        for row in out:
            while len(row) < self.chans: row.append([0, 0, 0, 0, 0])
        return out

    def strip(self, cells, drum_ins, chans=None):
        """clear every cell whose effective instrument is a drum"""
        last = [0] * self.chans
        for row in cells:
            for c, cell in enumerate(row[:self.base]):
                if cell[1]: last[c] = cell[1]
                if last[c] in drum_ins and (chans is None or c in chans):
                    if cell[3] == 0xF:          # keep tempo commands
                        row[c] = [0, 0, 0, 0xF, cell[4]]
                    else:
                        row[c] = [0, 0, 0, 0, 0]
        return cells

    def put(self, cells, r, c, n, ins, vol=None, fx=0, par=0):
        if r >= len(cells): return
        cell = cells[r][c]
        cell[0] = n; cell[1] = ins
        if vol is not None: cell[2] = V(round(vol * self.scale))
        if fx or par: cell[3] = fx; cell[4] = par

    def fx(self, cells, r, c, fx, par):
        cells[r][c][3] = fx; cells[r][c][4] = par

    def write(self, path, patterns, orders, restart, bpm, tempo, title):
        extra = b''.join(d for _, d in self.bank)
        xm.write(self.x, path, patterns, orders, self.chans, bpm, tempo,
                 extra_instruments=extra, n_extra=len(self.bank), restart=restart, name=title)

def roots16(x, pattern, melodic_ins):
    """root pitch class per 16-row block, by the lowest melodic note"""
    p = pattern; out = []
    last = [0] * len(p[0])
    for block in range(0, len(p), 16):
        ns = []
        for r in range(block, min(block + 16, len(p))):
            for c, (n, i, v, f, pp) in enumerate(p[r][:x.chans]):
                if i: last[c] = i
                if 0 < n < 97 and last[c] in melodic_ins: ns.append(n)
        out.append(NAMES[(min(ns) - 1) % 12] if ns else None)
    # fill gaps with the previous root
    prev = out[0] or 'A'
    for k in range(len(out)):
        if out[k] is None: out[k] = prev
        prev = out[k]
    return out

# low register for the guitars: E2..D#3
LOW = {'E': note('E',2), 'F': note('F',2), 'F#': note('F#',2), 'G': note('G',2), 'G#': note('G#',2),
       'A': note('A',2), 'A#': note('A#',2), 'B': note('B',2), 'C': note('C',3), 'C#': note('C#',3),
       'D': note('D',3), 'D#': note('D#',3)}

# the reference groove (Juhani Junkala's Title Screen, CC0), at 8 rows a beat:
# kick on rows 0, 4, 12, 16, 28 of each 32-row bar, snare on 8 and 24
GROOVE_KICK = (0, 4, 12, 16, 28)
GROOVE_SNARE = (8, 24)
