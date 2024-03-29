mod core;
mod ui;
use crate::ui::cli::CLI;
use crate::ui::ui::Interface;

/* Feature ideas:
- "units" command: show categories
- Add more units, duh!
- Make the program locally executable via a command (e.g. `convert 100 m -> km`)
- Provide web API for the program
 */

fn main() {
    let cli = CLI::new();
    cli.interact();
}
