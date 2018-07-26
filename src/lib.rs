//! Library for rotating desktop wallpapers using local and Unsplash images.

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate reqwest;

use std::error::Error;
use std::process::Command;
use std::thread;
use std::time::Duration;

mod errors;
mod fetchers;

use fetchers::{Fetch, LocalFetcher, UnsplashFetcher};

/// Information needed by the engine to know what and how to run.
#[derive(Debug)]
pub struct Context {
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

impl Context {
    pub fn new(dir: &str, token: &str, limit: u32, timeout: Duration, refresh: Duration) -> Context {
        Context {
            dir: dir.to_owned(),
            token: token.to_owned(),
            limit: limit,
            timeout: timeout,
            refresh: refresh,
        }
    }
}

/// Execute the never-ending engine!
pub fn run(ctx: &Context) -> Result<(), Box<Error>> {
    debug!("{:?}\n", ctx);

    let mut unsplash = UnsplashFetcher::new(ctx.token.as_str(), ctx.limit, ctx.refresh)?;
    let mut local = LocalFetcher::new(ctx.dir.as_str());

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
                error!("{}", e);
            }
        }

        do_local = !do_local;
        thread::sleep(ctx.timeout);
    }
}
