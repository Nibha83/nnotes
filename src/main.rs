use clap::Parser;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Result;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter, ReloadPolicy, doc};
use uuid::Uuid;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct Note {
    id: String,
    title: String,
    content: String,
}

#[derive(Parser, Debug, Serialize, Deserialize)]
struct NoteInput {
    title: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NoteFound {
    id: Vec<String>,
    title: Vec<String>,
    content: Vec<String>,
}

#[derive(Parser, Debug, Serialize, Deserialize)]
struct Search {
    query: String,
}


fn get_paths() -> (PathBuf, PathBuf) {
    let base = dirs::data_dir().unwrap().join("nnotes");
    std::fs::create_dir_all(&base).unwrap();

    let path = base.join("notes.json");
    let index_path = base.join("index_tantivy");

    (path, index_path)
}
fn main() -> Result<()> {
    let (path, index_path) = get_paths();
    let path = path.to_str().unwrap();
    let args: Vec<String> = env::args().collect();
    let num_of_args = args.len() - 1;

    if num_of_args != 1 && num_of_args != 2 {
        println!("Invalid number of arguments");
        return Ok(());
    }
    let (index, schema) = create_or_open_index(&index_path).unwrap();

    if num_of_args == 1 && args[1] == "-l" { //List all notes
        match read_notes(path) {
            Ok(notes) => {
                if notes.is_empty() {
                    println!("No notes found");
                } else {
                    for note in notes {
                        println!("Note Id: {}", note.id);
                        println!("Title: {}", note.title);
                        println!("Content: {}", note.content);
                        println!("-----------------------");
                    }
                }
            }
            Err(e) => eprintln!("Error reading notes: {}", e),
        }
        return Ok(());
    } else if num_of_args == 1 {
        let search = Search::parse();

        match search_index(&index, &search.query, &schema) {
            Ok(results) => {
                if results.is_empty() {
                    println!("No notes found");
                    return Ok(());
                }
                for result in results {
                    let note: NoteFound = serde_json::from_str(&result)?;
                    println!("Note Id: {}", note.id.join(", "));
                    println!("Title: {}", note.title.join(", "));
                    println!("Content: {}", note.content.join(", "));
                    println!("-----------------------");
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
        return Ok(());
    } else if num_of_args == 2 && args[1] == "-d" { //Delete note by id
        let note_id = &args[2];
        match delete_note(&index, path, note_id) {
            Ok(_) => println!("Note {} deleted successfully!", note_id),
            Err(e) => eprintln!("Error deleting note: {}", e),
        }
        return Ok(());
    } else {
        let note_input = NoteInput::parse();

        let id = Uuid::new_v4();
        let note = Note {
            id: id.to_string(),
            title: note_input.title,
            content: note_input.content,
        };
        match add_note_to_json(path, &note) {
            Ok(_) => (),
            Err(e) => eprintln!("Error: {}", e),
        }
        match add_to_index(&index, &note) {
            Ok(_) => println!("Note saved successfully!"),
            Err(e) => eprintln!("Error: {}", e),
        }
        return Ok(());
    }
}

fn create_or_open_index(index_path: &std::path::Path) -> tantivy::Result<(Index, Schema)> {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("id", STRING | STORED);
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("content", TEXT | STORED);
    let schema = schema_builder.build();

    let index = if index_path.exists() {
        Index::open_in_dir(index_path)?
    } else {
        std::fs::create_dir_all(index_path)?;
        Index::create_in_dir(index_path, schema.clone())?
    };

    Ok((index, schema))
}

fn read_notes(path: &str) -> Result<Vec<Note>> {
    let json = fs::read_to_string(path)?;
    let notes: Vec<Note> = serde_json::from_str(&json)?;
    Ok(notes)
}

fn add_note_to_json(path: &str, note: &Note) -> Result<()> {
    let mut notes = read_notes(path).unwrap_or_else(|_| Vec::new());

    notes.push(Note {
        id: note.id.clone(),
        title: note.title.clone(),
        content: note.content.clone(),
    });

    let json = serde_json::to_string(&notes)?;
    fs::write(path, json)?;
    Ok(())
}

fn add_to_index(index: &Index, note: &Note) -> tantivy::Result<()> {
    let schema = index.schema();
    let id = schema.get_field("id").unwrap();
    let title = schema.get_field("title").unwrap();
    let content = schema.get_field("content").unwrap();
    let mut index_writer: IndexWriter = index.writer(50_000_000)?;
    index_writer.add_document(doc!(
        id => note.id.to_string(),
        title => note.title,
        content => note.content
    ))?;
    index_writer.commit()?;
    Ok(())
}

fn search_index(index: &Index, query_str: &str, schema: &Schema) -> tantivy::Result<Vec<String>> {
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()?;
    let searcher = reader.searcher();
    let title = schema.get_field("title").unwrap();
    let content = schema.get_field("content").unwrap();
    let query_parser = QueryParser::for_index(&index, vec![title, content]);
    let query = query_parser.parse_query(query_str)?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    let mut results = Vec::new();
    for (_score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
        let json = retrieved_doc.to_json(&schema);
        results.push(json);
    }

    Ok(results)
}

fn delete_note(index: &Index, path: &str, note_id: &str) -> tantivy::Result<()> {
    // --- delete from JSON ---
    let mut notes = read_notes(path).unwrap_or_else(|_| Vec::new());
    let original_len = notes.len();
    notes.retain(|n| n.id != note_id);

    if notes.len() == original_len {
        return Err(tantivy::error::TantivyError::InvalidArgument(
            "Note with given id not found".to_string(),
        ));
    }

    let json = serde_json::to_string(&notes)?;
    fs::write(path, json)?;

    // --- delete from index ---
    let schema = index.schema();
    let id_field = schema.get_field("id").unwrap();
    let mut index_writer: IndexWriter = index.writer(50_000_000)?;
    index_writer.delete_term(tantivy::Term::from_field_text(id_field, note_id));
    index_writer.commit()?;

    Ok(())
}
