//! Module for command-line related things.

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use clap::App;
use clap::Arg;
use clap::ArgMatches;
use toml;

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

#[derive(Debug, Deserialize)]
pub struct ConfigTable {
    timeout: Option<u32>,
    local: Option<LocalTable>,
    unsplash: Option<UnsplashTable>,
}

#[derive(Debug, Deserialize)]
struct LocalTable {
    dir: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UnsplashTable {
    token: Option<String>,
    limit: Option<u32>,
    refresh: Option<u32>,
}

impl Default for ConfigTable {
    fn default() -> ConfigTable {
        ConfigTable {
            timeout: Some(30 * 60),
            local: Default::default(),
            unsplash: Default::default(),
        }
    }
}

impl Default for LocalTable {
    fn default() -> LocalTable {
        LocalTable { dir: None }
    }
}

impl Default for UnsplashTable {
    fn default() -> UnsplashTable {
        UnsplashTable {
            token: None,
            limit: Some(10),
            refresh: Some(24 * 60 * 60),
        }
    }
}

fn default_config_path() -> PathBuf {
    let mut p = env::home_dir().unwrap();
    p.push(".config");
    p.push("wallsplash");
    p.push("config.toml");
    p
}

pub fn parse_config_file(matches: &ArgMatches) -> Result<ConfigTable, Box<Error>> {
    let path = match matches.value_of("config") {
        Some(p) => Path::new(p).to_path_buf(),
        None => {
            let p = default_config_path();
            debug!(
                "config file not specified, trying default config file at {}",
                p.display()
            );
            p
        }
    };
    if !path.is_file() {
        return Ok(ConfigTable::default());
    }
    let mut content = String::new();
    File::open(path)?.read_to_string(&mut content)?;
    match toml::from_str::<ConfigTable>(&content) {
        Ok(t) => Ok(t),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn parse_arg_timeout(
    matches: &ArgMatches,
    table: &ConfigTable,
) -> Result<Duration, Box<Error>> {
    let secs = match matches.value_of("timeout") {
        Some(secs) => Some(secs.parse::<u64>()?),
        None => None,
    };
    let dur = match secs {
        Some(secs) => Some(Duration::from_secs(secs)),
        None => match table.timeout {
            Some(int) => Some(Duration::from_secs(int as u64)),
            None => None,
        },
    };
    match dur {
        Some(dur) => Ok(dur),
        None => Ok(Duration::from_secs(30 * 60)),
    }
}

pub fn parse_arg_local_dir(
    matches: &ArgMatches,
    table: &ConfigTable,
) -> Result<String, Box<Error>> {
    let dir = match matches.value_of("dir") {
        Some(path) => path.to_string(),
        None => table
            .local
            .as_ref()
            .expect("need a local directory")
            .dir
            .as_ref()
            .expect("need a local directory")
            .to_string(),
    };
    Ok(dir)
}

pub fn parse_arg_token(matches: &ArgMatches, table: &ConfigTable) -> Result<String, Box<Error>> {
    Ok(match matches.value_of("token") {
        Some(tok) => tok.to_string(),
        None => table
            .unsplash
            .as_ref()
            .expect("need unsplash token")
            .token
            .as_ref()
            .expect("need unsplash token")
            .to_string(),
    })
}

pub fn parse_arg_limit(matches: &ArgMatches, table: &ConfigTable) -> Result<u32, Box<Error>> {
    Ok(match matches.value_of("limit") {
        Some(num) => num.parse::<u32>()?,
        None => {
            let limit = match table.unsplash {
                Some(ref t) => t.limit,
                None => None,
            };
            match limit {
                Some(num) => num as u32,
                None => 10,
            }
        }
    })
}

pub fn parse_arg_refresh(
    matches: &ArgMatches,
    table: &ConfigTable,
) -> Result<Duration, Box<Error>> {
    Ok(match matches.value_of("refresh") {
        Some(secs) => Duration::from_secs(secs.parse::<u64>()?),
        None => {
            let refresh = match table.unsplash {
                Some(ref t) => t.refresh,
                None => None,
            };
            match refresh {
                Some(r) => Duration::from_secs(r as u64),
                None => Duration::from_secs(24 * 60 * 60),
            }
        }
    })
}
