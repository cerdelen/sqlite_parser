use crate::utils::VarInt;
// use core::{panic;
use crate::page::PageType;
use anyhow::{bail, Ok, Result};
// use core::slice::SlicePattern;
use std::{fmt, usize};

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Cell {{")?;
        writeln!(f, "\tsize_record: {}", self.size_record)?;
        writeln!(f, "\trowId: {:?}", self.rowid)?;
        writeln!(f, "\tcontent: {:?}", self.content)?;
        writeln!(f, "}}")
    }
}


pub fn find_table_by_name<'a>(cells: &'a Vec<Cell>, target: &str) -> Option<&'a Cell> {
    cells.iter().find(|t|{
            if let Content::TableCell(content) = &t.content {
                content.get_table_name().ok().map_or(false, |name| name == target)
            }
            else {
                false
            }
        }
    )
}


#[derive(Debug)]
pub enum Record {
    Null,
    I8(i8),
    I16(i16),
    I24(i32),
    I32(i32),
    I48(i64),
    I64(i64),
    F64(f64),
    Val0,
    Val1,
    Reserved,
    Blob,
    String(String),
}

impl Record {
    pub fn get_string_val(&self) -> &String {
        match self {
            Record::String(s) => s,
            _ => panic!("Record is not Strign!"),
        }
    }

    pub fn get_numeric_val(&self) -> u64 {
        match self {
            Record::I8(v) => *v as u64,
            Record::I16(v) => *v as u64,
            Record::I24(v) => *v as u64,
            Record::I32(v) => *v as u64,
            Record::I48(v) => *v as u64,
            Record::I64(v) => *v as u64,
            Record::F64(v) => *v as u64,
            Record::Val0 => 0,
            Record::Val1 => 1,
            _ => panic!("Record is not numeric!"),
        }
    }

    pub fn mem_size(&self) -> usize {
        match self {
            Record::Null => 0,
            Record::I8(_) => 1,
            Record::I16(_) => 2,
            Record::I24(_) => 3,
            Record::I32(_) => 4,
            Record::I48(_) => 6,
            Record::I64(_) => 8,
            Record::F64(_) => 8,
            Record::Val0 => 0,
            Record::Val1 => 0,
            Record::Reserved => panic!("Should never encounter Reserved Record"),
            Record::Blob => todo!(),
            Record::String(s) => s.len(),
        }
    }

    fn new(bytes: &[u8], record_type: &VarInt) -> Result<Self> {
        let res = match record_type.val {
            0 => Self::Null,
            1 => {
                if bytes.len() < 1 {
                    bail!("expect I8 but buffer only size of {}", bytes.len());
                };
                Self::I8(bytes[0] as i8)
            }
            2 => {
                if bytes.len() < 2 {
                    bail!("expect I16 but buffer only size of {}", bytes.len());
                };
                Self::I16(i16::from_be_bytes([bytes[0], bytes[1]]))
            }
            3 => {
                if bytes.len() < 3 {
                    bail!("expect I24 but buffer only size of {}", bytes.len());
                }
                Self::I24(i32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]))
            }
            4 => {
                if bytes.len() < 4 {
                    bail!("expect I32 but buffer only size of {}", bytes.len());
                }
                Self::I32(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
            }
            5 => {
                if bytes.len() < 6 {
                    bail!("expect I48 but buffer only size of {}", bytes.len());
                }
                Self::I48(i64::from_be_bytes([
                    0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5],
                ]))
            }
            6 => {
                if bytes.len() < 8 {
                    bail!("expect I64 but buffer only size of {}", bytes.len());
                }
                Self::I64(i64::from_be_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]))
            }
            7 => {
                if bytes.len() < 8 {
                    bail!("expect F64 but buffer only size of {}", bytes.len());
                }
                Self::F64(f64::from_be_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]))
            }
            8 => Self::Val0,
            9 => Self::Val1,
            10 | 11 => Self::Reserved,
            val => match val % 2 {
                1 => {
                    let str_len = ((val - 13) / 2) as usize;
                    if bytes.len() < str_len {
                        println!(
                            "Error this is the entry \"{}\"",
                            String::from_utf8_lossy(bytes)
                        );
                        bail!(
                            "expected String of size {}, but buffer only size of {}",
                            ((val - 13) / 2) as usize,
                            bytes.len()
                        );
                    }
                    let s = String::from_utf8_lossy(&bytes[..str_len]);
                    Self::String(s.to_string())
                }
                0 => {
                    if bytes.len() < ((val - 12) / 2) as usize {
                        bail!(
                            "expected Blob of size {}, but buffer only size of {}",
                            ((val - 12) / 2) as usize,
                            bytes.len()
                        );
                    }
                    Self::Blob
                }
                err => panic!("modulo 2 of {} returned neither 0 nor 1: {}", val, err),
            },
        };
        Ok(res)
    }
}

fn parse_cell_header(bytes: &[u8]) -> Result<(Vec<VarInt>, usize)> {
    let mut ind: usize = 0;
    let header_size = VarInt::from_mem(&bytes[..10])?;
    ind += header_size.len;

    let mut types = Vec::new();
    while ind < header_size.val as usize {
        let serial_type = VarInt::from_mem(&bytes[ind..ind + 10])?;
        ind += serial_type.len;
        types.push(serial_type);
    }

    assert!(ind == header_size.val as usize);

    Ok((types, ind))
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct RowCell{
    pub row: Vec<Record>
}

impl RowCell {
    fn new(bytes: &[u8]) -> Result<Self> {
        let (types, mut ind) = parse_cell_header(bytes)?;
        let mut row = Vec::new();

        for serial_type in types {
            let r = Record::new(&bytes[ind..], &serial_type)?;
            ind += r.mem_size();
            // println!("r: {:?}", r);
            row.push(r);
        }

        Ok(Self { row })
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ContentVariant {
    TableCell,
    RowCell
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Content {
    TableCell(TableCell),
    RowCell(RowCell)
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct TableCell {
    header_size: VarInt,
    schema_type: Record,
    schema_name: Record,
    schema_tbl_name: Record,
    schema_rootpage: Record,
    schema_sql: Record,
}

impl TableCell {
    pub fn get_sql(&self) -> &Record {
        &self.schema_sql
    }

    pub fn get_rootpage(&self) -> &Record {
        &self.schema_rootpage
    }

    pub fn is_table(&self) -> bool {
        if let Record::String(s) = &self.schema_type {
            if s == "table" {
                return true;
            }
        }
        false
    }

    pub fn get_table_name(&self) -> Result<&str> {
        if self.is_table() {
            if let Record::String(s) = &self.schema_tbl_name {
                return Ok(s);
            }
        }
        bail!("not a table or not a string type")
    }

    fn new(bytes: &[u8]) -> Result<Self> {
        let header_size = VarInt::from_mem(&bytes[..10])?;
        let mut c = RowCell::new(bytes)?;

        let schema_sql = c.row.pop().expect("TableRow doesnt have schema sql");
        let schema_rootpage = c.row.pop().expect("TableRow doesnt have schema rootpage");
        let schema_tbl_name = c.row.pop().expect("TableRow doesnt have schema table name");
        let schema_name = c.row.pop().expect("TableRow doesnt have schema name");
        let schema_type = c.row.pop().expect("TableRow doesnt have schema type");
        // let mut ind: usize = 0;
        // let schema_type_size = VarInt::from_mem(&bytes[ind..ind + 10])?;
        // ind += schema_type_size.len;
        // let schema_name_size = VarInt::from_mem(&bytes[ind..ind + 10])?;
        // ind += schema_name_size.len;
        // let table_name_size = VarInt::from_mem(&bytes[ind..ind + 10])?;
        // ind += table_name_size.len;
        // let rootpage_type = VarInt::from_mem(&bytes[ind..ind + 10])?;
        // ind += rootpage_type.len;
        // let sql_size = VarInt::from_mem(&bytes[ind..ind + 10])?;
        // ind += sql_size.len;
        //
        // let schema_type = Record::new(&bytes[ind..], &schema_type_size)?;
        // ind += schema_type.mem_size();
        // let schema_name = Record::new(&bytes[ind..], &schema_name_size)?;
        // ind += schema_name.mem_size();
        // let schema_tbl_name = Record::new(&bytes[ind..], &table_name_size)?;
        // ind += schema_tbl_name.mem_size();
        // let schema_rootpage = Record::new(&bytes[ind..], &rootpage_type)?;
        // ind += schema_rootpage.mem_size();
        // let schema_sql = Record::new(&bytes[ind..], &sql_size)?;
        Ok(Self {
            header_size,
            schema_type,
            schema_name,
            schema_tbl_name,
            schema_rootpage,
            schema_sql,
        })
    }
}

pub struct Cell {
    size_record: VarInt,
    pub rowid: VarInt,
    pub content: Content,
}

#[allow(dead_code)]
impl Cell {
    pub fn record_size(&self) -> usize {
        self.size_record.val as usize
    }

    pub fn cell_size(&self) -> usize {
        (self.size_record.val as usize) + self.rowid.len + self.size_record.len
    }

    pub fn new(bytes: &[u8], page_type: &PageType, content_type: ContentVariant) -> Result<Self> {
        match page_type {
            PageType::LeafTable => {
                // there can be overflow for cell spillage
                let size_record = VarInt::from_mem(&bytes[..9])?;
                let rowid = VarInt::from_mem(&bytes[size_record.len..size_record.len + 9])?;

                let content = match content_type {
                    ContentVariant::TableCell => Content::TableCell(TableCell::new(&bytes[&size_record.len + &rowid.len..])?),
                    ContentVariant::RowCell => {
                        Content::RowCell(RowCell::new(&bytes[&size_record.len + &rowid.len..])?)
                        // println!("page: {}",)
                    },
                };

                return Ok(Self {
                    size_record,
                    rowid,
                    content,
                });
            }
            _ => todo!(),
            // PageType::InteriorTable => todo!(),
            // PageType::LeafIndex => todo!(),
            // PageType::InteriorIndex => todo!(),
        };
    }
}
