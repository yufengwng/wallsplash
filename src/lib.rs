#[macro_use] extern crate serde_derive;
extern crate reqwest;
extern crate serde;
extern crate tempdir;

use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

mod errors;
mod unsplash;

use errors::WallsplashError;

#[derive(Debug)]
pub struct Config {
    /// Local directory path to find user wallpapers.
    dir: String,
    /// Unsplash API Client token.
    token: String,
    /// Number of images to cache, max 30.
    limit: u32,
    /// Seconds timeout before displaying next wallpaper.
    timeout: Duration,
    /// Seconds timeout before refreshing Unsplash images.
    refresh: Duration,
}

impl Config {
    pub fn new(dir: &str, token: &str, limit: u32, timeout: Duration, refresh: Duration) -> Self {
        Config {
            dir: dir.to_owned(),
            token: token.to_owned(),
            limit: limit,
            timeout: timeout,
            refresh: refresh,
        }
    }
}

struct LocalFetcher {
    dir: String,
    curr: usize,
}

impl LocalFetcher {
    fn new(dir: &str) -> LocalFetcher {
        LocalFetcher {
            dir: dir.to_owned(),
            curr: 0,
        }
    }
}

impl LocalFetcher {
    fn next_image_path(&mut self) -> Result<PathBuf, Box<Error>> {
        let mut images = Vec::new();

        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                images.push(path);
            }
        }

        if images.len() > 0 {
            if self.curr >= images.len() {
                self.curr = 0;
            }

            let path = images[self.curr].clone();
            self.curr += 1;

            println!("local: {:?}", path);
            return Ok(path);
        }

        Err(Box::new(WallsplashError::UnsplashNoImage))
    }
}

pub fn run(config: &Config) -> Result<(), Box<Error>> {
    println!("{:?}\n", config);

    let mut unsplash = unsplash::Client::new(config.token.as_str(), config.limit, config.refresh)?;
    let mut local = LocalFetcher::new(config.dir.as_str());

    let mut do_local = true;

    loop {
        let path = if do_local {
            local.next_image_path()
        } else {
            unsplash.next_image_path()
        };

        match path {
            Ok(path) => {
                Command::new("feh").arg("--bg-fill").arg(path).output()?;
            }
            Err(e) => {
                println!("{}", e);
            }
        }

        do_local = !do_local;
        thread::sleep(config.timeout);
    }
}
