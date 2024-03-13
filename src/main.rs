use console::Term;
use dialoguer::Input;

mod core;
use crate::core::commands::Command;

/* Feature ideas:
- "units" command: show categories
- Add more units, duh!
- Request currency exchange rates from an API (storing them in my current model might be non-trivial)
- Make the program locally executable via a command (e.g. `convert 100 m -> km`)
- Provide web API for the program
 */

fn main() {
    // term can be exchanged with other UI
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
