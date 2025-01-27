use crate::cell::*;
use crate::page::*;

use anyhow::{Ok, Result};
use std::fs::File;
use std::io::prelude::*;

pub fn tables(p: &str) -> Result<()> {
    let mut file = File::open(p)?;
    let db_header = DataBaseHeader::new(&mut file)?;

    file.rewind()?;
    let page = Page::new(&mut file, db_header.page_size, 100)?;

    let mut cells = vec![];
    for cell_start in page.cell_ptrs {
        cells.push(Cell::new(&page.raw[cell_start..], &page.page_type)?);
    }

    let mut tables = vec![];
    for cell in &cells {
        if cell.content.is_table() {
            let table_name = cell.content.get_table_name()?;
            if table_name != "sqlite_sequence" {
                tables.push(table_name);
            }
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
