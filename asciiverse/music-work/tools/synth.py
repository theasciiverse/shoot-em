"""Our own synthesised instruments (no third-party samples): drums, a
distorted power-chord guitar (open and palm-muted) and a bass, written as
XM instruments with 16-bit samples."""
import numpy as np, struct
from scipy.signal import butter, sosfilt

REL, FINE = 29, -28
R = 8363 * 2 ** ((REL + FINE / 128) / 12)      # playback rate of note C-4
C4 = 261.6255653
rng = np.random.default_rng(1234)

def env(n, a, d, sr=R):
    t = np.arange(n) / sr
    e = np.exp(-t / d)
    na = max(1, int(a * sr))
    e[:na] *= np.linspace(0, 1, na)
    return e

def lp(x, f, o=2): return sosfilt(butter(o, f, 'low', fs=R, output='sos'), x)
def hp(x, f, o=2): return sosfilt(butter(o, f, 'high', fs=R, output='sos'), x)
def bp(x, lo, hi, o=2): return sosfilt(butter(o, [lo, hi], 'band', fs=R, output='sos'), x)

def norm(x, peak=0.95):
    m = np.max(np.abs(x)) or 1
    return x / m * peak

def kick():
    n = int(0.35 * R); t = np.arange(n) / R
    f = 45 + 120 * np.exp(-t / 0.035)
    ph = 2 * np.pi * np.cumsum(f) / R
    body = np.sin(ph) * env(n, 0.001, 0.16)
    click = hp(rng.standard_normal(n), 2500) * env(n, 0.0005, 0.004) * 0.6
    return norm(np.tanh(2.2 * (body + click)))

def snare():
    n = int(0.32 * R); t = np.arange(n) / R
    tone = np.sin(2 * np.pi * 185 * t) * env(n, 0.001, 0.05) + 0.5 * np.sin(2 * np.pi * 330 * t) * env(n, 0.001, 0.03)
    noise = bp(rng.standard_normal(n), 1200, 9000) * env(n, 0.001, 0.11)
    return norm(np.tanh(1.6 * (0.8 * tone + noise)))

def hat(d=0.045):
    n = int(0.25 * R)
    x = hp(rng.standard_normal(n), 7000, 4) * env(n, 0.0005, d)
    return norm(x, 0.8)

def crash():
    n = int(2.2 * R)
    x = hp(rng.standard_normal(n), 3500, 2) * env(n, 0.002, 0.7)
    x += 0.3 * bp(rng.standard_normal(n), 400, 1500) * env(n, 0.002, 0.25)
    return norm(x, 0.9)

def ride():
    n = int(0.6 * R); t = np.arange(n) / R
    metal = sum(np.sin(2 * np.pi * f * t) for f in (3150, 4210, 5320, 6510)) / 4
    x = (0.6 * metal + 0.5 * hp(rng.standard_normal(n), 6000)) * env(n, 0.001, 0.18)
    return norm(x, 0.8)

def tom(f0):
    n = int(0.45 * R); t = np.arange(n) / R
    f = f0 * (1 + 0.6 * np.exp(-t / 0.05))
    x = np.sin(2 * np.pi * np.cumsum(f) / R) * env(n, 0.001, 0.18)
    x += 0.15 * bp(rng.standard_normal(n), 200, 3000) * env(n, 0.001, 0.03)
    return norm(np.tanh(1.5 * x))

def saw(f, n, phase=0.0):
    """band-limited saw by additive synthesis"""
    t = np.arange(n) / R
    x = np.zeros(n)
    k = 1
    while k * f < R / 2.2 and k < 60:
        x += np.sin(2 * np.pi * k * f * t + k * phase) / k
        k += 1
    return x

def guitar_cab(x):
    x = hp(x, 90)
    x = lp(x, 4800, 4)
    # a little scoop in the mids, the "metal" sound
    x = x - 0.35 * bp(x, 500, 900)
    return x

def power_chord(f, n):
    # root, fifth, octave, two slightly detuned saws per voice
    x = sum(saw(f * m, n) + saw(f * m * 1.003, n, 0.7) for m in (1, 1.5, 2)) / 6
    return x

def guitar_open():
    """sustained distorted power chord at C-4, looped seamlessly"""
    loop_len = 6742
    cycles = round(loop_len * C4 / R)
    f = cycles * R / loop_len                     # exact so the loop is seamless
    # detune of 1.003 breaks periodicity: use exact multiples instead for the loop
    pre = int(0.6 * R)
    n = pre + loop_len
    t = np.arange(n) / R
    x = sum(sum(np.sin(2*np.pi*k*f*m*t)/k for k in range(1, int(R/2.2/(f*m)))) for m in (1, 1.5, 2))
    x = x / 3
    x = np.tanh(18 * x)
    x = guitar_cab(x)
    # make the loop seamless: crossfade the end into the loop start region
    y = norm(x[-(loop_len + int(0.6*R) - pre):] if False else x, 0.9)
    return y, pre, loop_len

def guitar_mute():
    """palm-muted chug: short, thumpy power chord"""
    n = int(0.22 * R)
    x = power_chord(C4, n)
    x = np.tanh(14 * x * env(n, 0.002, 0.06))
    x = lp(x, 2600, 2)
    x = guitar_cab(x) * env(n, 0.001, 0.09)
    return norm(x, 0.9)

def bass():
    loop_len = 6742
    cycles = round(loop_len * (C4 / 2) / R)
    f = cycles * R / loop_len
    pre = int(0.3 * R); n = pre + loop_len
    t = np.arange(n) / R
    x = sum(np.sin(2*np.pi*k*f*t)/k for k in range(1, 30)) * 0.6 + np.sin(2*np.pi*f/2*t*2)*0.0
    x = np.tanh(2.5 * x)
    x = lp(x, 1400, 2)
    # make the head a bit punchier
    x[:pre] *= np.linspace(1.4, 1.0, pre)
    return norm(x, 0.9), pre, loop_len


def kick_heavy():
    """big, deep kick: a low sine body you feel, a punchy attack, long tail"""
    n = int(0.6 * R); t = np.arange(n) / R
    f = 38 + 95 * np.exp(-t / 0.05) + 40 * np.exp(-t / 0.008)
    ph = 2 * np.pi * np.cumsum(f) / R
    body = np.sin(ph) * env(n, 0.001, 0.32)
    sub = np.sin(2 * np.pi * 42 * t) * env(n, 0.005, 0.38) * 0.5
    punch = lp(rng.standard_normal(n), 1800) * env(n, 0.0005, 0.012) * 0.8
    x = np.tanh(2.8 * (body + sub + punch))
    return norm(lp(x, 6000, 2), 0.97)

def snare_heavy():
    """hard backbeat snare: a low body and a long, bright crack"""
    n = int(0.45 * R); t = np.arange(n) / R
    f = 160 * (1 + 0.5 * np.exp(-t / 0.01))
    tone = np.sin(2 * np.pi * np.cumsum(f) / R) * env(n, 0.001, 0.08)
    crack = bp(rng.standard_normal(n), 900, 7000) * env(n, 0.001, 0.17)
    snap = hp(rng.standard_normal(n), 3000) * env(n, 0.0005, 0.02) * 0.7
    x = np.tanh(2.4 * (0.9 * tone + crack + snap))
    return norm(x, 0.95)

def to16(x):
    return np.clip(np.round(x * 32767), -32768, 32767).astype(np.int16)

def xm_instrument(name, data, loop=None, vol=64, pan=128, rel=REL, fine=FINE, fadeout=0, volenv=None):
    """one instrument with one 16-bit sample. loop=(start,len) for forward loop.
    volenv: list of (tick, value) points, sustain at last point."""
    s = to16(data)
    nbytes = len(s) * 2
    hdr = bytearray(263)
    struct.pack_into('<I', hdr, 0, 263)
    hdr[4:26] = name.encode('latin1')[:22].ljust(22, b'\0')
    hdr[26] = 0
    struct.pack_into('<HI', hdr, 27, 1, 40)
    # keymap all zeros (sample 0)
    if volenv:
        for i, (tk, v) in enumerate(volenv[:12]):
            struct.pack_into('<HH', hdr, 129 + i * 4, tk, v)
        hdr[225] = len(volenv)
        hdr[227] = len(volenv) - 1            # sustain point: last
        hdr[233] = 1 | 2                       # on, sustain
    struct.pack_into('<H', hdr, 239, fadeout)
    sh = bytearray(40)
    ls, ll = (loop if loop else (0, 0))
    typ = 0x10 | (1 if loop else 0)
    struct.pack_into('<IIIBbBBbB', sh, 0, nbytes, ls * 2, ll * 2, vol, fine, typ, pan, rel, 0)
    sh[18:40] = name.encode('latin1')[:22].ljust(22, b'\0')
    delta = np.diff(np.concatenate([[0], s.astype(np.int32)])).astype(np.int16)
    return bytes(hdr) + bytes(sh) + delta.tobytes()

def bank():
    """returns list of (name, bytes) in a fixed order"""
    go, gpre, glen = guitar_open()
    bs, bpre, blen = bass()
    return [
        ('kick (Asciiverse synth)', xm_instrument('kick', kick())),
        ('snare (Asciiverse synth)', xm_instrument('snare', snare())),
        ('hat (Asciiverse synth)', xm_instrument('hat', hat())),
        ('crash (Asciiverse synth)', xm_instrument('crash', crash())),
        ('ride (Asciiverse synth)', xm_instrument('ride', ride())),
        ('tom hi', xm_instrument('tom hi', tom(170))),
        ('tom mid', xm_instrument('tom mid', tom(125))),
        ('tom lo', xm_instrument('tom lo', tom(90))),
        ('guitar open', xm_instrument('guitar open', go, loop=(gpre, glen))),
        ('guitar mute', xm_instrument('guitar mute', guitar_mute())),
        ('bass', xm_instrument('bass', bs, loop=(bpre, blen))),
        ('kick heavy', xm_instrument('kick heavy', kick_heavy())),
        ('snare heavy', xm_instrument('snare heavy', snare_heavy())),
    ]
