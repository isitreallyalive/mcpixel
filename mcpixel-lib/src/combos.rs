use kiddo::{ImmutableKdTree, SquaredEuclidean};
use serde::Deserialize;
use std::num::NonZero;

pub static DATA: &[u8] = include_bytes!("../../assets/1.21.6.msgpack");

pub type ComboTree = ImmutableKdTree<f32, 3>;

#[derive(Deserialize)]
pub struct Combo {
    #[serde(rename = "b")]
    pub(crate) base: String,
    #[serde(rename = "o")]
    pub(crate) overlay: Option<String>,
    #[serde(rename = "a")]
    average_rgb: [f32; 3],
    /// (r, g, b, count)
    #[serde(rename = "c")]
    colour_freq: Vec<[u32; 4]>,
}

/// See: https://en.wikipedia.org/wiki/Color_difference#sRGB
fn redmean_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let r_mean = (a[0] + b[0]) / 2.;
    let dr = a[0] - b[0];
    let dg = a[1] - b[1];
    let db = a[2] - b[2];

    ((2. + r_mean / 256.) * dr * dr + 4. * dg * dg + (2. + (255. - r_mean) / 256.) * db * db).sqrt()
}

fn score_combo(target: [f32; 3], combo: &Combo) -> f32 {
    let total: u32 = combo.colour_freq.iter().map(|e| e[3]).sum();

    combo
        .colour_freq
        .iter()
        .map(|e| {
            let colour = [e[0] as f32, e[1] as f32, e[2] as f32];
            let weight = e[3] as f32 / total as f32;
            redmean_distance(target, colour) * weight
        })
        .sum()
}

pub fn find_best(target: [f32; 3], combos: &[Combo], tree: &ComboTree) -> usize {
    tree.nearest_n::<SquaredEuclidean>(&target, NonZero::new(10).unwrap())
        .iter()
        .map(|n| {
            (
                n.item as usize,
                score_combo(target, &combos[n.item as usize]),
            )
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(idx, _)| idx).unwrap_or(0)
}

/// Build a k-d tree containing all combos.
pub fn build_tree(combos: &[Combo]) -> ComboTree {
    let entries: Vec<[f32; 3]> = combos.iter().map(|c| c.average_rgb).collect();
    ImmutableKdTree::new_from_slice(&entries)
}
