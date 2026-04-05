use crate::PixelArt;
use crate::proto::PlacedBlock;
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

/// The plane to render the schematic on.
pub enum Plane {
    Standing,
    Flat,
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

fn get_block(found: &PlacedBlock, plane: &Plane) -> Option<Block> {
    let mut block = Block::from_id(&format!("minecraft:{}", found.id)).ok()?;

    // set axis
    let axis = match (plane, found.top) {
        (Plane::Standing, false) => 'y',
        (Plane::Standing, true) => 'x',
        (Plane::Flat, false) => 'x',
        (Plane::Flat, true) => 'y',
    };
    block.attributes.insert("axis".into(), axis.into());

    Some(block)
}

impl PixelArt {
    /// Does the pixel art have an overlay?
    fn has_overlay(&self) -> bool {
        self.blocks
            .iter()
            .any(|row| row.iter().any(|b| b.overlay.is_some()))
    }

    /// Turn the pixel art into a schematic.
    pub fn schematic(&self, plane: Plane) -> Schematic {
        // create
        let mut region = Region::new();
        region.name = String::from("pixel_art");

        // resize
        let (width, height) = self.dimensions();
        let depth = if self.has_overlay() { 2 } else { 1 } + 1;

        // populate
        match plane {
            // stood up
            Plane::Standing => {
                // x=width, y=depth, z=height
                region.reshape(&[depth, height as i32, width as i32]); // yzx
                for (y, row) in self.blocks.iter().enumerate() {
                    let y = (height - 1 - y) as i32; // flip

                    for (z, block) in row.iter().enumerate() {
                        let z = (width - 1 - z) as i32; // flip

                        if let Some(base) = block.base.as_ref().and_then(|b| get_block(b, &plane)) {
                            region.set_block([0, y as i32, z as i32], &base).ok();
                        }
                        if let Some(overlay) = block.overlay.as_ref().and_then(|b| get_block(b, &plane)) {
                            region.set_block([1, y as i32, z as i32], &overlay).ok();
                        }
                    }
                }

            }
            // laying down
            Plane::Flat => {
                // x=width, y=height, z=depth
                region.reshape(&[height as i32, depth, width as i32]); // yzx
                for (x, row) in self.blocks.iter().enumerate() {
                    for (z, block) in row.iter().enumerate() {
                        if let Some(base) = block.base.as_ref().and_then(|b| get_block(b, &plane)) {
                            region.set_block([x as i32, 0, z as i32], &base).ok();
                        }
                        if let Some(overlay) = block.overlay.as_ref().and_then(|b| get_block(b, &plane)) {
                            region.set_block([x as i32, 1, z as i32], &overlay).ok();
                        }
                    }
                }
            }
        }

        region.into()
    }
}
