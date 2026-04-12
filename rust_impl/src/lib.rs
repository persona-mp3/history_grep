#![allow(dead_code)]
use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use rusqlite::{Connection, Result};

use home;
#[derive(Debug)]
pub struct Config {
    pub browser: String,
    pub limit: u32,
    pub fzf: String,
}

impl Config {
    pub fn default() -> Config {
        Config {
            browser: String::from("chrome"),
            limit: 600,
            fzf: String::from("fzf"),
        }
    }

    pub fn build(args: &[String]) -> Result<Config, Box<dyn std::error::Error>> {
        if args.len() < 4 {
            return Err("Not enough arguments\
                hist <browser-name> <limit> <fzf>
                "
            .into());
        }

        let browser = &args[1];
        let limit = args[2].clone().trim().parse::<u32>()?;
        Ok(Config {
            browser: browser.clone(),
            limit: limit,
            fzf: String::from("fzf"),
        })
    }
}
pub const CHROME_HISTORY_FILE_MACOS: &'static str =
    "Library/Application Support/Google/Chrome/Default/History";

// https://issarice.com/export-chrome-history
pub const CHROME_HISTORY_FILE_LINUX: &'static str = ".config/google-chrome/Default/History";

#[derive(Debug)]
pub struct VisitedUrl {
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

/// Matches the OS the application is running on an automatically resolves the possible file-path
/// of the browser. If successful, it returns the temporary  file path, as `.temp_chrome_history.db`
/// Callers need to call `cleanup()` to delete the file
pub fn copy_browing_history(config: &Config) -> Result<PathBuf, Box<dyn std::error::Error>> {
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

    let exit_status = Command::new("cp")
        .args([&browser_history_path, &temp_file])
        .output()?;

    if !exit_status.status.success() {
        eprintln!("Failed to copy history file {:?}", browser_history_path);
        let stderr_msg = exit_status.stderr;
        return Err(String::from_utf8_lossy(&stderr_msg).into());
    }

    println!("Copy was successfull");
    Ok(temp_file)
}

/// Parses the browsing history from the temp_file path using sqlite, reading the rows into `VisitedUrl`
pub fn parse_browsing_history(
    temp_file: &PathBuf,
) -> Result<Vec<VisitedUrl>, Box<dyn std::error::Error>> {
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

    let mut valid_data = vec![];
    for v_url in visited_urls_iter {
        valid_data.push(v_url.unwrap());
    }

    Ok(valid_data)
}

fn get_fzf(fzf: &String) -> Result<String, Box<dyn std::error::Error>> {
    // TODO:(persona):
    // 1. Check if commands were executed successfully
    let exit_status = Command::new("which").arg(fzf).output()?;
    let stdout = String::from_utf8(exit_status.stdout)?;
    let stdout = stdout.trim_end();

    Ok(stdout.to_string())
}

pub fn collect_input(
    config: &Config,
    temp_file: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let fzf_executable = get_fzf(&config.fzf)?;
    let browsing_history = parse_browsing_history(temp_file)?;

    // NOTE: I'd like it to panic here, so I can get a stack trace of what and where something
    // failed
    let mut fzf_process = Command::new(fzf_executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut fzf_stdin = if let Some(stdin) = fzf_process.stdin.take() {
        stdin
    } else {
        return Err("Could not get stdin of fzf_process".into());
    };

    let mut fzf_stdout = if let Some(stdout) = fzf_process.stdout.take() {
        stdout
    } else {
        return Err("Could not get stdout of fzf_process".into());
    };

    let mut all_urls: Vec<String> = vec![];
    for url in browsing_history {
        all_urls.push(url.frame_url);
    }

    let all_urls = all_urls.join("\n");

    write!(fzf_stdin, "{}", all_urls).expect("Could not write to fzf's stdin!");
    drop(fzf_stdin);
    fzf_process.wait().expect("fzf exited with an error");

    let mut buffer = String::new();
    fzf_stdout
        .read_to_string(&mut buffer)
        .expect("Could not read from fzfs stdout!");

    println!("User selected {}", buffer);
    Ok(())
}

pub fn cleanup(temp_file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let exit_status = Command::new("rm").arg(&temp_file).output()?;
    // NOTE: This is a 0 based exit status check?
    if !exit_status.status.success() {
        eprintln!("Failed to remove history file {:?}", temp_file);
        let stderr_msg = exit_status.stderr;
        return Err(String::from_utf8_lossy(&stderr_msg).into());
    }
    Ok(())
}
