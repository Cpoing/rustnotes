use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Write};
use std::process::Command;
use tempfile::NamedTempFile;

const FILE_PATH: &str = "notes.json";

#[derive(Serialize, Deserialize)]
struct Notes {
    entries: IndexMap<String, String>,
}

impl Notes {
    fn load() -> Self {
        let mut file = match File::open(FILE_PATH) {
            Ok(f) => f,
            Err(_) => {
                return Notes {
                    entries: IndexMap::new(),
                };
            }
        };

        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            serde_json::from_str(&contents).unwrap_or_else(|_| Notes {
                entries: IndexMap::new(),
            })
        } else {
            Notes {
                entries: IndexMap::new(),
            }
        }
    }

    fn save(&self) {
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(FILE_PATH)
        {
            let _ = file.write_all(serde_json::to_string_pretty(&self).unwrap().as_bytes());
        }
    }
}

fn list_notes(notes: &Notes) {
    for (key, value) in &notes.entries {
        println!("{}: {}", key, value);
    }
}

fn renumber_notes(notes: &mut Notes) {
    let values: Vec<String> = notes.entries.values().cloned().collect();
    notes.entries.clear();
    for (i, val) in values.into_iter().enumerate() {
        notes.entries.insert((i + 1).to_string(), val);
    }
}

fn edit_with_editor(initial: &str) -> Option<String> {
    let mut file = NamedTempFile::new().ok()?;
    writeln!(file, "{}", initial).ok()?;

    let path = file.path().to_str()?;

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    Command::new(editor).arg(path).status().ok()?;

    let mut new_content = String::new();
    File::open(path)
        .ok()?
        .read_to_string(&mut new_content)
        .ok()?;
    Some(new_content.trim().to_string())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut notes = Notes::load();

    let command = if let Some(cmd) = args.get(1) {
        cmd
    } else {
        eprintln!("Error: No command provided.");
        std::process::exit(1);
    };

    match command.as_str() {
        "list" | "ls" => {
            if notes.entries.is_empty() {
                println!("No notes found");
            } else {
                for (key, value) in &notes.entries {
                    println!("{}: {}", key, value);
                }
            }
        }

        "add" => {
            if let Some(val) = args.get(2) {
                let next_idx = notes.entries.len() + 1;
                notes.entries.insert(next_idx.to_string(), val.clone());
                notes.save();
                println!("Note added: {} -> {}", next_idx, val);
                list_notes(&notes);
            } else {
                eprintln!("Usage: add <value>");
            }
        }

        "delete" | "del" => {
            if let Some(key) = args.get(2) {
                if notes.entries.shift_remove(key).is_some() {
                    renumber_notes(&mut notes);
                    notes.save();
                    println!("Note deleted: {}", key);
                    list_notes(&notes);
                } else {
                    println!("Note not found: {}", key);
                }
            } else {
                eprintln!("Usage: delete <key>");
            }
        }

        "edit" | "ed" => {
            if let Some(key) = args.get(2) {
                if let Some(old_value) = notes.entries.get(key) {
                    if let Some(new_value) = edit_with_editor(old_value) {
                        notes.entries.insert(key.clone(), new_value);
                        notes.save();
                        println!("Note updated.");
                        list_notes(&notes);
                    } else {
                        eprintln!("Edit aborted or failed.");
                    }
                } else {
                    eprintln!("Note not found: {}", key);
                }
            } else {
                eprintln!("Usage: edit <key>");
            }
        }

        "swap" => {
            if let (Some(arg1), Some(arg2)) = (args.get(2), args.get(3)) {
                if notes.entries.contains_key(arg1) && notes.entries.contains_key(arg2) {
                    let val1 = notes.entries.get(arg1).cloned().unwrap();
                    let val2 = notes.entries.get(arg2).cloned().unwrap();
                    notes.entries.insert(arg1.clone(), val2);
                    notes.entries.insert(arg2.clone(), val1);
                    notes.save();
                    println!("Swapped notes {} and {}", arg1, arg2);
                    list_notes(&notes);
                } else {
                    eprintln!("One or both keys not found.");
                }
            } else {
                eprintln!("Usage: swap <key1> <key2>");
            }
        }

        "clear" | "cl" => {
            println!("Are you sure you want to delete all notes? (y/n): ");
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_ok() {
                match input.trim().to_lowercase().as_str() {
                    "y" | "yes" => {
                        notes.entries.clear();
                        notes.save();
                        println!("All notes deleted.");
                    }
                    "n" | "no" => {
                        println!("Aborted. No notes were deleted.");
                    }
                    _ => {
                        println!("Invalid input. Please enter 'y' or 'n'.");
                    }
                }
            }
        }

        _ => {
            eprintln!("Unknown command: {}", command)
        }
    }
}

// OPTIONS:
//  - LIST (no args)
//  - ADD (1 arg)
//  - DELETE (1 or more args)
//  - EDIT (1 arg)
//  - CLEAR
//
// - colored text
// - Project spaces
