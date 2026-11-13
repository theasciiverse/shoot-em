import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import java.io.*;
public class DecompileAll extends GhidraScript {
  public void run() throws Exception {
    String out = getScriptArgs()[0];
    DecompInterface d = new DecompInterface();
    d.openProgram(currentProgram);
    PrintWriter w = new PrintWriter(new FileWriter(out));
    for (Function f : currentProgram.getFunctionManager().getFunctions(true)) {
      if (f.isExternal() || f.isThunk()) continue;
      DecompileResults r = d.decompileFunction(f, 120, monitor);
      w.println("// ===== " + f.getName() + " @ " + f.getEntryPoint() + " size " + f.getBody().getNumAddresses());
      if (r.decompileCompleted()) w.println(r.getDecompiledFunction().getC()); else w.println("// failed: " + r.getErrorMessage());
    }
    w.close();
  }
}
