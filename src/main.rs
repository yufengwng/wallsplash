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

    let args = match args::Args::parse() {
        Ok(a) => a,
        Err(e) => {
            error!("{}", e);
            process::exit(1);
        }
    };

    let ctx = wallsplash::Context::new(
        &args.local_dir,
        &args.unsplash_token,
        args.unsplash_limit,
        Duration::from_secs(args.timeout as u64),
        Duration::from_secs(args.unsplash_refresh as u64),
    );

    process::exit(match wallsplash::run(&ctx) {
        Ok(_) => 0,
        Err(err) => {
            error!("{}", err);
            2
        }
    });
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
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    use toml;

    use ResBoxErr;

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
                timeout: None,
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
                limit: None,
                refresh: None,
            }
        }
    }

    pub fn parse_file(path: &Path) -> ResBoxErr<ConfigTable> {
        if !path.is_file() {
            debug!("config file {} does not exist", path.display());
            return Ok(ConfigTable::default());
        }
        let mut content = String::new();
        File::open(path)?.read_to_string(&mut content)?;
        match toml::from_str::<ConfigTable>(&content) {
            Ok(t) => Ok(t),
            Err(e) => Err(Box::new(e)),
        }
    }
}

mod defaults {
    use std::env;
    use std::path::PathBuf;

    pub const TIMEOUT: u32 = 30 * 60;
    pub const UNSPLASH_LIMIT: u32 = 10;
    pub const UNSPLASH_REFRESH: u32 = 24 * 60 * 60;

    pub fn config_path() -> PathBuf {
        let mut p = env::home_dir().unwrap();
        p.push(".config");
        p.push("wallsplash");
        p.push("config.toml");
        p
    }
}

mod args {
    use std::path::Path;

    use clap::ArgMatches;

    use cli;
    use config::{self, ConfigTable};
    use defaults;

    use ResBoxErr;

    pub struct Args {
        pub timeout: u32,
        pub local_dir: String,
        pub unsplash_token: String,
        pub unsplash_limit: u32,
        pub unsplash_refresh: u32,
    }

    impl Args {
        pub fn parse() -> ResBoxErr<Args> {
            let matches = cli::build_app().get_matches();
            let table = parse_config_file(&matches)?;

            let timeout = parse_arg_timeout(&matches, &table)?;
            let dir = parse_arg_local_dir(&matches, &table)?;
            let token = parse_arg_token(&matches, &table)?;
            let limit = parse_arg_limit(&matches, &table)?;
            let refresh = parse_arg_refresh(&matches, &table)?;

            Ok(Args {
                timeout: timeout,
                local_dir: dir,
                unsplash_token: token,
                unsplash_limit: limit,
                unsplash_refresh: refresh,
            })
        }
    }

    fn parse_config_file(matches: &ArgMatches) -> ResBoxErr<ConfigTable> {
        let path = matches
            .value_of("config")
            .map(|p| Path::new(p).to_path_buf())
            .unwrap_or_else(|| {
                let p = defaults::config_path();
                debug!("falling back to default config path {}", p.display());
                p
            });
        config::parse_file(&path)
    }

    fn parse_arg_timeout(matches: &ArgMatches, table: &ConfigTable) -> ResBoxErr<u32> {
        let secs = match matches.value_of("timeout") {
            Some(secs) => Some(secs.parse::<u32>()?),
            None => None,
        };
        Ok(secs.or(table.timeout).unwrap_or(defaults::TIMEOUT))
    }

    fn parse_arg_local_dir(matches: &ArgMatches, table: &ConfigTable) -> ResBoxErr<String> {
        Ok(matches
            .value_of("dir")
            .map(|s| s.to_string())
            .or(table.local.as_ref().and_then(|t| t.dir.to_owned()))
            .expect("need a local directory"))
    }

    fn parse_arg_token(matches: &ArgMatches, table: &ConfigTable) -> ResBoxErr<String> {
        Ok(matches
            .value_of("token")
            .map(|s| s.to_string())
            .or(table.unsplash.as_ref().and_then(|t| t.token.to_owned()))
            .expect("need unsplash token"))
    }

    fn parse_arg_limit(matches: &ArgMatches, table: &ConfigTable) -> ResBoxErr<u32> {
        let num = match matches.value_of("limit") {
            Some(n) => Some(n.parse::<u32>()?),
            None => None,
        };
        Ok(num.or(table.unsplash.as_ref().and_then(|t| t.limit))
            .unwrap_or(defaults::UNSPLASH_LIMIT))
    }

    fn parse_arg_refresh(matches: &ArgMatches, table: &ConfigTable) -> ResBoxErr<u32> {
        let secs = match matches.value_of("refresh") {
            Some(secs) => Some(secs.parse::<u32>()?),
            None => None,
        };
        Ok(secs.or(table.unsplash.as_ref().and_then(|t| t.refresh))
            .unwrap_or(defaults::UNSPLASH_REFRESH))
    }
}
