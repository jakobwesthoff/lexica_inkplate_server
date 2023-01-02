mod dithering;
mod image_data;
mod lexica;
mod my_curl;
mod posterity;

use std::ops::Deref;
use std::sync::Mutex;

use figment::providers::Env;
use figment::Figment;

use lexica::{fetch_lexica, LazyLexicaImage};
use posterity::{create_posterity_db, give_image_to_posterity, store_image_and_prompt};
use rand::Rng;
use rocket::http::{ContentType, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::Json;
use rocket::{Request, State};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct AppConfig {
    storage_path: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct PersistedConfig {
    update_at_night: bool,
    update_interval: usize,
}

struct DbFile(pub String);
impl Deref for DbFile {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
pub struct DbConn(pub rusqlite::Connection);

impl Deref for DbConn {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DbConn {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<DbConn, Self::Error> {
        let db_file = request.guard::<&State<DbFile>>().await.unwrap();

        match Connection::open(db_file.as_str()) {
            Ok(mut connection) => {
                create_posterity_db(&mut connection);
                Outcome::Success(DbConn(connection))
            }
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

#[derive(Clone)]
pub struct ProcessedImage {
    pub cropped: Vec<u8>,
    pub dithered: Vec<u8>,
    pub rotated: Vec<u8>,
    pub inkplate: Vec<u8>,
}

fn process_lazy_lexica_image(lexica_image: &LazyLexicaImage) -> ProcessedImage {
    let cropped = image_data::scale_and_crop_image(&lexica_image.image().unwrap());
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
    let mut rng = rand::thread_rng();
    let image_index = rng.gen_range(0..lexica.len());
    let processed_image = process_lazy_lexica_image(&lexica[image_index]);
    {
        let processed_image = processed_image.clone();
        tokio::spawn(async move {
            store_image_and_prompt(&connection, &lexica[0]);
            give_image_to_posterity(&connection, &lexica[0], &processed_image);
            // Fetch other listed images for later ;)
            // TODO: Implement loading from those images, as well as using it in
            // all retrieval functions.
            // SELECT COUNT(l.id) from lexica_image as l WHERE NOT EXISTS (SELECT p.id from posterity p WHERE p.lexica_image = l.id);
            // for image in &lexica[1..11] {
            //     store_image_and_prompt(&connection, image);
            // }
        });
    }
    return Some((ContentType::PNG, processed_image.cropped));
}

#[rocket::get("/lexica/png/dithered")]
async fn lexica_png_dithered(connection: DbConn) -> Option<(ContentType, Vec<u8>)> {
    let lexica = fetch_lexica().unwrap();
    let mut rng = rand::thread_rng();
    let image_index = rng.gen_range(0..lexica.len());
    let processed_image = process_lazy_lexica_image(&lexica[image_index]);
    {
        let processed_image = processed_image.clone();
        tokio::spawn(async move {
            store_image_and_prompt(&connection, &lexica[0]);
            give_image_to_posterity(&connection, &lexica[0], &processed_image);
        });
    }
    return Some((ContentType::PNG, processed_image.dithered));
}

#[rocket::get("/lexica/inkplate")]
async fn lexica_inkplate(connection: DbConn) -> Option<Vec<u8>> {
    let lexica = fetch_lexica().unwrap();
    let mut rng = rand::thread_rng();
    let image_index = rng.gen_range(0..lexica.len());
    let processed_image = process_lazy_lexica_image(&lexica[image_index]);

    {
        let processed_image = processed_image.clone();
        tokio::spawn(async move {
            store_image_and_prompt(&connection, &lexica[0]);
            give_image_to_posterity(&connection, &lexica[0], &processed_image);
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

    let _rocket = rocket::build()
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
