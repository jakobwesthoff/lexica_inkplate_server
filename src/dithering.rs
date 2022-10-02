use image::{DynamicImage, ImageBuffer, Luma};

type Kernel5x5 = [[u32; 5]; 5];
#[derive(Debug, Copy, Clone)]
pub struct Dithering {
    normalization: u32,
    kernel: Kernel5x5,
}

impl Dithering {
    #[inline(always)]
    fn new(kernel: Kernel5x5) -> Self {
        let mut normalization: u32 = 0;
        for row in 0..5 {
            for col in 0..5 {
                normalization += kernel[row][col];
            }
        }

        Dithering {
            kernel,
            normalization,
        }
    }
}

#[allow(dead_code)]
#[inline(always)]
pub fn jarvis_judice_ninke() -> Dithering {
    Dithering::new([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 7, 5],
        [3, 5, 7, 5, 3],
        [1, 3, 5, 3, 1],
    ])
}

#[allow(dead_code)]
#[inline(always)]
pub fn floyd_steinberg() -> Dithering {
    Dithering::new([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 7, 0],
        [0, 3, 5, 1, 0],
        [0, 0, 0, 0, 0],
    ])
}

#[allow(dead_code)]
#[inline(always)]
pub fn atkinson() -> Dithering {
    Dithering::new([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 1, 1],
        [0, 1, 1, 1, 0],
        [0, 0, 1, 0, 0],
    ])
}

#[allow(dead_code)]
#[inline(always)]
pub fn none() -> Dithering {
    Dithering::new([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ])
}

#[inline(always)]
pub fn palette_8_grayscale() -> Vec<u32> {
    vec![
        0x000000, 0x202020, 0x404040, 0x606060, 0x808080, 0xa0a0a0, 0xc0c0c0, 0xe0e0e0,
    ]
}

#[inline(always)]
pub fn palette_7_acep() -> Vec<u32> {
    vec![
        0x000000, // black
        0xFFFFFF, // white
        0x00FF00, // green
        0x0000FF, // blue
        0xFF0000, // red
        0xFFFF00, // yellow
        0xFF8000, // orange
    ]
}

fn color_distance(color1: u32, color2: u32) -> f64 {
    let r1 = (color1 >> 16 & 0xff) as u8;
    let r2 = (color2 >> 16 & 0xff) as u8;
    let g1 = (color1 >> 8 & 0xff) as u8;
    let g2 = (color2 >> 8 & 0xff) as u8;
    let b1 = (color1 >> 0 & 0xff) as u8;
    let b2 = (color2 >> 0 & 0xff) as u8;

    let dr = r1 as i64 - r2 as i64;
    let dg = g1 as i64 - g2 as i64;
    let db = b1 as i64 - b2 as i64;

    let distance_squared = (dr * dr + dg * dg + db * db) as f64;
    distance_squared.sqrt()
}

pub fn map_color_to_palette_index(color: u32, palette: &Vec<u32>) -> usize {
    let mut current_distance = f64::INFINITY;
    let mut current_index = 0;
    for i in 0..palette.len() {
        let distance = color_distance(color, palette[i]);
        if distance < current_distance {
            current_distance = distance;
            current_index = i;
        }
    }
    current_index
}

#[inline(always)]
fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[inline(always)]
fn assert_within_range<T>(value: T, min: T, max: T)
where
    T: PartialOrd,
    T: std::fmt::Debug,
{
    if value < min || value > max {
        panic!(
            "Given value is not within range: {:?} < {:?} < {:?}",
            min, value, max
        );
    }
}

#[inline(always)]
fn is_inside_image(image: &ImageBuffer<image::Rgba<u8>, Vec<u8>>, x: i64, y: i64) -> bool {
    !(x < 0 || y < 0 || x > image.width() as i64 - 1 || y > image.height() as i64 - 1)
}

#[inline(always)]
fn kernel_by_delta(kernel: &Kernel5x5, dx: i64, dy: i64) -> u32 {
    assert_within_range(dx, -2, 2);
    assert_within_range(dy, -2, 2);

    let vx = (dx + 2) as usize;
    let vy = (dy + 2) as usize;

    kernel[vy][vx]
}

#[inline(always)]
pub fn get_pixel(image: &ImageBuffer<image::Rgba<u8>, Vec<u8>>, x: u32, y: u32) -> u32 {
    let pixel = image.get_pixel(x, y).data;
    (pixel[0] as u32) << 16 | (pixel[1] as u32) << 8 | (pixel[2] as u32) << 0
}

#[inline(always)]
fn set_pixel(image: &mut ImageBuffer<image::Rgba<u8>, Vec<u8>>, x: u32, y: u32, new_pixel: u32) {
    let mut pixel = image.get_pixel_mut(x, y);
    pixel.data[0] = (new_pixel >> 16 & 0xff) as u8;
    pixel.data[1] = (new_pixel >> 8 & 0xff) as u8;
    pixel.data[2] = (new_pixel >> 0 & 0xff) as u8;
    pixel.data[3] = 0xffu8;
}

pub fn apply_error_diffusion(
    mut image: ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    dither: Dithering,
    palette: Vec<u32>,
) -> ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    for y in 0..image.height() {
        for x in 0..image.width() {
            // // Original pixel
            // let r = img.data[idx];
            // let g = img.data[idx + 1];
            // let b = img.data[idx + 2];

            let original_pixel = get_pixel(&image, x, y);

            // // Quantized pixel
            // let nearestPaletteIndex = mapColorToPalette(bytesToColor(r, g, b), palette);
            // const [qr, qg, qb] = colorToBytes(palette[nearestPaletteIndex]);

            let nearest_palette_index = map_color_to_palette_index(original_pixel, &palette);

            // img.data[idx] = qr;
            // img.data[idx + 1] = qg;
            // img.data[idx + 2] = qb;
            // img.indexed[toIndex(img, x, y, 1)] = nearestPaletteIndex;
            let new_pixel = palette[nearest_palette_index];
            set_pixel(&mut image, x, y, new_pixel);

            // // Quantization error
            // let er = r - qr;
            // let eg = g - qg;
            // let eb = b - qb;
            let er: i16 = (original_pixel >> 16 & 0xff) as i16 - (new_pixel >> 16 & 0xff) as i16;
            let eg: i16 = (original_pixel >> 8 & 0xff) as i16 - (new_pixel >> 8 & 0xff) as i16;
            let eb: i16 = (original_pixel >> 0 & 0xff) as i16 - (new_pixel >> 0 & 0xff) as i16;

            // Apply quantization error to surrounding pixels according to diffusion kernel
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let kernel_value = kernel_by_delta(&dither.kernel, dx, dy);

                    let kx = i64::from(x) + dx;
                    let ky = i64::from(y) + dy;

                    if kernel_value != 0 && is_inside_image(&image, kx, ky) {
                        // // Original not yet diffused pixel
                        // let r = img.data[idx];
                        // let g = img.data[idx + 1];
                        // let b = img.data[idx + 2];
                        let original = get_pixel(&image, kx as u32, ky as u32);

                        // // Pixel with propagated error
                        // let dr = clampPixel(r + Math.floor((er * matrixValue) / normalization));
                        // let dg = clampPixel(g + Math.floor((eg * matrixValue) / normalization));
                        // let db = clampPixel(b + Math.floor((eb * matrixValue) / normalization));
                        let dr = clamp(
                            (original >> 16 & 0xff) as i64
                                + (er as i64 * kernel_value as i64 / dither.normalization as i64),
                            0,
                            255,
                        ) as u8;
                        let dg = clamp(
                            (original >> 8 & 0xff) as i64
                                + (eg as i64 * kernel_value as i64 / dither.normalization as i64),
                            0,
                            255,
                        ) as u8;
                        let db = clamp(
                            (original >> 0 & 0xff) as i64
                                + (eb as i64 * kernel_value as i64 / dither.normalization as i64),
                            0,
                            255,
                        ) as u8;

                        set_pixel(
                            &mut image,
                            kx as u32,
                            ky as u32,
                            (dr as u32) << 16 | (dg as u32) << 8 | (db as u32) << 0,
                        );
                    }
                }
            }
        }
    }
    image
}

// pub fn quantize_to_3bit(
//     image: &DynamicImage,
//     dithering: Dithering,
// ) -> ImageBuffer<Luma<u8>, Vec<u8>> {
//     let grayscale = image.grayscale().to_luma();
//     apply_error_diffusion(grayscale, dithering)
// }
