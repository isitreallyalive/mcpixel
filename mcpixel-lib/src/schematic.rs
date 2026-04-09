use crate::PixelArt;
#[cfg(feature = "clap")]
use clap::ValueEnum;
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
#[derive(Default)]
#[cfg_attr(feature = "clap", derive(Clone, Debug, ValueEnum))]
pub enum Orientation {
    /// Standing, like a wall
    #[default]
    Vertical,
    /// Laid flat on the ground
    Horizontal,
}

/// All supported schematic formats.
#[derive(Default)]
#[cfg_attr(feature = "clap", derive(Clone, Debug, ValueEnum))]
pub enum SchematicFormat {
    /// Vanilla structure (.nbt)
    Vanilla,
    /// Litematica schematic (.litematic)
    #[default]
    Litematica,
    /// WorldEdit schematic (1.13+, .schem)
    WorldEdit,
}

impl SchematicFormat {
    pub const fn extension(&self) -> &'static str {
        match self {
            SchematicFormat::Vanilla => "nbt",
            SchematicFormat::Litematica => "litematic",
            SchematicFormat::WorldEdit => "schem",
        }
    }
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
            SchematicFormat::WorldEdit => self
                .0
                .save_world_edit_13_writer(dest, &WorldEdit13SaveOption::default()),
        }
        .map_err(Box::new)
    }
}

fn get_block(id: &str, top: bool, plane: &Orientation) -> Option<Block> {
    let mut block = Block::from_id(&format!("minecraft:{id}")).ok()?;

    // set axis
    let axis = match (plane, top) {
        (Orientation::Vertical, false) => 'y',
        (Orientation::Vertical, true) => 'x',
        (Orientation::Horizontal, false) => 'x',
        (Orientation::Horizontal, true) => 'y',
    };
    block.attributes.insert("axis".into(), axis.into());

    Some(block)
}

impl PixelArt {
    /// Does the pixel art have an overlay?
    fn has_overlay(&self) -> bool {
        self.blocks.iter().any(|row| {
            row.iter()
                .any(|cell| matches!(cell, Some((_, overlay)) if overlay.is_some()))
        })
    }

    /// Turn the pixel art into a schematic.
    pub fn schematic(&self, plane: Orientation) -> Schematic {
        // create
        let mut region = Region::new();
        region.name = String::from("pixel_art");

        // resize
        let (width, height) = self.dimensions();
        let has_overlay = self.has_overlay();

        // populate
        match plane {
            // stood up
            Orientation::Vertical => {
                let depth = if has_overlay { 3 } else { 1 };
                let base_x = if has_overlay { 1 } else { 0 };

                // x=width, y=depth, z=height
                region.reshape(&[depth, height as i32, width as i32]); // yzx
                for (y, row) in self.blocks.iter().enumerate() {
                    let y = (height - 1 - y) as i32; // flip

                    for (z, cell) in row.iter().enumerate() {
                        let z = (width - 1 - z) as i32; // flip

                        let Some((base, overlay)) = cell else {
                            continue;
                        };

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
            Orientation::Horizontal => {
                let depth = if has_overlay { 2 } else { 1 };

                // x=width, y=height, z=depth
                region.reshape(&[height as i32, depth, width as i32]); // yzx
                for (x, row) in self.blocks.iter().enumerate() {
                    for (z, cell) in row.iter().enumerate() {
                        let Some((base, overlay)) = cell else {
                            continue;
                        };

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
