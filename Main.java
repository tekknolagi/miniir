import java.util.Iterator;
import java.util.Collections;
import java.util.ArrayList;
import java.util.HashMap;

class Insn {
  Insn(Insn... operands) { this.operands = operands; this.subst = this; }
  Insn[] operands() { return operands; }
  int valueNumber() { return 0; }
  int defaultValueNumber() {
    int result = System.identityHashCode(this.getClass());
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
  Insn apply() {
    for (int i = 0; i < operands.length; i++) {
      operands[i] = operands[i].find();
    }
    return this;
  }

  protected Insn[] operands = null;
  protected Insn subst = null;
}

class Const extends Insn {
  public Const(int value) { this.value = value; }
  public int value() { return value; }
  @Override
  int valueNumber() { return defaultValueNumber(); }
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
  int valueNumber() { return defaultValueNumber(); }
  @Override
  boolean valueEqual(Insn other) {
    System.out.println("Comparing " + this + " and " + other);
    System.out.println("Operands: " + operands()[0] + ", " + operands()[1]);
    System.out.println("Other operands: " + other.operands()[0] + ", " + other.operands()[1]);
    boolean result = defaultValueEqual(other);
    System.out.println("Result: " + result);
    return defaultValueEqual(other); }
}

class Return extends Insn {
  public Return(Insn value) { super(value); }
}

class Block {
  Block() { insns = new ArrayList<>(); }
  Insn append(Insn insn) { insns.add(insn); return insn; }
  void replaceInsns(ArrayList<Insn> new_insns) { insns = new_insns; }
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

/* BEGIN From Maxine VM */
/* Copyright (c) 2009, 2011, Oracle and/or its affiliates. GPLv2 licensed. */
class ValueMap {
    private static class Link {
        final ValueMap map;
        final int valueNumber;
        final Insn value;
        final Link next;

        Link(ValueMap map, int valueNumber, Insn value, Link next) {
            this.map = map;
            this.valueNumber = valueNumber;
            this.value = value;
            this.next = next;
        }
    }

    private final ValueMap parent;
    private Link[] table;
    private int count;
    private int max;

    public ValueMap() {
        parent = null;
        table = new Link[19];
    }

    public ValueMap(ValueMap parent) {
        this.parent = parent;
        this.table = parent.table.clone();
        this.count = parent.count;
        this.max = table.length + table.length / 2;
    }

    public Insn findInsert(Insn x) {
        int valueNumber = x.valueNumber();
        if (valueNumber != 0) {
            // value number != 0 means the instruction can be value numbered
            int index = indexOf(valueNumber, table);
            Link l = table[index];
            // hash and linear search
            while (l != null) {
                if (l.valueNumber == valueNumber && l.value.valueEqual(x)) {
                    return l.value;
                }
                l = l.next;
            }
            // not found; insert
            table[index] = new Link(this, valueNumber, x, table[index]);
            if (count > max) {
                resize();
            }
        }
        return x;
    }

    public void killAll() {
        assert parent == null : "should only be used for local value numbering";
        for (int i = 0; i < table.length; i++) {
            table[i] = null;
        }
        count = 0;
    }

    private void resize() {
        Link[] ntable = new Link[table.length * 3 + 4];
        if (parent != null) {
            // first add all the parent's entries by cloning them
            for (int i = 0; i < table.length; i++) {
                Link l = table[i];
                while (l != null && l.map == this) {
                    l = l.next; // skip entries in this map
                }
                while (l != null) {
                    // copy entries from parent
                    int index = indexOf(l.valueNumber, ntable);
                    ntable[index] = new Link(l.map, l.valueNumber, l.value, ntable[index]);
                    l = l.next;
                }
            }
        }

        for (int i = 0; i < table.length; i++) {
            Link l = table[i];
            // now add all the entries from this map
            while (l != null && l.map == this) {
                int index = indexOf(l.valueNumber, ntable);
                ntable[index] = new Link(l.map, l.valueNumber, l.value, ntable[index]);
                l = l.next;
            }
        }
        table = ntable;
        max = table.length + table.length / 2;
    }

    private int indexOf(int valueNumber, Link[] t) {
        return (valueNumber & 0x7fffffff) % t.length;
    }
}
/* END from Maxine VM */

class Function {
  Function() { blocks = new ArrayList<>(); }

  void addBlock(Block block) {
    blocks.add(block);
  }

  void localValueNumbering() {
    for (Block block : blocks) {
      ValueMap vmap = new ValueMap();
      ArrayList<Insn> new_insns = new ArrayList<>();
      for (Insn insn : block.insns) {
        Insn found = insn.find().apply();
        System.out.println("found: " + found);
        Insn valueNumbered = vmap.findInsert(found);
        if (valueNumbered != found) {
          System.out.println("replacing " + found + " with " + valueNumbered);
          found.replace(valueNumbered);
        } else {
          new_insns.add(found);
        }
      }
      block.replaceInsns(new_insns);
    }
  }

  protected ArrayList<Block> blocks;
}

class Main {
    public static void main(String[] args) {
      Block block = new Block();
      Insn left = block.append(new Const(1));
      Insn right = block.append(new Const(2));
      Insn add = block.append(new Add(left, right));
      Insn left1 = block.append(new Const(1));
      Insn right1 = block.append(new Const(2));
      Insn add1 = block.append(new Add(left, right));
      Insn ret = block.append(new Return(add1));
      Function function = new Function();
      function.addBlock(block);
      System.out.println("Before LVN:");
      block.print();
      function.localValueNumbering();
      System.out.println("After LVN:");
      block.print();
    }
}
