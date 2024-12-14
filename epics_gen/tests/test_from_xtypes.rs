use std::str::FromStr;

use epics_gen::{DataType, FromXlsxData, FromXlsxRow, XlsxData};
use epics_gen_macros::{FromXlsxFloat, FromXlsxString};

#[derive(FromXlsxString, strum_macros::EnumString, PartialEq, Eq, Debug)]
enum TestEnum {
    First,
    Second,
    Third,
}

#[test]
fn test_from_xlsx_string() {
    let result = TestEnum::from_xlsx_data(XlsxData::String("Third".into()));
    assert!(matches!(result, Ok(t) if t == TestEnum::Third));

    let result = TestEnum::from_xlsx_data(XlsxData::String("ThrowError".into()));
    assert!(matches!(result, Err(t) if t == epics_gen::ParseErrorKind::InvalidValue));

    let result = TestEnum::from_xlsx_data(XlsxData::Empty);
    assert!(matches!(result, Err(t) if t == epics_gen::ParseErrorKind::ValueMissing));
}

#[derive(FromXlsxFloat, PartialEq, Debug)]
struct TestFloat(f64);

impl From<f64> for TestFloat {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

#[test]
fn test_from_xlsx_float() {
    let result = TestFloat::from_xlsx_data(XlsxData::Float(0.1));
    assert!(matches!(result, Ok(t) if t == TestFloat(0.1)));

    let result = TestFloat::from_xlsx_data(XlsxData::String("ThrowError".into()));
    assert!(matches!(result, Err(t) if t == epics_gen::ParseErrorKind::ValueMissing));

    let result = TestFloat::from_xlsx_data(XlsxData::Empty);
    assert!(matches!(result, Err(t) if t == epics_gen::ParseErrorKind::ValueMissing));
}

#[test]
fn test_to_primitive() {
    let result = String::from_xlsx_data(XlsxData::String("Third".into()));
    assert!(matches!(result, Ok(t) if t == String::from_str("Third").unwrap()));

    let result = f64::from_xlsx_data(XlsxData::Float(0.1));
    assert!(matches!(result, Ok(t) if t == 0.1f64));
}

#[test]
fn test_from_xlsx_row() {
    #[derive(FromXlsxRow, PartialEq, Debug)]
    pub struct BuiltStruct {
        pub enm: TestEnum,
        pub flt: TestFloat,
    }
    let row: Vec<XlsxData> = vec![
        XlsxData::String("Second".into()),
        XlsxData::Float(std::f64::consts::PI),
    ];
    let parsed = BuiltStruct::from_xlsx_row(row, 0, "test_table").unwrap();

    assert_eq!(parsed.enm, TestEnum::Second);
    assert_eq!(parsed.flt, TestFloat(std::f64::consts::PI));

    // TODO: Test Optional arguments
}
