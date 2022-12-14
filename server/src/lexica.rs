use curl::easy::Easy;
use serde_json::Value;

use crate::my_curl::{get_with_curl, post_with_curl};

pub struct LexicaImage {
    pub id: String,
    pub url: String,
    pub prompt: Value,
    pub metadata: Value,
    pub image: image::DynamicImage,
}


fn fetch_prompt_images(easy: &mut Easy, prompt_json: &str) -> anyhow::Result<Vec<LexicaImage>> {
    let v: serde_json::Value = serde_json::from_str(prompt_json)?;
    let prompts = &v["prompts"];

    // FIXME: Fetch all (ideally async)
    let prompt = &prompts[0];
    let images = prompt["images"].as_array().unwrap();
    let image_metadata = &images[0];
    let image_url = format!(
        "https://image.lexica.art/md/{}",
        image_metadata["id"].as_str().unwrap()
    );
    let image_data = get_with_curl(easy, "image/jpeg,*/*", &image_url)?;

    // let mut file = std::fs::File::create(format!(
    //     "v_{}_{}",
    //     &image["id"].as_str().unwrap(),
    //     "output.jpg"
    // ))?;
    // file.write_all(&image_data)?;
    // drop(file);

    let loaded_image = image::load_from_memory(&image_data)?;

    // let mut rng = rand::thread_rng();
    // let prompt_index = rng.gen_range(0..prompts.len());
    // let prompt = &prompts[prompt_index];
    // let images = prompt["images"].as_array().unwrap();
    // let image_index = rng.gen_range(0..images.len());
    // let image_metadata = &images[image_index];

    // let id = image_metadata["id"].as_str().unwrap();

    Ok(vec![LexicaImage {
        id: image_metadata["id"].as_str().unwrap().to_string(),
        url: image_url,
        prompt: prompt.to_owned(),
        metadata: image_metadata.to_owned(),
        image: loaded_image,
    }])
}

pub fn fetch_lexica() -> anyhow::Result<LexicaImage> {
    // Tried this with request. However then it is detected as "non browser" and
    // terminates in the cloudflare captcha.
    // With curl we do not have this problem however. No real idea why. However I don't really bother ;)
    // All the headers are simply duplicated from the request the browser makes
    // on my system.
    let mut easy = curl::easy::Easy::new();
    // Initialize cookie tracking
    easy.cookie_file("")?;

    // Just a base request to get the CSRF cookies ;)
    get_with_curl(
        &mut easy,
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        "https://lexica.art",
    )?;
    // That is the intersting part ;)
    let infinity_prompts_json = post_with_curl(
        &mut easy,
        "https://lexica.art/api/infinite-prompts",
        "{\"text\":\"\",\"searchMode\":\"images\",\"source\":\"search\",\"cursor\":0}",
    )?;

    Ok(
        fetch_prompt_images(&mut easy, std::str::from_utf8(&infinity_prompts_json)?)?
            .pop()
            .unwrap(),
    )
}
