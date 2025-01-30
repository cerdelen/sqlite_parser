mod cell;
mod commands;
mod page;
mod utils;

use anyhow::{bail, Ok, Result};

// const ELEMENTS_OFFSETS: usize = 2;

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            commands::db_info(&args[1])?;
        }
        ".tables" => {
            commands::tables(&args[1])?;
        }
        _ => {
            commands::sql_query(&args[1], command)?;
        },
    }

    Ok(())
}
