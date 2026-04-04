use kiddo::{ImmutableKdTree, SquaredEuclidean};
use serde::Deserialize;
use std::cmp::Ordering;
use std::num::NonZero;

#[derive(Deserialize)]
pub(crate) struct BlockAnalysis {
    #[serde(rename = "b")]
    base: String,
    #[serde(rename = "o")]
    pub(crate) overlay: Option<String>,
    #[serde(rename = "a")]
    average_rgb: [f32; 4],
    #[serde(rename = "c")]
    colour_freq: Vec<[u32; 5]>,
}

/// See: https://en.wikipedia.org/wiki/Color_difference#sRGB
fn redmean_distance([r1, g1, b1, _]: [f32; 4], [r2, g2, b2, _]: [f32; 4]) -> f32 {
    let r_mean = (r1 + r2) / 2.;
    let dr = r1 - r2;
    let dg = g1 - g2;
    let db = b1 - b2;

    ((2. + r_mean / 256.) * dr * dr + 4. * dg * dg + (2. + (255. - r_mean) / 256.) * db * db).sqrt()
}

impl BlockAnalysis {
    fn score(&self) -> f32 {
        let total: u32 = self.colour_freq.iter().map(|e| e[4]).sum();

        self.colour_freq
            .iter()
            .map(|[r, g, b, a, count]| {
                let colour = [*r as f32, *g as f32, *b as f32, *a as f32];
                let weight = *count as f32 / total as f32;
                redmean_distance(self.average_rgb, colour) * weight
            })
            .sum()
    }
}

pub(crate) struct Block {
    pub(crate) base: String,
    pub(crate) overlay: Option<String>,
}

impl From<&BlockAnalysis> for Block {
    fn from(analysis: &BlockAnalysis) -> Self {
        Self {
            base: analysis.base.clone(),
            overlay: analysis.overlay.clone(),
        }
    }
}

pub(crate) type AnalysisTree = ImmutableKdTree<f32, 4>;

/// Build a k-d tree containing the average RGB value of each block.
pub(crate) fn build_tree(analyses: &[BlockAnalysis]) -> AnalysisTree {
    let entries = analyses.iter().map(|c| c.average_rgb).collect::<Vec<_>>();

    ImmutableKdTree::new_from_slice(&entries)
}

/// Find the closest match in the k-d tree given a target colour.
pub(crate) fn find_best(
    target: [f32; 4],
    analyses: &[BlockAnalysis],
    tree: &AnalysisTree,
) -> Option<usize> {
    tree.nearest_n::<SquaredEuclidean>(&target, NonZero::new(10)?)
        .iter()
        .map(|n| (n.item as usize, analyses[n.item as usize].score()))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
        .map(|(idx, _)| idx)
}
