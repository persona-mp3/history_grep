use rust_impl::{Config, copy_browing_history};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub struct TempFile(PathBuf);

impl TempFile {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        TempFile(path.into())
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if self.0.exists() {
            let _ = fs::remove_file(&self.0);
        }
    }
}

// NOTE: If you're on a linux distro, you need to install sqlite so the application
// can interface with it. Spefically this was built on a late night cafee, on debian distro
// and I ran `sudo apt-get install libsqlite3-dev`. Since I choose to use the `rusqlite` crate
// it screams alot when it can't find it

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let config = if args.len() >= 3 {
        Config::build(&args)?
    } else {
        Config::default()
    };

    let _browser_info = copy_browing_history(&config)?;
    let _cleanup = TempFile::new(rust_impl::TEMP_FILE);
    let _target_link = rust_impl::collect_target_link(&config)?;

    Ok(())
}
