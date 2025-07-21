use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

const FILE_PATH: &str = "notes.json";

#[derive(Serialize, Deserialize)]
struct Notes {
    entries: HashMap<String, String>,
}

impl Notes {
    fn load() -> Self {
        let mut file = match File::open(FILE_PATH) {
            Ok(f) => f,
            Err(_) => {
                return Notes {
                    entries: HashMap::new(),
                };
            }
        };

        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            serde_json::from_str(&contents).unwrap_or_else(|_| Notes {
                entries: HashMap::new(),
            })
        } else {
            Notes {
                entries: HashMap::new(),
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
            if let (Some(key), Some(value)) = (args.get(2), args.get(3)) {
                notes.entries.insert(key.clone(), value.clone());
                notes.save();
                println!("Note added: {} -> {}", key, value);
            } else {
                eprintln!("Usage: ADD <key> <value>");
            }
        }

        _ => {
            eprintln!("Unknown command: {}", command)
        }
    }
}

// Make it so that the list reorders itself on item delete

// have a map contain all the notes
// {
//  1 : Do this
//  2 : Do that
// }
//
// OPTIONS:
//  - LIST (no args)
//  - ADD (1 arg)
//  - DELETE (1 or more args)
//  - EDIT (1 arg)
//  - CLEAR (no args)
