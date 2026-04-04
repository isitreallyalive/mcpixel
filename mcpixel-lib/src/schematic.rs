use crate::PixelArt;
use mc_schem::schem::{VanillaStructureSaveOption, WorldEdit13SaveOption};
use mc_schem::{Block, LitematicaSaveOption, Region};
use std::io::Write;

// thin-wrapper around mc_schem
#[non_exhaustive]
pub struct Schematic(mc_schem::Schematic);

impl From<Region> for Schematic {
    fn from(region: Region) -> Self {
        let mut schematic = mc_schem::Schematic::new();
        schematic.regions.push(region);
        Self(schematic)
    }
}

/// All supported schematic formats.
pub enum SchematicFormat {
    Vanilla,
    Litematica,
    WorldEdit13,
}

impl Schematic {
    /// Save the schematic to the given format.
    pub fn save(
        &self,
        dest: &mut dyn Write,
        format: SchematicFormat,
    ) -> Result<(), mc_schem::error::Error> {
        match format {
            SchematicFormat::Vanilla => self
                .0
                .save_vanilla_structure_writer(dest, &VanillaStructureSaveOption::default()),
            SchematicFormat::Litematica => self
                .0
                .save_litematica_writer(dest, &LitematicaSaveOption::default()),
            SchematicFormat::WorldEdit13 => self
                .0
                .save_world_edit_13_writer(dest, &WorldEdit13SaveOption::default()),
        }
    }
}

impl PixelArt<'_> {
    /// Turn the pixel art into a schematic.
    pub fn schematic(&self) -> Schematic {
        // create
        let mut region = Region::new();
        region.name = String::from("pixel_art");

        // resize
        let (width, height) = self.dimensions();
        let depth = if self.has_overlay() { 2 } else { 1 } + 1;
        region.reshape(&[height as i32, depth, width as i32]); // yzx

        // populate
        // todo: make sure it stands up
        for (y, row) in self.blocks.iter().enumerate() {
            for (x, block) in row.iter().enumerate() {
                // base
                if let Ok(base) = Block::from_id(&format!("minecraft:{}", block.base)) {
                    region.set_block([y as i32, 0, x as i32], &base).ok();
                }

                // overlay
                if let Some(overlay) = block
                    .overlay
                    .and_then(|o| Block::from_id(&format!("minecraft:{o}")).ok())
                {
                    region.set_block([y as i32, 1, x as i32], &overlay).ok();
                }
            }
        }

        region.into()
    }
}
