use std::{fs::File, io::Read};
use anyhow::{Ok, Result};
use std::io::SeekFrom;
use std::io::prelude::*;

#[derive(Debug)]
#[allow(dead_code)]
pub enum StringEncoding {
    Utf8,
    Utf16le,
    Utf16be,
}

#[allow(dead_code)]
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
            1 => StringEncoding::Utf8,
            2 => StringEncoding::Utf16le,
            3 => StringEncoding::Utf16be,
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



pub struct DB {
    pub header: DataBaseHeader,
    pub file: File
}

impl DB {
    pub fn new(p: &str) -> Result<Self> {
        let mut file = File::open(p)?;
        let header = DataBaseHeader::new(&mut file)?;

        Ok(Self{
            header,
            file
        })
    }

    pub fn root_page(&mut self, rp: u64) -> Result<()> {
        let offset = (rp - 1) * self.header.page_size as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        Ok(())
    }
}

