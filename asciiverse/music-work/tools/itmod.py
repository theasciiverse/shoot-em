"""Minimal Impulse Tracker (.it) editing: read patterns and orders, add new
patterns and a new order list, and write the file back with every original
instrument, sample and message kept byte for byte (offsets relocated)."""
import struct

class IT:
    pass

def read(path):
    b = open(path, 'rb').read()
    t = IT(); t.raw = b
    (t.ordnum, t.insnum, t.smpnum, t.patnum, t.cwt, t.cmwt, t.flags, t.special) = struct.unpack_from('<8H', b, 0x20)
    t.gv, t.mv, t.speed, t.tempo = b[0x30], b[0x31], b[0x32], b[0x33]
    t.msglen, t.msgoff = struct.unpack_from('<HI', b, 0x36)
    o = 0xC0
    t.orders = list(b[o:o + t.ordnum]); o += t.ordnum
    t.insoff = list(struct.unpack_from('<%dI' % t.insnum, b, o)); o += 4 * t.insnum
    t.smpoff = list(struct.unpack_from('<%dI' % t.smpnum, b, o)); o += 4 * t.smpnum
    t.patoff = list(struct.unpack_from('<%dI' % t.patnum, b, o)); o += 4 * t.patnum
    t.tables_end = o
    t.patterns = []
    for po in t.patoff:
        if po == 0:
            t.patterns.append([[None] * 64 for _ in range(64)]); continue
        ln, rows = struct.unpack_from('<HH', b, po)
        t.patterns.append(unpack(b[po + 8:po + 8 + ln], rows))
    return t

# a cell is (note, ins, volpan, cmd, param) with None for "absent"
def unpack(data, rows):
    grid = [[None] * 64 for _ in range(rows)]
    lastmask = [0] * 64; last = [[None] * 5 for _ in range(64)]
    i = 0; r = 0
    while r < rows:
        cv = data[i]; i += 1
        if cv == 0:
            r += 1; continue
        ch = (cv - 1) & 63
        if cv & 128:
            lastmask[ch] = data[i]; i += 1
        m = lastmask[ch]
        cell = [None] * 5
        if m & 1: cell[0] = data[i]; last[ch][0] = data[i]; i += 1
        if m & 2: cell[1] = data[i]; last[ch][1] = data[i]; i += 1
        if m & 4: cell[2] = data[i]; last[ch][2] = data[i]; i += 1
        if m & 8: cell[3], cell[4] = data[i], data[i + 1]; last[ch][3], last[ch][4] = data[i], data[i + 1]; i += 2
        if m & 16: cell[0] = last[ch][0]
        if m & 32: cell[1] = last[ch][1]
        if m & 64: cell[2] = last[ch][2]
        if m & 128: cell[3], cell[4] = last[ch][3], last[ch][4]
        grid[r][ch] = cell
    return grid

def pack(grid):
    out = bytearray()
    for row in grid:
        for ch, cell in enumerate(row):
            if cell is None or all(v is None for v in cell): continue
            m = 0; body = bytearray()
            if cell[0] is not None: m |= 1; body.append(cell[0])
            if cell[1] is not None: m |= 2; body.append(cell[1])
            if cell[2] is not None: m |= 4; body.append(cell[2])
            if cell[3] is not None: m |= 8; body += bytes([cell[3], cell[4] or 0])
            out.append((ch + 1) | 128); out.append(m); out += body
        out.append(0)
    return bytes(out)

def write(t, path, patterns, orders, new_tempo=None, mix_volume=None, chvol=None, chpan=None, global_volume=None):
    """patterns: full list (originals first, unchanged ones may be passed as
    the parsed grids); orders: list of pattern numbers (255 end)."""
    b = t.raw
    orders = list(orders)
    if not orders or orders[-1] != 255: orders.append(255)
    if len(orders) % 2: orders.append(255)
    patnum = len(patterns)
    head = bytearray(b[:0xC0])
    struct.pack_into('<HHHH', head, 0x20, len(orders), t.insnum, t.smpnum, patnum)
    if new_tempo: head[0x33] = new_tempo
    if mix_volume: head[0x31] = mix_volume
    if global_volume: head[0x30] = global_volume
    for c, v in (chvol or {}).items(): head[0x80 + c] = v
    for c, v in (chpan or {}).items(): head[0x40 + c] = v
    tables_len = len(orders) + 4 * (t.insnum + t.smpnum + patnum)
    new_tables_end = 0xC0 + tables_len
    # new patterns go between the tables and the original body, so OpenMPT's
    # extension block after the last sample stays where libopenmpt looks for it
    packed = [pack(g) for g in patterns]
    pat_block = bytearray(); offs = []
    for g, data in zip(patterns, packed):
        offs.append(new_tables_end + len(pat_block))
        pat_block += struct.pack('<HH4x', len(data), len(g)) + data
    delta = new_tables_end + len(pat_block) - t.tables_end
    body = bytearray(b[t.tables_end:])
    for so in t.smpoff:
        rel = so - t.tables_end
        ptr = struct.unpack_from('<I', body, rel + 0x48)[0]
        if ptr: struct.pack_into('<I', body, rel + 0x48, ptr + delta)
    if t.msglen: struct.pack_into('<I', head, 0x38, t.msgoff + delta)
    out = bytearray(head) + bytes(orders)
    out += struct.pack('<%dI' % t.insnum, *[x + delta for x in t.insoff])
    out += struct.pack('<%dI' % t.smpnum, *[x + delta for x in t.smpoff])
    out += struct.pack('<%dI' % patnum, *offs)
    assert len(out) == new_tables_end
    out += pat_block + body
    open(path, 'wb').write(out)
