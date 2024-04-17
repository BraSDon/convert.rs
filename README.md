# Basic Unit Converter

This is a command-line application written in Rust that allows you to convert between different units of measurement, including currencies.

Currently only a hand full units are supported, but more can be added by creating a corresponding enum in `src/units.rs`, implementing the Unitlike trait for the enum, and adding the new unit type to the high-level `Unit` enum.

Currency conversion is supported by using the Open Exchange Rates API. The application will fetch the latest exchange rates on request (if the stored rates are older than 1 week, see `src/currency.rs`) and cache them. On exit, the cache will be saved to a SQLite database and reloaded on startup.

## Usage Example
```sh
$ 1 km -> m
1000 meter (m)

$ 1 USD -> EUR
0.940365 EUR

$ help
Commands:
- <value> <unit> -> <unit>: Convert a value to another unit.
- units: List all available units.
- help: Show this help message.
- exit: Exit the program.

$ exit
```

## Getting Started

### Pre-requisites
1. [Rust + Cargo](https://www.rust-lang.org/tools/install)
2. [SQLite](https://www.sqlite.org/download.html)
3. [Open Exchange Rates API key](https://openexchangerates.org/) stored as environment variable `OPENEXCHANGERATES_APP_ID`

### Running the application
1. Clone the repository
    ```sh
    git clone https://github.com/BraSDon/convert.rs.git
    ```
2. Change into the project directory
    ```sh
    cd convert.rs
    ```
3. Build the project
    ```sh
    cargo build
    ```
4. Run the project
    ```sh
    cargo run
    ```


