mod demangle;
mod masm;
mod parser;
mod src_mgr;
mod trace;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    path: String,

    #[arg(short, long, help("Full entry function symbol"))]
    entry_func: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_str = std::fs::read_to_string(cli.path)?;
    let (src_items, trace) = parser::parse_trace(log_str.as_str())?;

    let mut srcs = src_mgr::SourceManager::new(src_items);
    let mut demangled_symbols = fxhash::FxHashMap::default();
    let mut trace_idx = 0;

    srcs.set_entry(&trace, &cli.entry_func)?;

    println!("ENTRY AT {}", srcs.get_src_func_name()?);

    if let Some(entry_func) = &cli.entry_func {
        // We need to skip along the trace until we hit the entry.
        while &trace[trace_idx].func != entry_func {
            trace_idx += 1;
        }
    }

    let mut prior_top_of_stack = [0, 0];
    let mut pending_trace_skip = false;

    loop {
        if let Some(frame) = srcs.check_leave()? {
            match frame {
                src_mgr::BlockType::Exec => {
                    println!("  RETURN TO {}", srcs.get_src_func_name()?);
                    println!();
                }

                src_mgr::BlockType::TrueBlock => {
                    println!("  end");
                }

                src_mgr::BlockType::FalseBlock => {
                    println!("  else");
                    println!("    (SKIPPING)");
                    println!("  end");
                }
            }

            continue;
        }

        if pending_trace_skip {
            // We need to skip the trace along until it arrives at the current function.
            let ret_func_str = srcs.get_src_func_name()?;
            loop {
                trace_idx += 1;

                let demangled_sym = demangled_symbols
                    .entry(&trace[trace_idx].func)
                    .or_insert_with(|| demangle::demangle(&trace[trace_idx].func));

                if demangled_sym == ret_func_str {
                    break;
                }
            }

            pending_trace_skip = false;
        }

        let Some(trace::Trace {
            func,
            op,
            cycle,
            total,
            stack,
            ..
        }) = trace.get(trace_idx)
        else {
            // End of trace.
            assert!(trace_idx == trace.len());
            break;
        };

        if cycle != total {
            // Skip the intermediate micro-ops.
            trace_idx += 1;
            continue;
        }

        let src_op = srcs.get_src_op();

        if src_op.opcode() == Some("trace") {
            // Skip `trace` ops in the source; they're not in the actual trace.
            srcs.next_op();
            continue;
        }

        prior_top_of_stack[1] = prior_top_of_stack[0];
        prior_top_of_stack[0] = stack[0];

        // Usually the op just matches; we'll assume it's all lined up.
        if src_op == op {
            print_op(op, func, Some(stack));

            srcs.next_op();
            trace_idx += 1;

            continue;
        }

        // We have a mismatch; could be a call or conditional.
        match src_op {
            masm::Op::Op { opcode, arg } => {
                if opcode == "exec" || opcode == "call" {
                    print_op(&src_op, func, None);

                    let callee_func_name =
                        &arg.as_ref().expect("CALL/EXEC must have an argument")[2..];
                    if let Some(callee_block_key) = srcs.find_block_key(callee_func_name) {
                        // It seems maybe functions beginning with '__' are not actually run, or
                        // traced, or... not sure.

                        if final_sym_element(callee_func_name).starts_with("__") {
                            println!("    (SKIPPING)");

                            srcs.next_op();
                        } else {
                            srcs.enter(src_mgr::BlockType::Exec, callee_block_key);

                            println!();
                            println!("ENTERING {}", srcs.get_src_func_name()?);
                        }
                    } else {
                        // Skip the unknown (probably intrinsic) function until it returns.
                        println!("    (SKIPPING)");

                        // We could be at the end of a function, so the function we're actually
                        // skipping to is not this one, but the caller.  So we need to know that
                        // function name before we can skip.
                        pending_trace_skip = true;

                        // Skip the exec to unknown too.
                        srcs.next_op();
                    }
                } else {
                    println!();
                    //println!("src func: {}", srcs[src_item_idx].name);
                    //println!("  pc: {pc}");
                    println!("  src op {src_op:?}");
                    println!("trace func: {func}");
                    println!("  idx: {trace_idx}");
                    println!("  op {op:?}");
                    todo!()
                }
            }

            masm::Op::Conditional(t_block_key, f_block_key) => {
                let cond = prior_top_of_stack[1] != 0;

                println!("  if.true");
                if !cond {
                    println!("    (SKIPPING)");
                    println!("  else");

                    srcs.enter(src_mgr::BlockType::FalseBlock, *f_block_key);
                } else {
                    srcs.enter(src_mgr::BlockType::TrueBlock, *t_block_key);
                }
            }
        }
    }

    println!("END OF TRACE");

    Ok(())
}

fn print_op(op: &masm::Op, func: &str, stack: Option<&[u64]>) {
    let masm::Op::Op { opcode, arg } = op else {
        unreachable!("Unexpected non-regular op in {func} ({op:?})",);
    };

    // Print opcode.
    print!("  {opcode}");
    if let Some(arg) = arg {
        print!(".{arg}");
    }
    println!();

    if let Some(stack) = stack {
        // Print the stack.  Find the index to the last non-zero value first.
        let nz_idx = stack
            .iter()
            .rev()
            .position(|item| *item != 0)
            .unwrap_or(stack.len());
        let num_items_to_print = (stack.len() + 2 - nz_idx).min(stack.len());

        print!("    [");
        for idx in 0..num_items_to_print {
            print!(" {}", stack[idx]);
        }
        if num_items_to_print < stack.len() {
            print!(" ...");
        }
        println!(" ]")
    }
}

fn final_sym_element(sym: &str) -> &str {
    // Find the substring following the final "::" separator.
    sym.split("::").last().unwrap()
}
