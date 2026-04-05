use crate::lab;
use crate::proto::Texture;
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use std::cmp::Ordering;
use std::num::NonZero;

impl Texture {
    fn score(&self, target_lab: [f32; 3], smoothness_penalty: f32) -> f32 {
        let mut total_distance = 0f32;
        let mut total_weight = 0f32;

        for (c, w) in self.colours.iter().zip(&self.weights) {
            total_distance += lab::distance(target_lab, [c.l, c.a, c.b]) * w;
            total_weight += w;
        }

        if total_weight < f32::EPSILON {
            return f32::MAX;
        }

        total_distance / total_weight + smoothness_penalty * self.smoothness
    }
}

pub(crate) struct BlockIndex {
    pub tree: ImmutableKdTree<f32, 3>,
    pub index_map: Vec<usize>,
}

pub(crate) fn build_tree(textures: &[Texture]) -> BlockIndex {
    let (entries, index_map): (Vec<_>, Vec<_>) = textures
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
pub(crate) fn find_best(
    target: [u8; 3],
    textures: &[Texture],
    index: &BlockIndex,
    candidate_count: NonZero<usize>,
    smoothness_penalty: f32,
) -> usize {
    let transformed = lab::from_rgb(target);

    index
        .tree
        .nearest_n::<SquaredEuclidean>(&transformed, candidate_count)
        .iter()
        .map(|n| {
            let stats_idx = index.index_map[n.item as usize];
            (
                stats_idx,
                textures[stats_idx].score(transformed, smoothness_penalty),
            )
        })
        .min_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        })
        .map(|(idx, _)| idx)
        .unwrap_or_default()
}
