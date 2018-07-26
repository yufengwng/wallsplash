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

    let ctx = args.into_context();
    let status = match wallsplash::run(&ctx) {
        Ok(_) => 0,
        Err(err) => {
            error!("{}", err);
            2
        }
    };

    process::exit(status);
}

mod cli {
    //! Module for defining the application command-line interface.

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

mod cfg {
    //! Module for application-specific configuration file. Defines the structure of the
    //! configuration file format and how to read it into the application.

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

    /// Read the configuration file into a structure. Will default to an empty structure when the
    /// file does not exist, which may happen if user did not specific the file on the command-line
    /// or has a configuration file in the default path.
    ///
    /// # Errors
    ///
    /// Returns an error when problem reading the file or converting to structure.
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

mod def {
    //! Module for application-specific default values. Fallback to these when user does not
    //! provide or set these using other means.

    use std::env;
    use std::path::PathBuf;

    /// 30 minutes in seconds.
    pub const TIMEOUT: u32 = 30 * 60;

    /// 10 images from Unsplash.
    pub const UNSPLASH_LIMIT: u32 = 10;

    /// 24 hours in seconds.
    pub const UNSPLASH_REFRESH: u32 = 24 * 60 * 60;

    /// Get the default configuration file path expected by the application. This assumes that the
    /// user has a valid home directory.
    pub fn config_path() -> PathBuf {
        let mut p = env::home_dir().unwrap();
        p.push(".config");
        p.push("wallsplash");
        p.push("config.toml");
        p
    }
}

mod args {
    //! Module for parsing and massaging application-specific arguments.

    use std::path::Path;
    use std::time::Duration;

    use clap::ArgMatches;
    use wallsplash;

    use cfg;
    use cli;
    use def;

    use ResBoxErr;

    /// Arguments that are merged, normalized, and flattened.
    pub struct Args {
        pub timeout: u32,
        pub local_dir: String,
        pub unsplash_token: String,
        pub unsplash_limit: u32,
        pub unsplash_refresh: u32,
    }

    impl Args {
        /// Arguments to the application comes from 3 different sources:
        ///
        /// 1. command-line arguments
        /// 2. configuration file
        /// 3. default settings
        ///
        /// All these sources are merged into a normalized argument structure. Preference is given
        /// in the listed order, from high to low.
        ///
        /// # Errors
        ///
        /// Possible errors including file I/O issues, configuration file convertion issues,
        /// missing required arguments, or invalid argument formats.
        pub fn parse() -> ResBoxErr<Args> {
            let matches = cli::build_app().get_matches();
            let table = ArgsParser::parse_config_file(&matches)?;
            let parser = ArgsParser::new(matches, table);
            parser.to_args()
        }

        /// Consume and convert arguments to a context object understood by the application engine.
        pub fn into_context(self) -> wallsplash::Context {
            wallsplash::Context::new(
                &self.local_dir,
                &self.unsplash_token,
                self.unsplash_limit,
                Duration::from_secs(self.timeout as u64),
                Duration::from_secs(self.unsplash_refresh as u64),
            )
        }
    }

    struct ArgsParser<'a> {
        matches: ArgMatches<'a>,
        table: cfg::ConfigTable,
    }

    impl<'a> ArgsParser<'a> {
        fn parse_config_file(matches: &ArgMatches) -> ResBoxErr<cfg::ConfigTable> {
            let path = matches
                .value_of("config")
                .map(|p| Path::new(p).to_path_buf())
                .unwrap_or_else(|| {
                    let p = def::config_path();
                    debug!("falling back to default config path {}", p.display());
                    p
                });
            cfg::parse_file(&path)
        }

        fn new(m: ArgMatches<'a>, t: cfg::ConfigTable) -> ArgsParser<'a> {
            ArgsParser {
                matches: m,
                table: t,
            }
        }

        fn to_args(&self) -> ResBoxErr<Args> {
            Ok(Args {
                timeout: self.parse_timeout()?,
                local_dir: self.parse_local_dir()?,
                unsplash_token: self.parse_token()?,
                unsplash_limit: self.parse_limit()?,
                unsplash_refresh: self.parse_refresh()?,
            })
        }

        fn parse_timeout(&self) -> ResBoxErr<u32> {
            let secs = match self.matches.value_of("timeout") {
                Some(secs) => Some(secs.parse::<u32>()?),
                None => None,
            };
            Ok(secs.or(self.table.timeout).unwrap_or(def::TIMEOUT))
        }

        fn parse_local_dir(&self) -> ResBoxErr<String> {
            Ok(self.matches
                .value_of("dir")
                .map(|s| s.to_string())
                .or(self.table.local.as_ref().and_then(|t| t.dir.to_owned()))
                .expect("need a local directory"))
        }

        fn parse_token(&self) -> ResBoxErr<String> {
            Ok(self.matches
                .value_of("token")
                .map(|s| s.to_string())
                .or(self.table
                    .unsplash
                    .as_ref()
                    .and_then(|t| t.token.to_owned()))
                .expect("need unsplash token"))
        }

        fn parse_limit(&self) -> ResBoxErr<u32> {
            let num = match self.matches.value_of("limit") {
                Some(n) => Some(n.parse::<u32>()?),
                None => None,
            };
            Ok(num.or(self.table.unsplash.as_ref().and_then(|t| t.limit))
                .unwrap_or(def::UNSPLASH_LIMIT))
        }

        fn parse_refresh(&self) -> ResBoxErr<u32> {
            let secs = match self.matches.value_of("refresh") {
                Some(secs) => Some(secs.parse::<u32>()?),
                None => None,
            };
            Ok(
                secs.or(self.table.unsplash.as_ref().and_then(|t| t.refresh))
                    .unwrap_or(def::UNSPLASH_REFRESH),
            )
        }
    }
}
