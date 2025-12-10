use clap::Parser;
use image::ImageReader;
use std::cmp::min;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "kcomprs", about = "Reduce number of colors used in image")]
pub struct Cli {
    #[arg(help = "Image files to compress", required = true)]
    files: Vec<String>,

    #[arg(long, short = 'n', default_value = "15", help = "Number of colors to use")]
    colors: u32,

    #[arg(long, short, default_value = ".", help = "Output directory name")]
    output: String,

    #[arg(
        long,
        short,
        default_value = "1",
        help = "Number of image to generate, series of output with increasing number of colors up util reached --colors parameter [min:1]"
    )]
    series: u32,

    #[arg(
        long,
        short = 'i',
        default_value = "100",
        help = "Maximum number of round before stop adjusting (number of kmeans iterations)"
    )]
    round: u32,

    #[arg(long, short = 'q', action, help = "Increase speed in exchange of accuracy")]
    quick: bool,

    #[arg(long, short = 'w', action, help = "Overwrite output if exists")]
    overwrite: bool,

    #[arg(
        long,
        short = 't',
        default_value = "8",
        help = "Maximum number image process at a time [min:1]"
    )]
    concurrency: usize,

    #[arg(
        long,
        short,
        default_value = "0.005",
        help = "Delta threshold of convergence (delta between kmeans old and new centroid’s values)"
    )]
    delta: f64,

    #[arg(
        name = "dalgo",
        long,
        default_value = "EuclideanDistance",
        help = "Distance algo for kmeans [EuclideanDistance,EuclideanDistanceSquared]"
    )]
    distance_algo: String,

    #[arg(
        long,
        default_value = "0",
        help = "Specify quality of output jpeg compression [0-100] (set to 0 to output png)"
    )]
    jpeg: u32,

    #[arg(long, action, help = "Enable debug mode")]
    pub debug: bool,
}

struct DecodedImage {
    img: image::DynamicImage,
    format: image::ImageFormat,
    path: String,
    config: ProcessImageConfig,
}

struct ProcessImageConfig {
    colors: u32,
    round: u32,
}

impl Cli {
    pub fn new() -> Self {
        let mut cli = Self::parse();
        if cli.quick {
            cli.delta = 0.01;
            cli.round = 50
        }
        cli
    }

    pub fn execute(self) -> Result<(), Box<dyn Error>> {
        let images = self.scan_images();
        if images.is_empty() {
            return Ok(());
        }

        let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(min(self.concurrency, images.len()));
        // TODO: is there any other way to do this?
        // Also, we could provide a fast path when concurrency and series is disabled, or there are only one image.
        let images = Arc::new(Mutex::new(images));

        for _ in 0..threads.capacity() {
            let images = Arc::clone(&images);
            let config: ProcessImageConfig = (&self).into();
            let handle = thread::spawn(move || {
                let mut images = images.lock().unwrap();
                let image = images.pop();
                drop(images);

                if image.is_none() {
                    return;
                }

                let image = image.unwrap();
                let res = handle_image(&image, &config);
                if res.is_err() {
                    error!(error = res.unwrap_err(), path = image.path, "Error processing image");
                }
            });

            threads.push(handle);
        }

        for thread in threads {
            thread
                .join()
                .expect("Error when joining thread waiting for all jobs to finish")
        }
        Ok(())
    }

    fn scan_images(&self) -> Vec<DecodedImage> {
        let mut images: Vec<DecodedImage> = Vec::with_capacity(self.files.len());
        for path in &self.files {
            let reader = ImageReader::open(&path);
            if reader.is_err() {
                error!(
                    path = &path,
                    error = reader.err().unwrap().get_ref(),
                    "Error opening image"
                );
                continue;
            }
            let reader = reader.unwrap().with_guessed_format();
            if reader.is_err() {
                error!(
                    path = &path,
                    error = reader.err().unwrap().get_ref(),
                    "Error reading image"
                );
                continue;
            }
            let reader = reader.unwrap();

            let format = reader.format();
            if format.is_none() {
                warn!(path = &path, "Unsupported image format");
                continue;
            }

            let img = reader.decode();
            if img.is_err() {
                let error = img.err().unwrap();
                error!(path = &path, error = &error as &dyn Error, "Error decoding image");
                continue;
            }
            let img = img.unwrap();

            if self.series > 1 {
                let mut s = self.series;
                let mut step = self.colors / s;
                let mut start = 1;
                if step <= 1 {
                    start = 2;
                    step = 1;
                    s = self.colors
                }

                for i in start..s {
                    let mut config: ProcessImageConfig = self.into();
                    config.colors = step * i;
                    images.push(DecodedImage {
                        // TODO: any better way instead of cloning?
                        img: img.clone(),
                        format: format.unwrap(),
                        path: path.to_string(),
                        config,
                    })
                }
            }

            images.push(DecodedImage {
                img,
                format: format.unwrap(),
                path: path.to_string(),
                config: self.into(),
            })
        }
        images
    }
}

impl Into<ProcessImageConfig> for &Cli {
    fn into(self) -> ProcessImageConfig {
        ProcessImageConfig {
            colors: self.colors,
            round: self.round,
        }
    }
}

fn handle_image(image: &DecodedImage, conf: &ProcessImageConfig) -> Result<(), Box<dyn Error>> {
    let path = Path::new(&image.path);
    let filename = path.file_name().expect("Missing filename what the fuck").to_str();
    if filename.is_none() {
        return Err(format!("Invalid filename: {}", path.display()).into());
    }

    let format = image.format.extensions_str().join("|");
    info!(
        cp = conf.colors,
        round = conf.round,
        img = filename.unwrap(),
        dimension = format!("{}x{}", image.img.width(), image.img.height()),
        format = format,
        "Processing image"
    );
    Ok(())
}
