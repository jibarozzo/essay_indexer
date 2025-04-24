mod cli;
mod db;

use cli::DmpCli;
use db::DatabaseManager;
use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{self, Write};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    name: String,
    affiliation: Option<String>,
    identifier: Option<String>,
    #[serde(rename = "id_type")]
    id_type: Option<AuthorIdType>,
    email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthorIdType {
    Orcid,
    Isni,
    OpenId,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identifier {
    identifier: String,
    id_type: String, // doi, handle, ark, url, other
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    project_title: String,
    grant_id: Option<String>,
    funder: Option<String>,
    institution: Option<String>,
    start_date: Option<DateTime>,
    end_date: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelatedDMP {
    dmp_id: Identifier,
    relationship_type: String,
    title: String,
    relationship_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelatedDataset {
    dataset_id: Identifier,
    relationship_type: String,
    title: String,
    repository: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelatedPublication {
    publication_id: Identifier,
    relationship_type: String,
    title: String,
    authors: Vec<String>,
    journal: Option<String>,
    publication_date: Option<DateTime>,
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelatedSoftware {
    software_id: Identifier,
    name: String,
    version: Option<String>,
    relationship: String,
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SectionCrossReference {
    section_title: String,
    reference_note: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subsection {
    title: String,
    rating: Option<u8>,
    tags: Vec<String>,
    comments: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    title: String,
    rating: Option<u8>,
    tags: Vec<String>,
    comments: Option<String>,
    subsections: Vec<Subsection>,
    cross_references: Option<Vec<SectionCrossReference>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OverallRating {
    score: f32,
    reviewer: String,
    review_date: DateTime,
    comments: String,
    overall_tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineActionable {
    is_machine_actionable: bool,
    format: Option<String>,
    validation_date: Option<DateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionHistory {
    version: String,
    date: DateTime,
    reviewer: String,
    changes: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metrics {
    completeness_score: Option<u8>,
    fair_readiness_level: Option<String>,
    reusability_score: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataManagementPlan {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<mongodb::bson::oid::ObjectId>,
    title: String,
    dmp_id: Identifier,
    authors: Vec<Author>,
    project_info: Option<ProjectInfo>,
    created_date: DateTime,
    last_modified: DateTime,
    version: String,
    overall_rating: Option<OverallRating>,
    sections: Vec<Section>,
    machine_actionable: Option<MachineActionable>,
    history: Option<Vec<VersionHistory>>,
    metrics: Option<Metrics>,
    related_dmps: Option<Vec<RelatedDMP>>,
    related_datasets: Option<Vec<RelatedDataset>>,
    related_publications: Option<Vec<RelatedPublication>>,
    related_software: Option<Vec<RelatedSoftware>>,
}

impl std::str::FromStr for AuthorIdType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "orcid" => Ok(AuthorIdType::Orcid),
            "isni" => Ok(AuthorIdType::Isni),
            "openid" => Ok(AuthorIdType::OpenId),
            "other" => Ok(AuthorIdType::Other),
            _ => Err(format!("'{}' is not a valid author ID type", s)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("DMP Rating System");
    println!("================");
    println!("Connecting to MongoDB...");

    // Initialize the database manager
    match DatabaseManager::new("mongodb://localhost:27017", "dmp_rating").await {
        Ok(db_manager) => {
            println!("Connected to MongoDB successfully!");

            // Use an Arc to share the database manager across different handlers
            let db_manager = Arc::new(db_manager);

            // Main menu
            loop {
                println!("\nMain Menu:");
                println!("1. Manage DMP Ratings (Add/Search)");
                println!("2. Export DMP Ratings to JSON");
                println!("3. Import DMP Ratings from JSON");
                println!("4. Exit");

                print!("Choose an option (1-4): ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                match input.trim() {
                    "1" => {
                        // Create and run the CLI with a clone of the database manager
                        let cli = DmpCli::new(Arc::clone(&db_manager));
                        if let Err(e) = cli.run().await {
                            eprintln!("Error in DMP management: {}", e);
                        }
                    }
                    "2" => {
                        print!("Enter export file path: ");
                        io::stdout().flush()?;
                        let mut path = String::new();
                        io::stdin().read_line(&mut path)?;

                        match db_manager.export_all_dmps(path.trim()).await {
                            Ok(count) => println!("Successfully exported {} DMPs.", count),
                            Err(e) => eprintln!("Error exporting DMPs: {}", e),
                        }
                    }
                    "3" => {
                        print!("Enter import file path: ");
                        io::stdout().flush()?;
                        let mut path = String::new();
                        io::stdin().read_line(&mut path)?;

                        match db_manager.import_from_file(path.trim()).await {
                            Ok(count) => println!("Successfully imported {} DMPs.", count),
                            Err(e) => eprintln!("Error importing DMPs: {}", e),
                        }
                    }
                    "4" => {
                        println!("Exiting program...");
                        break;
                    }
                    _ => println!("Invalid option. Please try again."),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to MongoDB: {}", e);
            eprintln!("Make sure MongoDB is running on localhost:27017");
        }
    }

    Ok(())
}
