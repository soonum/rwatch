//! # Data Generator
//!
//! `data_generator` is simple test plugin able to store and send data to
//! rwatch core instance over the network.
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::io::BufRead;

use serde_json::{self, Value}; //{Result, Value};

#[derive(Debug)]
pub struct Config {
    pub port: u32,
    pub random: bool,
}

impl Config {
    pub fn new<T>(mut args: T) -> Result<Config, &'static str>
    where T: Iterator<Item = std::string::String>
    {
        args.next();  // Discard args 0 which is program name

        let parsed_args = Config::parse_option_args(args).unwrap();

        let mut port: Option<String> = None;
        let mut random: bool = false;
        // Trouver un mécanisme pour rendre des options obligatoires
        for option in parsed_args {
            match option.name.as_str() {
                "port" | "p" => port = option.value,
                "random" | "r" => random = true,
                _ => {
                    eprintln!("Unknown option name: {}", option.name);
                    return Err("Configuration parsing failed due to unknown option");
                }
            }
        }

	    Ok(Config {
            port: Config::check_port(port)?,
            random,
	    })
    }

    fn parse_option_args<T>(args: T) -> Result<Vec<OptionArg>, &'static str>
    where T: Iterator<Item = std::string::String>
    {
        let mut args_map: HashMap<String, Option<String>> = HashMap::new();

        let mut previous_item = String::new();
        let mut previous_is_name = false;

        for item in args {
            let item_is_name = item.starts_with("-");

            match item_is_name {
                false => {
                    if previous_is_name == true {
                        args_map.insert(previous_item, Some(item.clone()));
                    } else {
                        eprintln!("Option value '{}' must be preceded with an option name", &item);
                        return Err("Error while parsing option arguments");
                    }
                },
                true => {
                    args_map.entry(item.clone()).or_insert(None);
                }
            };

            previous_item = item.clone();
            previous_is_name = item_is_name;
        }

        let mut parsed_args: Vec<OptionArg> = vec![];

        for (key, value) in args_map {
            let option = OptionArg::new(key, value);
            parsed_args.push(option);
        }

        println!("PARSED {:#?}", parsed_args);  // DEBUG
        Ok(parsed_args)
    }

    fn check_port(port: Option<String>) -> Result<u32, &'static str> {
        let port: String = match port {
            Some(n) => n,
            None => return Err("No port value provided"),
        };

        let port: u32 = match port.parse() {
            Ok(n) => n,
            Err(..) => return Err("Error with port argument"),
        };

        if port < 1001 || port > 65535 {
            return Err("Port value must be between 1001 and 65535");
        };

        Ok(port)
    }
}

#[derive(Debug)]
struct OptionArg {
    pub name: String,
    pub value: Option<String>,
}

impl OptionArg {
    pub fn new(name: String, value: Option<String>) -> OptionArg {
        let name = name.trim_start_matches("-").to_string();

        OptionArg {
            name,
            value,
        }
    }
}

#[derive(Debug)]
struct Record {
    pub list: Vec<Value>,
}

impl Record {
    pub fn new() -> Result<Record, Box<dyn Error>> {
        let list: Vec<Value> = Vec::new();
        Ok(Record {list})
    }
}

impl Record {
    pub fn store_data(&mut self, args: Vec<String>) {
        for (i, line) in args.iter().enumerate() {
            match serde_json::from_str(line.as_str()) {
                Ok(n) => self.list.push(n),
                Err(err) => {
                    eprintln!("Failed to parse line {} : {}", i + 1, err);
                    continue;
                }
            };
        }
        println!("Record: {:#?}", self);  // DEBUG
    }

    pub fn get_data(&mut self, args: Vec<String>) {
        // effectuer la récupération des différents args ici (une option et une valeur pa ligne)
    }
}

pub fn read_lines() -> Vec<String> {
    let stdin = io::stdin();
    let stdin_lock = stdin.lock();
    let vec = stdin_lock.lines().filter_map(|l| l.ok()).collect();

    vec
}

fn parse_lines() -> Result<Vec<String>, &'static str> {
    println!("Enter a command:");
    print!(">>> ");
    io::stdout().flush().unwrap();

    let lines: Vec<String> = read_lines();
    if lines.len() == 0 {
        return Err("Missing command")
    }


    Ok(lines)
}

fn execute_command(name: &String, args: Vec<String>, record: &mut Record) {
    name.to_lowercase();

    match name.as_str() {
	"send" => record.get_data(args),
	"store" => record.store_data(args),
	"quit" => println!("quit entered"),
	_ => eprintln!("'{}' is not a valid command", name)
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut record = Record::new().unwrap();

    for n in 0..8 {
        let parsed_lines = parse_lines();
        let parsed_lines: Vec<String> = match parsed_lines {
            Ok(n) => n,
            Err(err) => {
                eprintln!("User input error: {}", err);
                continue;
            }
        };

        let name = &parsed_lines[0];
        let args = (&parsed_lines[1..]).to_vec();
        execute_command(name, args, &mut record);
    }
    // Idiomatic, call run() for its side-effects and not the value
    // returned in case of success.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_new_no_arguments() {
	let no_args: Vec<String> = Vec::new();

	let config = Config::new(no_args.into_iter());

	assert_eq!(config.err(), Some("Didn't get a port"));
    }

    #[test]
    fn config_new_port_argument_ok() {
	let port = 1002;
	let args: Vec<String> = vec![
	    String::from("program_name"),
	    port.to_string(),
	];

	let config = Config::new(args.into_iter()).unwrap();

	assert_eq!(config.port, port);
    }

    #[test]
    fn config_new_port_argument_has_not_numeric_value() {
	let bad_port = String::from("abc1234");
	let args: Vec<String> = vec![
	    String::from("program_name"),
	    bad_port,
	];

	let config = Config::new(args.into_iter());

	assert_eq!(config.err(), Some("Error with port argument"));
    }

    #[test]
    fn config_new_port_argument_value_is_too_high() {
	let port = String::from("123456789");
	let args: Vec<String> = vec![
	    String::from("program_name"),
	    port,
	];

	let config = Config::new(args.into_iter());

	assert_eq!(config.err(), Some("Port value must be between 1001 and 65535"));
    }

    #[test]
    fn config_new_port_argument_value_is_too_low() {
	let port = String::from("123");
	let args: Vec<String> = vec![
	    String::from("program_name"),
	    port,
	];

	let config = Config::new(args.into_iter());

	assert_eq!(config.err(), Some("Port value must be between 1001 and 65535"));
    }

    #[test]
    fn store_data_push_result_into_record() {
        let raw_line = String::from(r#"{"json_field": 1}"#);
        let parsed_line: Value = serde_json::from_str(raw_line.as_str()).unwrap();
        let data = vec![raw_line];

        let mut record = Record::new().unwrap();
        record.store_data(data);

        assert_eq!(record.list[0], parsed_line);
    }

    #[test]
    fn store_data_handles_mutliple_lines() {
        let line_1 = String::from(r#"{"json_field": 1}"#);
        let line_2 = String::from(r#"{"another_field": "foo"}"#);
        let data = vec![line_1, line_2];

        let mut record = Record::new().unwrap();
        record.store_data(data);

        assert_eq!(record.list.len(), 2);
    }

    #[test]
    fn store_data_discard_invalid_json() {
        let line = String::from(r#"{"bad_json_field , spam}"#);
        let data = vec![line];

        let mut record = Record::new().unwrap();
        record.store_data(data);

        assert_eq!(record.list.len(), 0);

    }

    #[test]
    fn execute_command_name_is_case_insensitive() {
        // Mocker la fonction "store_data" et voir si elle appelé malgré
        // la valeur ci dessous
        let name = String::from("sEnD");
    }
}
