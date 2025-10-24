slotmap::new_key_type! { pub(crate) struct BlockKey; }

#[derive(Debug, Default)]
pub(crate) struct SourceBlocks(slotmap::SlotMap<BlockKey, Block>);

impl std::ops::Deref for SourceBlocks {
    type Target = slotmap::SlotMap<BlockKey, Block>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SourceBlocks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub(crate) struct Block {
    name: Option<String>,
    ops: Vec<Op>,
}

impl Block {
    pub(crate) fn new(name: String, ops: Vec<Op>) -> Self {
        Self {
            name: Some(name),
            ops,
        }
    }

    pub(crate) fn bare(ops: Vec<Op>) -> Self {
        Self { name: None, ops }
    }

    /// Special method to let the parser update the module name after the fact.
    pub(crate) fn prefix_module_name(&mut self, module_name: &str) {
        if let Some(name) = &mut self.name {
            let new_name = format!("{module_name}::{name}");
            *name = new_name;
        }
    }

    pub(crate) fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub(crate) fn op_at(&self, idx: usize) -> &Op {
        &self.ops[idx]
    }

    pub(crate) fn len(&self) -> usize {
        self.ops.len()
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Op {
    Op { opcode: String, arg: Option<String> },
    Conditional(BlockKey, BlockKey),
}

impl Op {
    pub(crate) fn opcode(&self) -> Option<&str> {
        if let Op::Op { opcode, .. } = self {
            Some(opcode)
        } else {
            None
        }
    }
}
