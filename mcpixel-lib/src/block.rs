use crate::proto::BlockStats;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
// use lab::Lab;
use std::cmp::Ordering;
use std::num::NonZero;

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
        self.weights
            .iter()
            .filter_map(|w| {
                if let Some(c) = w.colour {
                    Some(
                        redmean_distance(
                            target,
                            [c.r, c.g, c.b], // Lab {
                                             //     l: c.l,
                                             //     a: c.a,
                                             //     b: c.b,
                                             // },
                        ) * w.weight,
                    )
                } else {
                    None
                }
            })
            .sum()
    }
}
pub(crate) type AnalysisTree = ImmutableKdTree<f32, 3>;

/// Build a k-d tree containing the average LAB value of each block.
pub(crate) fn build_tree(stats: &[BlockStats]) -> AnalysisTree {
    let entries = stats
        .iter()
        .filter_map(|s| {
            if let Some(c) = s.average {
                Some([c.r, c.g, c.b])
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    ImmutableKdTree::new_from_slice(&entries)
}

/// Find the closest match in the k-d tree given a target colour.
pub(crate) fn find_best(
    target: [f32; 3],
    stats: &[BlockStats],
    tree: &AnalysisTree,
) -> Option<usize> {
    tree.nearest_n::<SquaredEuclidean>(&target, NonZero::new(10)?)
        .iter()
        .map(|n| (n.item as usize, stats[n.item as usize].score(target)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
        .map(|(idx, _)| idx)
}
