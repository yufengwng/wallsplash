//! Module for library specific errors.

use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub enum WallsplashError {
    LocalNoImage,
    UnsplashAPIFail,
    UnsplashNoImage,
}

impl fmt::Display for WallsplashError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl Error for WallsplashError {
    fn description(&self) -> &str {
        match *self {
            WallsplashError::LocalNoImage => "No local images found",
            WallsplashError::UnsplashAPIFail => "Unsplash /photos api failed",
            WallsplashError::UnsplashNoImage => "No images found from Unsplash",
        }
    }
}
