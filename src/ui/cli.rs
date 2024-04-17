use crate::core::commands::Command;
use crate::ui::ui::Interface;

use console::Term;
use dialoguer::Input;

pub struct Cli;

impl Interface for Cli {
    fn new() -> Self {
        Cli {}
    }

    fn interact(self) {
        let term = Term::stdout();
        term.write_line("Enter a conversion expression (e.g. 100 m -> km) or 'exit' to exit.")
            .unwrap();

        loop {
            let input: String = Input::new().interact().unwrap();

            let command: Result<Command, _> = input.trim().parse();
            match command {
                Ok(Command::Exit) => break,
                Ok(command) => term.write_line(&command.execute()).unwrap(),
                Err(e) => term.write_line(&e).unwrap(),
            }
        }
    }
}
