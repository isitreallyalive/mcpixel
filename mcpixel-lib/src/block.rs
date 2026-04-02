use crate::combos::Combo;

#[derive(Debug)]
pub struct Block {
    pub base: String,
    pub overlay: Option<String>
}

impl From<&Combo> for Block {
    fn from(combo: &Combo) -> Self {
        Block {
            base: combo.base.clone(),
            overlay: combo.overlay.clone()
        }
    }
}