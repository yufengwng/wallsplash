#[macro_use] extern crate log;
extern crate clap;
extern crate env_logger;
extern crate wallsplash;

use std::process;
use std::time::Duration;

use clap::{Arg, App};


fn main() {
    env_logger::init().unwrap();

    let matches = App::new("wallsplash")
        .version("0.0.1")
        .author("Yufeng Wang <yufengwang05@gmail.com>")
        .about("Display wallpapers from local image directory and Unsplash.")
        .arg(Arg::with_name("local_dir")
             .long("dir")
             .takes_value(true)
             .value_name("PATH")
             .required(true)
             .help("Local directory of images"))
        .arg(Arg::with_name("api_token")
             .long("token")
             .takes_value(true)
             .value_name("TOKEN")
             .required(true)
             .help("Unsplash API token"))
        .arg(Arg::with_name("limit")
             .long("limit")
             .takes_value(true)
             .value_name("NUM")
             .help("Max number of Unsplash images to download and cache, default 10"))
        .arg(Arg::with_name("timeout")
             .long("timeout")
             .takes_value(true)
             .value_name("SECS")
             .help("Seconds to wait before displaying next image, default 1800 (30 mins)"))
        .arg(Arg::with_name("refresh")
             .long("refresh")
             .takes_value(true)
             .value_name("SECS")
             .help("Seconds to wait before refreshing Unsplash image cache, default 86400 (1 day)"))
        .get_matches();

    let limit = match matches.value_of("limit") {
        None => {
            debug!("defaulting `limit` to 10");
            10
        }
        Some(num) => {
            match num.parse::<u32>() {
                Ok(n) => n,
                Err(e) => {
                    error!("`limit` arg: {}", e);
                    process::exit(1);
                }
            }
        }
    };

    let timeout = match matches.value_of("timeout") {
        None => {
            debug!("defaulting `timeout` to 30 mins");
            Duration::from_secs(30 * 60)
        }
        Some(secs) => {
            match secs.parse::<u64>() {
                Ok(s) => Duration::from_secs(s),
                Err(e) => {
                    error!("`timeout` arg: {}", e);
                    process::exit(1);
                }
            }
        }
    };

    let refresh = match matches.value_of("refresh") {
        None => {
            debug!("defaulting `refresh` to 1 day");
            Duration::from_secs(24 * 60 * 60)
        }
        Some(secs) => {
            match secs.parse::<u64>() {
                Ok(s) => Duration::from_secs(s),
                Err(e) => {
                    error!("`refresh` arg: {}", e);
                    process::exit(1);
                }
            }
        }
    };

    let config = wallsplash::Config::new(
        matches.value_of("local_dir").unwrap(),
        matches.value_of("api_token").unwrap(),
        limit,
        timeout,
        refresh,
    );

    let result = match wallsplash::run(&config) {
        Ok(_) => 0,
        Err(err) => {
            error!("{}", err);
            1
        }
    };

    process::exit(result);
}
