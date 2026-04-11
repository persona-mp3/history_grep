#![allow(dead_code)]
use std::env;
use std::path::PathBuf;
use std::process::Command;

use rusqlite::{Connection, Result};

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
pub const CHROME_HISTORY_FILE_LINUX: &'static str = ".config/google-chrome/Default/History";

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

    let browser_history_path: &str = match config.browser.as_str() {
        "chrome" => match os_arch {
            OSArch::Linux => CHROME_HISTORY_FILE_LINUX,
            OSArch::Windows => "windows\\ path \\ to \\ chrome \\ history \\ file",
            OSArch::MacOS => CHROME_HISTORY_FILE_MACOS,
            OSArch::Unknown => return Err("Could not identify OS".into()),
        },

        _ => return Err("Could not identify browser".into()),
    };

    let home_dir = match home::home_dir() {
        Some(v) => v,
        None => {
            return Err("Could not get home directory".into());
        }
    };
    let temp_file = PathBuf::from(".temp_chrome_history.db");
    let browser_history_path = home_dir.join(&browser_history_path);

    // NOTE: Do research if Rust translates it to Windows command
    let exit_status = Command::new("cp")
        .args([&browser_history_path, &temp_file])
        .output()?;

    if !exit_status.status.success() {
        eprintln!("Failed to copy history file {:?}", browser_history_path);
        let stderr_msg = exit_status.stderr;
        return Err(String::from_utf8_lossy(&stderr_msg).into());
    }

    println!("Copy was successfull");
    println!("exit_status -> {:?} ", exit_status);

    let conn = match Connection::open(&temp_file) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Could not open connection for file");
            return Err(err.into());
        }
    };

    let mut stmt = conn.prepare("SELECT * FROM visited_links ORDER BY id DESC LIMIT (?1)")?;
    let visited_urls_iter = stmt.query_map([5i32], |row| {
        Ok(VisitedUrl {
            id: row.get(0)?,
            link_url_id: row.get(1)?,
            top_level_url: row.get(2)?,
            frame_url: row.get(3)?,
            visit_count: row.get(4)?,
        })
    })?;

    for url in visited_urls_iter {
        println!("{:?}\n", url.unwrap());
    }

    // Cleanup
    let exit_status = Command::new("rm").arg(&temp_file).output()?;
    if !exit_status.status.success() {
        eprintln!("Failed to remove history file {:?}", temp_file);
        let stderr_msg = exit_status.stderr;
        return Err(String::from_utf8_lossy(&stderr_msg).into());
    }

    Ok(())
}
