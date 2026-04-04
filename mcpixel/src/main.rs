use clap::Parser;
use mcpixel::schematic::SchematicFormat;
use mcpixel::{Configuration, PixelArt};
use miette::{IntoDiagnostic, Result};
use std::fs::File;
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
}

fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(&args);

    if !args.input.is_file() {
        return Err(miette::miette!("Input path {:?} is not a file", args.input));
    }

    // determine configuration
    let mut config = Configuration::default();
    args.max_dimension.map(|d| config.max_dimension = d);
    args.palette_size.map(|p| config.palette_size = p);
    config.overlay = args.overlay;

    let image = std::fs::read(&args.input).into_diagnostic()?;
    let art = PixelArt::new(image, config).into_diagnostic()?;
    let schematic = art.schematic();
    let mut file = File::create("output.litematic").into_diagnostic()?;
    schematic
        .save(&mut file, SchematicFormat::Litematica)
        .into_diagnostic()?;

    Ok(())
}
