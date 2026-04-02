use crate::Block;
use mcdata::GenericBlockState;
use mcdata::util::BlockPos;
use rustmatica::{Litematic, Region};
use std::collections::HashMap;

pub(crate) fn create(blocks: Vec<Vec<Block>>) -> Litematic {
    let height = blocks.len() as i32;
    let width = blocks.first().map_or(0, |row| row.len() as i32);
    let mut region = Region::new(
        "pixel_art",
        BlockPos::new(0, 0, 0),
        BlockPos::new(width, height, 2),
    );

    for (y, row) in blocks.iter().enumerate() {
        for (x, block) in row.iter().enumerate() {
            let x_flipped = width - 1 - x as i32;
            let y_flipped = height - 1 - y as i32;
            let base_state = GenericBlockState {
                name: format!("minecraft:{}", block.base).into(),
                properties: HashMap::new(),
            };
            region.set_block(BlockPos::new(x_flipped, y_flipped, 1), base_state);

            if let Some(overlay) = &block.overlay {
                let overlay_state = GenericBlockState {
                    name: format!("minecraft:{}", overlay).into(),
                    properties: HashMap::new(),
                };
                region.set_block(BlockPos::new(x_flipped, y_flipped, 0), overlay_state);
            }
        }
    }

    let mut schematic = Litematic::new("", "", env!("CARGO_CRATE_NAME"));
    schematic.regions.push(region);

    schematic
}
