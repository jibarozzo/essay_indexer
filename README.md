# rateDMP

A command-line interface (CLI) application for rating and evaluating Data Management Plans (DMPs) written in Rust.

## Overview

rateDMP is a tool designed to help researchers, data managers, and institutional administrators evaluate the quality of Data Management Plans through a structured rating system. The application focuses on section-specific ratings, tags, and comments rather than storing the full content of DMPs.

## Features

- **DMP Rating Management**: Add, search, and update ratings for DMPs
- **Section-Based Evaluation**: Rate specific sections of DMPs (e.g., Data Description, Documentation & Metadata)
- **Tags and Comments**: Add tags and detailed comments to each section
- **Subsections Support**: Create hierarchical ratings with nested subsections
- **Search Capabilities**: Find DMPs by:
  - Section tags
  - Section ratings
  - Related entities (datasets, publications, software)
- **Export/Import**: Save and load DMP ratings in JSON format
- **Identifier Support**: Reference DMPs and related resources using DOIs and other persistent identifiers

## Installation

### Prerequisites

- Rust (latest stable version)
- MongoDB (running locally or accessible via network)

### Building from Source

```bash
git clone https://github.com/jibarozzo/rateDMP.git
cd rateDMP
cargo build --release
```

The compiled binary will be available at `target/release/ratedmp`.

## Usage

### Starting the Application

```bash
cargo run
```

### Main Menu Options

1. **Manage DMP Ratings**: Add new ratings or search existing ones
2. **Export DMP Ratings**: Save all ratings to a JSON file
3. **Import DMP Ratings**: Load ratings from a JSON file
4. **Exit**: Close the application

### Working with DMP Ratings

When choosing "Manage DMP Ratings" you can:

1. **Add a new DMP rating**:
   - Enter DMP metadata (title, identifiers, authors)
   - Rate individual sections (1-5 scale)
   - Add tags and comments
   - Include optional subsections

2. **Search DMPs by section tag**:
   - Find DMPs containing specific tags in their sections

3. **Search DMPs by section rating**:
   - Find DMPs with sections rated above a specified threshold

4. **Search DMPs by related entity**:
   - Locate DMPs connected to specific datasets, publications, or software
   - Search using persistent identifiers (DOIs, etc.)

5. **Update DMP section rating**:
   - Modify ratings for existing DMPs

## Data Model

The application uses a structured data model that includes:

- **Core DMP Metadata**: Title, identifiers, authors, project information
- **Sections**: Rating, tags, comments for standard DMP sections
- **Subsections**: More detailed ratings for components within sections
- **Relationships**: Links to related DMPs, datasets, publications, and software

## Contributing

Contributions to rateDMP are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Future Development

Future enhancements may include:
- Web interface for easier interaction
- Report generation capabilities
- Analytics for institutional DMP assessment
- Enhanced FAIR principles evaluation
- Integration with external repositories
