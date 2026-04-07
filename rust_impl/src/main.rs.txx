use std::env;
use std::io::Write;
use std::process;
use std::process::{Command, Stdio};

struct Config {
    browser: String,
    fzf: String,
}

impl Config {
    fn default() -> Config {
        Config {
            browser: "chrome".to_string(),
            fzf: "fzf".to_string(),
        }
    }

    fn build(args: &Vec<String>) -> Result<Config, &'static str> {
        if args.len() < 4 {
            return Err("Not enough arguments");
        }

        let browser = &args[1];
        let fzf = &args[2];

        Ok(Config {
            browser: browser.to_string(),
            fzf: fzf.to_string(),
        })
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config: Config;
    if args.len() <= 1 {
        println!("using default argumnets");
        config = Config::default();
    } else {
        config = Config::build(&args).unwrap_or_else(|err| {
            eprintln!("Error: {err}");
            process::exit(1);
        })
    };

    // let mut node_process = Command::new("tmux").args(["new", "-s", "spawn_process"]).spawn().unwrap_or_else(|err| {
    let mut node_process = Command::new("node")
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| {
            eprintln!("Could not run {}: {err}", config.fzf.to_string());
            process::exit(1);
        });

    let mut stdin = match node_process.stdin.take() {
        Some(v) => v,
        None => {
            eprintln!("Couldn't get stdin of node_process");
            process::exit(1);
        }
    };

    let msg = format!("console.log(\"{}\")", "Are we cool now Benson");
    //  NOTE hmm, rust doesnt block on writing to stdin, but goes straight
    // to the next function
    {
        println!("writing to stdin");
        stdin
            .write_all(msg.as_bytes())
            .expect("Could not write to fzf stdin");
    }

    run_fzf(&config, "Carvan".to_string());
}

fn run_fzf(config: &Config, input: String) {
    println!("Running fzf");
    Command::new("sh")
        .args(["-c", config.fzf.as_str()])
        .spawn()
        .unwrap_or_else(|err| {
            eprintln!("COULD NOT SPAWN FZF AS CMD: {err}");
            process::exit(1);
        });
}
