use anyhow::{bail, Ok, Result};
use core::panic;
use std::fs::File;
use std::io::prelude::*;

enum PageType {
    interior_index,
    interior_table,
    leaf_index,
    leaf_table,
}

struct Page {
    raw: Vec<u8>,
    size: u16,
    cell_count: u16,
    page_type: PageType,
}

impl Page {
    fn new(file: &mut File, size: u16) -> Result<Self> {
        if size < 8 {
            panic!("Page Size is smaller than page header!");
        }
        let mut page = Self { raw: vec![0u8; size as usize], cell_count: 0, page_type: PageType::interior_index, size };
        file.read_exact(&mut page.raw)?;
        page.cell_count = u16::from_be_bytes([page.raw[3], page.raw[4]]);
        match page.raw[0] {
            0x02 => page.page_type = PageType::interior_index,
            0x05 => page.page_type = PageType::interior_table,
            0x0a => page.page_type = PageType::leaf_index,
            0x0d => page.page_type = PageType::leaf_table,
            page_type => panic!("Unknown Page Type {}!", page_type),
        }
        Ok(page)
    }
}

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            let mut file = File::open(&args[1])?;
            let mut file_header = [0; 100];
            file.read_exact(&mut file_header)?;

            let page_size = u16::from_be_bytes([file_header[16], file_header[17]]);

            let page = Page::new(&mut file, page_size)?;


            println!("database page size: {}", page_size);
            println!("number of tables: {}", page.cell_count);
            // println!("number of cells: {}", page.cell_count);
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
