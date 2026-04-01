use clap::Parser;
use mcpixel::process;
use miette::{IntoDiagnostic, Result, WrapErr};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Input image file path")]
    input: PathBuf,

    #[arg(short, long, help = "Output image file path")]
    output: PathBuf,

    #[arg(
        short,
        long,
        default_value_t = 64,
        help = "Maximum dimension for resizing"
    )]
    max_dimension: u32,

    #[arg(
        short = 'p',
        long,
        default_value_t = 256,
        help = "Number of colors in the palette"
    )]
    palette_size: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(&args);

    if !args.input.is_file() {
        return Err(miette::miette!("Input path {:?} is not a file", args.input));
    }

    let image = image::open(&args.input).into_diagnostic()?;
    let processed_image =
        process(image, args.max_dimension, args.palette_size).into_diagnostic()?;

    processed_image
        .save(&args.output)
        .into_diagnostic()
        .wrap_err(format!("Failed to save output image to {:?}", args.output))?;

    Ok(())
}
