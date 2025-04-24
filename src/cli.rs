use futures_util::stream::TryStreamExt;
use mongodb::bson::DateTime;
use std::io::{self, Write};
use std::sync::Arc;

use crate::{
    db::DatabaseManager, Author, AuthorIdType, DataManagementPlan, Identifier, MachineActionable,
    Metrics, OverallRating, ProjectInfo, RelatedDMP, RelatedDataset, RelatedPublication,
    RelatedSoftware, Section, SectionCrossReference, Subsection, VersionHistory,
};

pub struct DmpCli {
    db_manager: Arc<DatabaseManager>,
}

impl DmpCli {
    pub fn new(db_manager: Arc<DatabaseManager>) -> Self {
        Self { db_manager }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("DMP Rating Management");
        println!("====================");

        loop {
            println!("\nOptions:");
            println!("1. Add a new DMP rating");
            println!("2. Search DMPs by section tag");
            println!("3. Search DMPs by section rating");
            println!("4. Search DMPs by related entity");
            println!("5. Update DMP section rating");
            println!("6. Return to main menu");

            print!("Choose an option (1-6): ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim() {
                "1" => self.add_dmp().await?,
                "2" => self.search_by_tag().await?,
                "3" => self.search_by_rating().await?,
                "4" => self.search_by_related_entity().await?,
                "5" => self.update_rating().await?,
                "6" => {
                    println!("Returning to main menu...");
                    break;
                }
                _ => println!("Invalid option. Please try again."),
            }
        }

        Ok(())
    }

    async fn add_dmp(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\nAdding a new DMP rating");

        print!("DMP title: ");
        io::stdout().flush()?;
        let mut title = String::new();
        io::stdin().read_line(&mut title)?;

        print!("DMP identifier (e.g., DOI or URL): ");
        io::stdout().flush()?;
        let mut identifier = String::new();
        io::stdin().read_line(&mut identifier)?;

        print!("Identifier type (doi, handle, ark, url, other): ");
        io::stdout().flush()?;
        let mut id_type = String::new();
        io::stdin().read_line(&mut id_type)?;

        // Create the DMP ID
        let dmp_id = Identifier {
            identifier: identifier.trim().to_string(),
            id_type: id_type.trim().to_string(),
        };

        // Author information
        let mut authors = Vec::new();
        loop {
            print!("Author name (leave empty to finish adding authors): ");
            io::stdout().flush()?;
            let mut name = String::new();
            io::stdin().read_line(&mut name)?;

            if name.trim().is_empty() {
                break;
            }

            print!("Author identifier (optional): ");
            io::stdout().flush()?;
            let mut identifier = String::new();
            io::stdin().read_line(&mut identifier)?;

            print!("Author ID type (orcid, isni, openid, other): ");
            io::stdout().flush()?;
            let mut id_type = String::new();
            io::stdin().read_line(&mut id_type)?;

            print!("Author affiliation (optional): ");
            io::stdout().flush()?;
            let mut affiliation = String::new();
            io::stdin().read_line(&mut affiliation)?;

            print!("Author email (optional): ");
            io::stdout().flush()?;
            let mut email = String::new();
            io::stdin().read_line(&mut email)?;

            authors.push(Author {
                name: name.trim().to_string(),
                identifier: if identifier.trim().is_empty() {
                    None
                } else {
                    Some(identifier.trim().to_string())
                },

                id_type: if id_type.trim().is_empty() {
                    Some(AuthorIdType::Other) // Default to Other if empty
                } else {
                    match id_type.trim().parse::<AuthorIdType>() {
                        Ok(parsed_id_type) => Some(parsed_id_type),
                        Err(_) => {
                            println!("Invalid ID type, using 'other'");
                            Some(AuthorIdType::Other)
                        }
                    }
                },
                affiliation: if affiliation.trim().is_empty() {
                    None
                } else {
                    Some(affiliation.trim().to_string())
                },
                email: if email.trim().is_empty() {
                    None
                } else {
                    Some(email.trim().to_string())
                },
            });
        }

        // Project information (optional)
        print!("Include project information? (y/n): ");
        io::stdout().flush()?;
        let mut include_project = String::new();
        io::stdin().read_line(&mut include_project)?;

        let project_info = if include_project.trim().to_lowercase() == "y" {
            print!("Project title: ");
            io::stdout().flush()?;
            let mut project_title = String::new();
            io::stdin().read_line(&mut project_title)?;

            print!("Grant ID (optional): ");
            io::stdout().flush()?;
            let mut grant_id = String::new();
            io::stdin().read_line(&mut grant_id)?;

            print!("Funder (optional): ");
            io::stdout().flush()?;
            let mut funder = String::new();
            io::stdin().read_line(&mut funder)?;

            print!("Institution (optional): ");
            io::stdout().flush()?;
            let mut institution = String::new();
            io::stdin().read_line(&mut institution)?;

            Some(ProjectInfo {
                project_title: project_title.trim().to_string(),
                grant_id: if grant_id.trim().is_empty() {
                    None
                } else {
                    Some(grant_id.trim().to_string())
                },
                funder: if funder.trim().is_empty() {
                    None
                } else {
                    Some(funder.trim().to_string())
                },
                institution: if institution.trim().is_empty() {
                    None
                } else {
                    Some(institution.trim().to_string())
                },
                start_date: None,
                end_date: None,
            })
        } else {
            None
        };

        // DMP Sections
        let mut sections = Vec::new();
        let section_titles = [
            "Data Description & Collection",
            "Documentation & Metadata",
            "Ethical & Legal Compliance",
            "Storage & Backup During the Project",
            "Data Sharing & Long-Term Preservation",
            "Responsibilities & Resources",
            "FAIR Principles",
        ];

        for section_title in &section_titles {
            println!("\nRating section: {}", section_title);

            print!("Rating (1-5): ");
            io::stdout().flush()?;
            let mut rating_input = String::new();
            io::stdin().read_line(&mut rating_input)?;
            let rating = rating_input.trim().parse::<u8>().ok();

            print!("Tags (comma-separated): ");
            io::stdout().flush()?;
            let mut tags_input = String::new();
            io::stdin().read_line(&mut tags_input)?;
            let tags = tags_input
                .trim()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();

            print!("Comments: ");
            io::stdout().flush()?;
            let mut comments = String::new();
            io::stdin().read_line(&mut comments)?;

            // Subsections (optional)
            print!("\nAdd subsections for this section? (y/n): ");
            io::stdout().flush()?;
            let mut add_subsections = String::new();
            io::stdin().read_line(&mut add_subsections)?;

            let mut subsections = Vec::new();

            if add_subsections.trim().to_lowercase() == "y" {
                loop {
                    print!("Subsection title (leave empty to finish): ");
                    io::stdout().flush()?;
                    let mut subsection_title = String::new();
                    io::stdin().read_line(&mut subsection_title)?;

                    if subsection_title.trim().is_empty() {
                        break;
                    }

                    print!("Rating (1-5): ");
                    io::stdout().flush()?;
                    let mut sub_rating_input = String::new();
                    io::stdin().read_line(&mut sub_rating_input)?;
                    let sub_rating = sub_rating_input.trim().parse::<u8>().ok();

                    print!("Tags (comma-separated): ");
                    io::stdout().flush()?;
                    let mut sub_tags_input = String::new();
                    io::stdin().read_line(&mut sub_tags_input)?;
                    let sub_tags = sub_tags_input
                        .trim()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<String>>();

                    print!("Comments: ");
                    io::stdout().flush()?;
                    let mut sub_comments = String::new();
                    io::stdin().read_line(&mut sub_comments)?;

                    subsections.push(Subsection {
                        title: subsection_title.trim().to_string(),
                        rating: sub_rating,
                        tags: sub_tags,
                        comments: if sub_comments.trim().is_empty() {
                            None
                        } else {
                            Some(sub_comments.trim().to_string())
                        },
                    });
                }
            }

            sections.push(Section {
                title: section_title.to_string(),
                rating,
                tags,
                comments: if comments.trim().is_empty() {
                    None
                } else {
                    Some(comments.trim().to_string())
                },
                subsections,
                cross_references: None,
            });
        }

        // Overall rating
        print!("\nOverall DMP rating (1-5): ");
        io::stdout().flush()?;
        let mut overall_rating_input = String::new();
        io::stdin().read_line(&mut overall_rating_input)?;
        let overall_score = match overall_rating_input.trim().parse::<f32>() {
            Ok(score) if score >= 1.0 && score <= 5.0 => score,
            _ => 3.0, // Default to middle value if invalid
        };

        print!("Reviewer name: ");
        io::stdout().flush()?;
        let mut reviewer = String::new();
        io::stdin().read_line(&mut reviewer)?;

        print!("Overall comments: ");
        io::stdout().flush()?;
        let mut overall_comments = String::new();
        io::stdin().read_line(&mut overall_comments)?;

        print!("Overall tags (comma-separated): ");
        io::stdout().flush()?;
        let mut overall_tags_input = String::new();
        io::stdin().read_line(&mut overall_tags_input)?;
        let overall_tags = overall_tags_input
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();

        let overall_rating = OverallRating {
            score: overall_score,
            reviewer: reviewer.trim().to_string(),
            review_date: DateTime::now(),
            comments: overall_comments.trim().to_string(),
            overall_tags,
        };

        // Create the DMP object
        let dmp = DataManagementPlan {
            id: None,
            title: title.trim().to_string(),
            dmp_id,
            authors,
            project_info,
            created_date: DateTime::now(),
            last_modified: DateTime::now(),
            version: "1.0".to_string(),
            overall_rating: Some(overall_rating),
            sections,
            machine_actionable: None,
            history: None,
            metrics: None,
            related_dmps: None,
            related_datasets: None,
            related_publications: None,
            related_software: None,
        };

        // Add the DMP to the database
        let id = self.db_manager.add_dmp(dmp).await?;
        println!("DMP rating added successfully with ID: {}", id);

        Ok(())
    }

    async fn search_by_tag(&self) -> Result<(), Box<dyn std::error::Error>> {
        print!("Section title to search (e.g., Data Description & Collection): ");
        io::stdout().flush()?;
        let mut section_title = String::new();
        io::stdin().read_line(&mut section_title)?;

        print!("Tag to search: ");
        io::stdout().flush()?;
        let mut tag = String::new();
        io::stdin().read_line(&mut tag)?;

        let dmps = self
            .db_manager
            .find_dmps_by_section_tag(section_title.trim(), tag.trim())
            .await?;

        println!(
            "\nFound {} DMP(s) with '{}' tag in '{}' section:",
            dmps.len(),
            tag.trim(),
            section_title.trim()
        );

        for (i, dmp) in dmps.iter().enumerate() {
            println!(
                "{}. {} (ID: {}/{})",
                i + 1,
                dmp.title,
                dmp.dmp_id.id_type,
                dmp.dmp_id.identifier
            );

            if let Some(ref rating) = dmp.overall_rating {
                println!(
                    "   Overall rating: {}/5 by {}",
                    rating.score, rating.reviewer
                );
            }

            // Find the specific section
            if let Some(section) = dmp
                .sections
                .iter()
                .find(|s| s.title == section_title.trim())
            {
                if let Some(rating) = section.rating {
                    println!("   Section rating: {}/5", rating);
                }
                if let Some(ref comments) = section.comments {
                    println!("   Comments: {}", comments);
                }
            }

            println!();
        }

        Ok(())
    }

    async fn search_by_rating(&self) -> Result<(), Box<dyn std::error::Error>> {
        print!("Section title to search (e.g., Data Description & Collection): ");
        io::stdout().flush()?;
        let mut section_title = String::new();
        io::stdin().read_line(&mut section_title)?;

        print!("Minimum rating (1-5): ");
        io::stdout().flush()?;
        let mut rating_input = String::new();
        io::stdin().read_line(&mut rating_input)?;
        let rating = match rating_input.trim().parse::<u8>() {
            Ok(r) if r >= 1 && r <= 5 => r,
            Ok(_) => {
                println!("Rating must be between 1 and 5, using default of 3");
                3
            }
            Err(_) => {
                println!("Invalid rating, using default of 3");
                3
            }
        };

        let dmps = self
            .db_manager
            .find_dmps_by_rating(section_title.trim(), rating)
            .await?;

        println!(
            "\nFound {} DMP(s) with '{}' section rated {} or higher:",
            dmps.len(),
            section_title.trim(),
            rating
        );

        for (i, dmp) in dmps.iter().enumerate() {
            println!(
                "{}. {} (ID: {}/{})",
                i + 1,
                dmp.title,
                dmp.dmp_id.id_type,
                dmp.dmp_id.identifier
            );

            if let Some(ref overall_rating) = dmp.overall_rating {
                println!(
                    "   Overall rating: {}/5 by {}",
                    overall_rating.score, overall_rating.reviewer
                );
            }

            // Find the specific section
            if let Some(section) = dmp
                .sections
                .iter()
                .find(|s| s.title == section_title.trim())
            {
                if let Some(section_rating) = section.rating {
                    println!("   Section rating: {}/5", section_rating);
                }
                if let Some(ref comments) = section.comments {
                    println!("   Comments: {}", comments);
                }
            }

            println!();
        }

        Ok(())
    }

    async fn search_by_related_entity(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Search DMPs by related entity:");
        println!("1. Dataset");
        println!("2. Publication");
        println!("3. Software");
        println!("4. Related DMP");

        print!("Choose entity type (1-4): ");
        io::stdout().flush()?;
        let mut entity_type_input = String::new();
        io::stdin().read_line(&mut entity_type_input)?;

        let entity_type = match entity_type_input.trim() {
            "1" => "dataset",
            "2" => "publication",
            "3" => "software",
            "4" => "dmp",
            _ => {
                println!("Invalid option, using 'dataset' as default");
                "dataset"
            }
        };

        print!("Enter entity identifier (e.g., DOI): ");
        io::stdout().flush()?;
        let mut entity_id = String::new();
        io::stdin().read_line(&mut entity_id)?;

        let dmps = self
            .db_manager
            .find_dmps_by_related_entity(entity_type, entity_id.trim())
            .await?;

        println!(
            "\nFound {} DMP(s) related to this {} identifier:",
            dmps.len(),
            entity_type
        );

        for (i, dmp) in dmps.iter().enumerate() {
            println!(
                "{}. {} (ID: {}/{})",
                i + 1,
                dmp.title,
                dmp.dmp_id.id_type,
                dmp.dmp_id.identifier
            );

            if let Some(ref overall_rating) = dmp.overall_rating {
                println!(
                    "   Overall rating: {}/5 by {}",
                    overall_rating.score, overall_rating.reviewer
                );
                println!("   Reviewed on: {}", overall_rating.review_date);
            }

            println!();
        }

        Ok(())
    }

    async fn update_rating(&self) -> Result<(), Box<dyn std::error::Error>> {
        print!("DMP identifier (e.g., DOI or URL): ");
        io::stdout().flush()?;
        let mut identifier = String::new();
        io::stdin().read_line(&mut identifier)?;

        print!("Identifier type (doi, handle, ark, url, other): ");
        io::stdout().flush()?;
        let mut id_type = String::new();
        io::stdin().read_line(&mut id_type)?;

        // Check if the DMP exists
        let dmp = self
            .db_manager
            .find_dmp_by_id(identifier.trim(), id_type.trim())
            .await?;

        match dmp {
            Some(dmp) => {
                println!("Found DMP: {}", dmp.title);

                println!("\nAvailable sections:");
                for (i, section) in dmp.sections.iter().enumerate() {
                    println!(
                        "{}. {} (Current rating: {})",
                        i + 1,
                        section.title,
                        section
                            .rating
                            .map_or("Not rated".to_string(), |r| r.to_string())
                    );
                }

                print!("\nSelect section number to update: ");
                io::stdout().flush()?;
                let mut section_num = String::new();
                io::stdin().read_line(&mut section_num)?;

                let section_index = match section_num.trim().parse::<usize>() {
                    Ok(num) if num > 0 && num <= dmp.sections.len() => num - 1,
                    _ => {
                        println!("Invalid section number");
                        return Ok(());
                    }
                };

                let section = &dmp.sections[section_index];
                println!("Updating section: {}", section.title);

                print!("New rating (1-5): ");
                io::stdout().flush()?;
                let mut rating_input = String::new();
                io::stdin().read_line(&mut rating_input)?;
                let rating = match rating_input.trim().parse::<u8>() {
                    Ok(r) if r >= 1 && r <= 5 => r,
                    _ => {
                        println!("Rating must be between 1 and 5");
                        return Ok(());
                    }
                };

                print!("New comments: ");
                io::stdout().flush()?;
                let mut comments = String::new();
                io::stdin().read_line(&mut comments)?;

                let success = self
                    .db_manager
                    .update_dmp_rating(
                        identifier.trim(),
                        id_type.trim(),
                        &section.title,
                        rating,
                        comments.trim(),
                    )
                    .await?;

                if success {
                    println!("Section rating updated successfully");
                } else {
                    println!("Failed to update section rating");
                }
            }
            None => println!("DMP with the given identifier was not found"),
        }

        Ok(())
    }
}
