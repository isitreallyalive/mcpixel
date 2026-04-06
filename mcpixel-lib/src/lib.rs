use crate::proto::PlacedBlock;
use crate::version::Version;
use std::collections::HashMap;
use std::num::NonZero;

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

pub struct Configuration {
    pub max_dimension: u32,
    pub palette_size: u32,
    pub gamma: f32,
    pub saturation: f32,
    pub smoothness_penalty: f32,
    pub overlay: bool,
    pub scale_to_fit: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            max_dimension: 32,
            palette_size: 256,
            gamma: 1.2,
            saturation: 1.2,
            smoothness_penalty: 0.3,
            overlay: false,
            scale_to_fit: false,
        }
    }
}

type StrippedTexture = (PlacedBlock, Option<PlacedBlock>);

pub struct PixelArt {
    ids: Vec<String>,
    blocks: Vec<Vec<Option<StrippedTexture>>>,
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
        let smoothness_penalty = smoothness::penalty(&image, &textures, config.smoothness_penalty);

        // compute candidate count
        let candidate_count = NonZero::new(
            10.max((config.palette_size as f32 * smoothness_penalty / 0.1).ceil() as usize),
        )
        .expect("this should always be at least 10");

        // convert to blocks
        let tree = texture::build_tree(&textures);
        let mut cache = HashMap::<[u8; 3], usize>::with_capacity(config.palette_size as usize);

        let blocks: Vec<Vec<Option<StrippedTexture>>> = (0..height)
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
}
