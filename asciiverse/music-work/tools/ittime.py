def order_times(t, orders, patterns=None):
    patterns = patterns or t.patterns
    speed, tempo = t.speed, t.tempo
    out = []; s = 0.0
    for o in orders:
        out.append(s)
        for row in patterns[o]:
            for cell in row:
                if cell and cell[3] == 1 and cell[4]: speed = cell[4]
                if cell and cell[3] == 20 and cell[4] and cell[4] >= 0x20: tempo = cell[4]
            s += 2.5 * speed / tempo
    out.append(s); return out
