use epics_gen::{DataType, FromXlsxData, ParseError, ParserBuilder};
use epics_gen_macros::{FromXlsxFloat, FromXlsxRow, FromXlsxString};
use regex::Regex;
#[derive(FromXlsxString, strum_macros::EnumString, PartialEq, Eq, Debug)]
enum RowId {
    First,
    Second,
    Third,
    Fourth,
}

#[derive(FromXlsxFloat, PartialEq, Debug)]
struct TestFloat(f64);

impl From<f64> for TestFloat {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

#[test]
fn test_parser1() {
    #[derive(FromXlsxRow, Debug, PartialEq)]
    struct TargetStruct {
        row_id: RowId,
        float1: TestFloat,
        float2: TestFloat,
    }
    impl TargetStruct {
        pub fn new(row_id: RowId, float1: TestFloat, float2: TestFloat) -> Self {
            Self {
                row_id,
                float1,
                float2,
            }
        }
    }

    let mut workbook: epics_gen::XlsxWorkbook = epics_gen::open_workbook("tests/test_parser1.xlsx")
        .expect("xlsx file for this test is missing!");

    let parser = epics_gen::ParserBuilder::new(&mut workbook)
        .add_sheet("Sheet1")
        .expect("Sheet1 does not exist.")
        .add_table("test_table_1")
        .expect("test_table_1 table does not exist.")
        .build();

    let parsed: Vec<TargetStruct> = parser.parse().unwrap();

    use RowId::*;
    let mut expected: Vec<TargetStruct> = vec![
        TargetStruct::new(First, 0.23.into(), 0.333.into()),
        TargetStruct::new(Second, 1.23.into(), 1.333.into()),
        TargetStruct::new(Third, 2.23.into(), 2.333.into()),
        TargetStruct::new(Fourth, 3.23.into(), 3.333.into()),
    ];
    expected.reverse();

    for obj in parsed.into_iter() {
        assert_eq!(obj, expected.pop().unwrap());
    }
}

#[test]
fn test_parser1_array() {
    #[derive(FromXlsxRow, Debug, PartialEq)]
    struct TargetStruct {
        row_id: RowId,
        floats: [TestFloat; 2],
    }

    impl TargetStruct {
        pub fn new(row_id: RowId, floats: [TestFloat; 2]) -> Self {
            Self { row_id, floats }
        }
    }

    let mut workbook: epics_gen::XlsxWorkbook = epics_gen::open_workbook("tests/test_parser1.xlsx")
        .expect("xlsx file for this test is missing!");

    let parser = epics_gen::ParserBuilder::new(&mut workbook)
        .add_sheet("Sheet1")
        .expect("Sheet1 does not exist.")
        .add_table("test_table_1")
        .expect("test_table_1 table does not exist.")
        .build();

    let parsed: Vec<TargetStruct> = parser.parse().unwrap();

    use RowId::*;
    let mut expected: Vec<TargetStruct> = vec![
        TargetStruct::new(First, [0.23.into(), 0.333.into()]),
        TargetStruct::new(Second, [1.23.into(), 1.333.into()]),
        TargetStruct::new(Third, [2.23.into(), 2.333.into()]),
        TargetStruct::new(Fourth, [3.23.into(), 3.333.into()]),
    ];
    expected.reverse();

    for obj in parsed.into_iter() {
        assert_eq!(obj, expected.pop().unwrap());
    }
}

#[test]
fn test_invalid_names() {
    let mut workbook: epics_gen::XlsxWorkbook = epics_gen::open_workbook("tests/test_parser1.xlsx")
        .expect("xlsx file for this test is missing!");

    let builder = ParserBuilder::new(&mut workbook).add_sheet("Unexisting_Sheet");
    assert!(builder.is_err());

    let builder =
        ParserBuilder::new(&mut workbook).add_sheets(Regex::new(r#"$SomePattern\d+^"#).unwrap());
    assert!(builder.is_err());

    let builder = ParserBuilder::new(&mut workbook)
        .add_sheet("Sheet1")
        .expect("Sheet1 exists.")
        .add_table("Unexisting_Table");
    assert!(builder.is_err());

    let builder = ParserBuilder::new(&mut workbook)
        .add_sheet("Sheet1")
        .expect("Sheet1 exists.")
        .add_tables(Regex::new(r#"$TablePattern\d+"#).unwrap());
    assert!(builder.is_err());
}
