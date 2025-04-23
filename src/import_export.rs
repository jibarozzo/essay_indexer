use crate::{db::DatabaseManager, Essay};
use futures_util::stream::TryStreamExt;
use mongodb::bson::doc;
use serde_json;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind};
use std::sync::Arc;

pub struct ImportExport {
    db_manager: Arc<DatabaseManager>,
}

impl ImportExport {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    pub async fn export_all_essays(
        &self,
        file_path: &str,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        // Get all essays from the database
        let collection = self.db_manager.get_essay_collection();
        let mut cursor = collection.find(doc! {}, None).await?;

        let mut essays = Vec::new();

        while let Some(essay) = cursor.try_next().await? {
            essays.push(essay);
        }

        // Write essays to file
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &essays)?;

        Ok(essays.len())
    }

    pub async fn import_from_file(
        &self,
        file_path: &str,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        // Read essays from file
        let file = File::open(file_path)
            .map_err(|e| Error::new(ErrorKind::NotFound, format!("Failed to open file: {}", e)))?;

        let reader = BufReader::new(file);
        let essays: Vec<Essay> = serde_json::from_reader(reader)?;

        if essays.is_empty() {
            return Ok(0);
        }

        // Insert essays into database
        let collection = self.db_manager.get_essay_collection();
        let mut count = 0;

        for essay in essays {
            // Make sure to clear any existing ID to prevent conflicts
            let mut essay_to_insert = essay;
            essay_to_insert.id = None;

            collection.insert_one(essay_to_insert, None).await?;
            count += 1;
        }

        Ok(count)
    }
}
