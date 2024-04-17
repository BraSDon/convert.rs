mod core;
mod ui;
use crate::ui::cli::Cli;
use crate::ui::ui::Interface;

fn main() {
    let cli = Cli::new();
    cli.interact();
}
