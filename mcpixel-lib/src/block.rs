use crate::proto::BlockStats;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use palette::{IntoColor, Lab, Srgb};
use std::cmp::Ordering;
use std::num::NonZero;

const CANDIDATE_COUNT: NonZero<usize> = NonZero::new(10).unwrap(); // 10 is enough now

fn to_lab([r, g, b]: [f32; 3]) -> [f32; 3] {
    let lab: Lab = Srgb::new(r / 255., g / 255., b / 255.).into_color();
    [lab.l, lab.a, lab.b]
}

fn lab_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let [l1, a1, b1] = a;
    let [l2, a2, b2] = b;
    (l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)
}

impl BlockStats {
    fn score(&self, target_lab: [f32; 3]) -> f32 {
        let mut total_distance = 0f32;
        let mut total_weight = 0f32;

        for w in &self.weights {
            if let Some(c) = w.colour {
                let sample_lab = [c.l, c.a, c.b];
                total_distance += lab_distance(target_lab, sample_lab) * w.weight;
                total_weight += w.weight;
            }
        }

        if total_weight == 0. {
            return f32::MAX;
        }

        total_distance / total_weight
    }
}

pub(crate) struct BlockIndex {
    pub tree: ImmutableKdTree<f32, 3>,
    pub index_map: Vec<usize>,
}

pub(crate) fn build_tree(stats: &[BlockStats]) -> BlockIndex {
    let (entries, index_map): (Vec<_>, Vec<_>) = stats
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.average.map(|c| ([c.l, c.a, c.b], i)))
        .unzip();

    BlockIndex {
        tree: ImmutableKdTree::new_from_slice(&entries),
        index_map,
    }
}

/// Find the closest match in the k-d tree given a target colour.
pub(crate) fn find_best(target: [f32; 3], stats: &[BlockStats], index: &BlockIndex) -> usize {
    let transformed = to_lab(target);

    index
        .tree
        .nearest_n::<SquaredEuclidean>(&transformed, CANDIDATE_COUNT)
        .iter()
        .map(|n| {
            let stats_idx = index.index_map[n.item as usize];
            (stats_idx, stats[stats_idx].score(transformed))
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
        .map(|(idx, _)| idx)
        .unwrap_or_default()
}
