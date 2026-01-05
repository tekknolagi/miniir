use declarative_enum_dispatch::enum_dispatch;

/// An index of an [`Insn`] in a [`Function`]. This is a popular
/// type since this effectively acts as a pointer to an [`Insn`].
/// See also: [`Function::find`].
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct InsnId(pub usize);

impl From<InsnId> for usize {
    fn from(val: InsnId) -> Self {
        val.0
    }
}

type ValueNumber = u32;
type Opcode = u16;

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
    fn opcode(&self) -> Opcode;
    fn value_number(&self) -> ValueNumber { 0 }
    fn value_equals(&self, _other: &Insn) -> bool { false }
    fn for_each_operand(&self, f: &mut dyn FnMut(InsnId));
    fn for_each_operand_mut(&mut self, f: &mut dyn FnMut(&mut InsnId));
    fn default_value_number(&self) -> ValueNumber {
        let mut result = self.opcode() as ValueNumber;
        self.for_each_operand(&mut |operand| {
            result <<= 8;
            result |= operand.0 as ValueNumber;
        });
        result
    }
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
    fn opcode(&self) -> Opcode { 1 }
    fn value_number(&self) -> ValueNumber { self.default_value_number() }
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
    fn opcode(&self) -> Opcode { 3 }
    create_iterators!(value);
}

#[derive(Debug)]
struct Add {
    lhs: InsnId,
    rhs: InsnId,
}

impl InsnTrait for Add {
    fn opcode(&self) -> Opcode { 2 }
    fn value_number(&self) -> ValueNumber { self.default_value_number() }
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

struct UnionFind<T: Copy + Into<usize>> {
    forwarded: Vec<Option<T>>,
}

impl<T: Copy + Into<usize> + PartialEq> UnionFind<T> {
    fn new() -> UnionFind<T> {
        UnionFind { forwarded: vec![] }
    }

    fn at(&self, idx: T) -> Option<T> {
        self.forwarded.get(idx.into()).copied().flatten()
    }

    fn set(&mut self, idx: T, value: T) {
        if idx.into() >= self.forwarded.len() {
            self.forwarded.resize(idx.into()+1, None);
        }
        self.forwarded[idx.into()] = Some(value);
    }

    pub fn find(&mut self, insn: T) -> T {
        let result = self.find_const(insn);
        if result != insn {
            // Path compression
            self.set(insn, result);
        }
        result
    }

    fn find_const(&self, insn: T) -> T {
        let mut result = insn;
        loop {
            match self.at(result) {
                None => return result,
                Some(insn) => result = insn,
            }
        }
    }

    pub fn make_equal_to(&mut self, insn: T, target: T) {
        let found = self.find(insn);
        self.set(found, target);
    }
}

impl UnionFind<InsnId> {
    pub fn apply(&mut self, insn: &mut Insn) {
        insn.for_each_operand_mut(&mut |operand| {
            *operand = self.find(*operand);
        });
    }
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
        let mut uf = UnionFind::<InsnId>::new();
        for &id in &self.block {
            let insn = &self.insns[id.0];
            if insn.value_number() == 0 {
                new_block.push(id);
                continue;
            }
            let Some(replacement) = map.get(&*insn) else {
                map.insert(&*insn, id);
                new_block.push(id);
                continue;
            };
            uf.make_equal_to(id, *replacement);
        }
        for &id in &new_block {
            uf.apply(&mut self.insns[id.0]);
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
