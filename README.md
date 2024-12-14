# epics-gen

epics-gen is a set of helper macros that help developers create parsers for
serializing xlsx spreadsheets into data structures and deserializing them as
EPICS PVs.

Macros exposed by this library:

- [`FromXlsxRow`]: implements support for converting xlsx rows into structures (deserialize)
  - [`FromXlsxString`]: implements [FromXlsxData] trait for xlsx
    conversion from `XlsxString` type to target type
  - [`FromXlsxFloat`]: implements [FromXlsxData] trait for
    conversion from `XlsxFloat` type to target type
- [`AsRecord`]: implements support for printing the record (serialize)

## Including epics-gen in Your Project

Import `epics_gen` and `epics_gen_macros` into your project by adding the
following lines to your Cargo.toml. `epics_gen_macros` contains the macros
needed to derive all the traits in epics-gen.

```toml
[dependencies]
epics_gen = "0.1"
epics_gen_macros = "0.1"
```

## Macros

| Macro | Description |
| --- | ----------- |
| [FromXlsxRow] | Converts xlsx table row to a target struct (deserialization). |
| [FromXlsxString] | Converts XlsxString to target type. |
| [FromXlsxFloat] | Converts XlsxFloat to target type. |
| [AsRecord] | Implements `as_record` function to type (serialization). |

```rust
#[derive(FromXlsxString)]
enum RowId {
    First,
    Second,
    Third,
    Fourth,
}

#[derive(FromXlsxFloat)]
struct TestFloat(f64);

#[derive(FromXlsxRow)]
struct TargetStruct {
    row_id: RowId,
    float1: CustomFloat,
    float2: CustomFloat,
}

let mut workbook: XlsxWorkbook = open_workbook("tests/test_parser1.xlsx")
    .expect("xlsx file for this test is missing!");

let parser = epics_gen::ParserBuilder::new(&mut workbook)
    .add_sheet("SheetName")
    .add_table("TableName")
    .build();

let parsed: Vec<TargetStruct> = parser.parse();
```

Note that an external library [`calamine`] is used to read and store `xlsx`
files and data. The used structures and functions from calamine are reexported
in this crate convenience.

### AsRecord attributes

| Attribute  | Description                           |
| ---        | -----------                           |
| [rec_name] | Sets record name of struct.           |
| [rec_type] | Define record type.                   |
| [field]    | Define field type.                    |
| [subst]    | Define substitution pattern.          |
| [repr]     | Define representation type of member. |
| [fmt]      | Override member format.               |

Example:

```rust
#[derive(AsRecord)]
#[record(rec_name = "$(P)Voltage", rec_type = "ao")]
struct TestStruct {
#[record(field = "DESC")]
desc: &'static str,
#[record(field = "EGU")]
egu: &'static str,
#[record(field = "VAL")]
val: f64,
}
```
