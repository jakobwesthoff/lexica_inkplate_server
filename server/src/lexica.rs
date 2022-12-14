use std::cell::RefCell;
use std::sync::Arc;

use curl::easy::Easy;
use serde_json::Value;

use crate::my_curl::{get_with_curl, post_with_curl};

fn fetch_image(easy: &mut Easy, url: &str) -> anyhow::Result<image::DynamicImage> {
    println!("Fetching image: {}", url);
    let image_data = get_with_curl(easy, "image/jpeg,*/*", url)?;
    Ok(image::load_from_memory(&image_data)?)
}

fn fetch_prompt_images(easy: &mut Easy, prompt_json: &str) -> anyhow::Result<Vec<LazyLexicaImage>> {
    // let start = Instant::now();
    let v: serde_json::Value = serde_json::from_str(prompt_json)?;
    let prompts = &v["prompts"];

    let mut lazy_images = Vec::new();
    for prompt in prompts.as_array().unwrap() {
        for image in prompt["images"].as_array().unwrap() {
            let image_url = format!(
                "https://image.lexica.art/md/{}",
                image["id"].as_str().unwrap()
            );
            lazy_images.push(LazyLexicaImage::new(
                image["id"].as_str().unwrap().to_string(),
                image_url,
                prompt.to_owned(),
                image.to_owned(),
            ));
        }

        // let mut rng = rand::thread_rng();
        // let prompt_index = rng.gen_range(0..prompts.len());
        // let prompt = &prompts[prompt_index];
        // let images = prompt["images"].as_array().unwrap();
        // let image_index = rng.gen_range(0..images.len());
        // let image_metadata = &images[image_index];
    }
    // let end = Instant::now();
    // println!("fetch_prompt_images run duration: {:?}", end - start);

    Ok(lazy_images)
}

pub fn fetch_lexica() -> anyhow::Result<Vec<LazyLexicaImage>> {
    // let start = Instant::now();

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

    // let end = Instant::now();
    // println!("fetch_lexica run duration: {:?}", end - start);

    let result = fetch_prompt_images(&mut easy, std::str::from_utf8(&infinity_prompts_json)?)?;
    Ok(result)
}

pub struct LazyLexicaImage {
    pub id: String,
    pub url: String,
    pub prompt: Value,
    pub metadata: Value,
    image: RefCell<Option<Arc<image::DynamicImage>>>,
}

impl LazyLexicaImage {
    pub fn new(id: String, url: String, prompt: Value, metadata: Value) -> Self {
        Self {
            id,
            url,
            prompt,
            metadata,
            image: RefCell::new(None),
        }
    }

    pub fn image(&self) -> anyhow::Result<Arc<image::DynamicImage>> {
        let mut mut_image = self.image.borrow_mut();
        match &*mut_image {
            Some(image) => Ok(Arc::clone(image)),
            None => {
                // let start = Instant::now();
                // dbg!(&self.url);
                let mut easy = curl::easy::Easy::new();
                let fetched_image = Arc::new(fetch_image(&mut easy, &self.url)?);
                *mut_image = Some(Arc::clone(&fetched_image));

                // let end = Instant::now();
                // println!("fetch lazy image duration: {:?}", end - start);

                Ok(Arc::clone(&fetched_image))
            }
        }
    }
}
