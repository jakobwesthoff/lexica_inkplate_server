use image::DynamicImage;

use crate::dithering;

pub fn png(image: &DynamicImage) -> Vec<u8> {
    let mut out_bytes: Vec<u8> = Vec::new();
    image
        .write_to(&mut out_bytes, image::ImageOutputFormat::PNG)
        .unwrap();

    out_bytes
}

pub fn png_dithered(image: &DynamicImage) -> Vec<u8> {
    let mut out_bytes: Vec<u8> = Vec::new();

    let dithered = dithering::apply_error_diffusion(
        image.to_rgba().clone(),
        dithering::jarvis_judice_ninke(),
        dithering::palette_7_acep(),
    );

    DynamicImage::ImageRgba8(dithered)
        .write_to(&mut out_bytes, image::ImageOutputFormat::PNG)
        .unwrap();

    out_bytes
}

#[inline(always)]
fn is_odd(value: u32) -> bool {
    value & 0x1 == 0x1
}

pub fn inkplate_raw(image: &DynamicImage) -> Vec<u8> {
    let dithered = dithering::apply_error_diffusion(
        image.to_rgba().clone(),
        dithering::jarvis_judice_ninke(),
        dithering::palette_7_acep(),
    );
    let (width, height) = dithered.dimensions();
    println!("dithered dimensions: {}x{}", width, height);

    // Minimize possible reallocations
    let mut out_bytes: Vec<u8> = Vec::with_capacity((width * height / 2) as usize);

    // We are encoding 2 3bit pixels per bit one in the upper byte, one in the
    // lower. Furthermore we are always starting a new byte at the beginning of
    // each row. Therefore the last one might need padding if the width is odd
    let odd_width = is_odd(width);

    for y in 0..height {
        let mut current_byte: u8 = 0x0;
        for x in 0..width {
            let color_pixel = dithering::get_pixel(&dithered, x, y);
            // We are mapping to this palette:
            // NAME            INDEX      COLOR
            // ---             ---        ---
            // INKPLATE_BLACK  0b00000000 0x000000
            // INKPLATE_WHITE  0b00000001 0xffffff
            // INKPLATE_GREEN  0b00000010 0x00ff00
            // INKPLATE_BLUE   0b00000011 0x0000ff
            // INKPLATE_RED    0b00000100 0xff0000
            // INKPLATE_YELLOW 0b00000101 0xffff00
            // INKPLATE_ORANGE 0b00000110 0xff8000
            let indexed_pixel = match color_pixel {
                0x000000 => 0u8,
                0xffffff => 1u8,
                0x00ff00 => 2u8,
                0x0000ff => 3u8,
                0xff0000 => 4u8,
                0xffff00 => 5u8,
                0xff8000 => 6u8,
                _ => panic!(
                    "Could not match dithered color {:x} to inkplate index",
                    color_pixel
                ),
            };

            if is_odd(x) {
                // First of two pixels (high nible)
                current_byte = ((indexed_pixel << 1) << 4) & 0xf0
            } else {
                // Second of two pixels (low nible)
                current_byte = current_byte | (indexed_pixel << 1);

                // Write finished byte
                out_bytes.push(current_byte);
            }

            if odd_width && x == width - 1 {
                // Write out last byte with padding before switching lines.
                out_bytes.push(current_byte);
            }
        }
    }

    out_bytes
}
