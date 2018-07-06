//! Module for image fetchers.

use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use reqwest;
use reqwest::header::{Authorization, ContentType};
use reqwest::mime::{Mime, SubLevel, TopLevel};

use errors::WallsplashError;

pub trait Fetch {
    /// Returns the file path for the next image to display.
    fn next_image_path(&mut self) -> Result<PathBuf, Box<Error>>;
}

/// Fetcher for local images.
#[derive(Debug)]
pub struct LocalFetcher {
    /// Local directory to search for images.
    dir: String,
    /// Index of next image to use.
    next: usize,
}

impl LocalFetcher {
    pub fn new(dir: &str) -> Self {
        LocalFetcher {
            dir: dir.to_owned(),
            next: 0,
        }
    }
}

impl Fetch for LocalFetcher {
    fn next_image_path(&mut self) -> Result<PathBuf, Box<Error>> {
        let mut images = Vec::new();

        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                images.push(path);
            }
        }

        if images.len() > 0 {
            self.next = self.next % images.len();

            let path = images[self.next].clone();
            self.next += 1;

            debug!("local: {:?}", path);
            return Ok(path);
        }

        Err(Box::new(WallsplashError::LocalNoImage))
    }
}

const UNSPLASH_API: &'static str = "https://api.unsplash.com";
const PHOTOS_ENDPOINT: &'static str = "/photos";

#[derive(Deserialize, Debug)]
struct Photo {
    id: String,
    links: Links,
}

#[derive(Deserialize, Debug)]
struct Links {
    download: String,
}

/// Fetcher for images provided by Unsplash.
#[derive(Debug)]
pub struct UnsplashFetcher {
    /// Unsplash API token.
    token: String,
    /// Max number of images to get from Unsplash.
    limit: u32,
    /// Directory for caching images.
    dir: PathBuf,
    /// Index of next image to use.
    next: usize,
    /// Total number of images cached.
    total: usize,
    /// Whether caching is complete.
    cached: bool,
    /// Time until next refresh of image cache.
    refresh: Duration,
    /// Time when successful cache is completed.
    timestamp: Instant,
}

impl UnsplashFetcher {
    pub fn new(token: &str, limit: u32, refresh: Duration) -> Result<Self, Box<Error>> {
        let mut cache = env::home_dir().unwrap();
        cache.push(".config");
        cache.push("wallsplash");
        cache.push("cache");

        if !cache.exists() || !cache.is_dir() {
            debug!("creating cache directory {:?}", cache);
            fs::create_dir_all(&cache)?;
        }

        Ok(UnsplashFetcher {
            token: token.to_owned(),
            limit: limit,
            dir: cache,
            next: 0,
            total: 0,
            cached: false,
            refresh: refresh,
            timestamp: Instant::now(),
        })
    }

    /// Calls Unsplash API to download and cache images.
    fn download_images(&mut self) -> Result<usize, Box<Error>> {
        let photos_uri = format!(
            "{}{}?per_page={}&order_by=latest",
            UNSPLASH_API, PHOTOS_ENDPOINT, self.limit
        );
        debug!("url: {}\n", photos_uri);

        let request = reqwest::Client::new()?;
        let mut resp = request
            .get(&photos_uri)
            .header(Authorization(format!("Client-ID {}", self.token)))
            .send()?;

        debug!("response: {}", resp.url());
        debug!("status:   {}", resp.status());
        debug!("headers:\n\n{}", resp.headers());

        if !resp.status().is_success() {
            return Err(Box::new(WallsplashError::UnsplashAPIFail));
        }

        let photos: Vec<Photo> = resp.json()?;
        debug!("json: {:?}", photos);

        let mut idx = 0;
        for photo in &photos {
            let img_url = &photo.links.download;
            debug!("downloading: {}", img_url);

            let mut resp = request.get(img_url.as_str()).send()?;

            debug!("response: {}", resp.url());
            debug!("status:   {}", resp.status());
            debug!("headers:\n\n{}", resp.headers());

            let mut img_file = match resp.headers().get::<ContentType>() {
                Some(mime) => match *mime.deref() {
                    Mime(TopLevel::Image, SubLevel::Jpeg, _) => {
                        let path = self.dir.join(format!("{}.jpg", idx));
                        fs::File::create(path)?
                    }
                    _ => continue,
                },
                None => continue,
            };

            debug!("writing image: {:?}\n", img_file);
            io::copy(&mut resp, &mut img_file)?;
            idx += 1;
        }

        Ok(idx)
    }
}

impl Fetch for UnsplashFetcher {
    fn next_image_path(&mut self) -> Result<PathBuf, Box<Error>> {
        if !self.cached || self.timestamp.elapsed() >= self.refresh {
            match self.download_images() {
                Ok(len) => {
                    self.cached = true;
                    self.total = len;
                }
                Err(err) => {
                    self.cached = false;
                    return Err(err);
                }
            }
            self.timestamp = Instant::now();
        }

        if self.total > 0 {
            self.next = self.next % self.total;

            let path = self.dir.join(format!("{}.jpg", self.next));
            self.next += 1;

            debug!("unsplash: {:?}", path);
            return Ok(path);
        }

        Err(Box::new(WallsplashError::UnsplashNoImage))
    }
}
