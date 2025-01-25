mod utils;
mod page;
use page::*;
mod cell;
use cell::*;

use anyhow::{bail, Ok, Result};
use std::fs::File;
use std::io::prelude::*;

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
            let mut file = File::open(&args[1])?;
            let db_header = DataBaseHeader::new(&mut file)?;

            file.rewind()?;
            let page = Page::new(&mut file, db_header.page_size, 100)?;

            let first_cell_offset = page.cell_start;
            let cell = Cell::new(&page.raw[(first_cell_offset as usize)..], &page.page_type)?;

            // println!("Page: {}", page);
            // println!("Cell: {}", cell);

            println!("database page size: {}", db_header.page_size);
            println!("number of tables: {}", page.cell_count);
        }
        ".tables" => {
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
