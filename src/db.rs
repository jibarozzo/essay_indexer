use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, Bson},
    options::ClientOptions,
    Client, Collection, Database,
};
use std::error::Error;

use crate::Essay;

pub struct DatabaseManager {
    client: Client,
    db: Database,
}

impl DatabaseManager {
    pub async fn new(connection_string: &str, db_name: &str) -> Result<Self, Box<dyn Error>> {
        let client_options = ClientOptions::parse(connection_string).await?;
        let client = Client::with_options(client_options)?;
        let db = client.database(db_name);

        Ok(Self { client, db })
    }

    pub fn get_essay_collection(&self) -> Collection<Essay> {
        self.db.collection("essays")
    }

    pub async fn add_essay(&self, essay: Essay) -> Result<String, Box<dyn Error>> {
        let collection = self.get_essay_collection();
        let result = collection.insert_one(essay, None).await?;

        Ok(result
            .inserted_id
            .as_object_id()
            .expect("Failed to get inserted ID")
            .to_hex())
    }

    pub async fn find_essays_by_section_tag(
        &self,
        section_title: &str,
        tag: &str,
    ) -> Result<Vec<Essay>, Box<dyn Error>> {
        let collection = self.get_essay_collection();
        let query = doc! {
            "sections": {
                "$elemMatch": {
                    "title": section_title,
                    "tags": tag
                }
            }
        };

        let mut cursor = collection.find(query, None).await?;
        let mut essays = Vec::new();

        while let Some(essay) = cursor.try_next().await? {
            essays.push(essay);
        }

        Ok(essays)
    }

    pub async fn find_essays_by_usefulness(
        &self,
        section_title: &str,
        min_rating: u8,
    ) -> Result<Vec<Essay>, Box<dyn Error>> {
        let collection = self.get_essay_collection();
        // Convert u8 to i32 for MongoDB compatibility
        let min_rating_i32: i32 = min_rating as i32;

        let query = doc! {
            "sections": {
                "$elemMatch": {
                    "title": section_title,
                    "usefulness_rating": { "$gte": min_rating_i32 }
                }
            }
        };

        let mut cursor = collection.find(query, None).await?;
        let mut essays = Vec::new();

        while let Some(essay) = cursor.try_next().await? {
            essays.push(essay);
        }

        Ok(essays)
    }
}
