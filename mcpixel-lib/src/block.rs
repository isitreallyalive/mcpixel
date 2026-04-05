use crate::proto::BlockStats;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use std::cmp::Ordering;
use std::num::NonZero;

const CANDIDATE_COUNT: NonZero<usize> = NonZero::new(30).unwrap();

/// Scale factors derived from redmean weights at r_mean=128.
const SCALE: [f32; 3] = [
    1.5811388300841898, // red:  w = 2 + 128/256 = 2.5
    2.,                 // green: w = 4.
    1.5798734126505198, // blue: w = 2 + 127/256 ~= 2.496
];

/// Transform an RGB colour into the approximate redmean-weighted space.
/// Both tree entries and query points must be transformed before use.
fn redmean_transform([r, g, b]: [f32; 3]) -> [f32; 3] {
    [r * SCALE[0], g * SCALE[1], b * SCALE[2]]
}

/// See: https://en.wikipedia.org/wiki/Color_difference#sRGB
fn redmean_distance([r1, g1, b1]: [f32; 3], [r2, g2, b2]: [f32; 3]) -> f32 {
    let r_mean = (r1 + r2) / 2.;
    let dr = r1 - r2;
    let dg = g1 - g2;
    let db = b1 - b2;

    ((2. + r_mean / 256.) * dr * dr + 4. * dg * dg + (2. + (255. - r_mean) / 256.) * db * db).sqrt()
}

impl BlockStats {
    fn score(&self, target: [f32; 3]) -> f32 {
        let mut total_distance = 0f32;
        let mut total_weight = 0f32;

        for w in &self.weights {
            if let Some(c) = w.colour {
                total_distance += redmean_distance(target, [c.r, c.g, c.b]) * w.weight;
                total_weight += w.weight;
            }
        }

        if total_weight == 0. {
            return f32::MAX; // no usable samples means worst possible score
        }

        total_distance / total_weight
    }
}
pub(crate) struct BlockIndex {
    pub tree: ImmutableKdTree<f32, 3>,
    /// Maps tree-entry index -> original index in the stats slice.
    pub index_map: Vec<usize>,
}

/// Build a k-d tree containing the average LAB value of each block.
pub(crate) fn build_tree(stats: &[BlockStats]) -> BlockIndex {
    let (entries, index_map): (Vec<_>, Vec<_>) = stats
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            s.average
                .map(|c| (redmean_transform([c.r, c.g, c.b]), i))
        })
        .unzip();

    BlockIndex {
        tree: ImmutableKdTree::new_from_slice(&entries),
        index_map,
    }
}

/// Find the closest match in the k-d tree given a target colour.
pub(crate) fn find_best(
    target: [f32; 3],
    stats: &[BlockStats],
    index: &BlockIndex,
) -> usize {
    // transform the query point into the same space as the tree
    let transformed = redmean_transform(target);

    index
        .tree
        .nearest_n::<SquaredEuclidean>(&transformed, CANDIDATE_COUNT)
        .iter()
        .map(|n| {
            let stats_idx = index.index_map[n.item as usize];
            (stats_idx, stats[stats_idx].score(target))
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
        .map(|(idx, _)| idx)
        .unwrap_or_default()
}