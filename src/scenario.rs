use std::{
    fs,
    fs::File,
    io,
    io::BufRead,
    // io::BufReader,
    // io::Lines,
    // io::Read,
    path,
};

#[derive(Debug, Default)]
pub struct ScenarioMetadata {
    pub name: String,
    pub description: String,
    pub description_long: String,
    pub scenario_type: String,
}

pub fn parse_scenario_metadata<P>(
    filename: P,
) -> Result<ScenarioMetadata, String>
where
    P: AsRef<path::Path>,
{
    let mut metadata = ScenarioMetadata::default();

    if let Ok(lines) = read_lines(filename) {
        for lines in lines {
            if let Ok(line) = lines {
                if !line.starts_with("--") {
                    break;
                }

                if line.starts_with("-- ") {
                    parse_key_values(&line, &mut metadata);
                }

                if line.starts_with("--- ") {
                    metadata.description_long.push_str(&line[4..]);
                    metadata.description_long.push_str("\n");
                }

                if line == "---" {
                    metadata.description_long.push_str("\n");
                }
            }
        }
    }

    Ok(metadata)
}

fn parse_key_values(line: &str, metadata: &mut ScenarioMetadata) {
    let parts = &mut line[3..].splitn(2, ":");

    if let Some(key) = parts.next() {
        if let Some(value) = parts.next() {
            match key.to_lowercase().trim() {
                "name" => metadata.name = value.trim().to_string(),
                "description" => {
                    metadata.description = value.trim().to_string()
                }
                "type" => metadata.scenario_type = value.trim().to_string(),
                _ => {}
            }
        }
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<path::Path>,
{
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
