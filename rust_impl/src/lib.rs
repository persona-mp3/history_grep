#![allow(dead_code)]
use std::env;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use home;
use rusqlite::{Connection, Result};

#[derive(Debug, PartialEq)]
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
                hist <browser-name> <limit> <fzf>"
                .into());
        }

        let browser = args[1].clone();
        let limit = args[2].clone().trim().parse::<u32>()?;
        let fzf = args[3].clone();

        Ok(Config {
            browser,
            limit,
            fzf,
        })
    }
}

#[derive(Debug)]
/// Table in sqlite format
pub struct VisitedUrl {
    id: i32,
    link_url_id: i32,
    top_level_url: String,
    frame_url: String,
    visit_count: i32,
}

// REVIEW:
//  Would need to refactor this to collect other params, for example
//  instead of just OS, it could also have the `history_path` and browser.exe
//  So we could just use it as:
//  parse_browing_histort(OSArch.history_path) -> Result<VistedUrl, Error>{}
//  collect_input(v: Vec[VisitedUrl>, OSArch.browser_exe) -> Result<(), Error>{}
#[derive(Debug)]
pub enum OSArch {
    Linux(String),
    MacOS(String),
    Windows(String),
    Unsupported(String),
}

// Would need a better name for this
pub struct OsInfo {
    os_arch: OSArch,
    browser_history_path: PathBuf,
    browser_exec: PathBuf,
}

impl OSArch {
    fn value(&self) -> &str {
        match self {
            OSArch::Linux(s) => s,
            OSArch::MacOS(s) => s,
            OSArch::Windows(s) => s,
            OSArch::Unsupported(s) => s,
        }
    }
}

pub fn get_system_os() -> OSArch {
    let user_os = env::consts::OS;
    let os_arch: OSArch = match user_os {
        "linux" => OSArch::Linux(String::from("linux")),
        "macos" => OSArch::MacOS(String::from("macos")),
        "windows" => OSArch::Windows(String::from("windows")),
        _ => {
            let msg = format!("{} is not yet supported. Feel free to make PRs!", user_os);
            OSArch::Unsupported(msg)
        }
    };

    os_arch
}

pub const CHROME_HISTORY_FILE_MACOS: &'static str =
    "Library/Application Support/Google/Chrome/Default/History";

// https://issarice.com/export-chrome-history
pub const CHROME_HISTORY_FILE_LINUX: &'static str = ".config/google-chrome/Default/History";

const CHROME_EXEC_PATH_MACOS: &'static str =
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const CHROME_EXEC_FILE_LINUX: &'static str = "/usr/bin/google-chrome";

pub const TEMP_FILE: &'static str = ".temp_chrome_history.db";

pub fn get_browser_info(config: &Config) -> Result<OsInfo, Box<dyn std::error::Error>> {
    let os_arch = get_system_os();
    let mut browser_history_path = PathBuf::new();
    let mut browser_exec = PathBuf::new();
    match config.browser.as_str() {
        "chrome" => {
            match &os_arch {
                OSArch::Linux(_) => {
                    browser_history_path.push(CHROME_HISTORY_FILE_LINUX);
                    browser_exec.push(CHROME_EXEC_FILE_LINUX)
                }
                OSArch::MacOS(_) => {
                    browser_history_path.push(CHROME_HISTORY_FILE_MACOS);
                    browser_exec.push(CHROME_EXEC_PATH_MACOS)
                }
                OSArch::Windows(_) => {
                    browser_history_path.push("Windows has not yet been developed")
                }
                OSArch::Unsupported(msg) => browser_history_path.push(msg),
            };
        }

        _ => {
            let err_msg = format!(
                "{} has not yet been supported, PR's are welcome!",
                config.browser
            );
            return Err(err_msg.into());
        }
    }

    Ok(OsInfo {
        os_arch,
        browser_history_path,
        browser_exec,
    })
}

/// Matches the OS the application is running on an automatically resolves the possible file-path
/// of the browser. If successful, it returns the temporary  file path, as `.temp_chrome_history.db`
/// Callers need to call `cleanup()` to delete the file
pub fn copy_browing_history(config: &Config) -> Result<OsInfo, Box<dyn std::error::Error>> {
    let info = get_browser_info(config)?;
    let home_dir = match home::home_dir() {
        Some(v) => v,
        None => {
            return Err("Could not get home directory".into());
        }
    };

    let temp_file = PathBuf::from(TEMP_FILE);
    let browser_history_path = home_dir.join(&info.browser_history_path);

    let exit_status = Command::new("cp")
        .args([&browser_history_path, &temp_file])
        .output()?;

    if !exit_status.status.success() {
        eprintln!("Failed to copy history file {:?}", browser_history_path);
        let stderr_msg = exit_status.stderr;
        return Err(String::from_utf8_lossy(&stderr_msg).into());
    }

    Ok(info)
}

/// Parses the browsing history from the temp_file path using sqlite, reading the rows into `VisitedUrl`
pub fn parse_browsing_history(
    search_limit: u32,
) -> Result<Vec<VisitedUrl>, Box<dyn std::error::Error>> {
    let conn = match Connection::open(TEMP_FILE) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Could not open connection for file");
            return Err(err.into());
        }
    };

    let mut stmt = conn.prepare("SELECT * FROM visited_links ORDER BY id DESC LIMIT (?1)")?;
    let visited_urls_iter = stmt.query_map([search_limit], |row| {
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

/// Sends a number of target links from the database into fzf and wait's for user
/// input. It panics when there are errors in piping stdout and stin processes of
/// fzf. When done, it returns the target link the user wants to visit
pub fn collect_target_link(config: &Config) -> Result<String, Box<dyn std::error::Error>> {
    let fzf_executable = get_fzf(&config.fzf)?;
    let browsing_history = parse_browsing_history(config.limit)?;

    // I'd like it to panic here, so I can get a stack trace of what and where something failed
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
        all_urls.push(url.top_level_url);
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
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // Tested on x86
    fn test_gets_os() {
        let actual_os = env::consts::OS;
        let expected_os = get_system_os();
        assert_eq!(actual_os, expected_os.value());
    }

    #[test]
    fn test_default_config() {
        let actual_result = Config::default();
        let expected_config = Config {
            browser: String::from("chrome"),
            limit: 600,
            fzf: String::from("fzf"),
        };
        assert_eq!(actual_result, expected_config)
    }

    #[test]
    fn test_build_config() {
        let args = vec![
            String::from("executable-path"),
            String::from("chrome"),
            String::from("600"),
            String::from("fzf"),
        ];
        let expected_config = Config {
            browser: args[1].clone(),
            limit: 600,
            fzf: args[3].clone(),
        };

        let actual_config = Config::build(&args).unwrap();
        assert_eq!(expected_config, actual_config)
    }

    #[test]
    fn test_build_config_fails() {
        let args = vec![
            String::from("executable-path"),
            String::from("chrome"),
            String::from("fzf"),
        ];

        let expected_err_msg = "Not enough arguments\
                                hist <browser-name> <limit> <fzf>";

        // Would also want to test against stderr without 3rd party crates but it can cause problems when running on Windows.
        let _ = Config::build(&args)
            .inspect_err(|err| assert_eq!(err.to_string().trim(), expected_err_msg.trim(),));
    }
}
