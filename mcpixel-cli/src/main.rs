use clap::Parser;
use hex::FromHex;
use mcpixel::Configuration;
use mcpixel::schematic::{Orientation, SchematicFormat};
use mcpixel::version::Version;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::PathBuf;

const DATA_URL: &str = "https://github.com/isitreallyalive/mcpixel/raw/refs/heads/main/data";

#[derive(Debug, Parser)]
#[command(bin_name = "mcpixel", author, version, about)]
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

fn download(url: &str) -> Result<Vec<u8>> {
    ureq::get(url)
        .call()
        .and_then(|mut r| r.body_mut().read_to_vec())
        .into_diagnostic()
}

/// Load version data.
fn load_version(version: &str) -> Result<Version> {
    let cache_dir = directories::ProjectDirs::from("dev", "newty", "mcpixel")
        .map(|d| d.cache_dir().to_path_buf())
        .or(std::env::current_dir().ok())
        .expect("there should be somewhere to cache version data");
    let data_path = cache_dir.join(version);

    if data_path.exists() {
        // make sure it is still up to date
        let latest_checksum = {
            let hex = download(&format!("{DATA_URL}/{version}.md5"))?;
            let digest = <[u8; 16]>::from_hex(hex.as_slice()).into_diagnostic()?;

            md5::Digest(digest)
        };

        let data = fs::read(&data_path).into_diagnostic()?;
        let checksum = md5::compute(&data);
        println!("{:?} {:?}", latest_checksum, checksum);

        if latest_checksum == checksum {
            return Version::read(&data[..]).into_diagnostic();
        }
    }

    // download the latest version data
    let data = download(&format!("{DATA_URL}/{version}"))?;
    fs::write(data_path, &data).into_diagnostic()?;

    Version::read(&data[..]).into_diagnostic()
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
    // let image = fs::read(&args.input).into_diagnostic()?;
    //
    // // generate schematic and material list
    // let art = PixelArt::new(image, version, args.config).into_diagnostic()?;
    // let materials = art.materials();
    // let schematic = art.schematic(args.orientation);
    //
    // // save schematic
    // if args.output.extension().is_none() {
    //     args.output.set_extension(args.format.extension());
    // }
    //
    // let mut file = fs::File::create(args.output).into_diagnostic()?;
    //
    // schematic.save(&mut file, args.format).into_diagnostic()?;
    //
    // #[cfg(debug_assertions)]
    // dbg!(materials.values().sum::<usize>());
    //
    // println!("{:?}", materials);

    Ok(())
}
