use std::num::ParseFloatError;
use console::Term;
use dialoguer::Input;
use regex::Regex;

mod units;
mod commands;
use crate::units::*;
use crate::commands::Command;

/* Feature ideas:
- REFACTOR: Using Bus-Booking example
- Commands:
    - 'units' command that outputs all available units
    - 'exit' command that exits the program
    - 'help' command that outputs a help message
- Add more units, duh!
- Request currency exchange rates from an API (storing them in my current model might be non-trivial)
- Make the program locally executable via a command (e.g. `convert 100 m -> km`)
- Provide web API for the program

 */

fn main() {
    let term = Term::stdout();
    term.write_line("Enter a conversion expression (e.g. 100 m -> km) or 'exit' to exit.").unwrap();
    
    loop {
        let input: String = Input::new()
            .interact()
            .unwrap();

        let command: Result<Command, _> = input.trim().parse();
        match command {
            Ok(Command::EXIT) => break,
            Ok(command) => term.write_line(&command.execute()).unwrap(),
            Err(_) => {}
        }
    }
}
