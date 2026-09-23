use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read};
use toml_edit::{DocumentMut, Item};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get a value by dot-path
    Get {
        /// The dot-separated path (e.g., package.version)
        path: String,
        /// The TOML file to read (or '-' for stdin)
        #[arg(default_value = "-")]
        file: String,
    },
    /// Set a value by dot-path
    Set {
        /// The dot-separated path (e.g., package.version)
        path: String,
        /// The value to set (always treated as string for now)
        value: String,
        /// The TOML file to read/write
        file: String,
    },
}

fn read_input(file: &str) -> Result<String> {
    if file == "-" {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .context("Failed to read from stdin")?;
        Ok(buf)
    } else {
        fs::read_to_string(file).with_context(|| format!("Failed to read file {}", file))
    }
}

fn get_item<'a>(doc: &'a DocumentMut, path: &str) -> Option<&'a Item> {
    let mut current: &Item = doc.as_item();
    for part in path.split('.') {
        if let Some(table) = current.as_table() {
            current = table.get(part)?;
        } else if let Some(table_like) = current.as_table_like() {
            current = table_like.get(part)?;
        } else {
            return None;
        }
    }
    Some(current)
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Get { path, file } => {
            let content = read_input(&file)?;
            let doc = content.parse::<DocumentMut>().context("Invalid TOML")?;
            if let Some(item) = get_item(&doc, &path) {
                if let Some(v) = item.as_value() {
                    if let Some(s) = v.as_str() {
                        println!("{}", s);
                    } else if let Some(i) = v.as_integer() {
                        println!("{}", i);
                    } else if let Some(f) = v.as_float() {
                        println!("{}", f);
                    } else if let Some(b) = v.as_bool() {
                        println!("{}", b);
                    } else {
                        println!("{}", item); // fallback for arrays/inline tables
                    }
                } else {
                    println!("{}", item); // for full tables
                }
            } else {
                std::process::exit(1);
            }
        }
        Commands::Set { path, value, file } => {
            let content = read_input(&file)?;
            let mut doc = content.parse::<DocumentMut>().context("Invalid TOML")?;

            let parts: Vec<&str> = path.split('.').collect();
            if parts.is_empty() {
                anyhow::bail!("Path cannot be empty");
            }

            let mut current = doc.as_item_mut();
            for (i, &part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    if let Some(table) = current.as_table_mut() {
                        table.insert(part, toml_edit::value(value.clone()));
                    } else if let Some(table_like) = current.as_table_like_mut() {
                        table_like.insert(part, toml_edit::value(value.clone()));
                    } else {
                        anyhow::bail!("Parent is not a table");
                    }
                } else {
                    if current.as_table_mut().is_some() || current.as_table_like_mut().is_some() {
                        if let Some(table_like) = current.as_table_like_mut() {
                            current = table_like.get_mut(part).context(
                                "Path does not exist and auto-creation is not yet implemented",
                            )?;
                        } else {
                            anyhow::bail!("Path structure error");
                        }
                    } else {
                        anyhow::bail!("Path structure error");
                    }
                }
            }

            if file == "-" {
                print!("{}", doc);
            } else {
                fs::write(&file, doc.to_string())
                    .with_context(|| format!("Failed to write to {}", file))?;
            }
        }
    }

    Ok(())
}
