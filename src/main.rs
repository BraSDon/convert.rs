mod core;
mod ui;
use crate::ui::cli::CLI;
use crate::ui::ui::Interface;

fn main() {
    let cli = CLI::new();
    cli.interact();
}
