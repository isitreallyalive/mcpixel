use crate::proto::Texture;
use prost::Message;
use prost::bytes::Buf;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Clone)]
pub struct Version(crate::proto::Version);

impl Version {
    pub fn read(buf: impl Buf) -> Result<Self, prost::DecodeError> {
        let version: crate::proto::Version = crate::proto::Version::decode(buf)?;
        Ok(Self(version))
    }

    pub(crate) fn into_parts(self) -> (Vec<String>, Vec<Texture>) {
        (self.0.ids, self.0.textures)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Version {
    #[wasm_bindgen(constructor)]
    pub fn new_wasm(version: &[u8]) -> Result<Self, JsValue> {
        Version::read(version).map_err(|_| JsValue::from_str("invalid version"))
    }
}
