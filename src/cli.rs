//! Module for command-line related things.

extern crate toml;
extern crate wallsplash;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::time::Duration;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use toml::Value;

pub fn build_cli_app() -> App<'static, 'static> {
    App::new("wallsplash")
        .version("0.0.3")
        .author("Yufeng Wang <yufengwang05@gmail.com>")
        .about("Display wallpapers from local image directory and Unsplash.")
        .arg(
            Arg::with_name("config")
                .long("config")
                .takes_value(true)
                .value_name("PATH")
                .help("Path to configuration file"),
        )
        .arg(
            Arg::with_name("dir")
                .long("dir")
                .takes_value(true)
                .value_name("PATH")
                .help("Path to local directory of images"),
        )
        .arg(
            Arg::with_name("limit")
                .long("limit")
                .takes_value(true)
                .value_name("NUM")
                .help("Max number of Unsplash images to download and cache, default 10"),
        )
        .arg(
            Arg::with_name("refresh")
                .long("refresh")
                .takes_value(true)
                .value_name("SECS")
                .help("Seconds before refreshing Unsplash image cache, default 86400 (1 day)"),
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .takes_value(true)
                .value_name("SECS")
                .help("Seconds before displaying next image, default 1800 (30 mins)"),
        )
        .arg(
            Arg::with_name("token")
                .long("token")
                .takes_value(true)
                .value_name("TOKEN")
                .help("Unsplash API token"),
        )
}

pub fn parse_config_file(matches: &ArgMatches) -> Result<Value, Box<Error>> {
    let path = match matches.value_of("config") {
        Some(p) => p.to_string(),
        None => {
            let mut p = env::home_dir().unwrap();
            p.push(".config");
            p.push("wallsplash");
            p.push("config.toml");
            p.to_str().unwrap().to_string()
        }
    };
    let mut content = String::new();
    File::open(path)?.read_to_string(&mut content)?;
    match content.parse::<Value>() {
        Ok(v) => Ok(v),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn parse_arg_timeout(matches: &ArgMatches, table: &Value) -> Result<Duration, Box<Error>> {
    let secs = match matches.value_of("timeout") {
        Some(secs) => Some(secs.parse::<u64>()?),
        None => None,
    };
    let dur = match secs {
        Some(secs) => Some(Duration::from_secs(secs)),
        None => match table["timeout"].as_integer() {
            Some(int) => Some(Duration::from_secs(int as u64)),
            None => None,
        },
    };
    match dur {
        Some(dur) => Ok(dur),
        None => Ok(Duration::from_secs(30 * 60)),
    }
}

pub fn parse_arg_local_dir(matches: &ArgMatches, table: &Value) -> Result<String, Box<Error>> {
    let dir = match matches.value_of("dir") {
        Some(path) => path,
        None => {
            let local = match table["local"].as_table() {
                Some(t) => t,
                None => panic!("need a local directory"),
            };
            match local["dir"].as_str() {
                Some(s) => s,
                None => panic!("need a local directory"),
            }
        }
    };
    Ok(dir.to_string())
}

pub fn parse_arg_token(matches: &ArgMatches, table: &Value) -> Result<String, Box<Error>> {
    Ok(match matches.value_of("token") {
        Some(tok) => tok.to_string(),
        None => table["unsplash"].as_table().expect("need unsplash token")["token"]
            .as_str()
            .expect("need unsplash token")
            .to_string(),
    })
}

pub fn parse_arg_limit(matches: &ArgMatches, table: &Value) -> Result<u32, Box<Error>> {
    Ok(match matches.value_of("limit") {
        Some(num) => num.parse::<u32>()?,
        None => {
            let unsplash = match table["unsplash"].as_table() {
                Some(t) => Some(t),
                None => None,
            };
            let limit = match unsplash {
                Some(t) => t["limit"].as_integer(),
                None => None,
            };
            match limit {
                Some(num) => num as u32,
                None => 10,
            }
        }
    })
}

pub fn parse_arg_refresh(matches: &ArgMatches, table: &Value) -> Result<Duration, Box<Error>> {
    Ok(match matches.value_of("refresh") {
        Some(secs) => Duration::from_secs(secs.parse::<u64>()?),
        None => {
            let refresh = match table["unsplash"].as_table() {
                Some(t) => t["refresh"].as_integer(),
                None => None,
            };
            match refresh {
                Some(r) => Duration::from_secs(r as u64),
                None => Duration::from_secs(24 * 60 * 60),
            }
        }
    })
}
