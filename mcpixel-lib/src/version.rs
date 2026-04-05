use crate::proto::BlockStats;
use prost::Message;
use prost::bytes::Buf;

pub struct Version(crate::proto::Version);

impl Version {
    pub fn read(buf: impl Buf) -> Result<Self, prost::DecodeError> {
        let version: crate::proto::Version = crate::proto::Version::decode(buf)?;
        Ok(Self(version))
    }

    pub(crate) fn stats(&self) -> &Vec<BlockStats> {
        &self.0.stats
    }
}
