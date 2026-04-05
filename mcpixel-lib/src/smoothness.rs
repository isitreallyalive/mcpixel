use crate::lab;
use crate::proto::Texture;
use image::RgbImage;
use std::cmp::Ordering;

pub(crate) fn penalty(image: &RgbImage, stats: &[Texture], target_penalty: f32) -> f32 {
    // compute per-pixel closest block distances
    let mut ratios = Vec::new();

    for pixel in image.pixels() {
        let target = lab::from_rgb(pixel.0);
        let mut min_dist = f32::MAX;
        let mut smoothness = 0f32;

        for block in stats {
            let score = {
                let mut total_distance = 0f32;
                let mut total_weight = 0f32;

                for (c, w) in block.colours.iter().zip(&block.weights) {
                    total_distance += lab::distance(target, [c.l, c.a, c.b]) * w;
                    total_weight += w;
                }

                if total_weight < f32::EPSILON {
                    f32::MAX
                } else {
                    total_distance / total_weight
                }
            };

            if score < min_dist {
                min_dist = score;
                smoothness = block.smoothness;
            }
        }

        // ignore zero smoothness
        if smoothness > 0. {
            ratios.push(min_dist / smoothness);
        }
    }

    // compute median ratio
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let median_ratio = ratios[ratios.len() / 2];

    // compute smoothness penalty
    median_ratio * target_penalty / (1. - target_penalty)
}
