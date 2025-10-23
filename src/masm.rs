#[derive(Debug)]
pub(crate) struct SourceItem {
    pub(crate) name: String,
    pub(crate) ops: Vec<Op>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Op {
    Op(String, Option<String>),
    Conditional(Vec<Op>, Vec<Op>),
}
