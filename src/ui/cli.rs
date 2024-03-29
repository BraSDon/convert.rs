use crate::core::commands::Command;
use crate::ui::ui::Interface;

use console::Term;
use dialoguer::Input;

pub struct CLI;

impl Interface for CLI {
    fn new() -> Self {
        CLI {}
    }

    fn interact(self) {
        let term = Term::stdout();
        term.write_line("Enter a conversion expression (e.g. 100 m -> km) or 'exit' to exit.")
            .unwrap();

        loop {
            let input: String = Input::new().interact().unwrap();

            let command: Result<Command, _> = input.trim().parse();
            match command {
                Ok(Command::EXIT) => break,
                Ok(command) => term.write_line(&command.execute()).unwrap(),
                Err(e) => term.write_line(&e).unwrap(),
            }
        }
    }
}
