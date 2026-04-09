use crate::proto::Texture;
use prost::Message;
use prost::bytes::Buf;

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
