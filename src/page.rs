use anyhow::{Ok, Result};
use core::panic;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use crate::cell::Cell;

#[derive(Debug)]
pub enum PageType {
    InteriorIndex,
    InteriorTable,
    LeafIndex,
    LeafTable,
}

#[derive(Debug)]
pub enum StringEncoding {
    UTF_8,
    UTF_16LE,
    UTF_16BE,
}

pub struct DataBaseHeader {
    pub page_size: u16,
    pub string_encoding: StringEncoding,
    pub database_size: u32,
    // Offset	Size	Description
    // 0	16	The header string: "SQLite format 3\000"
    // 16	2	The database page size in bytes. Must be a power of two between 512 and 32768 inclusive, or the value 1 representing a page size of 65536.
    // 18	1	File format write version. 1 for legacy; 2 for WAL.
    // 19	1	File format read version. 1 for legacy; 2 for WAL.
    // 20	1	Bytes of unused "reserved" space at the end of each page. Usually 0.
    // 21	1	Maximum embedded payload fraction. Must be 64.
    // 22	1	Minimum embedded payload fraction. Must be 32.
    // 23	1	Leaf payload fraction. Must be 32.
    // 24	4	File change counter.
    // 28	4	Size of the database file in pages. The "in-header database size".
    // 32	4	Page number of the first freelist trunk page.
    // 36	4	Total number of freelist pages.
    // 40	4	The schema cookie.
    // 44	4	The schema format number. Supported schema formats are 1, 2, 3, and 4.
    // 48	4	Default page cache size.
    // 52	4	The page number of the largest root b-tree page when in auto-vacuum or incremental-vacuum modes, or zero otherwise.
    // 56	4	The database text encoding. A value of 1 means UTF-8. A value of 2 means UTF-16le. A value of 3 means UTF-16be.
    // 60	4	The "user version" as read and set by the user_version pragma.
    // 64	4	True (non-zero) for incremental-vacuum mode. False (zero) otherwise.
    // 68	4	The "Application ID" set by PRAGMA application_id.
    // 72	20	Reserved for expansion. Must be zero.
    // 92	4	The version-valid-for number.
    // 96	4	SQLITE_VERSION_NUMBER
}

impl DataBaseHeader {
    pub fn new(file: &mut File) -> Result<Self> {
        let mut file_header = [0; 100];
        file.read_exact(&mut file_header)?;
        let page_size = u16::from_be_bytes([file_header[16], file_header[17]]);
        let string_encoding = match u32::from_be_bytes(file_header[56..60].try_into()?) {
            1 => StringEncoding::UTF_8,
            2 => StringEncoding::UTF_16LE,
            3 => StringEncoding::UTF_16BE,
            _ => panic!("Unknown String Encoding"),
        };
        let database_size = u32::from_be_bytes(file_header[28..32].try_into()?);
        Ok(Self {
            page_size,
            string_encoding,
            database_size,
        })
    }
}

pub fn tables_from_page(page: &Page) -> Result<Vec<Cell>> {
    let mut cells = vec![];
    for cell_start in &page.cell_ptrs {
        cells.push(Cell::new(&page.raw[cell_start.clone()..], &page.page_type)?);
    }

    cells.retain(|c| c.content.is_table());
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
    pub fn new(file: &mut File, size: u16, page_header_start: usize) -> Result<Self> {
        if size < 8 {
            panic!("Page Size is smaller than page header!");
        }
        let mut ind = page_header_start;
        let mut page = Self {
            raw: vec![0u8; size as usize],
            cell_count: 0,
            page_type: PageType::InteriorIndex,
            header_offset: 0,
            free_block_start: 0,
            size,
            cell_start: 0,
            cell_ptrs: vec![],
            free_block_size: 0,
            right_most_ptr: None,
        };
        file.read_exact(&mut page.raw)?;
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
