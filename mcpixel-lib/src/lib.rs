pub use crate::config::Configuration;
use crate::proto::PlacedBlock;
use crate::version::Version;
use std::collections::HashMap;
use std::num::NonZero;

mod config;
pub(crate) mod lab;
mod preprocess;
#[cfg(feature = "schematic")]
pub mod schematic;
mod smoothness;
mod texture;
pub mod version;

pub(crate) mod proto {
    include!(concat!(env!("OUT_DIR"), "/block.rs"));
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
}

type OptionalGrid<T> = Vec<Vec<Option<T>>>;
type Pair<T> = (T, Option<T>);

impl PlacedBlock {
    fn resolve(self, ids: &[String]) -> (String, bool) {
        (ids[self.i as usize].clone(), self.top)
    }
}

pub struct PixelArt {
    ids: Vec<String>,
    blocks: OptionalGrid<Pair<PlacedBlock>>,
}

impl PixelArt {
    pub fn new(
        image: impl AsRef<[u8]>,
        version: Version,
        config: Configuration,
    ) -> Result<PixelArt, Error> {
        // load image
        let image = image::load_from_memory(image.as_ref())?;

        // preprocess
        let image = preprocess::run(image, &config);
        let (width, height) = image.dimensions();
        let (ids, textures) = version.into_parts();

        // compute smoothness penalty
        let textures = textures
            .into_iter()
            .filter(|t| t.overlay.is_some() == config.overlay)
            .collect::<Vec<_>>();
        let smoothness_penalty = smoothness::penalty(&image, &textures, config.smoothing);

        // compute candidate count
        let candidate_count = NonZero::new(
            10.max((config.colours as f32 * smoothness_penalty / 0.1).ceil() as usize),
        )
        .expect("this should always be at least 10");

        // convert to blocks
        let tree = texture::build_tree(&textures);
        let mut cache = HashMap::<[u8; 3], usize>::with_capacity(config.colours as usize);

        let blocks: OptionalGrid<Pair<PlacedBlock>> = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| {
                        let pixel = image.get_pixel(x, y);

                        if pixel[3] == 0 {
                            return None;
                        }

                        let rgb = [pixel[0], pixel[1], pixel[2]];
                        let idx = *cache.entry(rgb).or_insert_with(|| {
                            texture::find_best(
                                rgb,
                                &textures,
                                &tree,
                                candidate_count,
                                smoothness_penalty,
                            )
                        });
                        let texture = &textures[idx];

                        texture.base.map(|base| (base, texture.overlay))
                    })
                    .collect()
            })
            .collect();

        Ok(Self { ids, blocks })
    }

    /// The size of the pixel art in blocks (width, height).
    pub fn dimensions(&self) -> (usize, usize) {
        let height = self.blocks.len();
        let width = self.blocks.first().map(|row| row.len()).unwrap_or_default();
        (width, height)
    }

    /// Calculate the materials required to build the pixel art.
    pub fn materials(&self) -> HashMap<&str, usize> {
        let mut materials = HashMap::new();

        for row in &self.blocks {
            for (base, overlay) in row.iter().flatten() {
                // base
                *materials
                    .entry(self.ids[base.i as usize].as_str())
                    .or_insert(0) += 1;

                // overlay
                if let Some(overlay) = &overlay {
                    *materials
                        .entry(self.ids[overlay.i as usize].as_str())
                        .or_insert(0) += 2; // 2 layers
                }
            }
        }

        materials
    }

    /// Returns the pixel art as a grid of blocks.
    pub fn blocks(&self) -> OptionalGrid<Pair<(String, bool)>> {
        self.blocks
            .iter()
            .map(|row| {
                row.iter()
                    .map(|texture| {
                        texture.map(|(base, overlay)| {
                            let base = base.resolve(&self.ids);
                            let overlay = overlay.map(|o| o.resolve(&self.ids));
                            (base, overlay)
                        })
                    })
                    .collect()
            })
            .collect()
    }
}
