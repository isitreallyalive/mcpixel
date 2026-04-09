use clap::Parser;
use mcpixel::schematic::{Orientation, SchematicFormat};
use mcpixel::version::Version;
use mcpixel::{Configuration, PixelArt};
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Input image file")]
    input: PathBuf,
    #[arg(help = "Schematic output")]
    output: PathBuf,
    #[arg(
        long = "format", short = 'f', value_enum,
        help = "Schematic format to output",
        default_value_t = SchematicFormat::Litematica
    )]
    format: SchematicFormat,
    #[arg(
        long, value_enum,
        default_value_t = Orientation::Vertical
    )]
    orientation: Orientation,
    #[arg(
        short = 'm',
        long,
        help = "Minecraft version to target",
        default_value = "1.21.11"
    )]
    minecraft: String,
    #[command(flatten)]
    config: Configuration,
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
    let mut args = Args::parse();

    #[cfg(debug_assertions)]
    dbg!(&args);

    if !args.input.is_file() {
        return Err(miette::miette!("Input path {:?} is not a file", args.input));
    }

    // load data
    let version = load_version(&args.minecraft)?;
    let image = fs::read(&args.input).into_diagnostic()?;

    // generate schematic and material list
    let art = PixelArt::new(image, version, args.config).into_diagnostic()?;
    let materials = art.materials();
    let schematic = art.schematic(args.orientation);

    // save schematic
    if args.output.extension().is_none() {
        args.output.set_extension(args.format.extension());
    }

    let mut file = fs::File::create(args.output).into_diagnostic()?;

    schematic.save(&mut file, args.format).into_diagnostic()?;

    #[cfg(debug_assertions)]
    dbg!(materials.values().sum::<usize>());

    println!("{:?}", materials);

    Ok(())
}
