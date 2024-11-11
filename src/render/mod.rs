pub mod frame;
pub mod palette;

use crate::ppu::NesPPU;
use frame::Frame;


fn bg_pallette(ppu: &NesPPU, tile_column: usize, tile_row: usize) -> [u8; 4] {
    let attr_table_idx = tile_row / 4 * 8 + tile_column / 4;
    // dividing by 4 to get index for a 2x2 meta-tile
    // *8 to move to next byte.
    let attr_byte = ppu.vram[0x3c0 + attr_table_idx]; // note: still using hardcoded first nametable: static screen

    let pallet_idx = match (tile_column % 4 / 2, tile_row % 4 / 2) {
        // determines which quadrant the tile is in
        (0, 0) => attr_byte & 0b11,        // top left
        (1, 0) => (attr_byte >> 2) & 0b11, // top right
        (0, 1) => (attr_byte >> 4) & 0b11, // bottom left
        (1, 1) => (attr_byte >> 6) & 0b11, // bottom right
        (_, _) => panic!("Impossible!"),
    };

    let pallete_start: usize = 1 + (pallet_idx as usize) * 4;

    // The background palette table in ppu.palette_table is arranged in groups of 4 colors per palette,
    // with each group starting after an initial global background color.
    // pallet_idx as usize * 4 calculates the offset for the chosen palette,
    // and 1 + ... skips the initial global background color, 0x00

    [
        ppu.palette_table[0],
        ppu.palette_table[pallete_start],
        ppu.palette_table[pallete_start + 1],
        ppu.palette_table[pallete_start + 2],
    ]

    // The function returns an array with the colors for the tile:
    // ppu.palette_table[0] is the universal background color.
    // ppu.palette_table[pallete_start], ppu.palette_table[pallete_start + 1],
    // and ppu.palette_table[pallete_start + 2] are the actual colors for this tile’s palette.
}

fn sprite_palette(ppu: &NesPPU, pallete_idx: u8) -> [u8; 4] {
    let start = 0x11 + (pallete_idx * 4) as usize;

    // 0x11 is the starting address in ppu.palette_table for sprite palettes.
    // The first byte (at 0x10) is usually ignored for transparency purposes.
    [
        0,
        ppu.palette_table[start],
        ppu.palette_table[start + 1],
        ppu.palette_table[start + 2],
    ]

    // 0: The first value is 0, which acts as a placeholder for transparency. In NES sprites,
    // color index 0 (0x00) is usually treated as transparent, so it doesn’t correspond to any visible color.
    // ppu.palette_table[start]: The first color for the sprite.
    // ppu.palette_table[start + 1]: The second color for the sprite.
    // ppu.palette_table[start + 2]: The third color for the sprite.
}

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    let bank = ppu.ctrl.bknd_pattern_addr();

    // background
    for i in 0..0x03c0 {
        // 960 bytes of memory needed in a nametable
        // just for now, lets use the first nametable, stored at PPU VRAM
        let tile = ppu.vram[i] as u16;
        let tile_x = i % 32; // number of pixels in row of 32 x 30 grid (matching 256 x 240)
        let tile_y = i / 32; // number of columns: caps at 960 / 32 = 30
        let tile = &ppu.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];

        let palette = bg_pallette(ppu, tile_x, tile_y);

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                upper = upper >> 1;
                lower = lower >> 1;

                // pick palette for this tile
                let rgb = match value {
                    0 => palette::SYSTEM_PALLETE[ppu.palette_table[0] as usize],
                    1 => palette::SYSTEM_PALLETE[palette[1] as usize],
                    2 => palette::SYSTEM_PALLETE[palette[2] as usize],
                    3 => palette::SYSTEM_PALLETE[palette[3] as usize],
                    _ => panic!("can't be"),
                };
                frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb)
            }
        }
    }

    // Sprites
    for i in (0..ppu.oam_data.len()).step_by(4).rev() {
        // The PPU’s Object Attribute Memory (OAM) contains 64 entries, each using 4 bytes, to represent up to 64 sprites.
        //
        //Each sprite entry uses:
        // Byte 0: Y-coordinate (position of the sprite on the screen).
        // Byte 1: Tile index (which tile to use from chr_rom).
        // Byte 2: Attributes (palette selection, flipping information).
        // Byte 3: X-coordinate.
        //
        // step_by(4).rev() iterates over the sprites in reverse order, ensuring that sprites drawn later
        // (higher priority) overwrite those drawn earlier.
        
        let tile_idx = ppu.oam_data[i + 1] as u16;
        let tile_x = ppu.oam_data[i + 3] as usize;
        let tile_y = ppu.oam_data[i] as usize;

        let flip_vertical = if ppu.oam_data[i + 2] >> 7 & 1 == 1 { // if bit 7 (flip vertical flag) is set, set it
            true
        } else {
            false
        };
        let flip_horizontal = if ppu.oam_data[i + 2] >> 6 & 1 == 1 { // if bit 6 (flip horizontal flag) is set, set it
            true
        } else {
            false
        };
        let pallette_idx = ppu.oam_data[i + 2] & 0b11; // extracts bit 1 and bit 0 which give the palette index
        let sprite_palette = sprite_palette(ppu, pallette_idx);

        let bank: u16 = ppu.ctrl.sprt_pattern_addr();

        let tile =
            &ppu.chr_rom[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];
            'label: for x in (0..=7).rev() {
            // rust label: Control flow returns to this label when it is encountered next.
                let value = (1 & lower) << 1 | (1 & upper);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => continue 'label, // skip coloring the pixel. 
                    // label makes continue apply only to the labeled loop, and not the outer loops.
                    1 => palette::SYSTEM_PALLETE[sprite_palette[1] as usize],
                    2 => palette::SYSTEM_PALLETE[sprite_palette[2] as usize],
                    3 => palette::SYSTEM_PALLETE[sprite_palette[3] as usize],
                    _ => panic!("can't be"),
                };
                match (flip_horizontal, flip_vertical) {
                    // tile_x and tile_y are the tile coordinates. x and y are the pixel coords
                    // within that tile.

                    (false, false) => frame.set_pixel(tile_x + x, tile_y + y, rgb),
                    // on no flip, just set pixels normally

                    (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, rgb),
                    // tile_x + 7 - x: By subtracting x from 7, we reverse the x-coordinates:
                    // When x is 0 (leftmost pixel), it maps to tile_x + 7 (rightmost position).
                    // When x is 7 (rightmost pixel), it maps to tile_x + 0 (leftmost position).
                    // This functions as a flip!

                    (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, rgb), // see prev 

                    (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb), // see prev prev

                    
                }
            }
        }
    }
}
