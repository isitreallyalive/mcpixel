use clap::Parser;
use mcpixel::schematic::{Plane, SchematicFormat};
use mcpixel::version::Version;
use mcpixel::{Configuration, PixelArt};
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Input image file path")]
    input: PathBuf,

    #[arg(name = "size", short, long, help = "Maximum width/height for resizing")]
    max_dimension: Option<u32>,

    #[arg(short = 'p', long, help = "Number of colors to quantize the image to")]
    palette_size: Option<u32>,

    #[arg(
        short = 'g',
        long,
        help = "Gamma correction factor for brightness adjustment"
    )]
    gamma: Option<f32>,

    #[arg(long, help = "Factor to boost image saturation")]
    saturation: Option<f32>,

    #[arg(
        short = 'o',
        long,
        help = "Include a glass overlay layer to help blend colours"
    )]
    overlay: bool,

    #[arg(long = "smooth", help = "Target smoothness penalty")]
    smoothness_penalty: Option<f32>,

    #[arg(
        short = 'm',
        long,
        help = "Minecraft version target for pixel art",
        default_value = "1.21.11"
    )]
    minecraft: String,
}

impl From<Args> for Configuration {
    fn from(args: Args) -> Self {
        let default = Self::default();
        Self {
            max_dimension: args.max_dimension.unwrap_or(default.max_dimension),
            palette_size: args.palette_size.unwrap_or(default.palette_size),
            gamma: args.gamma.unwrap_or(default.gamma),
            saturation: args.saturation.unwrap_or(default.saturation),
            overlay: args.overlay,
            smoothness_penalty: args
                .smoothness_penalty
                .unwrap_or(default.smoothness_penalty),
        }
    }
}

#[cfg(not(debug_assertions))]
fn download_version(version: &str) -> Result<Vec<u8>> {
    ureq::get(format!(
        "https://github.com/isitreallyalive/mcpixel/raw/refs/heads/main/data/{version}"
    ))
    .call()
    .and_then(|mut r| r.body_mut().read_to_vec())
    .into_diagnostic()
}

/// Load version data.
fn load_version(version: &str) -> Result<Version> {
    // read from disk in debug
    #[cfg(debug_assertions)]
    let data = fs::read(format!("./data/{version}")).into_diagnostic()?;

    // read the cache/download the file in release
    // todo: compare checksum for updates
    #[cfg(not(debug_assertions))]
    let data = if let Some(dirs) = directories::ProjectDirs::from("dev", "newty", "mcpixel") {
        // find cache directory
        let cache = dirs.cache_dir();
        fs::create_dir_all(cache).into_diagnostic()?;

        // check if the data has already been fetched
        let data_path = cache.join(version);

        if data_path.exists() {
            fs::read(data_path).into_diagnostic()?
        } else {
            // otherwise download it and save it
            let data = download_version(version)?;
            fs::write(data_path, &data).into_diagnostic()?;
            data
        }
    } else {
        download_version(version)?
    };

    Ok(Version::read(&data[..]).into_diagnostic()?)
}

fn main() -> Result<()> {
    let args = Args::parse();

    #[cfg(debug_assertions)]
    dbg!(&args);

    if !args.input.is_file() {
        return Err(miette::miette!("Input path {:?} is not a file", args.input));
    }

    let version = load_version(&args.minecraft)?;
    let image = fs::read(&args.input).into_diagnostic()?;
    let art = PixelArt::new(image, version, args.into()).into_diagnostic()?;
    let materials = art.materials();
    let schematic = art.schematic(Plane::Standing);
    let mut file = fs::File::create("output.litematic").into_diagnostic()?;

    schematic
        .save(&mut file, SchematicFormat::Litematica)
        .into_diagnostic()?;

    #[cfg(debug_assertions)]
    dbg!(materials.values().sum::<usize>());

    println!("{:?}", materials);

    Ok(())
}
