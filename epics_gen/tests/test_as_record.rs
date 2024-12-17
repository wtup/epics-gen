use epics_gen_macros::AsRecord;

#[test]
fn test_as_record_single() {
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

    let test_struct = TestStruct {
        desc: "Output Voltage",
        egu: "V",
        val: 0.5,
    };

    assert_eq!(
        test_struct.as_record(),
        r#"record(ao, "$(P)Voltage") {
  field(DESC, "Output Voltage")
  field(EGU, "V")
  field(VAL, "0.5")
}
"#
    );
}

#[test]
fn test_as_record_multiple() {
    #[derive(AsRecord)]
    struct TestStruct {
        #[record(rec_name = "$(P)Voltage", rec_type = "ao")]
        #[record(field = "VAL")]
        voltage: f64,
        #[record(rec_name = "$(P)Current", rec_type = "ao")]
        #[record(field = "VAL")]
        current: f64,
        #[record(rec_name = "$(P)SlewRate", rec_type = "ao")]
        #[record(field = "VAL")]
        slew_rate: f64,
    }

    let test_struct = TestStruct {
        voltage: 5.5,
        current: 0.5,
        slew_rate: 0.05,
    };

    assert_eq!(
        test_struct.as_record(),
        r#"record(ao, "$(P)Voltage") {
  field(VAL, "5.5")
}
record(ao, "$(P)Current") {
  field(VAL, "0.5")
}
record(ao, "$(P)SlewRate") {
  field(VAL, "0.05")
}
"#
    );
}

#[test]
fn test_as_record_fmt() {
    #[derive(AsRecord)]
    struct TestStruct {
        #[record(fmt = r#"record(waveform, "$(P)Label-I") {{ field(INP, "{{const:"{}"}}") }}"#)]
        txt: &'static str,
        #[record(fmt = r#"record(ao, "$(P)SomeOut") {{ field(VAL, "{}") }}"#)]
        val: f64,
    }

    let test_struct = TestStruct {
        txt: "Frac Syn",
        val: 0.5,
    };

    println!("test_struct: {}", test_struct.as_record());
    assert_eq!(
        test_struct.as_record(),
        r#"record(waveform, "$(P)Label-I") { field(INP, "{const:"Frac Syn"}") }
record(ao, "$(P)SomeOut") { field(VAL, "0.5") }
"#
    );
}

#[test]
fn test_as_record_named() {
    #[derive(Copy, Clone, Debug, Default, PartialEq)]
    pub enum MxcId {
        #[default]
        Mxc0,
    }

    impl std::fmt::Display for MxcId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let res = match self {
                MxcId::Mxc0 => "Mxc0",
            };
            write!(f, "{}", res)
        }
    }

    #[derive(AsRecord)]
    struct TestStruct {
        #[record(subst = "$(MxcId)")]
        name: MxcId,
        #[record(
            fmt = r#"record(waveform, "$(P)$(MxcId)Label-I") {{ field(INP, "{{const:"{}"}}") }}"#
        )]
        txt: String,
        #[record(fmt = r#"record(ao, "$(P)$(MxcId)SomeOut") {{ field(VAL, "{}") }}"#)]
        val: f64,
    }

    let test_struct = TestStruct {
        name: MxcId::Mxc0,
        txt: "Frac Syn".into(),
        val: 0.5,
    };

    assert_eq!(
        test_struct.as_record(),
        r#"record(waveform, "$(P)Mxc0Label-I") { field(INP, "{const:"Frac Syn"}") }
record(ao, "$(P)Mxc0SomeOut") { field(VAL, "0.5") }
"#
    );
}

#[test]
fn test_as_record_repr() {
    #[derive(Clone, Debug, Default, PartialEq)]
    enum MxcId {
        #[default]
        Mxc0,
        Mxc1,
        Mxc2,
    }

    #[derive(AsRecord)]
    struct TestStruct {
        #[record(rec_name = "TestRec", rec_type = "ao", field = "VAL", repr = u8)]
        name: MxcId,
    }

    let test_struct = TestStruct { name: MxcId::Mxc2 };

    assert_eq!(
        test_struct.as_record(),
        r#"record(ao, "TestRec") {
  field(VAL, "2")
}
"#
    );
}
