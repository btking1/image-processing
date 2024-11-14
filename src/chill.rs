use ab_glyph::{FontRef, PxScale};
use anyhow::Result;
use image::codecs::gif::GifEncoder;
use image::imageops::FilterType;
use image::Rgba;
use image::{Frame, GenericImageView, ImageBuffer, ImageReader};
use imageproc::map::map_pixels_mut;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt::Debug;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Instant;

#[derive(Clone)]
pub struct Chill {
    pub(crate) api_key: std::string::String,
    pub(crate) user: std::string::String,
    pub(crate) browser: std::string::String,
    pub(crate) page: u32,
    pub(crate) photo: u32,
    pub(crate) per_page: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Photos {
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
pub struct Pexel {
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
    pub fn save_image(chill: &Self, src: &String) -> Result<()> {
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
            anyhow::bail!("")
        }
    }
    pub fn display(chill: &Self) -> Result<()> {
        let browser = &chill.browser;
        let outdir = Chill::outdir(chill).unwrap().0;
        let src = outdir.join("chill-image-edit.jpg").clone();
        let open = Command::new("open")
            .args(&["-a", browser, &src.to_string_lossy()])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        if !&open.success() {
            anyhow::bail!("ERROR -> trying to open gif")
        } else {
            Ok(())
        }
    }

    pub fn process(chill: &Self) {
        let outdir = Chill::outdir(chill).unwrap().0;
        let image_path = outdir.join("chill-image.jpg");
        let proc_image_path = outdir.join("chill-image-edit.jpg");
        ImageReader::open(image_path)
            .unwrap()
            .decode()
            .unwrap()
            .blur(0.8)
            .resize_to_fill(2760, 1440, image::imageops::FilterType::Gaussian)
            .save(&proc_image_path)
            .expect("ERROR -> failed to process image");
        println!(
            "SUCCESS -> image processed {}",
            &proc_image_path.to_string_lossy()
        )
    }
    pub fn swirl_and_add_text(chill: &Self, text: &String) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        let outdir = Chill::outdir(chill).unwrap().0;
        let image_path = outdir.join("chill-image.jpg");

        OpenOptions::new()
            .create(true)
            .write(true)
            .open(&outdir.join("chill-image-edit-1.png"))
            .unwrap();

        let mut image = ImageReader::open(&image_path)
            .unwrap()
            .decode()
            .unwrap()
            .blur(0.9);
        let (width, height) = (image.width() as f32, image.height() as f32);
        let center_x = width / 2.0;
        let center_y = height / 2.0;

        let i = image.clone();
        map_pixels_mut(&mut image, |x, y, point| {
            static RADIUS: f32 = 200.0;
            static ROTATION: f32 = 1.0;
            static STRENGTH: f32 = 1.50;

            let delta_x = x as f32 - center_x;
            let delta_y = y as f32 - center_y;

            let ro = (delta_x.powi(2) + delta_y.powi(2)).sqrt();
            let theta = delta_y.atan2(delta_x);

            let decay = STRENGTH * (-ro / RADIUS).exp();
            let delta_theta = ROTATION * decay + theta;

            let new_x = (center_x + ro * delta_theta.cos()).round() as i32;
            let new_y = (center_y + ro * delta_theta.sin()).round() as i32;

            if new_x >= 0 && new_x < width as i32 && new_y >= 0 && new_y < height as i32 {
                i.get_pixel(new_x as u32, new_y as u32).clone()
            } else {
                point
            }
        });

        let mut rgba_image = image.into_rgba8();

        let color = Rgba([255, 255, 255, 255]); // White color with full opacity
        let scale = PxScale::from(72.0); // Increased font size
        let fira_code = FontRef::try_from_slice(include_bytes!(
            "/Users/godsiom/Library/Fonts/FiraCode-Medium.ttf"
        ))
        .unwrap();

        let text_width = text.len() as i32 * 30; // Approximate width
        let x = (width as i32 / 2) - (text_width / 2);
        let y = height as i32 / 2;

        let processed_image =
            imageproc::drawing::draw_text(&mut rgba_image, color, x, y, scale, &fira_code, text);
        processed_image
    }

    pub fn gif(chill: &Self) -> Result<(), anyhow::Error> {
        let tt = Instant::now();
        let outdir = Chill::outdir(&chill).unwrap();
        let outpath = outdir.0.join("chill.gif");

        let text = "hello world";

        let pb = ProgressBar::new(text.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("██░"),
        );

        // Create accumulating strings
        let accumulated_strings: Vec<String> = text
            .char_indices()
            .map(|(i, _)| text[..=i].to_string())
            .collect();

        // 3. Parallel frame generation
        let processed_frames: Vec<_> = accumulated_strings
            .par_iter()
            .map(|current_text| {
                let delay = image::Delay::from_numer_denom_ms(500, 1);
                let proc_image_buffer = Chill::swirl_and_add_text(&chill, current_text);

                // 4. Optimize image size if possible
                let resized_buffer = image::DynamicImage::ImageRgba8(proc_image_buffer)
                    .resize(2700, 1600, FilterType::Lanczos3)
                    .to_rgba8();

                let frame = Frame::from_parts(resized_buffer, 0, 0, delay);

                pb.inc(1);
                frame
            })
            .collect();

        pb.finish_with_message("Frames generated");

        // 5. Efficient GIF encoding
        println!("Starting GIF encoding...");
        let gif_file = BufWriter::new(File::create(&outpath)?);
        let mut encoder = GifEncoder::new(gif_file);
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite)?;

        // 6. Use a separate thread for encoding
        let (tx, rx) = mpsc::channel();
        let encode_handle = thread::spawn(move || {
            for frame in rx {
                encoder.encode_frame(frame).unwrap();
            }
            anyhow::anyhow!("ERROR -> encoding frames")
        });

        // Send frames to encoder
        for frame in processed_frames {
            tx.send(frame)?;
        }
        drop(tx);

        // Wait for encoding to complete
        encode_handle.join().unwrap();

        println!("Total time: {}s", tt.elapsed().as_secs());
        Ok(())
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
