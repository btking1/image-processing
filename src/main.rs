use image::ImageReader;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt::Debug;
use std::fs::{create_dir_all, write, File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone)]
struct Chill {
    api_key: std::string::String,
    user: std::string::String,
    browser: std::string::String,
    page: u32,
    photo: u32,
    per_page: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Photos {
    id: u32,
    width: u32,
    height: u32,
    url: String,
    photographer: String,
    photographer_url: String,
    avg_color: String,
    src: Map<std::string::String, Value>,
    liked: bool,
    alt: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct Pexel {
    page: u32,
    per_page: u32,
    photos: Vec<Photos>,
    total_results: u32,
    next_page: std::string::String,
}

impl Chill {
    pub fn outdir(chill: &Self) -> Result<(PathBuf, PathBuf), ()> {
        let chilldir = &format!("/Users/{}/.chill", chill.user);
        let outdir = Path::new(&chilldir);
        let json = Path::new("chill.json");
        let chill_json = Path::join(&outdir, json);

        if !chill_json.exists() {
            File::create_new(&chill_json).expect("ERROR -> issue creating file");
            println!("SUCCESS -> {:?}", &chill_json);

            return Ok((outdir.to_owned(), chill_json));
        }

        Ok((outdir.to_owned(), chill_json))
    }

    pub fn get_image(chill: &Self) {
        let api_auth = format!("Authorization: {}", chill.api_key);
        let query = format!(
            "https://api.pexels.com/v1/curated?page={}&per_page={}",
            &chill.page, &chill.per_page
        );

        let chill_json = Chill::outdir(chill).unwrap().1;
        let curl = Command::new("curl")
            .args(&[
                "-X",
                "GET",
                "-H",
                "Content-Type: application/json",
                "-H",
                &api_auth,
                &query,
            ])
            .output()
            .unwrap();

        match curl.stdout.is_empty() {
            true => {
                println!("ERROR -> stdout empty");
            }
            _ => {
                let mut file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(&chill_json)
                    .unwrap();

                file.write_all(&curl.stdout)
                    .expect("ERROR -> failed to write to chill.json");

                println!("SUCCESS -> image data added to chill.json")
            }
        }
    }
    pub fn save_image(chill: &Self, src: &String) -> Result<(), std::io::Error> {
        let api_auth = format!("Authorization: {}", &chill.api_key);
        let outdir = &Chill::outdir(&chill).unwrap().0;

        println!("{:?}", outdir);
        let image_path = outdir.join("chill-image.jpg");
        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&outdir.join("chill-image.jpg"))
            .expect("ERROR -> failed to create image");

        let get_image = Command::new("curl")
            .args(&[
                "-X",
                "GET",
                "-H",
                &api_auth,
                "-H",
                "Accept: image/jpeg, image/png",
                &src,
            ])
            .output()
            .unwrap();

        let image = image::load_from_memory(&get_image.stdout)
            .expect("ERROR -> failed to load image into memory");
        let save = image.save(&image_path);

        if save.is_ok() {
            println!("SUCCESS -> {}", &image_path.to_string_lossy());
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::Other,
                format!("ERROR -> failed to get image: {}", &src),
            ))
        }
    }
    pub fn display(chill: &Self, src: &String) {
        // let browser = &chill.browser;
        // let open = Command::new("open")
        //     .args(&["-a", browser, &src])
        //     .spawn()
        //     .unwrap()
        //     .wait()
        //     .unwrap();

        // if !&open.success() {
        //     Err(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!("{:?}", &open.code()),
        //     ))
        // } else {
        //     Ok(())
        // }
        //
    }

    pub fn read_from_json(chill: &Self) -> String {
        let outdir = Chill::outdir(chill).unwrap().0;
        let mut open_f = File::open(&outdir.as_path().join(Path::new("chill.json"))).unwrap();

        let mut buffer = String::new();
        open_f.read_to_string(&mut buffer).unwrap();

        let json_: Pexel = serde_json::from_str(&buffer).unwrap();
        let index: usize = (chill.photo - 1).try_into().unwrap();
        let srcs = &json_.photos[index].src;

        let v = srcs.get("large2x").to_owned();
        v.expect("ERROR -> couldnt find src: large2x")
            .as_str()
            .unwrap()
            .to_string()
    }
}

fn main() {
    let chill = Chill {
        //pexel api key
        api_key: "563492ad6f917000010000016271bb6cc0614b44bf7d12ba3a610eaa".to_string(),
        user: "godsiom".to_string(),
        browser: "firefox".to_string(),
        page: 1,
        photo: 1,
        per_page: 2,
    };
    let src = Chill::read_from_json(&chill);
    Chill::save_image(&chill, &src).unwrap();
}
