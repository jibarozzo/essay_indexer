use futures_util::stream::TryStreamExt;
use mongodb::{
    bson::{doc, Bson, Document},
    options::ClientOptions,
    Client, Collection, Database,
};
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error as IoError, ErrorKind};

use crate::DataManagementPlan;

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

    pub fn get_dmp_collection(&self) -> Collection<DataManagementPlan> {
        self.db.collection("dmps")
    }

    pub async fn add_dmp(&self, dmp: DataManagementPlan) -> Result<String, Box<dyn Error>> {
        let collection = self.get_dmp_collection();
        let result = collection.insert_one(dmp, None).await?;

        Ok(result
            .inserted_id
            .as_object_id()
            .expect("Failed to get inserted ID")
            .to_hex())
    }

    pub async fn find_dmp_by_id(
        &self,
        dmp_id: &str,
        id_type: &str,
    ) -> Result<Option<DataManagementPlan>, Box<dyn Error>> {
        let collection = self.get_dmp_collection();
        let filter = doc! {
            "dmp_id.identifier": dmp_id,
            "dmp_id.id_type": id_type
        };

        let result = collection.find_one(filter, None).await?;
        Ok(result)
    }

    pub async fn find_dmps_by_section_tag(
        &self,
        section_title: &str,
        tag: &str,
    ) -> Result<Vec<DataManagementPlan>, Box<dyn Error>> {
        let collection = self.get_dmp_collection();
        let query = doc! {
            "sections": {
                "$elemMatch": {
                    "title": section_title,
                    "tags": tag
                }
            }
        };

        let mut cursor = collection.find(query, None).await?;
        let mut dmps = Vec::new();

        while let Some(dmp) = cursor.try_next().await? {
            dmps.push(dmp);
        }

        Ok(dmps)
    }

    pub async fn find_dmps_by_rating(
        &self,
        section_title: &str,
        min_rating: u8,
    ) -> Result<Vec<DataManagementPlan>, Box<dyn Error>> {
        let collection = self.get_dmp_collection();
        // Convert u8 to i32 for MongoDB compatibility
        let min_rating_i32: i32 = min_rating as i32;

        let query = doc! {
            "sections": {
                "$elemMatch": {
                    "title": section_title,
                    "rating": { "$gte": min_rating_i32 }
                }
            }
        };

        let mut cursor = collection.find(query, None).await?;
        let mut dmps = Vec::new();

        while let Some(dmp) = cursor.try_next().await? {
            dmps.push(dmp);
        }

        Ok(dmps)
    }

    pub async fn find_dmps_by_related_entity(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<DataManagementPlan>, Box<dyn Error>> {
        let collection = self.get_dmp_collection();
        let field_name = match entity_type {
            "dataset" => "related_datasets.dataset_id.identifier",
            "publication" => "related_publications.publication_id.identifier",
            "software" => "related_software.software_id.identifier",
            "dmp" => "related_dmps.dmp_id.identifier",
            _ => {
                return Err(Box::new(IoError::new(
                    ErrorKind::InvalidInput,
                    "Invalid entity type",
                )))
            }
        };

        let query = doc! {
            field_name: entity_id
        };

        let mut cursor = collection.find(query, None).await?;
        let mut dmps = Vec::new();

        while let Some(dmp) = cursor.try_next().await? {
            dmps.push(dmp);
        }

        Ok(dmps)
    }

    pub async fn export_all_dmps(&self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let collection = self.get_dmp_collection();
        let mut cursor = collection.find(doc! {}, None).await?;

        let mut dmps = Vec::new();

        while let Some(dmp) = cursor.try_next().await? {
            dmps.push(dmp);
        }

        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &dmps)?;

        Ok(dmps.len())
    }

    pub async fn import_from_file(&self, file_path: &str) -> Result<usize, Box<dyn Error>> {
        let file = File::open(file_path).map_err(|e| {
            IoError::new(ErrorKind::NotFound, format!("Failed to open file: {}", e))
        })?;

        let reader = BufReader::new(file);
        let dmps: Vec<DataManagementPlan> = serde_json::from_reader(reader)?;

        if dmps.is_empty() {
            return Ok(0);
        }

        let collection = self.get_dmp_collection();
        let mut count = 0;

        for dmp in dmps {
            let mut dmp_to_insert = dmp;
            dmp_to_insert.id = None;

            collection.insert_one(dmp_to_insert, None).await?;
            count += 1;
        }

        Ok(count)
    }

    pub async fn update_dmp_rating(
        &self,
        dmp_id: &str,
        id_type: &str,
        section_title: &str,
        rating: u8,
        comments: &str,
    ) -> Result<bool, Box<dyn Error>> {
        let collection = self.get_dmp_collection();
        let filter = doc! {
            "dmp_id.identifier": dmp_id,
            "dmp_id.id_type": id_type,
            "sections.title": section_title
        };

        let update = doc! {
            "$set": {
                "sections.$.rating": rating as i32,
                "sections.$.comments": comments,
                "last_modified": mongodb::bson::DateTime::now()
            }
        };

        let result = collection.update_one(filter, update, None).await?;
        Ok(result.modified_count > 0)
    }
}
