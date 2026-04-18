use crate::proto::PlacedBlock;
use crate::version::Version;
use crate::{Configuration, Error, preprocess, smoothness, texture};
use std::collections::HashMap;
use std::num::NonZero;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

type Grid<T> = Vec<Vec<Option<T>>>;

pub(crate) struct PlacedBlockLayer {
    pub(crate) base: PlacedBlock,
    pub(crate) overlay: Option<PlacedBlock>,
}

#[cfg_attr(target_arch = "wasm32", derive(serde::Serialize, tsify::Tsify))]
#[cfg_attr(target_arch = "wasm32", tsify(into_wasm_abi))]
pub struct BlockLayer {
    pub base: String,
    pub overlay: Option<String>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct PixelArt {
    pub(crate) ids: Vec<String>,
    pub(crate) blocks: Grid<PlacedBlockLayer>,
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

        let blocks = (0..height)
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

                        texture.base.map(|base| PlacedBlockLayer {
                            base,
                            overlay: texture.overlay,
                        })
                    })
                    .collect()
            })
            .collect();

        Ok(Self { ids, blocks })
    }

    fn compute_materials(&self) -> HashMap<&str, usize> {
        let mut materials = HashMap::new();

        for layer in self.blocks.iter().flatten().flatten() {
            // base
            *materials
                .entry(self.ids[layer.base.i as usize].as_str())
                .or_insert(0) += 1;

            // overlay
            if let Some(overlay) = &layer.overlay {
                *materials
                    .entry(self.ids[overlay.i as usize].as_str())
                    .or_insert(0) += 2; // 2 layers
            }
        }

        materials
    }

    fn compute_blocks(&self) -> Grid<BlockLayer> {
        self.blocks
            .iter()
            .map(|row| {
                row.iter()
                    .map(|layer| {
                        layer.as_ref().map(|layer| BlockLayer {
                            base: self.ids[layer.base.i as usize].clone(),
                            overlay: layer.overlay.map(|o| self.ids[o.i as usize].clone()),
                        })
                    })
                    .collect()
            })
            .collect()
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PixelArt {
    /// The width of the pixel art in blocks.
    pub fn width(&self) -> usize {
        self.blocks.len()
    }

    /// The height of the pixel art in blocks.
    pub fn height(&self) -> usize {
        self.blocks.first().map(|row| row.len()).unwrap_or_default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PixelArt {
    /// Calculate the materials required to build the pixel art.
    pub fn materials(&self) -> HashMap<&str, usize> {
        self.compute_materials()
    }

    /// Returns the pixel art as a grid of blocks.
    pub fn blocks(&self) -> Grid<BlockLayer> {
        self.compute_blocks()
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;

    #[wasm_bindgen(typescript_custom_section)]
    const TS_APPEND_CONTENT: &'static str = r#"
export type Materials = { [key: string]: number };
export type Blocks = { base: string; overlay?: string; }[][];
"#;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(typescript_type = "Materials")]
        pub type Materials;

        #[wasm_bindgen(typescript_type = "Blocks")]
        pub type Blocks;
    }

    #[wasm_bindgen]
    impl PixelArt {
        #[wasm_bindgen(constructor)]
        pub fn new_wasm(
            image: &[u8],
            version: Version,
            config: Option<crate::config::Config>,
        ) -> Result<PixelArt, JsValue> {
            PixelArt::new(image, version, config.map(Into::into).unwrap_or_default())
                .map_err(|_| JsValue::from_str("invalid art"))
        }

        // todo: can this be turned into js_sys::Map and keep typing?
        /// Calculate the materials required to build the pixel art.
        #[allow(unused_variables)] // why is this even emitted???
        #[wasm_bindgen(typescript_type = "Materials")]
        pub fn materials(&self) -> Result<Materials, JsValue> {
            let materials = self.compute_materials();

            serde_wasm_bindgen::to_value(&materials)
                .map(Materials::from)
                .map_err(|_| JsValue::from_str("failed to serialize materials"))
        }

        /// Returns the pixel art as a grid of blocks.
        pub fn blocks(&self) -> Result<Blocks, JsValue> {
            let blocks = self.compute_blocks();

            serde_wasm_bindgen::to_value(&blocks)
                .map(Blocks::from)
                .map_err(|_| JsValue::from_str("failed to serialize blocks"))
        }
    }
}
