use crate::kmeans::model::{Dataset, Trainer};
use clap::Parser;
use image::buffer::ConvertBuffer;
use image::codecs::jpeg::JpegEncoder;
use image::{GenericImageView, ImageBuffer, ImageReader, RgbImage, Rgba};
use std::cmp::{max, min};
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;
use std::{fs, thread};
use tracing::{debug, error, info};

const PHI: f32 = 1.618033988749894848204586834365638118_f32;

#[derive(Parser)]
#[command(name = "kcomprs", about = "Reduce number of colors used in image")]
pub struct Cli {
    #[arg(help = "Image files to compress", required = true)]
    files: Vec<String>,

    #[arg(long, short = 'n', default_value = "15", help = "Number of colors to use")]
    colors: usize,

    #[arg(long, short, help = "Output directory name")]
    output: Option<String>,

    #[arg(
        long,
        short,
        help = "Number of image to generate, series of output with increasing number of colors up util reached --colors parameter"
    )]
    series: Option<usize>,

    #[arg(
        long,
        short = 'i',
        default_value = "100",
        help = "Maximum number of round before stop adjusting (number of kmeans iterations)"
    )]
    round: usize,

    #[arg(long, short = 'q', action, help = "Increase speed in exchange of accuracy")]
    quick: bool,

    #[arg(long, short = 'w', action, help = "Overwrite output if exists")]
    overwrite: bool,

    #[arg(
        long,
        short = 't',
        default_value = default_concurrency(),
        help = "Maximum number image process at a time [0=auto]"
    )]
    concurrency: usize,

    #[arg(long = "kcpu", help = "Maximum cpu used processing each image [unsupported]")]
    kmeans_concurrency: Option<usize>,

    #[arg(
        long,
        short,
        default_value = "0.005",
        help = "Delta threshold of convergence (delta between kmeans old and new centroid’s values)"
    )]
    delta: f64,

    #[arg(
        long = "dalgo",
        default_value = "EuclideanDistance",
        help = "Distance algo for kmeans [EuclideanDistance,EuclideanDistanceSquared]"
    )]
    distance_algo: String,

    #[arg(
        long,
        help = "Specify quality of output jpeg compression [0-100] [default 0 - output png]"
    )]
    jpeg: Option<u32>,

    #[arg(long, action, help = "Generate an additional palette image")]
    palette: bool,

    #[arg(long, action, global = true, help = "Enable debug mode")]
    pub debug: bool,
}

fn default_concurrency() -> String {
    let num = num_cpus::get();
    let default_concurrency = max(num, 1);
    default_concurrency.to_string()
}

struct DecodedImage {
    img: image::DynamicImage,
    format: image::ImageFormat,
    path: String,
    config: ProcessImageConfig,
}

struct ProcessImageConfig {
    colors: usize,
    round: usize,
    jpeg: u32,
    output: Option<String>,
    overwrite: bool,
    distance_algo: String,
    delta: f64,
    palette: bool,
}

impl Cli {
    pub fn new() -> Self {
        let num = num_cpus::get();
        let default_concurrency = min(num, 1);
        let mut cli = Self::parse();

        if cli.concurrency <= 0 {
            cli.concurrency = default_concurrency;
        }

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

        // Avoid concurrency overhead when disabled.
        if images.len() == 1 || self.concurrency <= 1 {
            for image in images {
                let res = handle_image(&image);
                if res.is_err() {
                    error!(error = res.unwrap_err(), path = image.path, "Error processing image");
                }
            }
            return Ok(());
        }

        // TODO: is there any other way to do this?
        let images = Arc::new(Mutex::new(images));
        thread::scope(|s| {
            for _ in 0..self.concurrency {
                let images = Arc::clone(&images);
                s.spawn(move || {
                    let mut images = images.lock().unwrap();
                    let image = images.pop();
                    drop(images);

                    if image.is_none() {
                        return;
                    }

                    let image = image.unwrap();
                    let res = handle_image(&image);
                    if res.is_err() {
                        error!(error = res.unwrap_err(), path = image.path, "Error processing image");
                    }
                });
            }
        });
        Ok(())
    }

    fn scan_images(&self) -> Vec<DecodedImage> {
        let mut images: Vec<DecodedImage> = Vec::with_capacity(self.files.len());
        for path in &self.files {
            match fs::metadata(path) {
                Ok(metadata) => {
                    if metadata.is_file() {
                        self.read_images(path, &mut images);
                        continue;
                    }
                    let paths = fs::read_dir(path).unwrap();
                    for path in paths {
                        if path.is_err() {
                            let err: Box<dyn Error> = path.unwrap_err().into();
                            debug!(error = err, "Error reading path metadata");
                            continue;
                        }
                        let path = path.unwrap();
                        match path.metadata() {
                            Ok(metadata) => {
                                if !metadata.is_file() {
                                    continue;
                                }
                                let path = path.path();
                                let path = path.to_str();
                                if path.is_some() {
                                    self.read_images(path.unwrap(), &mut images);
                                }
                            }
                            Err(err) => {
                                let err: Box<dyn Error> = err.into();
                                error!(path = path.path().to_str(), error = err, "Error reading file metadata");
                                continue;
                            }
                        }
                    }
                }
                Err(err) => {
                    let err: Box<dyn Error> = err.into();
                    error!(path = path, error = err, "Error reading file metadata");
                    continue;
                }
            }
        }
        images
    }

    fn read_images(&self, path: &str, images: &mut Vec<DecodedImage>) {
        let reader = ImageReader::open(&path);
        if reader.is_err() {
            error!(
                path = &path,
                error = reader.err().unwrap().get_ref(),
                "Error opening image"
            );
            return;
        }
        let reader = reader.unwrap().with_guessed_format();
        if reader.is_err() {
            error!(
                path = &path,
                error = reader.err().unwrap().get_ref(),
                "Error reading image"
            );
            return;
        }
        let reader = reader.unwrap();

        let format = reader.format();
        if format.is_none() {
            error!(path = &path, "Not an image");
            return;
        }

        let img = reader.decode();
        if img.is_err() {
            let error = img.err().unwrap();
            error!(path = &path, error = &error as &dyn Error, "Error decoding image");
            return;
        }
        let img = img.unwrap();

        if !self.series.is_none() {
            let mut s = self.series.unwrap();
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
}

impl Into<ProcessImageConfig> for &Cli {
    fn into(self) -> ProcessImageConfig {
        ProcessImageConfig {
            colors: self.colors,
            round: self.round,
            jpeg: self.jpeg.unwrap_or(0),
            output: self.output.clone(),
            overwrite: self.overwrite,
            distance_algo: self.distance_algo.clone(),
            delta: self.delta,
            palette: self.palette,
        }
    }
}

fn handle_image(image: &DecodedImage) -> Result<(), Box<dyn Error>> {
    let filepath = Path::new(&image.path);
    let filename = filepath.file_name().expect("Missing filename what the fuck").to_str();
    if filename.is_none() {
        return Err(format!("Invalid filename: {}", filepath.display()).into());
    }
    let filename = filename.unwrap();

    let format = image.format.extensions_str().join("|");
    info!(
        cp = image.config.colors,
        round = image.config.round,
        img = filename,
        dimension = format!("{}x{}", image.img.width(), image.img.height()),
        format = format,
        "Processing image"
    );

    let mut outfile_buf = PathBuf::new();
    if !image.config.output.is_none() {
        let output = image.config.output.as_ref();
        outfile_buf.push(output.unwrap());
    }
    outfile_buf.push(filename);
    outfile_buf.set_extension("");
    outfile_buf.set_file_name(format!(
        "{}.kcp{}n{}.",
        outfile_buf.file_name().unwrap().to_str().unwrap(),
        image.config.round,
        image.config.colors
    ));
    outfile_buf.set_extension(if image.config.jpeg > 0 { "jpeg" } else { "png" });
    let outfile = outfile_buf.to_str().unwrap();

    if let Ok(metadata) = fs::metadata(outfile) {
        info!(
            path = outfile,
            isDir = metadata.is_dir(),
            overwrite = image.config.overwrite,
            "File existed"
        );
        if !image.config.overwrite {
            return Ok(());
        }
        if !metadata.is_file() {
            return Ok(());
        }
    }

    let start = Instant::now();
    let mut matrix = Vec::with_capacity((image.img.width() * image.img.height()) as usize);
    image.img.pixels().for_each(|pixel| {
        matrix.push([
            pixel.2[0] as f64,
            pixel.2[1] as f64,
            pixel.2[2] as f64,
            pixel.2[3] as f64,
        ])
    });

    debug!(
        cp = image.config.colors,
        img = filename,
        round = image.config.round,
        ms = start.elapsed().as_millis(),
        "Start partitioning"
    );
    let trainer = Trainer {
        k: image.config.colors,
        max_iterations: image.config.round,
        delta: image.config.delta,
        distance_fn: match image.config.distance_algo.as_str() {
            // TODO: enum, maybe?.
            "EuclideanDistanceSquared" => crate::kmeans::cluster::euclidean_distance_squared,
            _ => crate::kmeans::cluster::euclidean_distance,
        },
    };

    let model = trainer.fit(matrix);

    let width = image.img.width();
    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, image.img.height());
    for (index, number) in model.mapping.iter().enumerate() {
        let cluster = model.centroids[*number];
        let y = index as u32 / width;
        let x = index as u32 % width;
        let r = cluster[0].round() as u8;
        let g = cluster[1].round() as u8;
        let b = cluster[2].round() as u8;
        let a = cluster[3].round() as u8;
        img.put_pixel(x, y, Rgba([r, g, b, a]));
    }

    let write_result = if image.config.jpeg > 0 {
        let file = File::create(outfile);
        match file {
            Err(err) => {
                let err: Box<dyn Error> = err.into();
                error!(error = err, out = outfile, "Error writing image");
                Ok(())
            }
            Ok(file) => {
                let encoder = JpegEncoder::new_with_quality(file, image.config.jpeg as u8);
                let img: RgbImage = img.convert();
                img.write_with_encoder(encoder)
            }
        }
    } else {
        img.save(outfile)
    };

    match write_result {
        Ok(_) => {
            let outfile = outfile.to_owned();
            if image.config.palette {
                gen_palette(model.centroids, outfile_buf)
            }
            info!(
                out = outfile,
                ms = start.elapsed().as_millis(),
                iter = model.iter,
                "Compress completed"
            );
        }
        Err(err) => {
            let err: Box<dyn Error> = err.into();
            error!(error = err, out = outfile, "Error writing image");
        }
    }
    Ok(())
}

fn gen_palette(centroids: Dataset, mut outfile: PathBuf) {
    outfile.set_extension("palette.png");
    let outfile = outfile.to_str().unwrap();

    let mut swatch_width = 400;
    if centroids.len() > 1 {
        swatch_width = 200 - min(7 * centroids.len() - 2, 140)
    }
    let width = (swatch_width * centroids.len()) as u32;
    let height = (width as f32 / PHI) as u32;

    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);
    centroids.iter().enumerate().for_each(|(i, cluster)| {
        let start_x = i * swatch_width;
        let end_x = (i + 1) * swatch_width;
        for y in 0..height {
            for x in start_x..end_x {
                let r = cluster[0].round() as u8;
                let g = cluster[1].round() as u8;
                let b = cluster[2].round() as u8;
                let a = cluster[3].round() as u8;
                img.put_pixel(x as u32, y, Rgba([r, g, b, a]));
            }
        }
    });

    let result = img.save(outfile);
    if result.is_err() {
        let err: Box<dyn Error> = result.unwrap_err().into();
        error!(error = err, out = outfile, "Error palette image");
    }
}
