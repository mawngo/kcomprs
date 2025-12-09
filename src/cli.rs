use clap::Parser;

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
    debug: bool,
}

impl Cli {
    pub fn execute() -> Self {
        let cli = Self::parse();
        return cli;
    }
}
