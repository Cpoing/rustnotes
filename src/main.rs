use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

const BASE_DIR_NAME: &str = ".my_notes";
const CURRENT_SPACES_FILE: &str = "current_space";
const SPACES_DIR: &str = "spaces";

#[derive(Serialize, Deserialize)]
struct Notes {
    entries: IndexMap<String, String>,
}

impl Notes {
    fn load() -> Self {
        let path = notes_path_for_current_spaces();
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                return serde_json::from_str(&contents).unwrap_or_else(|_| Notes {
                    entries: IndexMap::new(),
                });
            }
        }
        Notes {
            entries: IndexMap::new(),
        }
    }

    fn save(&self) {
        let path = notes_path_for_current_spaces();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
        {
            let _ = file.write_all(serde_json::to_string_pretty(&self).unwrap().as_bytes());
        }
    }
}

fn notes_path_for_current_spaces() -> PathBuf {
    let mut dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push(BASE_DIR_NAME);
    dir.push(SPACES_DIR);
    let proj = get_current_space().unwrap_or_else(|_| "default".to_string());
    dir.push(format!("{}.json", proj));
    dir
}

fn get_current_space() -> io::Result<String> {
    let mut path = dirs::home_dir().unwrap();
    path.push(BASE_DIR_NAME);
    path.push(CURRENT_SPACES_FILE);
    let mut f = File::open(path)?;
    let mut spaces = String::new();
    f.read_to_string(&mut spaces)?;
    Ok(spaces.trim().to_string())
}

fn set_current_spaces(name: &str) -> io::Result<()> {
    let mut base_dir = dirs::home_dir().unwrap();
    base_dir.push(BASE_DIR_NAME);
    std::fs::create_dir_all(&base_dir)?;

    let mut proj_file = base_dir.clone();
    proj_file.push(SPACES_DIR);
    proj_file.push(format!("{}.json", name));
    if !proj_file.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("spaces '{}' does not exist", name),
        ));
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(base_dir.join(CURRENT_SPACES_FILE))?;
    writeln!(file, "{}", name)?;
    Ok(())
}

fn list_notes(notes: &Notes) {
    println!(
        "[{}]",
        get_current_space().unwrap_or_else(|_| "default".into())
    );
    if notes.entries.is_empty() {
        println!("No notes found");
    } else {
        for (key, value) in &notes.entries {
            println!("{}: {}", key, value);
        }
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

fn list_spaces() -> io::Result<()> {
    let mut dir = dirs::home_dir().unwrap();
    dir.push(BASE_DIR_NAME);
    dir.push(SPACES_DIR);
    let entries = std::fs::read_dir(dir)?;
    println!("Available spaces:");
    for entry in entries.flatten() {
        if let Some(name) = entry.file_name().to_str() {
            if let Some(stem) = name.strip_suffix(".json") {
                println!("- {}", stem);
            }
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Error: No command provided.");
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "cd" | "spaces" => match command.as_str() {
            "cd" => {
                if let Some(proj) = args.get(2) {
                    if set_current_spaces(proj).is_ok() {
                        println!("Switched to spaces '{}'", proj);
                    } else {
                        eprintln!("Failed to switch spaces");
                        std::process::exit(1);
                    }
                } else {
                    eprintln!("Usage: cd <spaces_name>");
                }
            }
            "spaces" => match args.get(2).map(|s| s.as_str()) {
                Some("list") => {
                    if let Err(e) = list_spaces() {
                        eprintln!("Error listing spacess: {}", e);
                    }
                }
                Some("add") if args.len() > 3 => {
                    let newp = &args[3];
                    let mut path = dirs::home_dir().unwrap();
                    path.push(BASE_DIR_NAME);
                    path.push(SPACES_DIR);
                    std::fs::create_dir_all(&path).unwrap();
                    path.push(format!("{}.json", newp));
                    File::create(path).unwrap();
                    set_current_spaces(newp).unwrap();
                    println!("Created and switched to spaces '{}'", newp);
                }
                Some("rm") if args.len() > 3 => {
                    let rmproj = &args[3];
                    println!(
                        "Are you sure you want to remove space '{}' and all its notes? (y/n): ",
                        rmproj
                    );
                    let mut input = String::new();
                    if io::stdin().read_line(&mut input).is_ok() {
                        if matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
                            let mut path = dirs::home_dir().unwrap();
                            path.push(BASE_DIR_NAME);
                            path.push(SPACES_DIR);
                            path.push(format!("{}.json", rmproj));
                            if std::fs::remove_file(path).is_ok() {
                                println!("Removed spaces '{}'.", rmproj);
                                set_current_spaces("default").unwrap();
                                println!("Switched to 'default'.");
                            } else {
                                eprintln!("Failed to remove spaces '{}'.", rmproj);
                            }
                        } else {
                            println!("Aborted.");
                        }
                    }
                }
                Some("use") if args.len() > 3 => {
                    let useproj = &args[3];
                    if set_current_spaces(useproj).is_ok() {
                        println!("Using spaces '{}'.", useproj);
                    } else {
                        eprintln!("Failed to switch to spaces '{}'.", useproj);
                    }
                }
                _ => eprintln!("Usage: spaces [list|add|rm|use] <name>"),
            },
            _ => unreachable!(),
        },

        // Note commands
        "list" | "ls" => {
            let notes = Notes::load();
            list_notes(&notes);
        }

        "add" => {
            let mut notes = Notes::load();
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

        "delete" | "del" | "rm" => {
            let mut notes = Notes::load();
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
            let mut notes = Notes::load();
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
            let mut notes = Notes::load();
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
            let mut notes = Notes::load();
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

        cmd => {
            eprintln!("Unknown command: {}", cmd);
            std::process::exit(1);
        }
    }
}
