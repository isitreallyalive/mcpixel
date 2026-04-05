use crate::Configuration;
use exoquant::{Color, convert_to_indexed, ditherer, optimizer};
use image::imageops::FilterType;
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage, RgbaImage};

pub(crate) fn run(image: DynamicImage, config: &Configuration) -> RgbImage {
    let mut image = resize(image, config.max_dimension as f32);
    boost_saturation(&mut image, config.gamma, config.saturation);
    let image = quantize(image, config.palette_size as usize);
    #[cfg(debug_assertions)]
    image.save("processed.png").unwrap();
    image
}

/// Resize an image to fit within a maximum dimension while preserving aspect ratio.
fn resize(image: DynamicImage, max_dimension: f32) -> RgbaImage {
    let (width, height) = (image.width() as f32, image.height() as f32);
    let scale = max_dimension / width.max(height);
    let (new_width, new_height) = ((width * scale) as u32, (height * scale) as u32);

    image
        .resize_exact(new_width, new_height, FilterType::Lanczos3)
        .to_rgba8()
}

/// Boost the saturation of an image.
fn boost_saturation(image: &mut RgbaImage, gamma: f32, factor: f32) {
    for p in image.pixels_mut() {
        // linearise (undo gamma)
        let r = ((p[0] as f32) / 255.).powf(gamma);
        let g = ((p[1] as f32) / 255.).powf(gamma);
        let b = ((p[2] as f32) / 255.).powf(gamma);

        // RGB -> HSL
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.;

        let delta = max - min;
        let s = if delta < f32::EPSILON {
            0.
        } else {
            delta / (1. - (2. * l - 1f32).abs())
        };

        // compute hue
        let h = if delta < f32::EPSILON {
            0.
        } else if max == r {
            ((g - b) / delta).rem_euclid(6.)
        } else if max == g {
            ((b - r) / delta) + 2.
        } else {
            ((r - g) / delta) + 4.
        } * 60.;

        // boost saturation
        let s_new = (s * factor).min(1.);

        // HSL -> RGB
        let c = (1. - (2. * l - 1f32).abs()) * s_new;
        let x = c * (1. - ((h / 60.) % 2. - 1.).abs());
        let m = l - c / 2.;

        let (r1, g1, b1) = if (0.0..60.0).contains(&h) {
            (c, x, 0.)
        } else if (60.0..120.0).contains(&h) {
            (x, c, 0.)
        } else if (120.0..180.0).contains(&h) {
            (0., c, x)
        } else if (180.0..240.0).contains(&h) {
            (0., x, c)
        } else if (240.0..300.0).contains(&h) {
            (x, 0., c)
        } else {
            (c, 0., x)
        };

        // reapply gamma
        p[0] = ((r1 + m).powf(1. / gamma) * 255.).clamp(0., 255.) as u8;
        p[1] = ((g1 + m).powf(1. / gamma) * 255.).clamp(0., 255.) as u8;
        p[2] = ((b1 + m).powf(1. / gamma) * 255.).clamp(0., 255.) as u8;
    }
}

/// Quantize an image's colors to a limited palette.
fn quantize(image: RgbaImage, palette_size: usize) -> RgbImage {
    // convert to rgba8
    let pixels: Vec<Color> = image
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
