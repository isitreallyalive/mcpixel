use crate::proto::BlockStats;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use lab::Lab;
use std::cmp::Ordering;
use std::num::NonZero;

fn lab_distance(c1: Lab, c2: Lab) -> f32 {
    let dl = c1.l - c2.l;
    let da = c1.a - c2.a;
    let db = c1.b - c2.b;
    (dl * dl + da * da + db * db).sqrt()
}

impl BlockStats {
    fn score(&self, target: Lab) -> f32 {
        self.weights
            .iter()
            .filter_map(|w| {
                if let Some(c) = w.colour {
                    Some(
                        lab_distance(
                            target,
                            Lab {
                                l: c.l,
                                a: c.a,
                                b: c.b,
                            },
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
                Some([c.l, c.a, c.b])
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    ImmutableKdTree::new_from_slice(&entries)
}

/// Find the closest match in the k-d tree given a target colour.
pub(crate) fn find_best(
    target: lab::Lab,
    stats: &[BlockStats],
    tree: &AnalysisTree,
) -> Option<usize> {
    tree.nearest_n::<SquaredEuclidean>(&[target.l, target.a, target.b], NonZero::new(10)?)
        .iter()
        .map(|n| (n.item as usize, stats[n.item as usize].score(target)))
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
        .map(|(idx, _)| idx)
}
