mod masm;
mod parser;
mod trace;

fn main() -> anyhow::Result<()> {
    let log_path = std::env::args()
        .nth(1)
        .ok_or(anyhow::anyhow!("Missing trace log file name."))?;

    let log_str = std::fs::read_to_string(log_path)?;

    let (_srcs, _trace) = parser::parse_trace(log_str.as_str())?;

    Ok(())
}

