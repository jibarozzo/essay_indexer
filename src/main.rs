mod cli;
mod db;
mod import_export;

use cli::EssayCli;
use db::DatabaseManager;
use import_export::ImportExport;
use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{self, Write};
use std::sync::Arc;

// Define our schema using Rust structs
#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    title: String,
    content: String,
    tags: Vec<String>,
    usefulness_rating: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    word_count: Option<u32>,
    categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Essay {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<mongodb::bson::oid::ObjectId>,
    title: String,
    author: String,
    date: Option<DateTime>,
    sections: Vec<Section>,
    metadata: Option<Metadata>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Essay Indexing System");
    println!("====================");
    println!("Connecting to MongoDB...");

    // Initialize the database manager
    match DatabaseManager::new("mongodb://localhost:27017", "essay_database").await {
        Ok(db_manager) => {
            println!("Connected to MongoDB successfully!");

            // Use an Arc to share the database manager across different handlers
            let db_manager = Arc::new(db_manager);

            // Main menu
            loop {
                println!("\nMain Menu:");
                println!("1. Manage Essays (Add/Search)");
                println!("2. Export Essays to JSON");
                println!("3. Import Essays from JSON");
                println!("4. Exit");

                print!("Choose an option (1-4): ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                match input.trim() {
                    "1" => {
                        // Create and run the CLI with a clone of the database manager
                        let cli = EssayCli::new(Arc::clone(&db_manager));
                        if let Err(e) = cli.run().await {
                            eprintln!("Error in essay management: {}", e);
                        }
                    }
                    "2" => {
                        print!("Enter export file path: ");
                        io::stdout().flush()?;
                        let mut path = String::new();
                        io::stdin().read_line(&mut path)?;

                        let export_manager = ImportExport::new(Arc::clone(&db_manager));

                        match export_manager.export_all_essays(path.trim()).await {
                            Ok(count) => println!("Successfully exported {} essays.", count),
                            Err(e) => eprintln!("Error exporting essays: {}", e),
                        }
                    }
                    "3" => {
                        print!("Enter import file path: ");
                        io::stdout().flush()?;
                        let mut path = String::new();
                        io::stdin().read_line(&mut path)?;

                        let import_manager = ImportExport::new(Arc::clone(&db_manager));

                        match import_manager.import_from_file(path.trim()).await {
                            Ok(count) => println!("Successfully imported {} essays.", count),
                            Err(e) => eprintln!("Error importing essays: {}", e),
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
