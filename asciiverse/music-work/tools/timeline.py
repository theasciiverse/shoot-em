def order_times(x, orders=None, patterns=None):
    """start time in seconds of each order, following F (speed/BPM) commands"""
    orders = orders or x.orders; patterns = patterns or x.patterns
    speed, bpm = x.tempo, x.bpm
    t = 0.0; out = []
    for pi in orders:
        out.append(t)
        for row in patterns[pi]:
            for cell in row:
                if cell[3] == 0xF and cell[4]:
                    if cell[4] < 0x20: speed = cell[4]
                    else: bpm = cell[4]
            t += 2.5 * speed / bpm
    out.append(t)
    return out
