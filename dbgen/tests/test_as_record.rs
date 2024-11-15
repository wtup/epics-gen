use dbgen_macros::AsRecord;

#[test]
fn test_as_record_global() {
    #[derive(AsRecord)]
    #[dbgen(rec_name = "$(P)Voltage", rec_type = "ao")]
    struct TestStruct {
        #[dbgen(field = "DESC")]
        desc: &'static str,
        #[dbgen(field = "EGU")]
        egu: &'static str,
        #[dbgen(field = "VAL")]
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
fn test_as_record_local() {
    #[derive(AsRecord)]
    struct TestStruct {
        #[dbgen(rec_name = "$(P)Voltage", rec_type = "ao")]
        #[dbgen(field = "VAL")]
        voltage: f64,
        #[dbgen(rec_name = "$(P)Current", rec_type = "ao")]
        #[dbgen(field = "VAL")]
        current: f64,
        #[dbgen(rec_name = "$(P)SlewRate", rec_type = "ao")]
        #[dbgen(field = "VAL")]
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
        #[dbgen(fmt = r#"record(waveform, "$(P)Label-I") {{ field(INP, "{{const:"{}"}}") }}"#)]
        txt: &'static str,
        #[dbgen(fmt = r#"record(ao, "$(P)SomeOut") {{ field(VAL, "{}") }}"#)]
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
        #[dbgen(subst = "$(MxcId)")]
        name: MxcId,
        #[dbgen(
            fmt = r#"record(waveform, "$(P)$(MxcId)Label-I") {{ field(INP, "{{const:"{}"}}") }}"#
        )]
        txt: String,
        #[dbgen(fmt = r#"record(ao, "$(P)$(MxcId)SomeOut") {{ field(VAL, "{}") }}"#)]
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

// // TODO: Add test for global record
// #[test]
// fn test_as_record_attr_parsing() {
//     #[derive(Copy, Clone, Debug, Default, PartialEq)]
//     pub enum MxcId {
//         #[default]
//         Mxc0,
//     }
//
//     pub enum Pini {
//         NO = 0,
//         YES,
//         RUN,
//         RUNNING,
//         PAUSE,
//         PAUSED,
//     }
//
//     impl std::fmt::Display for MxcId {
//         fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//             let res = match self {
//                 MxcId::Mxc0 => "Mxc0",
//             };
//             write!(f, "{}", res)
//         }
//     }
// }
