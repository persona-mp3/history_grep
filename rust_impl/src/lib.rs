#![allow(dead_code)]
use std::env;
use std::path::PathBuf;
use std::process::Command;

// use rusqlite::{Connection, Result};

use home;
#[derive(Debug)]
pub struct Config {
    pub browser: String,
    pub limit: u32,
}

impl Config {
    pub fn default() -> Config {
        Config {
            browser: String::from("chrome"),
            limit: 600,
        }
    }

    pub fn build(args: &[String]) -> Result<Config, Box<dyn std::error::Error>> {
        if args.len() < 3 {
            return Err("Not enough arguments\
                hist <browser-name> <limit>
                "
            .into());
        }

        let browser = &args[1];
        let limit = args[2].clone().trim().parse::<u32>()?;
        Ok(Config {
            browser: browser.clone(),
            limit: limit,
        })
    }
}
pub const CHROME_HISTORY_FILE_MACOS: &'static str =
    "Library/Application Support/Google/Chrome/Default/History";

// https://issarice.com/export-chrome-history
pub const CHROME_HISTORY_FILE_LINUX: &'static str = "~/.config/google-chrome/Default/History";

#[derive(Debug)]
struct VisitedUrl {
    id: i32,
    link_url_id: i32,
    top_level_url: String,
    frame_url: String,
    visit_count: i32,
}

#[derive(Debug)]
pub enum OSArch {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

pub fn get_os() -> OSArch {
    let user_os = env::consts::OS;
    let os_arch: OSArch = match user_os {
        "linux" => OSArch::Linux,
        "macos" => OSArch::MacOS,
        "windows" => OSArch::Windows,
        _ => OSArch::Unknown,
    };

    os_arch
}

pub fn parse_browing_history(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let os_arch = get_os();
    // TODO(daniel) These paths are really fragile like this, because we'll need to access the
    // home_dirs of each os. For example, the chrome_history_linux is `~/.config...`
    // but for chrome_history_macos its `/Library/...`. So I think it's best if we just brute force
    // it as `~/path_to_chrome_history_file` on unix based oses and what windows calls its own
    // so theres no need to constantly try and resolve file-paths here
    let browser_history_path: &str = match config.browser.as_str() {
        "chrome" => match os_arch {
            OSArch::Linux => CHROME_HISTORY_FILE_LINUX,
            OSArch::Windows => "windows\\ path \\ to \\ chrome \\ history \\ file",
            OSArch::MacOS => CHROME_HISTORY_FILE_MACOS,
            OSArch::Unknown => return Err("Could not identify OS".into()),
        },

        _ => return Err("Could not identify browser".into()),
    };

    let temp_file = ".temp_chrome_history.db";
    let _ = temp_file;

    // NOTE: Do research if Rust translates it to Windows command
    let exit_status = Command::new("cp")
        .args([browser_history_path, temp_file])
        .output()?;

    if !exit_status.status.success() {
        eprintln!("Failed to copy history file {browser_history_path}");
        let stderr_msg = exit_status.stderr;
        return Err(String::from_utf8_lossy(&stderr_msg).into());
    }

    println!("Copy was successfull");
    println!("exit_status -> {:?} ", exit_status);

    Ok(())
}

pub fn get_browsing_history(config: &Config) -> Result<&str, Box<dyn std::error::Error>> {
    let file_path = match config.browser.as_str() {
        "chrome" => CHROME_HISTORY_FILE_MACOS,
        _ => return Err("Browser not supported".into()),
    };

    let homedir = home::home_dir().unwrap_or_else(|| PathBuf::from("~"));

    let full_path = homedir.join(file_path);

    let temp_file = PathBuf::from("temp_history.db");
    let cmd_status = Command::new("cp").args([&full_path, &temp_file]).output()?;
    println!("Command Status: {:?}", cmd_status);

    println!("Command for copying was successful, I assume..");
    // Docs: https://crates.io/crates/rusqlite/
    // let conn = Connection::open(&temp_file)?;
    // let mut stmt = conn.prepare("SELECT * FROM visited_links ORDER BY id DESC LIMIT (?1)")?;
    // let visited_urls_iter = stmt.query_map([5i32], |row| {
    //     Ok(VisitedUrl {
    //         id: row.get(0)?,
    //         link_url_id: row.get(1)?,
    //         top_level_url: row.get(2)?,
    //         frame_url: row.get(3)?,
    //         visit_count: row.get(4)?,
    //     })
    // })?;
    //
    // for url in visited_urls_iter {
    //     println!("{:?}\n", url.unwrap());
    // }
    //
    // Command::new("rm").arg(&temp_file).output()?;
    Ok("")
}
