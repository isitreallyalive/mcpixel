pub use crate::block::Block;
use crate::combos::{Combo, build_tree, find_best};
use image::imageops::FilterType;
use image::{DynamicImage, ImageBuffer, Rgb};
use std::collections::HashMap;
use zenquant::{OutputFormat, QuantizeConfig, QuantizeResult, RGB};

mod block;
mod combos;

/// Resize an image to fit within a maximum dimension while preserving aspect ratio.
fn resize(image: DynamicImage, max_dimension: f32) -> DynamicImage {
    let (width, height) = (image.width() as f32, image.height() as f32);
    let scale = max_dimension / width.max(height);
    let (new_width, new_height) = ((width * scale) as u32, (height * scale) as u32);

    image.resize_exact(new_width, new_height, FilterType::Triangle)
}

/// Apply a quantization palette to an image buffer in place.
fn apply_palette(
    buffer: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    quantized: QuantizeResult,
    width: u32,
    height: u32,
) {
    let palette = quantized.palette();
    let indices = quantized.indices();

    for y in 0..height {
        for x in 0..width {
            let idx = indices[(y * width + x) as usize] as usize;
            buffer.put_pixel(x, y, Rgb(palette[idx]));
        }
    }
}

/// Quantize an image to a fixed number of colours.
fn quantize(
    image: &mut DynamicImage,
    palette_size: u32,
) -> Result<(), zenquant::error::QuantizeError> {
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

    // modify image
    if let DynamicImage::ImageRgb8(buffer) = image {
        apply_palette(buffer, quant, width, height);
    } else {
        let mut buffer = ImageBuffer::new(width, height);
        apply_palette(&mut buffer, quant, width, height);
        *image = DynamicImage::ImageRgb8(buffer);
    }

    Ok(())
}

pub fn process(
    image: DynamicImage,
    max_dimension: u32,
    palette_size: u32,
) -> Result<Vec<Vec<Block>>, zenquant::error::QuantizeError> {
    let mut image = resize(image, max_dimension as f32);
    quantize(&mut image, palette_size)?;

    let combos: Vec<Combo> = rmp_serde::from_slice(combos::DATA).unwrap();
    let tree = build_tree(&combos);

    let rgb = image.to_rgb8();
    let (width, height) = rgb.dimensions();
    let mut cache: HashMap<[u8; 3], usize> = HashMap::new();

    let result: Vec<Vec<Block>> = (0..height)
        .map(|y| {
            (0..width)
                .map(|x| {
                    let Rgb(key) = *rgb.get_pixel(x, y); // [u8; 3] directly from image
                    let idx = *cache.entry(key).or_insert_with(|| {
                        let target = key.map(|c| c as f32);
                        find_best(target, &combos, &tree)
                    });
                    &combos[idx]
                })
                .map(Block::from)
                .collect()
        })
        .collect();

    Ok(result)
}
