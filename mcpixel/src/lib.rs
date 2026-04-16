pub use crate::art::PixelArt;
pub use crate::config::Configuration;

mod art;
mod config;
pub(crate) mod lab;
mod preprocess;
pub(crate) mod proto;
#[cfg(feature = "schematic")]
pub mod schematic;
mod smoothness;
mod texture;
pub mod version;

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
