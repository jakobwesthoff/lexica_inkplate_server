use image::{DynamicImage, GenericImage};
use jpegxl_rs::encode::{EncoderResult, EncoderSpeed};
use jpegxl_rs::encoder_builder;
use std::time::Instant;

use crate::dithering;

fn get_cover_dimensions(
    width: u32,
    height: u32,
    target_width: u32,
    target_height: u32,
) -> (u32, u32) {
    let aspect_ratio: f64 = width as f64 / height as f64;
    let target_aspect_ratio = target_width as f64 / target_height as f64;

    if aspect_ratio < target_aspect_ratio {
        // scale to width and cut height
        let new_width = target_width;
        let new_height = (new_width as f64 / aspect_ratio).round() as u32;
        return (new_width, new_height);
    } else {
        let new_height = target_height;
        let new_width = (new_height as f64 * aspect_ratio).round() as u32;
        return (new_width, new_height);
    }
}

pub fn scale_and_crop_image(image: &image::DynamicImage) -> image::DynamicImage {
    let (width, height) = image.dimensions();
    let target_width = 448u32;
    let target_height = 600u32;

    let (new_width, new_height) = get_cover_dimensions(width, height, target_width, target_height);

    let mut resized = image::imageops::resize(
        image,
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3,
    );
    let analyzer = smartcrop::Analyzer::new(smartcrop::CropSettings::default());
    let crop = analyzer
        .find_best_crop(
            &resized,
            std::num::NonZeroU32::new(target_width).unwrap(),
            std::num::NonZeroU32::new(target_height).unwrap(),
        )
        .unwrap()
        .crop;

    // println!("crop: {:?}", crop);

    let cropped = image::imageops::crop(
        &mut resized,
        crop.x,
        crop.y,
        crop.width.clamp(0, target_width),
        crop.height.clamp(0, target_height),
    )
    .to_image();
    // cropped.save(format!("v_{}_{}", id, "resized_cropped.png"))?;
    // let cropped = image::open("output_resized_cropped.png")?.to_rgba();

    // let dithered = apply_error_diffusion(cropped.clone(), floyd_steinberg(), palette_8_grayscale());
    // dithered.save(format!("v_{}_{}", id, "dithered_grayscale.png"))?;

    // let dithered = apply_error_diffusion(cropped.clone(), jarvis_judice_ninke(), palette_7_acep());
    // dithered.save(format!("v_{}_{}", id, "dithered_acep.png"))?;
    // let carved = seamcarving::resize(&resized, target_width, target_height);
    // carved.save("output_carved.png")?;

    image::DynamicImage::ImageRgba8(cropped)
}

pub fn rotate_image(image: &image::DynamicImage) -> image::DynamicImage {
    let rotated = image::imageops::rotate90(image);
    image::DynamicImage::ImageRgba8(rotated)
}

pub fn png(image: &DynamicImage) -> Vec<u8> {
    let mut out_bytes: Vec<u8> = Vec::new();
    image
        .write_to(&mut out_bytes, image::ImageOutputFormat::PNG)
        .unwrap();

    out_bytes
}

pub fn jpegxl(image: &DynamicImage) -> Vec<u8> {
    let start = Instant::now();

    let raw_image = image.to_rgb().into_raw();
    let mut encoder = encoder_builder()
        .lossless(true)
        .speed(EncoderSpeed::Falcon)
        .build()
        .unwrap();

    let encodedu8: EncoderResult<u8> = encoder
        .encode(&raw_image, image.width(), image.height())
        .unwrap();

    let end = Instant::now();
    println!("jpegxl encoding took {:?}", end - start);

    encodedu8.data
}

pub fn jpegxl_from_data(image_data: &Vec<u8>) -> Vec<u8> {
    let image = image::load_from_memory(&image_data).unwrap();
    jpegxl(&image)
}

pub fn image_dithered(image: &DynamicImage) -> DynamicImage {
    let dithered = dithering::apply_error_diffusion(
        image.to_rgba().clone(),
        dithering::jarvis_judice_ninke(),
        dithering::palette_7_acep(),
    );

    DynamicImage::ImageRgba8(dithered)
}

#[inline(always)]
fn is_odd(value: u32) -> bool {
    value & 0x1 == 0x1
}

// Input must be dithered
pub fn inkplate_raw(dithered_image: &DynamicImage) -> Vec<u8> {
    let dithered = dithered_image.as_rgba8().unwrap();

    let (width, height) = dithered.dimensions();
    // println!("dithered dimensions: {}x{}", width, height);

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
