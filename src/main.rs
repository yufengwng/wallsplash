extern crate wallsplash;

use std::process;
use std::time::Duration;

fn main() {
    let config = wallsplash::Config::new(
        "<LOCAL_WALLPAPER_DIR>",
        "<UNSPLASH_API_TOKEN>",
        10,
        Duration::from_secs(30 * 60),
        Duration::from_secs(24 * 60 * 60),
    );

    let result = match wallsplash::run(&config) {
        Ok(_) => 0,
        Err(err) => {
            println!("{}", err);
            1
        }
    };

    process::exit(result);
}
