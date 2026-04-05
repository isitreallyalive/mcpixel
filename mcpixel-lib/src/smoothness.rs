use crate::lab;
use crate::proto::Texture;
use image::RgbImage;
use std::cmp::Ordering;
use std::collections::HashMap;

pub(crate) fn penalty(image: &RgbImage, stats: &[Texture], target_penalty: f32) -> f32 {
    // collapse repeated colours first; preprocessing quantizes colours so duplicates are common.
    let mut counts = HashMap::<[u8; 3], usize>::new();
    for pixel in image.pixels() {
        *counts.entry(pixel.0).or_insert(0) += 1;
    }

    // compute per-colour closest block distance and keep multiplicity for weighted median
    let mut ratios = Vec::with_capacity(counts.len());

    for (rgb, count) in counts {
        let target = lab::from_rgb(rgb);
        let mut min_dist = f32::MAX;
        let mut smoothness = 0f32;

        for block in stats {
            let score = block.score(target, 0.);

            if score < min_dist {
                min_dist = score;
                smoothness = block.smoothness;
            }
        }

        // ignore zero smoothness
        if smoothness > 0. {
            ratios.push((min_dist / smoothness, count));
        }
    }

    if ratios.is_empty() {
        return 0.;
    }

    // compute weighted median ratio to preserve per-pixel behaviour after collapsing duplicates
    ratios.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    let total: usize = ratios.iter().map(|(_, count)| *count).sum();
    let mid = total / 2;
    let mut running = 0usize;
    let median_ratio = ratios
        .into_iter()
        .find_map(|(ratio, count)| {
            running += count;
            (running > mid).then_some(ratio)
        })
        .unwrap_or(0.);

    // compute smoothness penalty
    median_ratio * target_penalty / (1. - target_penalty)
}
