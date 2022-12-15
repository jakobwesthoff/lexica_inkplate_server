use std::time::SystemTime;

use rusqlite::{params, Connection};
use rusqlite_migration::{Migrations, M};

use crate::lexica::LazyLexicaImage;
use crate::{image_data, DbConn, ProcessedImage};

pub fn create_posterity_db(connection: &mut Connection) {
    let migrations = Migrations::new(vec![
        M::up(
            "CREATE TABLE IF NOT EXISTS lexica_image (
        id TEXT PRIMARY KEY,
        prompt TEXT NOT NULL,
        url TEXT NOT NULL,
        raw_document TEXT NOT NULL,
        image BLOB NOT NULL,
        stored_at INTEGER
    )",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS lexica_prompt (
        id TEXT PRIMARY KEY,
        prompt TEXT NOT NULL,
        raw_document TEXT NOT NULL,
        stored_at INTEGER
    )",
        ),
        M::up(
            "CREATE TABLE IF NOT EXISTS posterity (
        id INTEGER PRIMARY KEY,
        lexica_image TEXT NOT NULL,
        cropped_image BLOB NOT NULL,
        dithered_image BLOB NOT NULL,
        shown_at INTEGER
    )",
        ),
        M::up("CREATE INDEX IF NOT EXISTS idx_posterity_lexixa_image ON posterity(lexica_image)"),
        M::up("ALTER TABLE lexica_image ADD image_type TEXT NOT NULL DEFAULT \"png\""),
        M::up("ALTER TABLE posterity ADD image_type TEXT NOT NULL DEFAULT \"png\""),
    ]);

    migrations.to_latest(connection).unwrap();
}

pub fn store_image_and_prompt(connection: &DbConn, lexica_image: &LazyLexicaImage) {
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

    let image_data_jpegxl = image_data::jpegxl(&lexica_image.image().unwrap());

    connection
        .execute(
            "
            INSERT OR IGNORE INTO lexica_image
                (id, prompt, url, raw_document, image, image_type, stored_at)
            VALUES
                (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                image_id,
                prompt_id,
                lexica_image.url,
                serde_json::to_string(&lexica_image.metadata).unwrap(),
                "jxl",
                image_data_jpegxl,
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
                (lexica_image, cropped_image, dithered_image, image_type, shown_at)
            VALUES
                (?1, ?2, ?3, ?4, ?5)
            ",
            params![
                image_id,
                image_data::jpegxl_from_data(&processed_image.cropped),
                image_data::jpegxl_from_data(&processed_image.dithered),
                "jxl",
                now
            ],
        )
        .unwrap();
}
