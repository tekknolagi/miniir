/// An index of an [`Insn`] in a [`Function`]. This is a popular
/// type since this effectively acts as a pointer to an [`Insn`].
/// See also: [`Function::find`].
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct InsnId(pub usize);

type ValueNumber = u32;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Opcode {
    Const = 1,
    Add,
    Return,
}

pub trait Insn: std::any::Any + std::fmt::Debug {
    fn opcode(&self) -> Opcode;
    fn value_number(&self) -> ValueNumber { 0 }
    fn value_equals(&self, other: &dyn Insn) -> bool { false }
    fn for_each_operand(&self, f: &mut dyn FnMut(InsnId));
    fn for_each_operand_mut(&mut self, f: &mut dyn FnMut(&mut InsnId));
    fn as_any<'a>(&'a self) -> &'a dyn std::any::Any;
    fn number_operands(&self) -> ValueNumber {
        let mut result = self.opcode() as ValueNumber;
        self.for_each_operand(&mut |id| {
            result <<= 16;
            result |= id.0 as u32;
        });
        result
    }
}

#[derive(Debug)]
struct Const {
    value: u32,
}

impl Insn for Const {
    fn opcode(&self) -> Opcode { Opcode::Add }
    fn for_each_operand(&self, _f: &mut dyn FnMut(InsnId)) {}
    fn for_each_operand_mut(&mut self, _f: &mut dyn FnMut(&mut InsnId)) {}
    fn as_any<'a>(&'a self) -> &'a dyn std::any::Any { self }

    fn value_number(&self) -> ValueNumber {
        // Can't be 0 because that's reserved for "no value number"
        Opcode::Const as u32 | self.value
    }

    fn value_equals(&self, other: &dyn Insn) -> bool {
        other.as_any().downcast_ref::<Const>().map_or(false, |other_const| {
            self.value == other_const.value
        })
    }
}

#[derive(Debug)]
struct Return {
    value: InsnId,
}

impl Insn for Return {
    fn opcode(&self) -> Opcode { Opcode::Return }

    fn for_each_operand(&self, f: &mut dyn FnMut(InsnId)) {
        f(self.value);
    }

    fn for_each_operand_mut(&mut self, f: &mut dyn FnMut(&mut InsnId)) {
        f(&mut self.value);
    }

    fn as_any<'a>(&'a self) -> &'a dyn std::any::Any { self }
}

#[derive(Debug)]
struct Add {
    lhs: InsnId,
    rhs: InsnId,
}

impl Insn for Add {
    fn opcode(&self) -> Opcode { Opcode::Add }

    fn for_each_operand(&self, f: &mut dyn FnMut(InsnId)) {
        f(self.lhs);
        f(self.rhs);
    }

    fn for_each_operand_mut(&mut self, f: &mut dyn FnMut(&mut InsnId)) {
        f(&mut self.lhs);
        f(&mut self.rhs);
    }

    fn as_any<'a>(&'a self) -> &'a dyn std::any::Any { self }

    fn value_number(&self) -> ValueNumber { self.number_operands() }

    fn value_equals(&self, other: &dyn Insn) -> bool {
        other.as_any().downcast_ref::<Add>().map_or(false, |other_add| {
            self.lhs == other_add.lhs && self.rhs == other_add.rhs
        })
    }
}

#[derive(Debug)]
struct Function {
    insns: Vec<Box<dyn Insn>>,
    block: Vec<InsnId>,
}

impl Function {
    #[inline]
    fn push_insn<T: Insn>(&mut self, insn: T) -> InsnId {
        let id = InsnId(self.insns.len());
        self.insns.push(Box::new(insn));
        self.block.push(id);
        id
    }

    fn local_value_number(&mut self) {
        let mut map: std::collections::HashMap<&dyn Insn, InsnId> = std::collections::HashMap::new();
        let mut new_block = vec![];
        for &id in &self.block {
            let insn = &*self.insns[id.0];
            if insn.value_number() == 0 {
                new_block.push(id);
                continue;
            }
            let Some(replacement) = map.get(&*insn) else {
                map.insert(&*insn, id);
                new_block.push(id);
                continue;
            };
        }
        self.block = new_block;
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &id in &self.block {
            writeln!(f, "v{} = {:?}", id.0, self.insns[id.0])?;
        }
        Ok(())
    }
}

impl std::hash::Hash for dyn Insn {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let vn = self.value_number();
        assert_ne!(vn, 0, "Cannot hash instruction with no value number");
        vn.hash(state);
    }
}

impl std::cmp::PartialEq for dyn Insn {
    fn eq(&self, other: &Self) -> bool {
        self.value_equals(other)
    }
}

impl std::cmp::Eq for dyn Insn {}

fn main() {
    let mut function = Function { insns: vec![], block: vec![] };
    let v0 = function.push_insn(Const { value: 42 });
    let v1 = function.push_insn(Const { value: 42 });
    let v2 = function.push_insn(Const { value: 42 });
    let v3 = function.push_insn(Add { lhs: v0, rhs: v1 });
    let v4 = function.push_insn(Add { lhs: v0, rhs: v1 });
    eprintln!("fun:\n{}", function);
    function.local_value_number();
    eprintln!("fun:\n{}", function);
    // let right = function.push_insn(Const { value: 58 });
    // let add = function.push_insn(Add { lhs: left, rhs: right });
    // function.push_insn(Return { value: add });
}
