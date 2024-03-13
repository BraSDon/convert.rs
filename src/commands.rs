use std::{num::ParseFloatError, str::FromStr};

use crate::units::{Value, Unit};

use regex::Regex;

pub enum Command {
    CONVERT(Value, Unit),
    UNITS,
    HELP,
    EXIT,
}

impl Command {
    pub fn execute(&self) -> String {
        // String builder for the output as I might have clui and web output
        let mut output = String::new();

        match self {
            Command::CONVERT(value, to_unit) => {
                let result = value.convert_to(to_unit);
                match result {
                    Ok(v) => output.push_str(&format!("{}", v)),
                    Err(e) => output.push_str(&format!("{}", e)),
                }
            },
            Command::UNITS => {
                output.push_str("Available units:\n");
                let units = Unit::get_all_units();
                for unit in units {
                    output.push_str(&format!("{}\n", unit));
                }
            },
            Command::HELP => output.push_str("HEEEEELP!"),
            _ => {}
        };

        output
    }
}

impl Command {
    fn try_parse_conversion(s: &str) -> Result<Command, String> {
        // define regex pattern (<value> <unit> -> <unit>)
        let pattern = r"(\d+(?:\.\d+)?)\s(.+)\s->\s(.+)";
        let re = Regex::new(pattern).unwrap();

        match re.captures(s) {
            Some(caps) => {
                let value: f64 = caps[1].parse().map_err(|e: ParseFloatError| e.to_string())?;
                let from_unit = caps[2].parse()?;
                let to_unit = caps[3].parse()?;

                let v = Value::new(value, from_unit);
                Ok(Command::CONVERT(v, to_unit))
            },
            None => Err(format!("Invalid input. Expression should be in the form <value> <unit> -> <unit>."))
        }
    }
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // try to parse a conversion command seperate from the other commands
        let conversion_result = Command::try_parse_conversion(s);

        match s {
            "units" => Ok(Command::UNITS),
            "help" => Ok(Command::HELP),
            "exit" => Ok(Command::EXIT),
            _ => conversion_result,
        }
    }
}
