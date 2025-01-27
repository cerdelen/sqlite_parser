use crate::page::*;
use crate::cell::*;

use anyhow::{Ok, Result};
use std::io::prelude::*;
use std::fs::File;

pub fn tables(p: &str) -> Result<()> {
    let mut file = File::open(p)?;
    let db_header = DataBaseHeader::new(&mut file)?;

    file.rewind()?;
    let page = Page::new(&mut file, db_header.page_size, 100)?;

    println!("{}", page);

    let mut cells = vec![];
    for cell_start in page.cell_ptrs {
        cells.push(Cell::new(&page.raw[cell_start..], &page.page_type)?);
        // println!("cell: {}", cells.last().unwrap());
    }

    let mut tables = vec![];
    for cell in &cells {
        let table_name = cell.content.get_table_name()?;
        if table_name != "sqlite_sequence" {
            tables.push(table_name);
        }
    }
    tables.sort();
    for table in tables {
        print!("{} ", table);
    }
    println!("");

    Ok(())
}

pub fn db_info(p: &str) -> Result<()> {
    let mut file = File::open(p)?;
    let db_header = DataBaseHeader::new(&mut file)?;

    file.rewind()?;
    let page = Page::new(&mut file, db_header.page_size, 100)?;

    println!("database page size: {}", db_header.page_size);
    println!("number of tables: {}", page.cell_count);
    Ok(())
}

