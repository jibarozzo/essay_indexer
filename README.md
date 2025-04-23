# Essay Indexer API

## Overview

Essay Indexer is a Rust application that provides a database system for indexing academic essays and their sections. The system allows users to tag essay sections with descriptive keywords (like "helpful" or "unhelpful") and assign usefulness ratings, making it easier to search and retrieve specific content.

## Features

- **MongoDB Integration**: Store and retrieve essays using a flexible document-based model
- **Section-Level Tagging**: Mark individual sections of essays with keywords for improved searchability
- **Usefulness Ratings**: Assign ratings (1-5) to essay sections to indicate their quality or relevance
- **Command-Line Interface**: Easy-to-use CLI for managing essay data
- **Import/Export**: JSON import and export functionality for data backup and migration
- **REST API**: HTTP endpoints for programmatic access to the essay database

## Getting Started

### Prerequisites

- Rust (1.56.0 or newer)
- MongoDB (4.4 or newer)
- Cargo (Rust's package manager)

### Installation

1. Clone this repository
```bash
   git clone https://github.com/jibarozzo/essay-indexer.git
   cd essay-indexer
```
2. Build the project

```bash
cargo build --release
```
3. Make sure MongoDB is running
```bash
# On most systems
sudo systemctl start mongodb
```

4. Run the application

```bash
cargo run --release
```
## Database Schema
Each essay is stored as a document with the following structure:

```JSON
{
  "_id": ObjectId,
  "title": "Essay Title",
  "author": "Author Name",
  "date": ISODate,
  "sections": [
    {
      "title": "Introduction",
      "content": "The introduction content...",
      "tags": ["helpful", "concise"],
      "usefulness_rating": 5
    },
    ...
  ],
  "metadata": {
    "word_count": 2500,
    "categories": ["philosophy", "ethics"]
  }
}
```