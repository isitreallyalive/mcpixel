use crate::proto::Block;
use crate::version::Version;
use image::Rgb;
use lab::Lab;
use std::collections::HashMap;

mod block;
mod preprocess;
pub mod schematic;
pub mod version;

pub(crate) mod proto {
    include!(concat!(env!("OUT_DIR"), "/block.rs"));
}

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
        let stats = version
            .stats()
            .iter()
            .filter(|s| {
                s.block
                    .as_ref()
                    .expect("block is missing for this entry")
                    .overlay
                    .is_some()
                    == config.overlay
            })
            .cloned()
            .collect::<Vec<_>>();
        let tree = block::build_tree(&stats);
        let mut cache = HashMap::<[u8; 3], usize>::new();

        let blocks: Vec<Vec<_>> = (0..height)
            .map(|y| {
                (0..width)
                    .filter_map(|x| {
                        let Rgb(key) = *image.get_pixel(x, y);
                        let idx = *cache.entry(key).or_insert_with(|| {
                            block::find_best(Lab::from_rgb(&key), &stats, &tree).unwrap_or_default()
                        });
                        stats[idx].block.clone()
                    })
                    .collect()
            })
            .collect();

        Ok(Self { blocks })
    }

    /// The size of the pixel art in blocks.
    pub fn dimensions(&self) -> (usize, usize) {
        (
            self.blocks.len(),
            self.blocks.first().map(|row| row.len()).unwrap_or_default(),
        )
    }

    /// Calculate the materials required to build the pixel art.
    pub fn materials(&self) -> HashMap<&str, usize> {
        let mut materials = HashMap::new();

        for row in &self.blocks {
            for block in row {
                // base
                *materials.entry(block.base.as_str()).or_insert(0) += 1;

                // overlay
                if let Some(overlay) = &block.overlay {
                    *materials.entry(overlay.as_str()).or_insert(0) += 1;
                }
            }
        }

        materials
    }
}
