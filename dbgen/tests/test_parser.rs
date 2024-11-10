use dbgen::{open_workbook, DataType, Parser};
use dbgen_macros::{FromXlsxFloat, FromXlsxRow, FromXlsxString};
use regex::Regex;

#[test]
fn test_parser1() {
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

    let mut workbook: dbgen::XlsxWorkbook = dbgen::open_workbook("tests/test_parser1.xlsx")
        .expect("xlsx file for this test is missing!");

    let parser = dbgen::ParserBuilder::new(&mut workbook)
        .add_sheet("Sheet1")
        .add_table("test_table_1")
        .build();

    let parsed: Vec<TargetStruct> = parser.parse();

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
