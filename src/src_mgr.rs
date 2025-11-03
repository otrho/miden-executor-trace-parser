use crate::{masm, trace};

const INDENT_AMOUNT: usize = 4;

pub(crate) struct SourceManager {
    srcs: masm::SourceBlocks,
    src_block_key: masm::BlockKey,
    pc: usize,
    call_stack: Vec<(BlockType, masm::BlockKey, usize)>,
    indent: usize,
}

pub(crate) enum BlockType {
    Start,
    Exec,
    TrueBlock,
    FalseBlock,
}

impl SourceManager {
    // TODO: Intialise with an entry so that we don't have an empty call stack or invalid
    // src_item_idx.
    pub(crate) fn new(srcs: masm::SourceBlocks) -> Self {
        Self {
            srcs,
            src_block_key: masm::BlockKey::default(),
            pc: 0,
            call_stack: vec![(BlockType::Start, masm::BlockKey::default(), 0)],
            indent: 0,
        }
    }

    pub(crate) fn find_block_key(&self, func: &str) -> Option<masm::BlockKey> {
        self.srcs
            .iter()
            .find(|(_, block)| block.name().map(|name| name == func).unwrap_or(false))
            .map(|(key, _)| key)
    }

    pub(crate) fn fuzzy_find_block_key(&self, func: &str) -> Vec<masm::BlockKey> {
        self.srcs
            .iter()
            .filter_map(|(key, block)| {
                block
                    .name()
                    .and_then(|name| name.ends_with(func).then_some(key))
            })
            .collect()
    }

    pub(crate) fn get_src_func_name(&self) -> anyhow::Result<&String> {
        // Default to the current block name.
        self.srcs[self.src_block_key]
            .name()
            .or_else(|| {
                // Else search backwards in the call stack until one is found.
                self.call_stack
                    .iter()
                    .rev()
                    .find_map(|(_, block_key, _)| self.srcs[*block_key].name())
            })
            .ok_or_else(|| anyhow::anyhow!("Failed to find a current function name"))
    }

    pub(crate) fn get_src_op(&self) -> &masm::Op {
        self.srcs[self.src_block_key].op_at(self.pc)
    }

    pub(crate) fn next_op(&mut self) {
        self.pc += 1;
    }

    pub(crate) fn indent(&self) -> usize {
        self.indent
    }

    pub(crate) fn indent_next(&self) -> usize {
        self.indent + INDENT_AMOUNT
    }

    pub(crate) fn inc_indent(&mut self) {
        self.indent += INDENT_AMOUNT;
    }

    pub(crate) fn dec_indent(&mut self) {
        self.indent -= INDENT_AMOUNT;
    }

    pub(crate) fn enter(&mut self, frame: BlockType, target_block_key: masm::BlockKey) {
        self.call_stack
            .push((frame, self.src_block_key, self.pc + 1));
        self.src_block_key = target_block_key;
        self.pc = 0;
    }

    pub(crate) fn check_leave(&mut self) -> anyhow::Result<Option<BlockType>> {
        if self.pc >= self.srcs[self.src_block_key].len() {
            // We've run off the end of this block.  Need to return.
            let (frame, ret_block_key, ret_pc) = self
                .call_stack
                .pop()
                .ok_or(anyhow::anyhow!("Underflowed the call stack."))?;

            self.src_block_key = ret_block_key;
            self.pc = ret_pc;

            Ok(Some(frame))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn set_entry(
        &mut self,
        trace: &[trace::Trace],
        entry_func: &Option<String>,
    ) -> anyhow::Result<String> {
        let block_key = if let Some(entry_func) = entry_func {
            let entry_funcs = self.fuzzy_find_block_key(entry_func);
            match entry_funcs.len() {
                0 => anyhow::bail!("Failed to find requested entry function: {entry_func}"),
                1 => Ok(entry_funcs[0]),
                _ => {
                    let found_funcs_str = entry_funcs
                        .iter()
                        .filter_map(|key| self.srcs[*key].name())
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n  ");

                    anyhow::bail!("Found multiple potential entry functions:\n  {found_funcs_str}")
                }
            }
        } else {
            self.get_entry_func_block_key(trace).ok_or(anyhow::anyhow!(
                "Failed to determine default entry function."
            ))
        }?;

        // XXX: This could be re-thought.  A lot of re-fetching and cloning of strings going on.
        self.src_block_key = block_key;
        Ok(self.srcs[block_key].name().unwrap().clone())
    }

    fn get_entry_func_block_key(&self, trace: &[trace::Trace]) -> Option<masm::BlockKey> {
        // Get the first trace event.  If it's pointing to a `run` then that's our entry func.  If
        // it's pointing to an `init`, then replace `init` with `run` and try that.

        macro_rules! ret_if_found {
            ($func_name: expr) => {{
                let idx = self.find_block_key($func_name);
                if idx.is_some() {
                    return idx;
                }
            }};
        }

        if let Some(trace::Trace { func, .. }) = trace.first() {
            if func.ends_with("::run") {
                ret_if_found!(func)
            }

            if let Some(base_str) = func.strip_suffix("::init") {
                ret_if_found!(&(base_str.to_string() + "::run"))
            }
        }

        None
    }
}
