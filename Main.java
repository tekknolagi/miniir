import java.util.Iterator;
import java.util.Collections;
import java.util.ArrayList;
import java.util.HashMap;

class Insn {
  Insn(Insn... operands) { this.operands = operands; this.subst = this; }
  Insn[] operands() { return operands; }
  long valueNumber() { return 0; }
  long defaultValueNumber() {
    long result = System.identityHashCode(this.getClass());
    for (Insn operand : operands) {
      result <<= 8;
      result |= System.identityHashCode(operand);
    }
    // Set at least one bit in case it wraps to zero.
    return result | 0x1;
  }
  boolean valueEqual(Insn other) { return false; }
  boolean defaultValueEqual(Insn other) {
    if (this.getClass() != other.getClass()) {
      return false;
    }
    return this.operands() == other.operands();
  }
  String immediate() { return ""; }
  Insn find() {
    Insn result = this;
    while (result.subst != result) {
      result = result.subst;
    }
    return result;
  }
  Insn replace(Insn other) {
    Insn root1 = this.find();
    Insn root2 = other.find();
    if (root1 != root2) {
      root1.subst = root2;
    }
    return root2;
  }
  void apply() {
    for (int i = 0; i < operands.length; i++) {
      operands[i] = operands[i].find();
    }
  }

  protected Insn[] operands = null;
  protected Insn subst = null;
}

class Const extends Insn {
  public Const(int value) { this.value = value; }
  public int value() { return value; }
  @Override
  long valueNumber() { return defaultValueNumber(); }
  @Override
  boolean valueEqual(Insn other) {
    if (!(other instanceof Const)) {
      return false;
    }
    Const o = (Const) other;
    return this.value == o.value;
  }
  @Override
  String immediate() { return Integer.toString(value); }

  private int value;
}

class Add extends Insn {
  public Add(Insn left, Insn right) { super(left, right); }
  @Override
  long valueNumber() { return defaultValueNumber(); }
  @Override
  boolean valueEqual(Insn other) { return defaultValueEqual(other); }
}

class Return extends Insn {
  public Return(Insn value) { super(value); }
}

class Block {
  Block() { insns = new ArrayList<>(); }
  Insn append(Insn insn) { insns.add(insn); return insn; }
  void print() {
    int next_id = 0;
    HashMap<Insn, String> names = new HashMap<>();
    for (Insn insn : insns) {
      String name = names.get(insn);
      if (name == null) {
        name = "v" + next_id++;
        names.put(insn, name);
      }
      System.out.print(name + " = " + insn.getClass().getSimpleName());
      String imm = insn.immediate();
      if (!imm.isEmpty()) {
        System.out.print("<" + imm + ">");
      }
      String sep = " ";
      for (Insn operand : insn.operands()) {
        String operand_name = names.get(operand);
        if (operand_name == null) {
          operand_name = "v" + next_id++;
          names.put(operand, operand_name);
        }
        System.out.print(sep + operand_name);
        sep = ", ";
      }
      System.out.println();
    }
  }

  protected ArrayList<Insn> insns;
}

class Main {
    public static void main(String[] args) {
      Block block = new Block();
      Insn left = block.append(new Const(1));
      Insn right = block.append(new Const(2));
      Insn add = block.append(new Add(left, right));
      Insn ret = block.append(new Return(add));
      block.print();
    }
}
