//! # epics-gen
//!
//! epics-gen is a set of helper macros that help developers create parsers for serializing xlsx spreadsheets
//! and deserializing them as EPICS PVs.
//!
//! # Including epics-gen in Your Project
//!
//! Import `epics_gen` and `epics_gen_macros` into your project by adding the following lines to your
//! Cargo.toml. `epics_gen_macros` contains the macros needed to derive all the traits in epics-gen.
//!
//! ```toml
//! [dependencies]
//! epics_gen = "0.1"
//! epics_gen_macros = "0.1"
//!
//! # Usage
//!
//! The following code is a usage example:
//! ```rust
//! let mut workbook: epics-gen::XlsxWorkbook = epics-gen::open_workbook("tests/test_parser1.xlsx")
//!     .expect("xlsx file for this test is missing!");
//! ```

use std::collections::HashMap;

use calamine::{Cell, Data, Reader};

#[allow(unused_imports)]
#[cfg(feature = "derive")]
use epics_gen_macros::*;

use regex::Regex;

// Excel workbook. Reexported from calamine.
pub type XlsxWorkbook = calamine::Xlsx<std::io::BufReader<std::fs::File>>;
pub use calamine::DataType;

/// A struct that represents a row in a table/sheet. Reexported from calamine.
pub type XlsxData = calamine::Data;

/// A struct that represents a row in a table/sheet. Reexported from calamine.
pub type XlsxRow = Vec<XlsxData>;

/// A struct that represents an Excel Cell unit. Reexported from calamine.
pub type XlsxCell = calamine::Cell<XlsxData>;
pub use calamine::open_workbook;

pub struct ParserBuilder<'a> {
    workbook: &'a mut XlsxWorkbook,
    sheets: Vec<Entry>,
    tables: Vec<Entry>,
}

enum Entry {
    String(String),
    Regex(regex::Regex),
}

impl<'a> ParserBuilder<'a> {
    pub fn new(workbook: &'a mut calamine::Xlsx<std::io::BufReader<std::fs::File>>) -> Self {
        workbook
            .load_tables()
            .expect("Could not load workbook tables!");
        Self {
            workbook,
            sheets: Vec::new(),
            tables: Vec::new(),
        }
    }

    pub fn add_sheet(mut self, sheet: impl Into<String>) -> Self {
        self.sheets.push(Entry::String(sheet.into()));
        self
    }

    pub fn add_sheets(mut self, sheet_pattern: Regex) -> Self {
        self.sheets.push(Entry::Regex(sheet_pattern));
        self
    }

    pub fn add_table(mut self, table: impl Into<String>) -> Self {
        self.tables.push(Entry::String(table.into()));
        self
    }

    pub fn add_tables(mut self, table_pattern: Regex) -> Self {
        self.tables.push(Entry::Regex(table_pattern));
        self
    }

    fn get_valid_tables(&self, sheet_name: &str) -> Vec<String> {
        let table_names_in_sheet = self.workbook.table_names_in_sheet(sheet_name);
        let mut res: Vec<String> = Vec::new();
        self.tables
            .iter()
            .for_each(|table_entry| match table_entry {
                Entry::String(s) => {
                    if table_names_in_sheet.contains(&s) {
                        res.push(s.to_string());
                    }
                }
                Entry::Regex(r) => table_names_in_sheet.iter().for_each(|table_name| {
                    if r.is_match(table_name) {
                        res.push((*table_name).to_owned());
                    }
                }),
            });
        res
    }

    pub fn build(self) -> Parser<'a> {
        let mut sheets: HashMap<String, Vec<String>> = HashMap::new();
        let sheet_names = self.workbook.sheet_names();
        self.sheets
            .iter()
            .for_each(|sheet_entry| match sheet_entry {
                Entry::String(s) => {
                    if sheet_names.contains(s) {
                        sheets.insert(s.to_string(), self.get_valid_tables(s));
                    }
                }
                Entry::Regex(r) => self
                    .workbook
                    .sheet_names()
                    .into_iter()
                    .for_each(|sheet_name| {
                        if r.is_match(&sheet_name) {
                            sheets
                                .insert(sheet_name.to_string(), self.get_valid_tables(&sheet_name));
                        }
                    }),
            });

        Parser {
            workbook: self.workbook,
            sheets,
        }
    }
}

// First state of the Parser is just creating it with a reference to workbook
// Second state of parser is adding a sheet and table
// Third state of parser is parsing the sheets
// Final state of parser is returning a map of sheets with their associated objects
pub struct Parser<'a> {
    workbook: &'a mut XlsxWorkbook,
    sheets: HashMap<String, Vec<String>>,
}

impl<'a> Parser<'a> {
    fn parse_by_rows<O: FromXlsxRow>(&mut self, table_name: String) -> Vec<O>
    where
        <O as FromXlsxRow>::Error: std::fmt::Display,
    {
        let mut res = Vec::new();
        let table = self
            .workbook
            .table_by_name(&table_name)
            .unwrap_or_else(|e| panic!("Invalid table name {}", e));

        let rows = table.data().rows();

        for (i, row) in rows.enumerate() {
            res.push(
                O::from_xlsx_row(row.into(), i, table.name()).unwrap_or_else(|e| {
                    panic!(
                        "err: {}! Sheet: {}, Table: {}, Row: {}",
                        e,
                        table.sheet_name(),
                        table.name(),
                        i
                    )
                }),
            );
        }

        res
    }

    pub fn parse<O: FromXlsxRow>(mut self) -> Vec<O>
    where
        <O as FromXlsxRow>::Error: std::fmt::Display,
    {
        let mut res: Vec<O> = Vec::new();
        self.sheets.clone().into_iter().for_each(|(_, tables)| {
            for table in tables {
                res.extend(self.parse_by_rows(table));
            }
        });
        res
    }
}

/// The `ParseError` enum is a collection of all possible reasons
/// a value could not be parsed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    InvalidValue,
    ValueMissing,
}

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    location: Option<XlsxLocation>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ParseErrorKind::InvalidValue => {
                if let Some(location) = &self.location {
                    write!(f, "Invalid value. Location: {}", location)
                } else {
                    write!(f, "Invalid value.")
                }
            }
            ParseErrorKind::ValueMissing => {
                if let Some(location) = &self.location {
                    write!(f, "Value is missing, Location: {}", location)
                } else {
                    write!(f, "Value is missing.")
                }
            }
        }
    }
}

impl ParseError {
    pub fn kind(&self) -> ParseErrorKind {
        self.kind
    }

    pub fn new(kind: ParseErrorKind) -> ParseError {
        Self {
            kind,
            location: None,
        }
    }
    pub fn new_in_table(
        kind: ParseErrorKind,
        cell: Cell<Data>,
        table_name: impl Into<String>,
    ) -> ParseError {
        Self {
            kind,
            location: Some(XlsxLocation {
                cell,
                context: Context::Table(table_name.into()),
            }),
        }
    }
    pub fn new_in_sheet(
        kind: ParseErrorKind,
        cell: Cell<Data>,
        sheet_name: impl Into<String>,
    ) -> ParseError {
        Self {
            kind,
            location: Some(XlsxLocation {
                cell,
                context: Context::Table(sheet_name.into()),
            }),
        }
    }
}

impl std::error::Error for ParseError {}

/// `Location` represents a location in a xslx spreadsheet or table (depending on the context)
#[derive(Debug)]
struct XlsxLocation {
    cell: Cell<Data>,
    context: Context, //TODO: This could maybe be replaced with a simple string.
}

impl std::fmt::Display for XlsxLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (row, col) = self.cell.get_position();
        write!(
            f,
            "{}, Row: {}, Col: {}, Value: {} ",
            self.context,
            row,
            col,
            self.cell.get_value()
        )
    }
}

//TODO: Decide if we need this, or can it be replaced with a simple String
#[allow(dead_code)]
#[derive(Debug)]
enum Context {
    Sheet(String),
    Table(String),
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Context::Sheet(s) => write!(f, "Sheet: {}", s),
            Context::Table(s) => write!(f, "Table: {}", s),
        }
    }
}

/// Interface that supports converting a single row in a table to a structure. This should be
/// implemented from a derive macro (epics_gen::FromXlsxRow)!
///
///
/// For example, a table of 8 rows contains configuration for 8 `Prescalers`. To convert from a row
/// of data to a `Prescaler` structure this trait needs to be implemented on the `Prescaler`
/// struct.
///
// TODO: Modify this description
pub trait FromXlsxRow
where
    Self: Sized,
{
    type Error;

    fn from_xlsx_row(
        row: Vec<calamine::Data>,
        row_num: usize,
        table_name: &str,
    ) -> std::result::Result<Self, Self::Error>;
}
