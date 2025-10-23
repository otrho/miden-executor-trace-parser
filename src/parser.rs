use crate::{
    masm::{Op, SourceItem},
    trace::Trace,
};

pub(crate) fn parse_trace(input: &str) -> anyhow::Result<(Vec<SourceItem>, Vec<Trace>)> {
    trace_parser::parse(input).map_err(|err| {
        let l = err.location.offset;
        anyhow::anyhow!("{err}\nFound: {}...", &input[l..l + 20])
    })
}

peg::parser! {
    grammar trace_parser() for str {
        pub rule parse() -> (Vec<SourceItem>, Vec<Trace>)
            = _ srcs:module()* between_garbage() traces:trace_item()* end_garbage() {
                (srcs.into_iter().flatten().collect(), traces)
            }

        rule module() -> Vec<SourceItem>
            = m:mod_comment() srcs:src_item()* {
                srcs.into_iter().map(|src| {
                    SourceItem {
                        name: format!("{m}::{}", src.name),
                        ops: src.ops,
                    }
                }).collect()
            }

        rule mod_comment() -> String
            = "#" _ "mod" _ sym:symbol() {
                sym.to_string()
            }

        rule src_item() -> SourceItem
            = export()
            / proc()

        rule export() -> SourceItem
            = "export." name:symbol() ops:op()+ end() {
                SourceItem { name, ops }
            }

        rule proc() -> SourceItem
            = "proc." name:symbol() ops:op()+ end() {
                SourceItem { name, ops }
            }

        rule between_garbage()
            = "test" (!"FAILED" [_])* "FAILED" _ (!trace_marker() [_])*

        rule end_garbage()
            = "Stack Trace:" [_]*

        rule op() -> Op
            = cond_block()
            / op:ident() arg:op_arg()? {
                Op::Op(op, arg)
            }

        rule op_arg() -> String
            = "." arg:(decimal_str() / symbol() / array_op_arg()) {
                arg
            }

        rule array_op_arg() -> String
            = str:$("[" ['0'..='9' | ',']+ "]") _ {
                str.to_string()
            }

        rule cond_block() -> Op
            = "if.true" _ tops:op()* "else" _ fops:op()* end() {
                Op::Conditional(tops, fops)
            }

        rule keyword()
            = "export" / "proc" / "if" / "else" / "end"

        rule trace_item() -> Trace
            = func:trace_in() exe:trace_executed() stack:trace_stack() step_stack()? {
                let (masm_op, op, cycle, total) = exe;
                Trace { func, masm_op, op, cycle, total, stack }
            }

        rule trace_in() -> String
            = trace_marker() "in" _ sym:symbol() "(" (!")" [_])* ")" _ {
                sym
            }

        rule trace_executed() -> (String, Op, u64, u64)
            = trace_marker() "executed" _ ops:trace_ops() "(cycle" _ c:num() "/" t:num() ")" _ {
                let (a, b) = ops;
                (a, b, c, t)
            }

        rule trace_ops() -> (String, Op)
            = "`" masm_op:$((!"`" [_])*)  "`" _ "of" _ "`" op:op() "`" _ {
                (masm_op.to_string(), op)
            }

        rule trace_stack() -> Vec<u64>
            = trace_marker() "stack state:" _ "[" _ nums:csn() ","? _ "]" _ {
                nums
            }

        rule trace_marker() = "[TRACE executor]" _

        rule step_stack()
            = "[" (!"]" [_])* "]" _ "&step.stack =" _ "[" (!"]" [_])* _ "]" _

        rule symbol() -> String
            = s:$(sym_char() (sym_char() / ['0'..='9'])*) _ {
                s.to_string()
            }

        rule sym_char() -> char
            = quiet!{['a'..='z' | 'A'..='Z' | '.' | '/' | '@' | '_' | '-' | ':' | '#']}

        rule ident() -> String
            = !keyword() id:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '_']+) _ {
                id.to_string()
            }

        rule csn() -> Vec<u64>
            = num() ** ("," _)

        rule num() -> u64
            = s:decimal_str() {
                s.parse::<u64>().unwrap()
            }

        rule decimal_str() -> String
            = str:$(['0'..='9']+) _ {
                str.to_string()
            }

        rule end() = "end" _

        rule _ = quiet!{(ws() / pkg_spam())*}

        rule ws() = [' ' | '\n' | '\r' | '\t']
        rule pkg_spam() = "Creating Miden package" to_eol()
        rule to_eol() = (!['\n' | '\r'] [_])*
    }
}
