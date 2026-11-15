import sys,wave,numpy as np
def load(p,start=0,secs=None):
    w=wave.open(p); sr=w.getframerate(); w.setpos(int(start*sr)); n=w.getnframes()-int(start*sr) if secs is None else int(secs*sr)
    return np.frombuffer(w.readframes(n),dtype=np.int16).reshape(-1,2).astype(float)/32768, sr
a,sr=load(sys.argv[1],float(sys.argv[3]) if len(sys.argv)>3 else 0, float(sys.argv[4]) if len(sys.argv)>4 else None)
b,_=load(sys.argv[2])
n=min(len(a),len(b)); a,b=a[:n].mean(1),b[:n].mean(1)
def envl(x,w=sr//10): return np.sqrt(np.add.reduceat(x**2,np.arange(0,len(x),w))/w)
ea,eb=envl(a),envl(b)
print('len %.1fs rms a %.3f b %.3f peak a %.3f b %.3f envelope corr %.3f'%(n/sr,np.sqrt((a**2).mean()),np.sqrt((b**2).mean()),np.abs(a).max(),np.abs(b).max(),np.corrcoef(ea,eb)[0,1]))
