use clap::Parser;
use directories::ProjectDirs;
use mcpixel::schematic::SchematicFormat;
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

    #[arg(name = "size", short, long, help = "Maximum dimension for resizing")]
    max_dimension: Option<u32>,

    #[arg(short = 'p', long, help = "Number of colors in the palette")]
    palette_size: Option<u32>,

    #[arg(short = 'o', long, help = "Should there be a glass overlay?")]
    overlay: bool,

    #[arg(
        short = 'm',
        long,
        help = "Which version of Minecraft should the pixel art be for?",
        default_value = "1.21.11"
    )]
    minecraft: String,
}

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
    let data = if let Some(dirs) = ProjectDirs::from("dev", "newty", "mcpixel") {
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

    Ok(Version::try_from(data).into_diagnostic()?)
}

fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(&args);

    if !args.input.is_file() {
        return Err(miette::miette!("Input path {:?} is not a file", args.input));
    }

    // determine configuration
    let version = load_version(&args.minecraft)?;

    let mut config = Configuration::default();
    args.max_dimension.map(|d| config.max_dimension = d);
    args.palette_size.map(|p| config.palette_size = p);
    config.overlay = args.overlay;

    let image = fs::read(&args.input).into_diagnostic()?;
    let art = PixelArt::new(image, version, config).into_diagnostic()?;
    let schematic = art.schematic();
    let mut file = fs::File::create("output.litematic").into_diagnostic()?;

    schematic
        .save(&mut file, SchematicFormat::Litematica)
        .into_diagnostic()?;

    Ok(())
}
