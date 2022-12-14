use std::time::SystemTime;

use rusqlite::{params, Connection};

use crate::lexica::{LazyLexicaImage};
use crate::{image_data, DbConn, ProcessedImage};

pub fn create_posterity_db(connection: &Connection) {
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

    connection
        .execute(
            "CREATE INDEX IF NOT EXISTS idx_posterity_lexixa_image ON posterity(lexica_image)",
            [],
        )
        .unwrap();
}

pub fn store_image_and_prompt(
    connection: &DbConn,
    lexica_image: &LazyLexicaImage,
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
                image_data::optimized_png(&image_data::png(&lexica_image.image().unwrap())),
                now
            ],
        )
        .unwrap();
}

pub fn give_image_to_posterity(
    connection: &DbConn,
    lexica_image: &LazyLexicaImage,
    processed_image: &ProcessedImage,
) {
    let image_id = &lexica_image.id;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

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
