use crate::cell::*;
use crate::page::*;

use regex::Regex;

use anyhow::Ok;
use anyhow::Result;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

pub fn tables(p: &str) -> Result<()> {
    let mut file = File::open(p)?;
    let db_header = DataBaseHeader::new(&mut file)?;

    file.rewind()?;
    let page = Page::new(&mut file, db_header.page_size, 100)?;

    let tables = tables_from_page(&page)?;

    let mut tables_names: Vec<&str> = tables
        .iter()
        .filter_map(|table| table.content.get_table_name().ok())
        .collect();

    tables_names.sort();

    for table_name in tables_names {
        print!("{} ", table_name);
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

fn count_rows(p: &str, table: &str) -> Result<()> {
    let mut file = File::open(p)?;
    let db_header = DataBaseHeader::new(&mut file)?;

    file.rewind()?;
    let page = Page::new(&mut file, db_header.page_size, 100)?;

    let tables = tables_from_page(&page)?;

    if let Some(table) = find_table_by_name(&tables, table) {
        println!("{}", table);
        let root_page =
            (table.content.get_rootpage().get_numeric_val() - 1) * db_header.page_size as u64;

        file.seek(SeekFrom::Start(root_page))?;
        let table_page = Page::new(&mut file, db_header.page_size, 0)?;
        // println!("table_page: {}", table_page);

        if let PageType::LeafTable = table_page.page_type {
            println!("{}", table_page.cell_count);
        } else {
            println!("table is multipage table ... cant parse that yet");
        }
    } else {
        println!("no such table: {}", table);
    }

    Ok(())
}

pub fn select_x_from_y(p: &str, x: &str, y: &str) -> Result<()> {
    println!("Select {} from {}", x, y);
    let mut file = File::open(p)?;
    let db_header = DataBaseHeader::new(&mut file)?;

    file.rewind()?;
    let page = Page::new(&mut file, db_header.page_size, 100)?;

    let tables = tables_from_page(&page)?;

    if let Some(table) = find_table_by_name(&tables, y) {
        println!("table: {}", table);
        let table_regex = Regex::new(r"(?i)CREATE\s+TABLE\s+(\w+)\s*\(([^;]+)\)").unwrap();
        let field_regex = Regex::new(r"(\w+)\s+\w+").unwrap();

        if let Some(caps) = table_regex.captures(table.content.get_sql().get_string_val()) {
            let table_name = &caps[1];
            let fields_part = &caps[2];
            for field_cap in field_regex.captures_iter(fields_part) {
                println!("Field: {}", &field_cap[1]);
            }
        }
    }

    Ok(())
}

pub fn sql_query(p: &str, query: &str) -> Result<()> {
    let tokens: Vec<&str> = query.split(" ").collect();

    if tokens.len() < 4 {
        println!("syntax error, need more arguments");
    }

    if *tokens.get(1).unwrap() == "COUNT(*)" {
        return count_rows(p, tokens.last().unwrap());
    }
    select_x_from_y(p, tokens.get(1).unwrap(), tokens.get(3).unwrap())
}
