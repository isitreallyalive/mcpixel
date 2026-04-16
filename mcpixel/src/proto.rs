include!(concat!(env!("OUT_DIR"), "/block.rs"));

impl PlacedBlock {
    pub(crate) fn resolve(self, ids: &[String]) -> (String, bool) {
        (ids[self.i as usize].clone(), self.top)
    }
}
