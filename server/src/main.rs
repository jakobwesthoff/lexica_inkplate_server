mod dithering;
mod image_data;

use image::GenericImage;
use rand::Rng;

use rocket::http::ContentType;

fn create_fake_headers(accept: &str) -> anyhow::Result<curl::easy::List> {
    let mut headers: curl::easy::List = curl::easy::List::new();
    headers.append("User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:105.0) Gecko/20100101 Firefox/105.0")?;
    headers.append(format!("Accept: {}", accept).as_str())?;
    headers.append("Accept-Language: en-US,en;q=0.5")?;
    headers.append("Accept-Encoding: identity")?;
    headers.append("DNT: 1")?;
    headers.append("Connection: keep-alive")?;
    headers.append("Upgrade-Insecure-Requests: 1")?;
    headers.append("Sec-Fetch-Dest: document")?;
    headers.append("Sec-Fetch-Mode: navigate")?;
    headers.append("Sec-Fetch-Site: cross-site")?;
    headers.append("Pragma: no-cache")?;
    headers.append("Cache-Control: no-cache")?;
    headers.append("TE: trailers")?;
    Ok(headers)
}
fn fetch_lexica() -> anyhow::Result<image::DynamicImage> {
    // Tried this with request. However then it is detected as "non browser" and
    // terminates in the cloudflare captcha.
    // With curl we do not have this problem however. No real idea why. However I don't really bother ;)
    // All the headers are simply duplicated from the request the browser makes
    // on my system.
    let mut easy = curl::easy::Easy::new();
    easy.url("https://lexica.art")?;
    let headers = create_fake_headers("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")?;
    easy.http_headers(headers)?;

    let mut dst = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            dst.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }
    // println!("{}", std::str::from_utf8(&dst)?);

    let document = scraper::Html::parse_document(std::str::from_utf8(&dst)?);
    let selector = match scraper::Selector::parse("script#__NEXT_DATA__[type=\"application/json\"]")
    {
        Ok(selector) => Ok(selector),
        Err(err) => Err(anyhow::anyhow!("Error parsing selector: {:?}", err)),
    }?;

    let script = document.select(&selector).next().unwrap();

    let v: serde_json::Value = serde_json::from_str(script.inner_html().as_str())?;
    let prompts = v["props"]["pageProps"]["trpcState"]["json"]["queries"][0]["state"]["data"]
        ["pages"][0]["prompts"]
        .as_array()
        .unwrap();

    let mut rng = rand::thread_rng();
    let prompt_index = rng.gen_range(0..prompts.len());
    let prompt = &prompts[prompt_index];
    let images = prompt["images"].as_array().unwrap();
    let image_index = rng.gen_range(0..images.len());
    let image = &images[image_index];

    let id = image["id"].as_str().unwrap();

    let image_url = format!("https://image.lexica.art/md/{}", id);
    println!("{}", image_url);

    let mut easy = curl::easy::Easy::new();
    easy.url(image_url.as_str())?;

    let headers = create_fake_headers("image/jpeg,*/*")?;
    easy.http_headers(headers)?;

    let mut dst = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            dst.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    // let mut file = std::fs::File::create(format!("v_{}_{}", id, "output.jpg"))?;
    // file.write_all(&dst)?;
    // drop(file);

    let image = image::load_from_memory(&dst)?;
    let (width, height) = image.dimensions();
    let target_width = 448u32;
    let target_height = 600u32;

    let (new_width, new_height) = get_cover_dimensions(width, height, target_width, target_height);

    let mut resized = image::imageops::resize(
        &image,
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
    let rotated = image::imageops::rotate90(&cropped);

    Ok(image::DynamicImage::ImageRgba8(rotated))
}

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

#[rocket::get("/lexica/png/original")]
async fn lexica_png_original() -> Option<(ContentType, Vec<u8>)> {
    return Some((ContentType::PNG, image_data::png(&fetch_lexica().unwrap())));
}

#[rocket::get("/lexica/png/dithered")]
async fn lexica_png_dithered() -> Option<(ContentType, Vec<u8>)> {
    return Some((
        ContentType::PNG,
        image_data::png_dithered(&fetch_lexica().unwrap()),
    ));
}

#[rocket::get("/lexica/inkplate")]
async fn lexica_inkplate() -> Option<Vec<u8>> {
    return Some(image_data::inkplate_raw(&fetch_lexica().unwrap()));
}
#[tokio::main]
async fn main() {
    rocket::build()
        .mount(
            "/",
            rocket::routes![lexica_png_original, lexica_png_dithered, lexica_inkplate],
        )
        .launch()
        .await
        .unwrap();
}