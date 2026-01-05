use declarative_enum_dispatch::enum_dispatch;

/// An index of an [`Insn`] in a [`Function`]. This is a popular
/// type since this effectively acts as a pointer to an [`Insn`].
/// See also: [`Function::find`].
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct InsnId(pub usize);

type ValueNumber = u32;

macro_rules! create_iterators {
    ($($field:ident),*) => {
        #[allow(unused_variables)]
        fn for_each_operand(&self, f: &mut dyn FnMut(InsnId)) {
            $(f(self.$field);)*
        }
        #[allow(unused_variables)]
        fn for_each_operand_mut(&mut self, f: &mut dyn FnMut(&mut InsnId)) {
            $(f(&mut self.$field);)*
        }
    };
}

enum_dispatch!(
pub trait InsnTrait: std::fmt::Debug {
    fn value_number(&self) -> ValueNumber { 0 }
    fn value_equals(&self, _other: &Insn) -> bool { false }
    fn for_each_operand(&self, f: &mut dyn FnMut(InsnId));
    fn for_each_operand_mut(&mut self, f: &mut dyn FnMut(&mut InsnId));
}
#[derive(Debug)]
pub enum Insn {
    Const(Const),
    Add(Add),
    Return(Return),
}
);

#[derive(Debug)]
struct Const {
    value: u32,
}

impl InsnTrait for Const {
    fn value_number(&self) -> ValueNumber {
        0x500 | self.value
    }
    fn value_equals(&self, other: &Insn) -> bool {
        let Insn::Const(other_const) = other else { return false; };
        self.value == other_const.value
    }
    create_iterators!();
}

#[derive(Debug)]
struct Return {
    value: InsnId,
}

impl InsnTrait for Return {
    create_iterators!(value);
}

#[derive(Debug)]
struct Add {
    lhs: InsnId,
    rhs: InsnId,
}

impl InsnTrait for Add {
    fn value_number(&self) -> ValueNumber {
        0x1000 | (self.lhs.0 as ValueNumber) << 8 | (self.rhs.0 as ValueNumber)
    }
    fn value_equals(&self, other: &Insn) -> bool {
        let Insn::Add(other_add) = other else { return false; };
        self.lhs == other_add.lhs && self.rhs == other_add.rhs
    }
    create_iterators!(lhs, rhs);
}

#[derive(Debug)]
struct Function {
    insns: Vec<Insn>,
    block: Vec<InsnId>,
}

impl Function {
    #[inline]
    fn push_insn(&mut self, insn: Insn) -> InsnId {
        let id = InsnId(self.insns.len());
        self.insns.push(insn);
        self.block.push(id);
        id
    }

    fn local_value_number(&mut self) {
        let mut map: std::collections::HashMap<&Insn, InsnId> = std::collections::HashMap::new();
        let mut new_block = vec![];
        for &id in &self.block {
            let insn = &self.insns[id.0];
            if insn.value_number() == 0 {
                new_block.push(id);
                continue;
            }
            let Some(_) = map.get(&*insn) else {
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

impl std::hash::Hash for Insn {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let vn = self.value_number();
        assert_ne!(vn, 0, "Cannot hash instruction with no value number");
        vn.hash(state);
    }
}

impl std::cmp::PartialEq for Insn {
    fn eq(&self, other: &Self) -> bool {
        self.value_equals(other)
    }
}

impl std::cmp::Eq for Insn {}

fn main() {
    let mut function = Function { insns: vec![], block: vec![] };
    let v0 = function.push_insn(Insn::Const(Const { value: 42 }));
    let v1 = function.push_insn(Insn::Const(Const { value: 42 }));
    let _v2 = function.push_insn(Insn::Const(Const { value: 42 }));
    let _v3 = function.push_insn(Insn::Add(Add { lhs: v0, rhs: v1 }));
    let v4 = function.push_insn(Insn::Add(Add { lhs: v0, rhs: v1 }));
    function.push_insn(Insn::Return(Return { value: v4 }));
    eprintln!("fun:\n{}", function);
    function.local_value_number();
    eprintln!("fun:\n{}", function);
}
