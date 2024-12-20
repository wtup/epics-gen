//! # epics-gen
//!
//! epics-gen is a set of helper macros that helps create parsers for serializing xlsx
//! spreadsheets into data structures and deserializing them as EPICS PVs.
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
//! ```
//!
//! # Deserialization
//!
//! Note that an external library [`calamine`] is used to read and store `xlsx` files and
//! data. The used structures and functions from calamine are reexported in this crate convenience.
//! See `Type Aliases` for more information.
//!
//! The following code is a xlsx deserializing example:
//!
//! ```
//! use epics_gen::FromXlsxData;
//!
//! let mut workbook: epics_gen::XlsxWorkbook = epics_gen::open_workbook("tests/test_parser1.xlsx")
//!     .expect("xlsx file for this test is missing!");
//!
//! let parser = epics_gen::ParserBuilder::new(&mut workbook)
//!     .add_sheet("Sheet1")
//!     .add_table("test_table_1")
//!     .build();
//!
//!
//! #[derive(epics_gen::FromXlsxRow, Debug, PartialEq)]
//! struct TargetStruct {
//!    row_id: String,
//!    float1: f64,
//!    float2: f64,
//! }
//!
//! let parsed: Vec<TargetStruct> = parser.parse();
//! ```
//! Note that the struct members must implement traits that enable conversion from Xlsx types to
//! target type. See [`FromXlsxData`] trait for more details (and the macros that
//! automatically implement it).
//!
//! and this an example of serializing structures to PVs:
//!
//! ```rust
//! use epics_gen::AsRecord;
//!
//! #[derive(AsRecord)]
//! #[record(rec_name = "$(P)Voltage", rec_type = "ao")]
//! struct TestStruct {
//!     #[record(field = "DESC")]
//!     desc: &'static str,
//!     #[record(field = "EGU")]
//!     egu: &'static str,
//!     #[record(field = "VAL")]
//!     val: f64,
//! }
//!
//! let test_struct = TestStruct {
//!     desc: "Output Voltage",
//!     egu: "V",
//!     val: 0.5,
//! };
//!
//! assert_eq!(
//!     test_struct.as_record(),
//!     r#"record(ao, "$(P)Voltage") {
//!   field(DESC, "Output Voltage")
//!   field(EGU, "V")
//!   field(VAL, "0.5")
//! }
//! "#);
//! ```
//! see [`AsRecord`] for more details.
//!
//! # Serialization
//!
//! To serialize struct the `AsRecord` macro is used. It comes with attributes to help the user
//! define EPICS PVs for printing.
//!
//! ## Attributes
//!
//! To use the `AsRecord` macro, these attributes need to be defined:
//!
//! - record name: `#[record(rec_name = "<record_name>")]` (e.g.: "$(P)Voltage")
//! - record type: `#[record(rec_type = "<record_type>")]` (e.g.: "ao")
//! - record field: `#[record(field = "<field>")]` (e.g.: "DESC")
//!
//! Optional attributes:
//!
//! - subst: `#[record(subst = "<pattern>")]`; substitutes a pattern in other fields. Similar to EPICS
//!   macro definitions.
//! - fmt: `#[record(fmt = "<user_defined_string>"]`; overrides other attributes and lets the user
//!   define a custom output.
//!   (e.g.: `#[record(fmt = r#"record(ao, "$(P):Voltage"){field(VAL, "{{}}")"#]`)
//! - repr: `#[record(repr = <type>)]`; convert to type before printing the value; (e.g.: `#[record(repr = u32)]`)
//!
//! ## Usage
//!
//! The mandatory attributes `name` and `type` can be either set on the whole structure (global)
//! or on a per field basis (local), which allows defining more records per struct.
//!
//! Global record definition example:
//!
//! ```ignore
//! #[derive(AsRecord)]
//! #[record(rec_name = "$(P)Voltage", rec_type = "ao")]
//! struct TestStruct {
//! #[record(field = "DESC")]
//! desc: &'static str,
//! #[record(field = "EGU")]
//! egu: &'static str,
//! #[record(field = "VAL")]
//! val: f64,
//! }
//! ```
//!
//! local record definition example:
//!
//! ```ignore
//! #[derive(AsRecord)]
//! struct TestStruct {
//!     #[record(rec_name = "$(P)Voltage", rec_type = "ao")]
//!     #[record(field = "VAL")]
//!     voltage: f64,
//!     #[record(rec_name = "$(P)Current", rec_type = "ao")]
//!     #[record(field = "VAL")]
//!     current: f64,
//!     #[record(rec_name = "$(P)SlewRate", rec_type = "ao")]
//!     #[record(field = "VAL")]
//!     slew_rate: f64,
//! }
//! ```
//!
//! See tests for usage examples of other attributes.
//!

use std::collections::HashMap;

use calamine::{Cell, Data, Reader};

#[allow(unused_imports)]
#[cfg(feature = "derive")]
pub use epics_gen_macros::*;

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

/// Builder for parsers.
///
/// This is used to build parsers of excel tables. Use [`add_tables`](Self::add_tables) and
/// [`add_sheets`](Self::add_sheets) to specify which tables it needs to parse and
/// which sheets to find them in.
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
    /// Construct new parser builder.
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

    /// Adds single sheet to parser.
    pub fn add_sheet(mut self, sheet: impl Into<String>) -> Self {
        self.sheets.push(Entry::String(sheet.into()));
        self
    }

    /// Adds a pattern which is expanded to matched sheet names in the workbook.
    pub fn add_sheets(mut self, sheet_pattern: Regex) -> Self {
        self.sheets.push(Entry::Regex(sheet_pattern));
        self
    }

    /// Adds single table to parser.
    pub fn add_table(mut self, table: impl Into<String>) -> Self {
        self.tables.push(Entry::String(table.into()));
        self
    }

    /// Adds a pattern which is expanded to matched table names in the workbook.
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

    // Builds the parser.
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

/// Parser structure. It's only purpose is to call [`parse`](Self::parse) and convert tables into a
/// vector of user defined structs.
pub struct Parser<'a> {
    workbook: &'a mut XlsxWorkbook,
    sheets: HashMap<String, Vec<String>>,
}

impl<'a> Parser<'a> {
    fn parse_by_rows<O: FromXlsxRow>(&mut self, table_name: String) -> Result<Vec<O>, ParseError> {
        let mut res = Vec::new();
        let table = self.workbook.table_by_name(&table_name).map_err(|_| {
            ParseError::new_in_table(
                ParseErrorKind::InvalidTableName,
                Cell::new((0, 0), Data::Empty),
                table_name,
            )
        })?;

        let rows = table.data().rows();

        for (i, row) in rows.enumerate() {
            res.push(O::from_xlsx_row(row.into(), i, table.name())?);
        }

        Ok(res)
    }

    /// Parse tables to struct.
    pub fn parse<O: FromXlsxRow>(mut self) -> Result<Vec<O>, ParseError> {
        let mut res: Vec<O> = Vec::new();
        for (_, tables) in self.sheets.clone().into_iter() {
            for table in tables {
                res.extend(self.parse_by_rows(table)?);
            }
        }
        Ok(res)
    }
}

/// The `ParseError` enum is a collection of all possible reasons
/// a value could not be parsed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    InvalidValue,
    ValueMissing,
    InvalidTableName,
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
            ParseErrorKind::InvalidTableName => {
                if let Some(location) = &self.location {
                    write!(f, "Invalid table name, Location: {}", location)
                } else {
                    write!(f, "Invalid table name.")
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

//TODO: Decide if this is needed, or if it can be replaced with a simple String
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
/// implemented from a derive macro [FromXlsxRow](epics_gen_macros::FromXlsxRow)!
///
///
/// For example, a table of 8 rows contains configuration for 8 `Prescalers`. To convert from a row
/// of data to a `Prescaler` structure this trait needs to be implemented on the `Prescaler`
/// struct via `FromXlsxRow`.
pub trait FromXlsxRow
where
    Self: Sized,
{
    fn from_xlsx_row(
        row: Vec<calamine::Data>,
        row_num: usize,
        table_name: &str,
    ) -> std::result::Result<Self, ParseError>;
}

/// Interface that is used to convert XlsxData to target type.
///
/// This trait is used when traversing the XlsxRow and converting each cell to associated struct
/// member types.
pub trait FromXlsxData
where
    Self: Sized,
{
    type Error;

    fn from_xlsx_data(data: XlsxData) -> Result<Self, Self::Error>;
}

impl FromXlsxData for f64 {
    type Error = ParseErrorKind;

    fn from_xlsx_data(data: XlsxData) -> Result<Self, Self::Error> {
        data.get_float().ok_or(Self::Error::ValueMissing)
    }
}

impl FromXlsxData for String {
    type Error = ParseErrorKind;

    fn from_xlsx_data(data: XlsxData) -> Result<Self, Self::Error> {
        data.get_string()
            .map(String::from)
            .ok_or(Self::Error::ValueMissing)
    }
}
