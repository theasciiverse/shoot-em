import sys,subprocess,wave,numpy as np,itmod,os
S='/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad'
name=sys.argv[1]; secs=float(sys.argv[2]) if len(sys.argv)>2 else 45
t=itmod.read(f'{S}/pip/{name}.it')
orders=[o for o in t.orders if o<200]
used=sorted({c for o in orders for row in t.patterns[o] for c,cell in enumerate(row) if cell})
def load(p):
    w=wave.open(p); a=np.frombuffer(w.readframes(w.getnframes()),dtype=np.int16).reshape(-1,2).astype(float).mean(1)/32768
    return a
def env(x,sr=44100): w=sr//20; n=len(x)//w; return np.sqrt((x[:n*w].reshape(n,w)**2).mean(1))
def render(path,tag):
    subprocess.run(['ffmpeg','-hide_banner','-loglevel','error','-y','-i',path,'-t',str(secs),'-ar','44100','-ac','2',f'/tmp/claude-1000/-home-david-store-code-asciiverse/b526938b-385b-4d4e-8a3f-1d932f3d80dc/scratchpad/pip/{tag}-lo.wav'])
    subprocess.run([f'{S}/xmrender/target/release/xmrender',path,f'{S}/pip/{tag}-xm.wav',str(secs),'0'],stderr=subprocess.DEVNULL)
    a=load(f'{S}/pip/{tag}-lo.wav'); b=load(f'{S}/pip/{tag}-xm.wav'); n=min(len(a),len(b))
    ea,eb=env(a[:n]),env(b[:n])
    c=np.corrcoef(ea,eb)[0,1] if ea.std()>0 and eb.std()>0 else float('nan')
    return np.sqrt((a**2).mean()),np.sqrt((b**2).mean()),c
ra,rb,c=render(f'{S}/pip/{name}.it',name)
print('%s all channels: libopenmpt rms %.3f xmrs rms %.3f ratio %.2f corr %.2f'%(name,ra,rb,rb/max(ra,1e-9),c))
for ch in used:
    t.patterns=[p for p in t.patterns]
    itmod.write(t,f'{S}/pip/solo.it',t.patterns,orders,chvol={c:(64 if c==ch else 0) for c in used})
    ra,rb,c=render(f'{S}/pip/solo.it','solo')
    if ra<1e-4 and rb<1e-4: continue
    print('  ch %2d: libopenmpt rms %.3f xmrs rms %.3f ratio %.2f corr %.2f'%(ch,ra,rb,rb/max(ra,1e-9),c))
