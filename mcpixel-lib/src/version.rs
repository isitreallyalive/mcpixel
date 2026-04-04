use crate::block::BlockAnalysis;

#[non_exhaustive]
pub struct Version(pub(crate) Vec<BlockAnalysis>);

impl TryFrom<Vec<u8>> for Version {
    type Error = rmp_serde::decode::Error;

    fn try_from(data: Vec<u8>) -> Result<Self, Self::Error> {
        let analyses: Vec<BlockAnalysis> = rmp_serde::from_slice(&data)?;
        Ok(Self(analyses))
    }
}
