#!/usr/bin/env python3
"""Turns the intro logo routine (FUN_004098d0) in Ghidra's decompiled C
into src/logo.rs: every glBegin/glNormal3f/glVertex3f with its float
arguments (written by Ghidra as their bit patterns) decoded.

usage: extract_logo.py decompiled.c > src/logo.rs
"""
import re, struct, sys

src = open(sys.argv[1]).read()
body = re.search(r'// ===== FUN_004098d0 .*?(?=// ===== )', src, re.S).group(0)

def f(t):
    t = t.strip()
    if t.startswith('0x'):
        return struct.unpack('<f', struct.pack('<I', int(t, 16)))[0]
    return float(t)

def r(x):
    s = repr(round(x, 4))
    return s if '.' in s or 'e' in s else s + '.0'

prims, cur, normal = [], None, (0.0, 0.0, 1.0)
for line in body.splitlines():
    m = re.search(r'glBegin\((\d+)\)', line)
    if m:
        cur = {'mode': int(m.group(1)), 'v': []}
        prims.append(cur)
        continue
    m = re.search(r'glNormal3f\(([^)]*)\)', line)
    if m:
        normal = tuple(f(x) for x in m.group(1).split(','))
        continue
    m = re.search(r'glVertex3f\(([^)]*)\)', line)
    if m and len(m.group(1).split(',')) == 3:
        cur['v'].append((tuple(f(x) for x in m.group(1).split(',')), normal))

print("// The intro logo, extracted from the original program's drawing routine")
print("// (FUN_004098d0) by asciiverse/tools/extract_logo.py. Each entry is an OpenGL")
print("// primitive: (mode, [(vertex, normal)]), mode 4 triangles, 5 strip, 7 quads.")
print("pub const LOGO: &[(u8, &[([f32; 3], [f32; 3])])] = &[")
for p in prims:
    vs = ', '.join(f'([{r(a[0])}, {r(a[1])}, {r(a[2])}], [{r(n[0])}, {r(n[1])}, {r(n[2])}])' for a, n in p['v'])
    print(f'    ({p["mode"]}, &[{vs}]),')
print('];')
