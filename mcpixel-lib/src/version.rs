use crate::proto::Texture;
use prost::Message;
use prost::bytes::Buf;

pub struct Version(crate::proto::Version);

impl Version {
    pub fn read(buf: impl Buf) -> Result<Self, prost::DecodeError> {
        let version: crate::proto::Version = crate::proto::Version::decode(buf)?;
        Ok(Self(version))
    }

    pub(crate) fn ids(&self) -> Vec<String> {
        self.0.ids.clone()
    }

    pub(crate) fn textures(&self) -> &Vec<Texture> {
        &self.0.textures
    }
}
