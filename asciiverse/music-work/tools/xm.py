import struct

class XM:
    pass

def u8(b,o): return b[o]
def u16(b,o): return struct.unpack_from('<H',b,o)[0]
def u32(b,o): return struct.unpack_from('<I',b,o)[0]

def unpack_pattern(data, rows, chans):
    cells=[]; i=0
    for r in range(rows):
        row=[]
        for c in range(chans):
            b=data[i]; i+=1
            if b & 0x80:
                note=ins=vol=fx=par=0
                if b&1: note=data[i]; i+=1
                if b&2: ins=data[i]; i+=1
                if b&4: vol=data[i]; i+=1
                if b&8: fx=data[i]; i+=1
                if b&16: par=data[i]; i+=1
            else:
                note=b; ins,vol,fx,par=data[i],data[i+1],data[i+2],data[i+3]; i+=4
            row.append([note,ins,vol,fx,par])
        cells.append(row)
    return cells

def pack_pattern(cells):
    out=bytearray()
    for row in cells:
        for note,ins,vol,fx,par in row:
            if not (note or ins or vol or fx or par):
                out.append(0x80); continue
            flags=0x80; body=bytearray()
            for bit,v in ((1,note),(2,ins),(4,vol),(8,fx),(16,par)):
                if v: flags|=bit; body.append(v)
            out.append(flags); out+=body
    return bytes(out)

def read(path):
    b=open(path,'rb').read()
    x=XM()
    x.raw=b
    x.name=b[17:37]
    hsize=u32(b,60)
    x.hdr=bytearray(b[60:60+hsize])
    x.songlen=u16(b,64); x.restart=u16(b,66); x.chans=u16(b,68)
    x.npat=u16(b,70); x.nins=u16(b,72); x.flags=u16(b,74); x.tempo=u16(b,76); x.bpm=u16(b,78)
    x.orders=list(b[80:80+x.songlen])
    o=60+hsize
    x.patterns=[]
    for p in range(x.npat):
        phl=u32(b,o); rows=u16(b,o+5); psize=u16(b,o+7)
        data=b[o+phl:o+phl+psize]
        cells=unpack_pattern(data,rows,x.chans) if psize else [[[0,0,0,0,0] for _ in range(x.chans)] for _ in range(rows)]
        x.patterns.append(cells)
        o+=phl+psize
    x.instruments_raw=b[o:]  # instruments verbatim, appended after patterns
    # parse instrument names and sample counts
    x.inst=[]
    q=o
    for i in range(x.nins):
        isz=u32(b,q); name=b[q+4:q+26].rstrip(b'\0').decode('latin1'); ns=u16(b,q+27)
        start=q; q+=isz
        if ns:
            shs=u32(b,start+29)
            lens=[]
            for s in range(ns):
                lens.append(u32(b,q)); q+=40
            q+=sum(lens)
        x.inst.append((name,ns,start,q))
    x.inst_end=q
    return x

def write(x, path, patterns, orders, chans, bpm, tempo, extra_instruments=b'', n_extra=0, restart=0, name=None):
    b=x.raw
    out=bytearray(b[:60])
    if name:
        nm=name.encode('latin1')[:20].ljust(20,b' ')
        out[17:37]=nm
    hsize=276
    hdr=bytearray(hsize)
    struct.pack_into('<IHHHHHHHH',hdr,0,hsize,len(orders),restart,chans,len(patterns),x.nins+n_extra,x.flags,tempo,bpm)
    hdr[20:20+len(orders)]=bytes(orders)
    out+=hdr
    for cells in patterns:
        data=pack_pattern(cells)
        out+=struct.pack('<IBHH',9,0,len(cells),len(data))+data
    out+=b[x.inst[0][2]:x.inst_end] if x.nins else b''
    out+=extra_instruments
    open(path,'wb').write(out)
