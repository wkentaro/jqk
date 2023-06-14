use atty;
use calm_io;
use clap::{CommandFactory, Parser};
use std;
use std::io::Read;

fn format_json_keys(
    json: &serde_json::Value,
    parent_keys: &Vec<String>,
    mut formatted_string: String,
    colored: bool,
) -> String {
    let color_reset: &str = if colored { "\x1b[0m" } else { "" };
    let color_bracket_and_comma: &str = if colored { "\x1b[0m\x1b[1m" } else { "" };
    let color_key: &str = if colored { "\x1b[1;34m" } else { "" };

    match json {
        serde_json::Value::Object(map) => {
            for (i, (key, value)) in map.iter().enumerate() {
                if !(parent_keys.len() == 0 && i == 0) {
                    formatted_string.push_str("\n");
                }

                let formatted_parent_keys = parent_keys.join("");
                let formatted_key = format!(".{}", key);

                formatted_string.push_str(&format!(
                    "{}{}{}{}{}",
                    color_key,
                    formatted_parent_keys,
                    formatted_key,
                    color_bracket_and_comma,
                    color_reset,
                ));

                let mut parent_keys = parent_keys.clone();
                parent_keys.push(formatted_key);
                formatted_string = format_json_keys(value, &parent_keys, formatted_string, colored);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, value) in arr.iter().enumerate() {
                let mut parent_keys = parent_keys.clone();
                parent_keys.push(format!("[{}]", i));
                formatted_string = format_json_keys(value, &parent_keys, formatted_string, colored);
            }
        }
        _ => {}
    }

    return formatted_string;
}

fn format_json(
    json: &serde_json::Value,
    parent_keys: &Vec<String>,
    mut formatted_string: String,
    is_trailing: bool,
    colored: bool,
) -> String {
    fn get_indentation_string(level: usize) -> String {
        std::iter::repeat(" ").take(level * 2).collect()
    }

    let level = parent_keys.len();

    let color_reset: &str = if colored { "\x1b[0m" } else { "" };
    let color_bracket_and_comma: &str = if colored { "\x1b[0m\x1b[1m" } else { "" };
    let color_key: &str = if colored { "\x1b[1;34m" } else { "" };

    match json {
        serde_json::Value::Object(map) => {
            formatted_string.push_str(&format!(
                "{}{}{{{}",
                if is_trailing {
                    String::from(" ")
                } else {
                    get_indentation_string(level)
                },
                color_bracket_and_comma,
                color_reset
            ));

            for (i, (key, value)) in map.iter().enumerate() {
                let formatted_parent_keys = parent_keys.join("");
                let formatted_key = format!(".{}", key);

                if i != 0 {
                    formatted_string
                        .push_str(&format!("{},{}", color_bracket_and_comma, color_reset));
                }

                formatted_string.push_str(&format!(
                    "\n{}{}{}{}{}:{}",
                    color_key,
                    get_indentation_string(level + 1),
                    formatted_parent_keys,
                    formatted_key,
                    color_bracket_and_comma,
                    color_reset,
                ));

                let mut parent_keys = parent_keys.clone();
                parent_keys.push(formatted_key);
                formatted_string =
                    format_json(value, &parent_keys, formatted_string, true, colored);
            }

            if map.len() != 0 {
                formatted_string.push_str(&format!("\n{}", get_indentation_string(level)));
            }
            formatted_string.push_str(&format!("{}}}{}", color_bracket_and_comma, color_reset));
        }
        serde_json::Value::Array(arr) => {
            formatted_string.push_str(&format!(
                "{}{}[{}",
                if is_trailing {
                    String::from(" ")
                } else {
                    get_indentation_string(level)
                },
                color_bracket_and_comma,
                color_reset
            ));

            for (i, value) in arr.iter().enumerate() {
                if i != 0 {
                    formatted_string
                        .push_str(&format!("{},{}", color_bracket_and_comma, color_reset));
                }
                formatted_string.push_str("\n");

                let mut parent_keys = parent_keys.clone();
                parent_keys.push(format!("[{}]", i));
                formatted_string =
                    format_json(value, &parent_keys, formatted_string, false, colored);
            }

            if arr.len() != 0 {
                formatted_string.push_str(&format!("\n{}", get_indentation_string(level)));
            }
            formatted_string.push_str(&format!("{}]{}", color_bracket_and_comma, color_reset));
        }
        serde_json::Value::String(str) => {
            let color_start: &str = if colored { "\x1b[32m" } else { "" };
            formatted_string.push_str(&format!(
                "{}{}\"{}\"",
                if is_trailing {
                    String::from(" ")
                } else {
                    get_indentation_string(level)
                },
                color_start,
                str
            ));
        }
        _ => {
            formatted_string.push_str(&format!(
                "{}{}",
                if is_trailing {
                    String::from(" ")
                } else {
                    get_indentation_string(level)
                },
                json
            ));
        }
    }

    return formatted_string;
}

#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short('C'), help = "Colorize output even if stdout is not a tty")]
    color: bool,
    #[arg(short('M'), help = "Monochrome (don't colorize output)")]
    monochrome: bool,
    #[arg(short, help = "List all available patterns")]
    list: bool,
    #[arg(help = "JSON file to format")]
    json_file: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut json_file_content;
    if let Some(json_file) = args.json_file {
        json_file_content = std::fs::read_to_string(json_file)?;
    } else if atty::is(atty::Stream::Stdin) {
        Args::command().print_help()?;
        return Ok(());
    } else {
        json_file_content = String::new();
        std::io::stdin().read_to_string(&mut json_file_content)?;
    }

    let json: serde_json::Value =
        serde_json::from_str(&json_file_content).expect("Failed to parse json");

    let colored: bool = !args.monochrome && (args.color || atty::is(atty::Stream::Stdout));
    let list_keys: bool = args.list;

    let parent_keys: Vec<String> = vec![];
    let output = if list_keys {
        format_json_keys(&json, &parent_keys, String::new(), colored)
    } else {
        format_json(&json, &parent_keys, String::new(), false, colored)
    };

    match calm_io::stdoutln!("{}", output) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok(())
}
