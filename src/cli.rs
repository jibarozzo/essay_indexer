use crate::{db::DatabaseManager, Essay, Metadata, Section};
use futures_util::stream::TryStreamExt;
use std::io::{self, Write};
use std::sync::Arc;

pub struct EssayCli {
    db_manager: Arc<DatabaseManager>,
}

impl EssayCli {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Essay Indexing System");
        println!("====================");

        loop {
            println!("\nOptions:");
            println!("1. Add a new essay");
            println!("2. Search essays by section tag");
            println!("3. Search essays by section usefulness");
            println!("4. Exit");

            print!("Choose an option (1-4): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim() {
                "1" => self.add_essay().await?,
                "2" => self.search_by_tag().await?,
                "3" => self.search_by_usefulness().await?,
                "4" => break,
                _ => println!("Invalid option. Please try again."),
            }
        }

        Ok(())
    }

    async fn add_essay(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nAdding a new essay");

        print!("Essay title: ");
        io::stdout().flush()?;
        let mut title = String::new();
        io::stdin().read_line(&mut title)?;

        print!("Author: ");
        io::stdout().flush()?;
        let mut author = String::new();
        io::stdin().read_line(&mut author)?;

        let mut sections = Vec::new();
        let mut continue_adding = true;

        while continue_adding {
            print!("Section title: ");
            io::stdout().flush()?;
            let mut section_title = String::new();
            io::stdin().read_line(&mut section_title)?;

            print!("Section content: ");
            io::stdout().flush()?;
            let mut content = String::new();
            io::stdin().read_line(&mut content)?;

            print!("Tags (comma-separated): ");
            io::stdout().flush()?;
            let mut tags_input = String::new();
            io::stdin().read_line(&mut tags_input)?;
            let tags = tags_input
                .trim()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();

            print!("Usefulness rating (1-5): ");
            io::stdout().flush()?;
            let mut rating_input = String::new();
            io::stdin().read_line(&mut rating_input)?;
            let usefulness_rating = rating_input.trim().parse::<u8>().ok();

            sections.push(Section {
                title: section_title.trim().to_string(),
                content: content.trim().to_string(),
                tags,
                usefulness_rating,
            });

            print!("Add another section? (y/n): ");
            io::stdout().flush()?;
            let mut another = String::new();
            io::stdin().read_line(&mut another)?;
            continue_adding = another.trim().to_lowercase() == "y";
        }

        print!("Word count (optional): ");
        io::stdout().flush()?;
        let mut word_count_input = String::new();
        io::stdin().read_line(&mut word_count_input)?;
        let word_count = word_count_input.trim().parse::<u32>().ok();

        print!("Categories (comma-separated): ");
        io::stdout().flush()?;
        let mut categories_input = String::new();
        io::stdin().read_line(&mut categories_input)?;
        let categories = categories_input
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();

        let essay = Essay {
            id: None,
            title: title.trim().to_string(),
            author: author.trim().to_string(),
            date: Some(mongodb::bson::DateTime::now()),
            sections,
            metadata: Some(Metadata {
                word_count,
                categories,
            }),
        };

        let id = self.db_manager.add_essay(essay).await?;
        println!("Essay added successfully with ID: {}", id);

        Ok(())
    }

    async fn search_by_tag(&self) -> Result<(), Box<dyn std::error::Error>> {
        print!("Section title to search (e.g., Introduction): ");
        io::stdout().flush()?;
        let mut section_title = String::new();
        io::stdin().read_line(&mut section_title)?;

        print!("Tag to search: ");
        io::stdout().flush()?;
        let mut tag = String::new();
        io::stdin().read_line(&mut tag)?;

        let essays = self
            .db_manager
            .find_essays_by_section_tag(section_title.trim(), tag.trim())
            .await?;

        println!(
            "\nFound {} essay(s) with '{}' tag in '{}' section:",
            essays.len(),
            tag.trim(),
            section_title.trim()
        );

        for (i, essay) in essays.iter().enumerate() {
            println!("{}. {} by {}", i + 1, essay.title, essay.author);
        }

        Ok(())
    }

    async fn search_by_usefulness(&self) -> Result<(), Box<dyn std::error::Error>> {
        print!("Section title to search (e.g., Introduction): ");
        io::stdout().flush()?;
        let mut section_title = String::new();
        io::stdin().read_line(&mut section_title)?;

        print!("Minimum usefulness rating (1-5): ");
        io::stdout().flush()?;
        let mut rating_input = String::new();
        io::stdin().read_line(&mut rating_input)?;
        let rating = rating_input.trim().parse::<u8>()?;

        let essays = self
            .db_manager
            .find_essays_by_usefulness(section_title.trim(), rating)
            .await?;

        println!(
            "\nFound {} essay(s) with '{}' section rated {} or higher:",
            essays.len(),
            section_title.trim(),
            rating
        );

        for (i, essay) in essays.iter().enumerate() {
            println!("{}. {} by {}", i + 1, essay.title, essay.author);
        }

        Ok(())
    }
}
