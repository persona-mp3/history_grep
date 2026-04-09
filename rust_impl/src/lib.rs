#![allow(dead_code)]
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
pub const CHROME_HISTORY_FILE: &'static str =
    "Library/Application Support/Google/Chrome/Default/History";

#[derive(Debug)]
struct VisitedUrl {
    id: i32,
    link_url_id: i32,
    top_level_url: String,
    frame_url: String,
    visit_count: i32,
}

pub fn get_browsing_history(config: &Config) -> Result<&str, Box<dyn std::error::Error>> {
    let file_path = match config.browser.as_str() {
        "chrome" => CHROME_HISTORY_FILE,
        _ => return Err("Browser not supported".into()),
    };

    let homedir = home::home_dir().unwrap_or_else(|| PathBuf::from("~"));

    let full_path = homedir.join(file_path);

    let temp_file = PathBuf::from("temp_history.db");
    Command::new("cp").args([&full_path, &temp_file]).output()?;

    // Docs: https://crates.io/crates/rusqlite/
    let conn = Connection::open(&temp_file)?;
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

    Command::new("rm").arg(&temp_file).output()?;
    Ok("")
}
