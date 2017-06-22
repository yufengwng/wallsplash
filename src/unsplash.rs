use std::error::Error;
use std::fs;
use std::io;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use reqwest;
use reqwest::header::{Authorization, ContentType};
use reqwest::mime::{Mime, SubLevel, TopLevel};
use tempdir::TempDir;

use errors::WallsplashError;

const UNSPLASH_API: &'static str = "https://api.unsplash.com";
const PHOTOS_ENDPOINT: &'static str = "/photos/curated";

#[derive(Deserialize, Debug)]
struct Photo {
    id: String,
    links: Links,
}

#[derive(Deserialize, Debug)]
struct Links {
    download: String,
}

#[derive(Debug)]
pub struct Client {
    token: String,
    limit: u32,
    dir: TempDir,
    curr: usize,
    total: usize,
    cached: bool,
    period: Duration,
    timestamp: Instant,
}

impl Client {
    pub fn new(token: &str, limit: u32, period: Duration) -> Result<Client, Box<Error>> {
        let tempdir = TempDir::new("unsplash")?;
        println!("{:?}", tempdir.path());

        Ok(Client {
            token: token.to_owned(),
            limit: limit,
            dir: tempdir,
            curr: 0,
            total: 0,
            cached: false,
            period: period,
            timestamp: Instant::now(),
        })
    }
}

impl Client {
    pub fn next_image_path(&mut self) -> Result<PathBuf, Box<Error>> {
        if !self.cached || self.timestamp.elapsed() >= self.period {
            match self.download_images() {
                Ok(len) => {
                    self.cached = true;
                    self.total = len;
                },
                Err(err) => {
                    self.cached = false;
                    return Err(err);
                }
            }
            self.timestamp = Instant::now();
        }

        if self.total > 0 {
            if self.curr >= self.total {
                self.curr = 0;
            }

            let path = self.dir.path().join(format!("{}.jpg", self.curr));
            self.curr += 1;

            println!("unsplash: {:?}", path);
            return Ok(path);
        }

        Err(Box::new(WallsplashError::UnsplashNoImage))
    }

    fn download_images(&mut self) -> Result<usize, Box<Error>> {
        let photos_uri = format!("{}{}?per_page={}&order_by=latest",
                                 UNSPLASH_API, PHOTOS_ENDPOINT, self.limit);
        println!("url: {}\n", photos_uri);

        let request = reqwest::Client::new()?;
        let mut resp = request.get(&photos_uri)
            .header(Authorization(format!("Client-ID {}", self.token)))
            .send()?;

        println!("response: {}", resp.url());
        println!("status:   {}", resp.status());
        println!("headers:\n\n{}", resp.headers());

        if !resp.status().is_success() {
            return Err(Box::new(WallsplashError::UnsplashAPIFail));
        }

        let photos: Vec<Photo> = resp.json()?;
        println!("json: {:?}", photos);

        let mut idx = 0;
        for photo in photos.iter() {
            let img_url = &photo.links.download;
            println!("downloading: {}", img_url);

            let mut resp = request.get(img_url.as_str()).send()?;

            println!("response: {}", resp.url());
            println!("status:   {}", resp.status());
            println!("headers:\n\n{}", resp.headers());

            let mut img_file = match resp.headers().get::<ContentType>() {
                Some(mime) => {
                    match *mime.deref() {
                        Mime(TopLevel::Image, SubLevel::Jpeg, _) => {
                            let path = self.dir.path().join(format!("{}.jpg", idx));
                            fs::File::create(path)?
                        }
                        _ => continue,
                    }
                }
                None => continue,
            };

            println!("writing image: {:?}\n", img_file);
            io::copy(&mut resp, &mut img_file)?;
            idx += 1;
        }

        Ok(idx)
    }
}
