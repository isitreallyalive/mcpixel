pub use crate::art::PixelArt;
pub use crate::config::Configuration;

mod art;
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}
