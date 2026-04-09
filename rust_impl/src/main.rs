use rust_impl::{Config, get_browsing_history};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let config = if args.len() >= 3 {
        Config::build(&args)?
    } else {
        Config::default()
    };

    let _history = get_browsing_history(&config)?;

    // TODO:
    // [] Pipe content to fzf
    // [] Collect output from fzf and use it to open browser
    Ok(())
}
