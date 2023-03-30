use home::home_dir;
use std::{fs, path, process::exit};

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
