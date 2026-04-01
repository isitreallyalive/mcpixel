use image::imageops::FilterType;
use image::{DynamicImage, ImageBuffer, Rgb};
use zenquant::{OutputFormat, QuantizeConfig, RGB};

fn resize(image: DynamicImage, max_dimension: f32) -> DynamicImage {
    let (width, height) = (image.width() as f32, image.height() as f32);
    let scale = (max_dimension / width.max(height)).min(1.);
    let (new_width, new_height) = ((width * scale) as u32, (height * scale) as u32);
    image.resize_exact(new_width, new_height, FilterType::Triangle)
}

fn quantize(
    image: DynamicImage,
    palette_size: u32,
) -> Result<DynamicImage, zenquant::error::QuantizeError> {
    let rgb_image = image.to_rgb8();
    let (width, height) = rgb_image.dimensions();

    // prepare pixels for zenquant
    let pixels: Vec<_> = rgb_image
        .pixels()
        .map(|p| RGB::new(p[0], p[1], p[2]))
        .collect();

    // quantize
    let config = QuantizeConfig::new(OutputFormat::Png).with_max_colors(palette_size);
    let quant = zenquant::quantize(&pixels, width as usize, height as usize, &config)?;
    let palette = quant.palette();
    let indices = quant.indices();

    // reconstruct image
    let mut out = ImageBuffer::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let idx = indices[(y * width + x) as usize] as usize;
            out.put_pixel(x, y, Rgb(palette[idx]));
        }
    }

    Ok(DynamicImage::ImageRgb8(out))
}

pub fn process(
    image: DynamicImage,
    max_dimension: u32,
    palette_size: u32,
) -> Result<DynamicImage, zenquant::error::QuantizeError> {
    let image = resize(image, max_dimension as f32);
    quantize(image, palette_size)
}
