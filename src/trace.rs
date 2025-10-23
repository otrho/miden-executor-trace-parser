use crate::masm::Op;

#[derive(Debug)]
pub(crate) struct Trace {
    pub(crate) func: String,
    pub(crate) masm_op: String,
    pub(crate) op: Op,
    pub(crate) cycle: u64,
    pub(crate) total: u64,
    pub(crate) stack: Vec<u64>,
}
