
pub mod config {
    use std::collections::HashMap;
    use std::time::Duration;

    #[derive(Debug)]
    pub struct Config {
        // TODO: add documentation
        pub port: u32,
        pub random: bool,
        pub random_generate_interval: Duration,
        pub random_store_interval: Duration,
    }

    impl Config {
        pub fn new<T>(mut args: T) -> Result<Config, &'static str>
        where T: Iterator<Item = std::string::String>
        {
            args.next();  // Discard args 0 which is program name

            let parsed_args = Config::parse_option_args(args).unwrap();

            let mut port: Option<String> = None;
            let mut random: bool = false;
            let mut random_generate_interval: Option<String> = Some(String::from("500"));
            let mut random_store_interval: Option<String> = Some(String::from("30"));

            for option in parsed_args {
                match option.name.as_str() {
                    "port" | "p" => port = option.value,
                    "random" | "r" => random = true,
                    "generate-interval" => random_generate_interval = option.value,
                    "store-interval" => random_store_interval = option.value,
                    _ => {
                        eprintln!("Unknown option name: {}", option.name);
                        return Err("Configuration parsing failed due to unknown option");
                    }
                }
            }

    	    Ok(Config {
                port: Config::check_port(port)?,
                random,
                random_generate_interval: Config::check_duration(
                    random_generate_interval, &Duration::from_millis)?,
                random_store_interval: Config::check_duration(
                    random_store_interval, &Duration::from_secs)?,
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

            Ok(parsed_args)
        }

        fn check_port(port: Option<String>) -> Result<u32, &'static str> {
            let port: String = match port {
                Some(n) => n,
                None => return Err("No port value provided"),
            };

            let port: u32 = match port.parse() {
                Ok(n) => n,
                Err(_) => return Err("Error with port argument"),
            };

            if port < 1001 || port > 65535 {
                return Err("Port value must be between 1001 and 65535");
            };

            Ok(port)
        }

        fn check_duration<T>(duration: Option<String>, func: T) -> Result<Duration, &'static str>
            where
                T: Fn(u64) -> Duration {

            let duration: String = match duration {
                Some(n) => n,
                None => return Err("No duration value provided"),
            };

            let parsed_duration = duration.parse::<u64>();
            match  parsed_duration {
                Ok(n) => Ok(func(n)),
                Err(_) => {
                    eprintln!("Error while parsing duration: {}", duration);
                    Err("Error with duration option")
                }
            }
        }
    }

    // TODO: create a custom Error type ConfigError instead of returning a `static str`

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
}
