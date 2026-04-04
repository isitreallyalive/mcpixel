use crate::block::Block;
use crate::version::Version;
use image::{GenericImageView, Rgba};
use std::collections::HashMap;

mod block;
mod preprocess;
pub mod schematic;
pub mod version;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
}

#[derive(Clone, Copy)]
pub struct Configuration {
    pub max_dimension: u32,
    pub palette_size: u32,
    pub overlay: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            max_dimension: 64,
            palette_size: 256,
            overlay: false,
        }
    }
}

pub struct PixelArt {
    blocks: Vec<Vec<Block>>,
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

        // convert to blocks
        let analyses = version
            .0
            .into_iter()
            .filter(|a| a.overlay.is_some() == config.overlay)
            .collect::<Vec<_>>();
        let tree = block::build_tree(&analyses);
        let mut cache = HashMap::<[u8; 4], usize>::new();

        let blocks: Vec<Vec<_>> = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| {
                        let Rgba(key) = *image.get_pixel(x, y);
                        let idx = *cache.entry(key).or_insert_with(|| {
                            block::find_best(key.map(|c| c as f32), &analyses, &tree)
                                .unwrap_or_default()
                        });
                        &analyses[idx]
                    })
                    .map(Block::from)
                    .collect()
            })
            .collect();

        Ok(Self { blocks })
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (
            self.blocks.len(),
            self.blocks.first().map(|row| row.len()).unwrap_or_default(),
        )
    }

    pub(crate) fn has_overlay(&self) -> bool {
        self.blocks
            .iter()
            .any(|row| row.iter().any(|b| b.overlay.is_some()))
    }
}
