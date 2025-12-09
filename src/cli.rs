use clap::Parser;
use image::ImageReader;
use std::error::Error;
use std::path::Path;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "kcomprs", about = "Reduce number of colors used in image")]
pub struct Cli {
    #[arg(help = "Image files to compress", required = true)]
    files: Vec<String>,

    #[arg(
        long,
        short = 'n',
        default_value = "15",
        help = "Number of colors to use"
    )]
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

    #[arg(
        long,
        short = 'q',
        action,
        help = "Increase speed in exchange of accuracy"
    )]
    quick: bool,

    #[arg(long, short = 'w', action, help = "Overwrite output if exists")]
    overwrite: bool,

    #[arg(
        long,
        short = 't',
        default_value = "8",
        help = "Maximum number image process at a time [min:1]"
    )]
    concurrency: u32,

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
    // TODO: use &str?
    path: String,
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

    pub fn execute(self: Self) -> Result<usize, Box<dyn Error>> {
        let mut count = 0;
        let images = scan_images(&self.files);
        for image in images {
            match self.handle_image(&image) {
                Ok(_) => count += 1,
                Err(err) => {
                    error!(error = err, path = image.path, "Error processing image");
                }
            }
        }
        Ok(count)
    }

    fn handle_image(&self, image: &DecodedImage) -> Result<(), Box<dyn Error>> {
        let path = Path::new(&image.path);
        let filename = path
            .file_name()
            .expect("Missing filename what the fuck")
            .to_str();
        if filename.is_none() {
            return Err(format!("Invalid filename: {}", path.display()).into());
        }

        let format = image.format.extensions_str().join("|");
        info!(
            cp = self.colors,
            round = self.round,
            img = filename.unwrap(),
            dimension = format!("{}x{}", image.img.width(), image.img.height()),
            format = format,
            "Processing image"
        );
        Ok(())
    }
}

fn scan_images(files: &[String]) -> Vec<DecodedImage> {
    let mut images: Vec<DecodedImage> = Vec::with_capacity(files.len());
    for path in files {
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
            error!(
                path = &path,
                error = &error as &dyn Error,
                "Error decoding image"
            );
            continue;
        }

        images.push(DecodedImage {
            img: img.unwrap(),
            format: format.unwrap(),
            path: path.to_string(),
        })
    }
    images
}
