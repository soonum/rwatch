//! # Data Generator
//!
//! `data_generator` is simple test plugin able to store and send data to
//! rwatch core instance over the network.

mod config;

use std::error::Error;
use std::io::{self, Write};
use std::io::BufRead;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;
use rand::Rng;

use serde_json::{self, Value};

pub use crate::config::config::Config;

#[derive(Debug)]
struct Record {
    pub list: Vec<Value>,
}

impl Record {
    // DEV Note: de manière idiomatique, il n'est pas judicieux de retourner un Box<dyn Error>
    // dans le code d'une lib. En effet l'appelant peut vouloir gérer les erreurs de manière particulière.
    // Il vaut mieux créer un type d'erreur (ou utiliser un type existant)
    pub fn new() -> Result<Record, Box<dyn Error>> {
        let list: Vec<Value> = Vec::new();
        Ok(Record {list})
    }
}

impl Record {
    pub fn store_data(&mut self, data: &Vec<String>) -> Result<(), Box<dyn Error>>{
        for (i, line) in data.iter().enumerate() {
            match serde_json::from_str(line.as_str()) {
                Ok(n) => self.list.push(n),
                Err(err) => {
                    eprintln!("Failed to parse line {} : {}", i + 1, err);
                    continue;
                }
            };
        }
        //println!("Record: {:#?}", self);  // DEBUG

        Ok(())
    }

    pub fn get_data(&mut self, args: Vec<String>) {
        // effectuer la récupération des différents args ici (une option et une valeur par ligne)
    }
}

pub fn read_lines() -> Vec<String> {
    let stdin = io::stdin();
    let stdin_lock = stdin.lock();
    let vec = stdin_lock
        .lines()
        .filter_map(|l| l.ok()).collect();

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

fn execute_command(name: &String, args: Vec<String>, record: &mut Record)
    -> Result<(), Box<dyn Error>> {

    name.to_lowercase();

    match name.as_str() {
        "send" => record.get_data(args),
	    "store" => record.store_data(&args)?,
	    "quit" => println!("quit entered"),
	    _ => eprintln!("'{}' is not a valid command", name)
    };

    Ok(())
}

fn generate_random_data(data_container: Arc<Mutex<Vec<String>>>, generate_interval: Duration)
    {
    let mut pending_data: Vec<String> = vec![];

    loop {
        thread::sleep(generate_interval);

        let now = SystemTime::now();
        let since_epoch = match now.duration_since(UNIX_EPOCH) {
            Ok(n) => n,
            Err(..) => {
                eprintln!("Clock may have gone backwards");
                continue;
            }
        };

        let number = rand::thread_rng().gen_range(1..101);

        let entry = format!(
            r#"{{"time": {}, "random_value": {}}}"#,
            since_epoch.as_millis(), &number);
        let entry = String::from(entry);
        println!("{}", entry);  // DEBUG
        pending_data.push(entry);

        match data_container.try_lock() {
            Ok(mut container) => {
                for item in pending_data.drain(..) {
                    container.push(item);
                }
            },
            Err(_) => continue,
        };

    }
}

fn store_random_data(
    data_container: Arc<Mutex<Vec<String>>>,
    record: Mutex<Record>,
    store_interval: Duration) {

    loop {
        thread::sleep(store_interval);

        // A context is used so as to unlock resources at the end of each iteration.
        {
            let mut container = match data_container.lock() {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Failed to store random data: {}",e);
                    break;
                }
            };

            let mut rec = match record.lock() {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Failed to store random data: {}",e);
                    break;
                }
            };

            rec.store_data(&container);
            container.clear();
        }
    }
}

fn handle_random_data(record: Record, generate_interval: Duration, store_interval: Duration) {
    let container = Arc::new(Mutex::new(vec![]));
    let record = Mutex::new(record);

    let container_ref_1 = Arc::clone(&container);
    let handle_generate = thread::spawn(move || {generate_random_data(container_ref_1, generate_interval)});

    let container_ref_2 = Arc::clone(&container);
    let handle_store = thread::spawn(move || {store_random_data(container_ref_2, record, store_interval)});

    handle_generate.join().unwrap();
    handle_store.join().unwrap();  // DEBUG
}

fn handle_user_data(record: &mut Record) -> Result<(), Box<dyn Error>> {
    loop {
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
        if let Err(..) = execute_command(name, args, record) {
            break;
        }
    }

    Ok(())

}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let mut record = Record::new().unwrap();

    if config.random == true {
        handle_random_data(
            record,
            config.random_generate_interval,
            config.random_store_interval
        );
    } else {
        handle_user_data(&mut record)?;
    }
    // Idiomatic, call run() for its side-effects and not the value
    // returned in case of success.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // TODO: Complete the test
        let name = String::from("sEnD");
    }
}
