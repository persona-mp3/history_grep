use std::fs;
use std::path::PathBuf;

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

pub fn get_browsing_history(config: &Config) -> Result<&str, Box<dyn std::error::Error>> {
    let file_path = match config.browser.as_str() {
        "chrome" => CHROME_HISTORY_FILE,
        _ => return Err("Browser not supported".into()),
    };

    let homedir = home::home_dir().unwrap_or_else(|| PathBuf::from("~"));

    let full_path = homedir.join(file_path);

    let metadata = match fs::metadata(&full_path) {
        Ok(md) => md,
        Err(err) => {
            eprintln!("Could not get metdata for file: {:?}", full_path);
            return Err(err.into());
        }
    };

    let fsize = metadata.len();
    println!("fsize: {}", fsize);
    Ok("")
}
