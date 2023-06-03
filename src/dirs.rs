use home::home_dir;
use std::{fs, path, process::exit};
use walkdir::WalkDir;

pub fn get_esm_root_dir() -> path::PathBuf {
    let esm_root_dir = match home_dir() {
        Some(path) => path.join(".ee-scenario-manager"),
        None => {
            log::error!("Could not find or access user's home directory");
            exit(1);
        }
    };

    if !esm_root_dir.exists() {
        match fs::create_dir_all(&esm_root_dir) {
            Ok(_) => {
                log::info!(
                    "Created root directory: {}",
                    esm_root_dir.display()
                );
            }
            Err(error) => {
                log::error!("Could not create root directory: {}", error);
                exit(1);
            }
        }
    }

    return esm_root_dir;
}

pub fn get_esm_scenarios_dir() -> path::PathBuf {
    let esm_scenarios_dir = get_esm_root_dir().join("scenarios");

    if !esm_scenarios_dir.exists() {
        match fs::create_dir_all(&esm_scenarios_dir) {
            Ok(_) => {
                log::info!(
                    "Created scenarios directory: {}",
                    esm_scenarios_dir.display()
                );
            }
            Err(error) => {
                log::error!("Could not create scenarios directory: {}", error);
                exit(1);
            }
        }
    }

    return esm_scenarios_dir;
}

pub fn is_in_empty_epsilon_dir() -> bool {
    println!("Current dir: {:?}\n", std::env::current_dir());

    let pwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(error) => {
            log::error!("Could not get current directory: {}", error);
            exit(1);
        }
    };

    let walker = WalkDir::new("/home/johan/esm-tests/win/EmptyEpsilon")
        .max_depth(2)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok());

    for entry in walker {
        println!("{:?}", entry);
    }

    println!("-------\n\n\n");

    return true;
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".lua"))
        .unwrap_or(false)
}

fn has_scripts_dir(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with("ee.lua") || s.ends_with("luax.lua"))
        .unwrap_or(false)
}
