use dbgen_macros::AsRecord;

#[test]
fn test_as_record() {
    #[derive(AsRecord)]
    struct TestStruct {
        #[dbgen(record = r#"record(waveform, "$(P)Label-I") {{ field(INP, "{{const:"{}"}}") }}"#)]
        txt: &'static str,
        #[dbgen(record = r#"record(ao, "$(P)SomeOut") {{ field(VAL, "{}") }}"#)]
        val: f64,
    }

    let test_struct = TestStruct {
        txt: "Frac Syn",
        val: 0.5,
    };

    assert_eq!(
        test_struct.txt_as_record(),
        r#"record(waveform, "$(P)Label-I") { field(INP, "{const:"Frac Syn"}") }"#
    );

    assert_eq!(
        test_struct.val_as_record(),
        r#"record(ao, "$(P)SomeOut") { field(VAL, "0.5") }"#
    );

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
            record = r#"record(waveform, "$(P)$(MxcId)Label-I") {{ field(INP, "{{const:"{}"}}") }}"#
        )]
        txt: String,
        #[dbgen(record = r#"record(ao, "$(P)$(MxcId)SomeOut") {{ field(VAL, "{}") }}"#)]
        val: f64,
    }

    let test_struct = TestStruct {
        name: MxcId::Mxc0,
        txt: "Frac Syn".into(),
        val: 0.5,
    };

    assert_eq!(
        test_struct.txt_as_record(),
        r#"record(waveform, "$(P)$(MxcId)Label-I") { field(INP, "{const:"Frac Syn"}") }"#
    );

    assert_eq!(
        test_struct.val_as_record(),
        r#"record(ao, "$(P)$(MxcId)SomeOut") { field(VAL, "0.5") }"#
    );

    assert_eq!(
        test_struct.as_record(),
        r#"record(waveform, "$(P)Mxc0Label-I") { field(INP, "{const:"Frac Syn"}") }
record(ao, "$(P)Mxc0SomeOut") { field(VAL, "0.5") }
"#
    );
}

#[test]
fn test_attr_parsing() {
    #[derive(Copy, Clone, Debug, Default, PartialEq)]
    pub enum MxcId {
        #[default]
        Mxc0,
    }

    pub enum Pini {
        NO = 0,
        YES,
        RUN,
        RUNNING,
        PAUSE,
        PAUSED,
    }

    impl std::fmt::Display for MxcId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let res = match self {
                MxcId::Mxc0 => "Mxc0",
            };
            write!(f, "{}", res)
        }
    }

    // TODO: Implement this attribute set
    // #[derive(AsRecord)]
    // #[dbgen(rec_name = "$(P)$(MxcId)Label-I")]
    // #[dbgen(rec_type = "waveform")]
    // struct TestStruct {
    //     #[dbgen(field = "DESC")]
    //     name: MxcId,
    //     #[dbgen(field = "INP", fmt = r#"const: "{}""#)]
    //     txt: String,
    //     #[dbgen(field = "VAL", repr = "u8")]
    //     val: Pini,
    // }
}
