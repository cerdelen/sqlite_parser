mod cell;
mod commands;
mod page;
mod utils;
mod db;

use db::DB;

use anyhow::{bail, Ok, Result};

// const ELEMENTS_OFFSETS: usize = 2;

fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    let mut db = DB::new(&args[1])?;

    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            commands::db_info(&mut db)?;
        }
        ".tables" => {
            commands::tables(&mut db)?;
        }
        _ => {
            commands::sql_query(&mut db, command)?;
        },
    }

    Ok(())
}
