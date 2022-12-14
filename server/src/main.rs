mod dithering;
mod image_data;

use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

use curl::easy::Easy;
use figment::providers::Env;
use figment::Figment;
use rand::Rng;

use rocket::http::{ContentType, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::{Request, State};
use rusqlite::{params, Connection};
use scraper::Html;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct AppConfig {
    storage_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PersistedConfig {
    update_at_night: bool,
    update_interval: usize,
}

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

struct LexicaImage {
    pub id: String,
    pub url: String,
    pub prompt: Value,
    pub metadata: Value,
    pub image: image::DynamicImage,
}

fn get_with_curl(easy: &mut Easy, accept: &str, url: &str) -> anyhow::Result<Vec<u8>> {
    easy.reset();
    easy.url(url)?;
    let headers = create_fake_headers(accept)?;
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

    Ok(dst)
}

fn post_with_curl(easy: &mut Easy, url: &str, post_data: &str) -> anyhow::Result<Vec<u8>> {
    easy.reset();
    easy.url(url)?;
    let mut headers = create_fake_headers(
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
    )?;
    headers.append("Content-Type: application/json")?;
    easy.http_headers(headers)?;

    easy.post_fields_copy(post_data.as_bytes())?;

    let mut dst = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            dst.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    Ok(dst)
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

fn fetch_lexica() -> anyhow::Result<LexicaImage> {
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

struct DbFile(pub String);
impl Deref for DbFile {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
struct DbConn(pub rusqlite::Connection);

impl Deref for DbConn {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn create_posterity_db(connection: &Connection) {
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS lexica_image (
        id TEXT PRIMARY KEY,
        prompt TEXT NOT NULL,
        url TEXT NOT NULL,
        raw_document TEXT NOT NULL,
        image BLOB NOT NULL,
        stored_at INTEGER
    )",
            [],
        )
        .unwrap();

    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS lexica_prompt (
        id TEXT PRIMARY KEY,
        prompt TEXT NOT NULL,
        raw_document TEXT NOT NULL,
        stored_at INTEGER
    )",
            [],
        )
        .unwrap();

    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS posterity (
        id INTEGER PRIMARY KEY,
        lexica_image TEXT NOT NULL,
        cropped_image BLOB NOT NULL,
        dithered_image BLOB NOT NULL,
        shown_at INTEGER
    )",
            [],
        )
        .unwrap();
}

fn give_image_to_posterity(
    connection: DbConn,
    lexica_image: &LexicaImage,
    processed_image: &ProcessedImage,
) {
    let image_id = &lexica_image.id;
    let prompt_id = lexica_image.prompt["id"].as_str().unwrap();
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    connection
        .execute(
            "
            INSERT OR IGNORE INTO lexica_prompt
                (id, prompt, raw_document, stored_at)
            VALUES
                (?1, ?2, ?3, ?4)
            ",
            params![
                prompt_id,
                &lexica_image.prompt["prompt"].as_str().unwrap(),
                serde_json::to_string(&lexica_image.prompt).unwrap(),
                now
            ],
        )
        .unwrap();

    connection
        .execute(
            "
            INSERT OR IGNORE INTO lexica_image
                (id, prompt, url, raw_document, image, stored_at)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6)
            ",
            params![
                image_id,
                prompt_id,
                lexica_image.url,
                serde_json::to_string(&lexica_image.metadata).unwrap(),
                image_data::optimized_png(&image_data::png(&lexica_image.image)),
                now
            ],
        )
        .unwrap();

    connection
        .execute(
            "
            INSERT INTO posterity
                (lexica_image, cropped_image, dithered_image, shown_at)
            VALUES
                (?1, ?2, ?3, ?4)
            ",
            params![
                image_id,
                image_data::optimized_png(&processed_image.cropped),
                image_data::optimized_png(&processed_image.dithered),
                now
            ],
        )
        .unwrap();
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DbConn {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<DbConn, Self::Error> {
        let db_file = request.guard::<&State<DbFile>>().await.unwrap();

        match Connection::open(db_file.as_str()) {
            Ok(connection) => {
                create_posterity_db(&connection);
                Outcome::Success(DbConn(connection))
            }
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

#[derive(Clone)]
struct ProcessedImage {
    pub cropped: Vec<u8>,
    pub dithered: Vec<u8>,
    pub rotated: Vec<u8>,
    pub inkplate: Vec<u8>,
}

fn process_lexica_image(lexica_image: &LexicaImage) -> ProcessedImage {
    let cropped = image_data::scale_and_crop_image(&lexica_image.image);
    let dithered = image_data::image_dithered(&cropped);
    let rotated = image_data::rotate_image(&dithered);
    let inkplate = image_data::inkplate_raw(&rotated);

    ProcessedImage {
        cropped: image_data::png(&cropped),
        dithered: image_data::png(&dithered),
        rotated: image_data::png(&rotated),
        inkplate,
    }
}

#[rocket::get("/lexica/png/cropped")]
async fn lexica_png_original(connection: DbConn) -> Option<(ContentType, Vec<u8>)> {
    let lexica = fetch_lexica().unwrap();
    let processed_image = process_lexica_image(&lexica);
    {
        let processed_image = processed_image.clone();
        tokio::spawn(async move {
            give_image_to_posterity(connection, &lexica, &processed_image);
        });
    }
    return Some((ContentType::PNG, processed_image.cropped));
}

#[rocket::get("/lexica/png/dithered")]
async fn lexica_png_dithered(connection: DbConn) -> Option<(ContentType, Vec<u8>)> {
    let lexica = fetch_lexica().unwrap();
    let processed_image = process_lexica_image(&lexica);
    {
        let processed_image = processed_image.clone();
        tokio::spawn(async move {
            give_image_to_posterity(connection, &lexica, &processed_image);
        });
    }
    return Some((ContentType::PNG, processed_image.dithered));
}

#[rocket::get("/lexica/inkplate")]
async fn lexica_inkplate(connection: DbConn) -> Option<Vec<u8>> {
    let lexica = fetch_lexica().unwrap();
    let processed_image = process_lexica_image(&lexica);

    {
        let processed_image = processed_image.clone();
        tokio::spawn(async move {
            give_image_to_posterity(connection, &lexica, &processed_image);
        });
    }

    return Some(processed_image.inkplate);
}

#[rocket::get("/config")]
async fn get_config(config: &State<Mutex<PersistedConfig>>) -> Json<PersistedConfig> {
    return Json(config.lock().unwrap().clone());
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    flexi_logger::Logger::try_with_str("info, oxipng=error")?.start()?;

    let figment = Figment::from(Env::prefixed("LEXICA_INKPLATE_"));
    let config: AppConfig = figment.extract()?;
    let db_file = format!("{}/posterity.sqlite", config.storage_path);
    let persistent_config = Mutex::new(PersistedConfig {
        update_at_night: false,
        update_interval: 15,
    });

    let rocket = rocket::build()
        .manage(config)
        .manage(DbFile(db_file))
        .manage(persistent_config)
        .mount(
            "/",
            rocket::routes![
                lexica_png_original,
                lexica_png_dithered,
                lexica_inkplate,
                get_config,
            ],
        )
        .launch()
        .await
        .unwrap();

    Ok(())
}
