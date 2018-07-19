#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate wallsplash;

use std::error::Error;
use std::process;
use std::time::Duration;

type ResBoxErr<T> = Result<T, Box<Error>>;

fn main() {
    env_logger::init().unwrap();

    let app = cli::build_app();
    let matches = app.get_matches();
    let table = unwrap_log(args::parse_config_file(&matches));

    let timeout = unwrap_log(args::parse_arg_timeout(&matches, &table));
    let dir = unwrap_log(args::parse_arg_local_dir(&matches, &table));
    let token = unwrap_log(args::parse_arg_token(&matches, &table));
    let limit = unwrap_log(args::parse_arg_limit(&matches, &table));
    let refresh = unwrap_log(args::parse_arg_refresh(&matches, &table));

    let config = wallsplash::Config::new(&dir, &token, limit, timeout, refresh);

    process::exit(match wallsplash::run(&config) {
        Ok(_) => 0,
        Err(err) => {
            error!("{}", err);
            2
        }
    });
}

fn unwrap_log<T>(res: ResBoxErr<T>) -> T {
    match res {
        Ok(t) => t,
        Err(e) => {
            error!("{}", e);
            process::exit(1);
        }
    }
}

mod cli {
    use clap::App;
    use clap::Arg;

    pub fn build_app() -> App<'static, 'static> {
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
}

mod config {
    #[derive(Debug, Deserialize)]
    pub struct ConfigTable {
        pub timeout: Option<u32>,
        pub local: Option<LocalTable>,
        pub unsplash: Option<UnsplashTable>,
    }

    #[derive(Debug, Deserialize)]
    pub struct LocalTable {
        pub dir: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    pub struct UnsplashTable {
        pub token: Option<String>,
        pub limit: Option<u32>,
        pub refresh: Option<u32>,
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
}

mod defaults {
    use std::env;
    use std::path::PathBuf;

    pub fn config_path() -> PathBuf {
        let mut p = env::home_dir().unwrap();
        p.push(".config");
        p.push("wallsplash");
        p.push("config.toml");
        p
    }
}

mod args {
    use super::config::*;
    use super::*;
    use clap::ArgMatches;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    use toml;

    pub fn parse_config_file(matches: &ArgMatches) -> ResBoxErr<ConfigTable> {
        let path = match matches.value_of("config") {
            Some(p) => Path::new(p).to_path_buf(),
            None => {
                let p = defaults::config_path();
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

    pub fn parse_arg_timeout(matches: &ArgMatches, table: &ConfigTable) -> ResBoxErr<Duration> {
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

    pub fn parse_arg_local_dir(matches: &ArgMatches, table: &ConfigTable) -> ResBoxErr<String> {
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

    pub fn parse_arg_token(matches: &ArgMatches, table: &ConfigTable) -> ResBoxErr<String> {
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

    pub fn parse_arg_limit(matches: &ArgMatches, table: &ConfigTable) -> ResBoxErr<u32> {
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

    pub fn parse_arg_refresh(matches: &ArgMatches, table: &ConfigTable) -> ResBoxErr<Duration> {
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
}
