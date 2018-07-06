#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate toml;
extern crate wallsplash;

use std::process;
use std::error::Error;

mod cli;

fn main() {
    env_logger::init().unwrap();

    let app = cli::build_cli_app();
    let matches = app.get_matches();
    let table = unwrap_log(cli::parse_config_file(&matches));

    let timeout = unwrap_log(cli::parse_arg_timeout(&matches, &table));
    let dir = unwrap_log(cli::parse_arg_local_dir(&matches, &table));
    let token = unwrap_log(cli::parse_arg_token(&matches, &table));
    let limit = unwrap_log(cli::parse_arg_limit(&matches, &table));
    let refresh = unwrap_log(cli::parse_arg_refresh(&matches, &table));

    let config = wallsplash::Config::new(&dir, &token, limit, timeout, refresh);

    process::exit(match wallsplash::run(&config) {
        Ok(_) => 0,
        Err(err) => {
            error!("{}", err);
            2
        }
    });
}

fn unwrap_log<T>(res: Result<T, Box<Error>>) -> T {
    match res {
        Ok(t) => t,
        Err(e) => {
            error!("{}", e);
            process::exit(1);
        }
    }
}
