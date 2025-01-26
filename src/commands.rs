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


    let first_cell_offset = page.cell_start;
    let cell = Cell::new(&page.raw[(first_cell_offset as usize)..], &page.page_type)?;
    cell.content.print_table();

    // println!("Page: {}", page);
    // println!("Cell: {}", cell);
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
