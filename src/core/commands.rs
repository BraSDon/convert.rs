use regex::Regex;
use std::{num::ParseFloatError, str::FromStr};

use crate::core::units::{Unit, Value};

/// Command enum to represent the different commands the user can input.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Convert a value to another unit.
    Convert(Value, Unit),
    /// List all available units.
    Units,
    /// Show help.
    Help,
    /// Exit the program.
    Exit,
}

impl Command {
    /// Execute the command and return the output as a string.
    /// String output is chosen to support different UIs.
    pub fn execute(&self) -> String {
        let mut output = String::new();

        match self {
            Command::Convert(value, to_unit) => {
                let result = value.convert_to(to_unit);
                match result {
                    Ok(v) => output.push_str(&v.to_string()),
                    Err(e) => output.push_str(&e.to_string()),
                }
            }
            Command::Units => {
                output.push_str("Available units:\n");
                let units = Unit::get_all_units();
                for unit in units {
                    output.push_str(&format!("{}\n", unit));
                }
            }
            Command::Help => output.push_str("HEEEEELP!"),
            _ => {}
        };

        output
    }
}

impl Command {
    /// Try parsing a conversion command from a string.
    fn try_parse_conversion(s: &str) -> Result<Command, String> {
        // define regex pattern (<value> <unit> -> <unit>)
        let pattern = r"(\d+(?:\.\d+)?)\s(.+)\s->\s(.+)";
        let re = Regex::new(pattern).unwrap();

        match re.captures(s) {
            Some(caps) => {
                let value: f64 = caps[1]
                    .parse()
                    .map_err(|e: ParseFloatError| e.to_string())?;
                let from_unit = caps[2].parse()?;
                let to_unit = caps[3].parse()?;

                let v = Value::new(value, from_unit);
                Ok(Command::Convert(v, to_unit))
            }
            None => Err(
                "Invalid input. Expression should be in the form <value> <unit> -> <unit>."
                    .to_string(),
            ),
        }
    }
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // try to parse a conversion command seperate from the other commands
        let conversion_result = Command::try_parse_conversion(s);

        match s {
            "units" => Ok(Command::Units),
            "help" => Ok(Command::Help),
            "exit" => Ok(Command::Exit),
            _ => conversion_result,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::units::LengthUnit;

    use super::*;

    #[test]
    fn test_command_from_str() {
        let command = "100 m -> km".parse::<Command>();
        assert!(command.is_ok());
        assert_eq!(
            command.unwrap(),
            Command::Convert(
                Value::new(100.0, Unit::Length(LengthUnit::Meter)),
                Unit::Length(LengthUnit::Kilometer)
            )
        );

        let command = "units".parse::<Command>();
        assert!(command.is_ok());
        assert_eq!(command.unwrap(), Command::Units);

        let command = "help".parse::<Command>();
        assert!(command.is_ok());
        assert_eq!(command.unwrap(), Command::Help);

        let command = "exit".parse::<Command>();
        assert!(command.is_ok());
        assert_eq!(command.unwrap(), Command::Exit);

        let command = "invalid".parse::<Command>();
        assert!(command.is_err());
    }
}
