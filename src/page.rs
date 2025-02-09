use anyhow::{Ok, Result};
use core::panic;
use std::fmt;
// use std::fs::File;
use std::io::prelude::*;

use crate::{cell::{Cell, Content, ContentVariant}, db::DB};

#[derive(Debug)]
pub enum PageType {
    InteriorIndex,
    InteriorTable,
    LeafIndex,
    LeafTable,
}


pub fn tables_from_page(page: &Page) -> Result<Vec<Cell>> {
    let mut cells = vec![];
    for cell_start in &page.cell_ptrs {
        cells.push(Cell::new(&page.raw[cell_start.clone()..], &page.page_type, ContentVariant::TableCell)?);
    }

    cells.retain(|c| {
        if let Content::TableCell(c) = &c.content {
            c.is_table()
        } else { false }
    });
    Ok(cells)
}

pub fn rows_from_page(page: &Page) -> Result<Vec<Content>> {
    let mut cells = vec![];
    for cell_start in &page.cell_ptrs {
        cells.push(Cell::new(&page.raw[cell_start.clone()..], &page.page_type, ContentVariant::RowCell)?.content);
    }

    Ok(cells)
}



#[derive(Debug)]
pub struct Page {
    pub raw: Vec<u8>,
    pub size: u16,
    pub cell_count: u16,
    pub page_type: PageType,
    pub header_offset: usize,
    pub free_block_start: u16,
    pub free_block_size: u8,
    pub cell_start: u32,
    pub cell_ptrs: Vec<usize>,
    pub right_most_ptr: Option<u32>,
}

impl Page {
    pub fn new(db: &mut DB, page_ind: u64) -> Result<Self> {
        db.root_page(page_ind)?;
        if db.header.page_size < 8 {
            panic!("Page Size is smaller than page header!");
        }
        let page_header_start = if db.file.stream_position()? == 0 {
            100
        } else {
            0
        };
        let mut ind = page_header_start;
        let mut page = Self {
            raw: vec![0u8; db.header.page_size as usize],
            cell_count: 0,
            page_type: PageType::InteriorIndex,
            header_offset: 0,
            free_block_start: 0,
            size: db.header.page_size,
            cell_start: 0,
            cell_ptrs: vec![],
            free_block_size: 0,
            right_most_ptr: None,
        };
        db.file.read_exact(&mut page.raw)?;
        match page.raw[ind] {
            0x02 => page.page_type = PageType::InteriorIndex,
            0x05 => page.page_type = PageType::InteriorTable,
            0x0a => page.page_type = PageType::LeafIndex,
            0x0d => page.page_type = PageType::LeafTable,
            page_type => panic!("Unknown Page Type {}!", page_type),
        }
        ind += 1;
        page.free_block_start = u16::from_be_bytes([page.raw[ind], page.raw[ind + 1]]);
        ind += 2;
        page.cell_count = u16::from_be_bytes([page.raw[ind], page.raw[ind + 1]]);
        ind += 2;
        page.cell_start = u16::from_be_bytes([page.raw[ind], page.raw[ind + 1]]) as u32;
        ind += 2;
        if page.cell_start == 0 {
            page.cell_start = 65536;
        }
        page.free_block_size = page.raw[ind];
        ind += 1;
        match &page.page_type {
            PageType::InteriorIndex | PageType::InteriorTable => {
                page.right_most_ptr = Some(u32::from_be_bytes([
                    page.raw[ind],
                    page.raw[ind + 1],
                    page.raw[ind + 2],
                    page.raw[ind + 3],
                ]));
                ind += 4;
            }
            _ => (),
        }
        for _ in 0..page.cell_count {
            page.cell_ptrs
                .push(u16::from_be_bytes([page.raw[ind], page.raw[ind + 1]]) as usize);
            ind += 2;
        }
        match page.page_type {
            PageType::InteriorIndex | PageType::InteriorTable => page.header_offset = 12,
            _ => page.header_offset = 8,
        }
        Ok(page)
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Page {{")?;
        writeln!(f, "\tPageType: {:?}", self.page_type)?;
        writeln!(f, "\tsize: {}", self.size)?;
        writeln!(f, "\tcell_count: {}", self.cell_count)?;
        writeln!(f, "\theader_offset: {}", self.header_offset)?;
        writeln!(f, "\tfree_block_start: {}", self.free_block_start)?;
        writeln!(f, "\tcell_start: {}", self.cell_start)?;
        writeln!(f, "}}")
    }
}
