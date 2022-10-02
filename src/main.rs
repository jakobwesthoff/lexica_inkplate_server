use std::io::Write;

use anyhow::Context;
use rand::Rng;

fn create_fake_headers() -> anyhow::Result<curl::easy::List> {
    let mut headers: curl::easy::List = curl::easy::List::new();
    headers.append("User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:105.0) Gecko/20100101 Firefox/105.0")?;
    headers.append("Accept-Language: en-US,en;q=0.5")?;
    headers.append("DNT: 1")?;
    headers.append("Upgrade-Insecure-Requests: 1")?;
    headers.append("Sec-Fetch-Dest: document")?;
    headers.append("Sec-Fetch-Mode: navigate")?;
    headers.append("Sec-Fetch-Site: cross-site")?;
    headers.append("Pragma: no-cache")?;
    headers.append("Cache-Control: no-cache")?;
    headers.append("TE: trailers")?;
    Ok(headers)
}
fn main() -> anyhow::Result<()> {
    // Tried this with request. However then it is detected as "non browser" and
    // terminates in the cloudflare captcha.
    // With curl we do not have this problem however. No real idea why. However I don't really bother ;)
    // All the headers are simply duplicated from the request the browser makes
    // on my system.
    let mut easy = curl::easy::Easy::new();
    easy.url("https://lexica.art")?;
    let mut headers = create_fake_headers()?;
    headers.append("Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")?;
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
    let prompts = v["props"]["pageProps"]["trpcState"]["json"]["queries"][0]["state"]["data"]["pages"][0]["prompts"].as_array().unwrap();

    let mut rng = rand::thread_rng();
    let prompt_index = rng.gen_range(0..prompts.len());
    let prompt = &prompts[prompt_index];
    let images = prompt["images"].as_array().unwrap();
    let image_index = rng.gen_range(0..images.len());
    let image = &images[image_index];

    let id = &image["id"].as_str().unwrap();

    let image_url = format!("https://image.lexica.art/md/{}", id);
    println!("{}", image_url);


    let mut easy = curl::easy::Easy::new();
    easy.url(image_url.as_str())?;

    let mut headers = create_fake_headers()?;
    headers.append("Accept: image/avif,image/webp,*/*")?;
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

    let mut file = std::fs::File::create("output.jpg")?;
    file.write_all(&dst)?;
    drop(file);

    Ok(())
}
