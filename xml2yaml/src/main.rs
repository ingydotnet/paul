use anyhow::{Context, Result};
use clap::Parser;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "xml2yaml",
    about = "Convert XML to YAML",
    version,
    author
)]
struct Cli {
    /// Input XML file (use - for stdin)
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,

    /// Output YAML file (use - for stdout)
    #[arg(value_name = "OUTPUT")]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    
    // Read input
    let xml_content = match args.input {
        Some(ref path) if path.to_string_lossy() != "-" => {
            let file = File::open(path)
                .with_context(|| format!("Failed to open file: {}", path.display()))?;
            let mut reader = BufReader::new(file);
            let mut content = String::new();
            reader.read_to_string(&mut content)?;
            content
        }
        _ => {
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            content
        }
    };

    // Convert XML to YAML
    let yaml_content = convert_xml_to_yaml(&xml_content)?;

    // Write output
    match args.output {
        Some(ref path) if path.to_string_lossy() != "-" => {
            let mut file = File::create(path)
                .with_context(|| format!("Failed to create file: {}", path.display()))?;
            file.write_all(yaml_content.as_bytes())?;
        }
        _ => {
            io::stdout().write_all(yaml_content.as_bytes())?;
        }
    }

    Ok(())
}

fn convert_xml_to_yaml(xml_content: &str) -> Result<String> {
    let mut reader = Reader::from_str(xml_content);
    reader.trim_text(true);

    let mut stack = Vec::new();
    let mut current = YamlValue::Mapping(serde_yaml::Mapping::new());
    
    loop {
        match reader.read_event()? {
            Event::Start(ref e) => {
                let name = String::from_utf8_lossy(e.name().into_inner()).to_string();
                stack.push((name, current));
                current = YamlValue::Mapping(serde_yaml::Mapping::new());
            }
            Event::Text(ref e) => {
                if !e.is_empty() {
                    let text = e.unescape()?.to_string();
                    current = YamlValue::String(text);
                }
            }
            Event::End(_) => {
                if let Some((name, mut parent)) = stack.pop() {
                    if let YamlValue::Mapping(ref mut map) = parent {
                        let key = YamlValue::String(name);
                        map.insert(key, current);
                    }
                    current = parent;
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(serde_yaml::to_string(&current)?)
}
