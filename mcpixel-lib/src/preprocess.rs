use crate::Configuration;
use exoquant::{Color, convert_to_indexed, ditherer, optimizer};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb, RgbImage};

pub(crate) fn run(image: DynamicImage, config: &Configuration) -> RgbImage {
    let image = resize(image, config.max_dimension as f32);
    quantize(image, config.palette_size as usize)
}

/// Resize an image to fit within a maximum dimension while preserving aspect ratio.
fn resize(image: DynamicImage, max_dimension: f32) -> DynamicImage {
    let (width, height) = (image.width() as f32, image.height() as f32);
    let scale = max_dimension / width.max(height);
    let (new_width, new_height) = ((width * scale) as u32, (height * scale) as u32);

    image.resize_exact(new_width, new_height, FilterType::Lanczos3)
}

/// Quantize the image's palette to a set number of colours.
fn quantize(image: DynamicImage, palette_size: usize) -> RgbImage {
    // convert to rgba8
    let rgba = image.to_rgba8();
    let pixels: Vec<Color> = rgba
        .pixels()
        .map(|p| Color::new(p[0], p[1], p[2], p[3]))
        .collect();

    // quantize
    let (width, height) = image.dimensions();
    let (palette, indices) = convert_to_indexed(
        &pixels,
        width as usize,
        palette_size,
        &optimizer::KMeans,
        &ditherer::FloydSteinberg::checkered(),
    );

    // rebuild the image
    let mut img = ImageBuffer::new(width, height);

    for (i, pixel) in img.pixels_mut().enumerate() {
        let idx = indices[i] as usize;
        let Color { r, g, b, .. } = palette[idx];
        *pixel = Rgb([r, g, b])
    }

    img
}
