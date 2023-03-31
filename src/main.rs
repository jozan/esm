//#![allow(unused)]

use clap::{Parser, Subcommand};
use dialoguer::Confirm;
use error_chain::error_chain;
use human_bytes::human_bytes;
use std::{fs, fs::File, io::copy, process::exit};
use tempfile::Builder;

mod dirs;
use dirs::get_esm_scenarios_dir;

mod config;
use config::{create_config, get_config};

mod scenario;
use scenario::parse_scenario_metadata;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[derive(Parser)]
#[command(name = "esm")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(visible_alias = "list", about = "List installed scenarios")]
    Ls,

    #[command(
        about = "Add a scenario with identifier, from an URL or local file"
    )]
    Add { uri: String },

    #[command(visible_alias = "remove", about = "List installed scenarios")]
    Rm { identifier: String },

    #[command(about = "Open the scenario directory in browser")]
    OpenDir,

    #[command(about = "Clean installed scenarios")]
    Clean,

    #[command(about = "Configure esm", arg_required_else_help = true)]
    Config {
        #[arg(short, long, value_hint = clap::ValueHint::DirPath, help = "Path to the Empty Epsilon installation")]
        empty_epsilon_path: Option<Option<std::path::PathBuf>>,
        #[arg(short, long, help = "URL of the registry to use")]
        registry: Option<Option<url::Url>>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let esm_scenarios_dir = get_esm_scenarios_dir();

    match &cli.command {
        Commands::Ls => {
            let mut scenarios = vec![];

            let scenarios_dir = match fs::read_dir(&esm_scenarios_dir) {
                Ok(scenarios_dir) => scenarios_dir,
                Err(error) => {
                    log::error!("Could not read scenario directory: {}", error);
                    exit(1);
                }
            };

            for scenario_file in scenarios_dir {
                let scenario_file = match scenario_file {
                    Ok(file) => file,
                    Err(error) => {
                        log::error!("Could not read scenario file: {}", error);
                        exit(1);
                    }
                };

                scenarios.push(scenario_file);
            }

            if scenarios.is_empty() {
                println!("No scenarios installed.");
                exit(0);
            }

            println!("Installed Empty Epsilon scenarios:");

            for scenario in scenarios {
                let len = scenario.metadata()?.len() as f64;
                let scenario_file_name = scenario.file_name();
                let scenario_name = scenario_file_name.to_str().unwrap_or(
                    "Failed to convert file name to readable format",
                );
                let scenario_metadata =
                    parse_scenario_metadata(&scenario.path())?;

                println!("name: {}", scenario_metadata.name);
                println!("description: {}", scenario_metadata.description);
                println!(
                    "long description: {}",
                    scenario_metadata.description_long
                );

                println!(" - {} ({})", scenario_name, human_bytes(len));
            }

            Ok(())
        }
        Commands::Add { uri } => {
            println!("Installing scenario: {}", uri);

            let temp_dir = Builder::new().prefix("esm").tempdir()?;
            let target = construct_url(uri);
            let response = reqwest::get(target).await?;

            let (mut temp_dest_file, temp_dest_path) = {
                let fname = response
                    .url()
                    .path_segments()
                    .and_then(|segments| segments.last())
                    .and_then(
                        |name| if name.is_empty() { None } else { Some(name) },
                    )
                    .unwrap_or("tmp.bin");

                let fname = temp_dir.path().join(fname);
                log::info!("Will be located under: '{:?}'", fname);

                (File::create(&fname)?, fname)
            };

            let content = response.text().await?;
            copy(&mut content.as_bytes(), &mut temp_dest_file)?;

            let len = temp_dest_file.metadata()?.len() as f64;
            println!("Downloaded {}", human_bytes(len));

            let dest = esm_scenarios_dir.join(uri).with_extension("lua");
            if dest.exists() {
                if !Confirm::new()
                    .with_prompt("Scenario already exists. Overwrite?")
                    .default(true)
                    .interact()?
                {
                    exit(0);
                }
            }

            match fs::rename(temp_dest_path, &dest) {
                Ok(_) => (),
                Err(error) => {
                    log::error!("Could not move file: {}", error);
                    exit(1);
                }
            }

            println!("Installed scenario {} ({})", uri, dest.display());

            Ok(())
        }
        Commands::Rm { identifier } => {
            log::info!("Removing scenario: {}", identifier);

            let scenario_file_name =
                esm_scenarios_dir.join(identifier).with_extension("lua");
            if !scenario_file_name.exists() {
                log::error!("Scenario {} is not installed", identifier);
                exit(1);
            }

            match fs::remove_file(&scenario_file_name) {
                Ok(_) => (),
                Err(error) => {
                    log::error!("Could not remove scenario file: {}", error);
                    exit(1);
                }
            }

            println!(
                "Removed scenario: {} ({})",
                identifier,
                scenario_file_name.display()
            );

            Ok(())
        }
        Commands::OpenDir => {
            println!("TODO: Opening scenario directory in browser");
            Ok(())
        }
        Commands::Clean => {
            log::info!(
                "Cleaning scenario directory: {}",
                esm_scenarios_dir.display()
            );

            if !Confirm::new()
                .with_prompt("Are you sure you want to delete all scenarios? There is no coming back.")
                .default(false)
                .interact()?
            {
                exit(0);
            }

            match fs::remove_dir_all(&esm_scenarios_dir) {
                Ok(_) => {
                    println!(
                        "Cleaned scenario directory: {}",
                        esm_scenarios_dir.display()
                    );
                    Ok(())
                }
                Err(error) => {
                    log::error!(
                        "Could not remove scenario directory: {}",
                        error
                    );
                    exit(1);
                }
            }
        }
        Commands::Config {
            empty_epsilon_path: config_empty_epsilon_path,
            registry: config_registry,
        } => {
            println!("{:?}", config_empty_epsilon_path);
            println!("{:?}", config_registry);

            let mut config = get_config().unwrap_or_else(|error| {
                if Confirm::new()
                    .with_prompt(
                        "No configuration file found. Create a new config?",
                    )
                    .default(true)
                    .interact()
                    .unwrap()
                {
                    create_config().unwrap_or_else(|error| {
                        log::error!("Could not create config: {}", error);
                        exit(1);
                    })
                } else {
                    log::info!("Config file not found: {}", error);
                    exit(0);
                }
            });

            // Double unwrapping is required because of the way clap works
            match config_empty_epsilon_path {
                Some(Some(config_empty_epsilon_path)) => {
                    log::info!(
                        "Setting empty epsilon path to: {:?}",
                        config_empty_epsilon_path
                    );

                    let path = config_empty_epsilon_path
                            .clone()
                            .into_os_string()
                            .into_string().unwrap_or_else(|error| {
                                log::error!(
                                "Could not save empty epsilon path ({:?}) error: {:?}",
                                config_empty_epsilon_path,
                                error
                            );
                                exit(1);
                            });

                    config.empty_epsilon_path = Some(path);
                }
                Some(None) => {
                    log::info!("Reading empty_epsilon_path from config");
                    match config.empty_epsilon_path {
                        Some(empty_epsilon_path) => {
                            println!(
                                "empty_epsilon_path = \"{}\"",
                                empty_epsilon_path
                            );
                        }
                        None => {
                            println!("empty_epsilon_path is not set");
                        }
                    }
                }
                None => (),
            }

            // Double unwrapping is required because of the way clap works
            match config_registry {
                Some(Some(registry)) => {
                    log::info!("Setting registry to: {:?}", registry);
                    config.registry = Some(registry.to_string());
                }
                Some(None) => {
                    log::info!("Reading registry from config");
                    match config.registry {
                        Some(registry) => {
                            println!("registry = \"{}\"", registry);
                        }
                        None => {
                            println!("registry is not set");
                        }
                    }
                }
                None => (),
            }

            Ok(())
        }
    }
}

// function to construct the url to download the file from
fn construct_url(identifier: &String) -> String {
    return format!(
        "https://raw.githubusercontent.com/daid/EmptyEpsilon/master/scripts/{identifier}.lua"
    );
}

#[test]
fn test_verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
