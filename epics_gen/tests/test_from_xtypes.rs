use epics_gen::{DataType, FromXlsxRow, XlsxData};
use epics_gen_macros::{FromXlsxFloat, FromXlsxRow, FromXlsxString};

#[derive(FromXlsxString, strum_macros::EnumString, PartialEq, Eq, Debug)]
enum TestEnum {
    First,
    Second,
    Third,
}

#[test]
fn test_from_xlsx_string() {
    let result = TestEnum::try_from(XlsxData::String("Fourth".into()));
    assert!(matches!(result, Ok(t) if t == TestEnum::Third));

    let result = TestEnum::try_from(XlsxData::String("ThrowError".into()));
    assert!(matches!(result, Err(t) if t == epics_gen::ParseErrorKind::InvalidValue));

    let result = TestEnum::try_from(XlsxData::Empty);
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
    let result = TestFloat::try_from(XlsxData::Float(0.1));
    assert!(matches!(result, Ok(t) if t == TestFloat(0.1)));

    let result = TestFloat::try_from(XlsxData::String("ThrowError".into()));
    assert!(matches!(result, Err(t) if t == epics_gen::ParseErrorKind::ValueMissing));

    let result = TestFloat::try_from(XlsxData::Empty);
    assert!(matches!(result, Err(t) if t == epics_gen::ParseErrorKind::ValueMissing));
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
