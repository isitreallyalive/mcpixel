use clap::Parser;
use mcpixel::process;
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Input image file path")]
    input: PathBuf,
    #[arg(help = "Output image file path")]
    output: PathBuf,

    #[arg(
        name = "size",
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

    #[arg(short = 'o', long, help = "Should there be a glass overlay?")]
    overlay: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    dbg!(&args);

    if !args.input.is_file() {
        return Err(miette::miette!("Input path {:?} is not a file", args.input));
    }

    let image = image::open(&args.input).into_diagnostic()?;
    let blocks =
        process(image, args.max_dimension, args.palette_size, args.overlay).into_diagnostic()?;

    blocks.write_file(args.output).into_diagnostic()?;

    Ok(())
}
