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
    ) -> Result<(), Box<mc_schem::error::Error>> {
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
        .map_err(Box::new)
    }
}

fn get_block(id: &str, top: bool, plane: &Plane) -> Option<Block> {
    let mut block = Block::from_id(&format!("minecraft:{id}")).ok()?;

    // set axis
    let axis = match (plane, top) {
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
            .any(|row| row.iter().any(|(_, overlay)| overlay.is_some()))
    }

    /// Turn the pixel art into a schematic.
    pub fn schematic(&self, plane: Plane) -> Schematic {
        // create
        let mut region = Region::new();
        region.name = String::from("pixel_art");

        // resize
        let (width, height) = self.dimensions();
        let has_overlay = self.has_overlay();

        // populate
        match plane {
            // stood up
            Plane::Standing => {
                let depth = if has_overlay { 3 } else { 1 };
                let base_x = if has_overlay { 1 } else { 0 };

                // x=width, y=depth, z=height
                region.reshape(&[depth, height as i32, width as i32]); // yzx
                for (y, row) in self.blocks.iter().enumerate() {
                    let y = (height - 1 - y) as i32; // flip

                    for (z, (base, overlay)) in row.iter().enumerate() {
                        let z = (width - 1 - z) as i32; // flip

                        // base
                        if let Some(base) = get_block(&self.ids[base.i as usize], base.top, &plane)
                        {
                            region.set_block([base_x, y, z], &base).ok();
                        }

                        // overlay
                        if let Some(overlay) =
                            overlay.and_then(|o| get_block(&self.ids[o.i as usize], o.top, &plane))
                        {
                            region.set_block([0, y, z], &overlay).ok();
                            region.set_block([2, y, z], &overlay).ok();
                        }
                    }
                }
            }
            // laying down
            Plane::Flat => {
                let depth = if has_overlay { 2 } else { 1 };

                // x=width, y=height, z=depth
                region.reshape(&[height as i32, depth, width as i32]); // yzx
                for (x, row) in self.blocks.iter().enumerate() {
                    for (z, (base, overlay)) in row.iter().enumerate() {
                        // base
                        if let Some(base) = get_block(&self.ids[base.i as usize], base.top, &plane)
                        {
                            region.set_block([x as i32, 0, z as i32], &base).ok();
                        }

                        // overlay
                        if let Some(overlay) = overlay
                            .as_ref()
                            .and_then(|o| get_block(&self.ids[o.i as usize], o.top, &plane))
                        {
                            region.set_block([x as i32, 1, z as i32], &overlay).ok();
                        }
                    }
                }
            }
        }

        region.into()
    }
}
