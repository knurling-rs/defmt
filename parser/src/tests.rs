use rstest::rstest;

use super::*;

#[rstest]
#[case::noo_param("", None, Type::Format, None)]
#[case::one_param_type("=u8", None, Type::U8, None)]
#[case::one_param_hint(":a", None, Type::Format, Some(DisplayHint::Ascii))]
#[case::one_param_index("1", Some(1), Type::Format, None)]
#[case::two_param_type_hint("=u8:x", None, Type::U8, Some(DisplayHint::Hexadecimal {alternate: false, uppercase: false, zero_pad: 0}))]
#[case::two_param_index_type("0=u8", Some(0), Type::U8, None)]
#[case::two_param_index_hint("0:a", Some(0), Type::Format, Some(DisplayHint::Ascii))]
#[case::two_param_type_hint("=[u8]:#04x", None, Type::U8Slice, Some(DisplayHint::Hexadecimal {alternate: true, uppercase: false, zero_pad: 4}))]
#[case::all_param("1=u8:b", Some(1), Type::U8, Some(DisplayHint::Binary { alternate: false, zero_pad: 0}))]
fn all_parse_param_cases(
    #[case] input: &str,
    #[case] index: Option<usize>,
    #[case] ty: Type,
    #[case] hint: Option<DisplayHint>,
) {
    assert_eq!(
        parse_param(input, ParserMode::Strict),
        Ok(Param {
            name: None,
            index,
            ty,
            hint
        })
    );
}

#[rstest]
#[case(":a", DisplayHint::Ascii)]
#[case(":b", DisplayHint::Binary { alternate: false, zero_pad: 0 })]
#[case(":#b", DisplayHint::Binary { alternate: true, zero_pad: 0 })]
#[case(":o", DisplayHint::Octal { alternate: false, zero_pad: 0 })]
#[case(":#o", DisplayHint::Octal { alternate: true, zero_pad: 0 })]
#[case(":x", DisplayHint::Hexadecimal { alternate: false, uppercase: false, zero_pad: 0 })]
#[case(":02x", DisplayHint::Hexadecimal { alternate: false, uppercase: false, zero_pad: 2 })]
#[case(":#x", DisplayHint::Hexadecimal { alternate: true, uppercase: false, zero_pad: 0 })]
#[case(":#04x", DisplayHint::Hexadecimal { alternate: true, uppercase: false, zero_pad: 4 })]
#[case(":X", DisplayHint::Hexadecimal { alternate: false, uppercase: true, zero_pad: 0 })]
#[case(":#X", DisplayHint::Hexadecimal { alternate: true, uppercase: true, zero_pad: 0 })]
#[case(":ms", DisplayHint::Seconds(TimePrecision::Millis))]
#[case(":us", DisplayHint::Seconds(TimePrecision::Micros))]
#[case(":ts", DisplayHint::Time(TimePrecision::Seconds))]
#[case(":tms", DisplayHint::Time(TimePrecision::Millis))]
#[case(":tus", DisplayHint::Time(TimePrecision::Micros))]
#[case(":iso8601ms", DisplayHint::ISO8601(TimePrecision::Millis))]
#[case(":iso8601s", DisplayHint::ISO8601(TimePrecision::Seconds))]
#[case(":?", DisplayHint::Debug)]
#[case(":02", DisplayHint::NoHint { zero_pad: 2 })]
fn all_display_hints(#[case] input: &str, #[case] hint: DisplayHint) {
    assert_eq!(
        parse_param(input, ParserMode::Strict),
        Ok(Param {
            name: None,
            index: None,
            ty: Type::Format,
            hint: Some(hint),
        })
    );
}

#[test]
// separate test, because of `ParserMode::ForwardsCompatible`
fn display_hint_unknown() {
    assert_eq!(
        parse_param(":unknown", ParserMode::ForwardsCompatible),
        Ok(Param {
            name: None,
            index: None,
            ty: Type::Format,
            hint: Some(DisplayHint::Unknown("unknown".to_string())),
        })
    );
}

#[rstest]
#[case("=i8", Type::I8)]
#[case("=i16", Type::I16)]
#[case("=i32", Type::I32)]
#[case("=i64", Type::I64)]
#[case("=i128", Type::I128)]
#[case("=isize", Type::Isize)]
#[case("=u8", Type::U8)]
#[case("=u16", Type::U16)]
#[case("=u32", Type::U32)]
#[case("=u64", Type::U64)]
#[case("=u128", Type::U128)]
#[case("=usize", Type::Usize)]
#[case("=f32", Type::F32)]
#[case("=f64", Type::F64)]
#[case("=bool", Type::Bool)]
#[case("=?", Type::Format)]
#[case("=str", Type::Str)]
#[case("=[u8]", Type::U8Slice)]
fn all_types(#[case] input: &str, #[case] ty: Type) {
    assert_eq!(
        parse_param(input, ParserMode::Strict),
        Ok(Param {
            name: None,
            index: None,
            ty,
            hint: None,
        })
    );
}

#[rstest]
#[case::implicit("{=u8}{=u16}", [(0, Type::U8), (1, Type::U16)])]
#[case::single_parameter_formatted_twice("{=u8}{0=u8}", [(0, Type::U8), (0, Type::U8)])]
#[case::explicit_index("{=u8}{1=u16}", [(0, Type::U8), (1, Type::U16)])]
#[case::reversed_order("{1=u8}{0=u16}", [(1, Type::U8), (0, Type::U16)])]
fn index(#[case] input: &str, #[case] params: [(usize, Type); 2]) {
    assert_eq!(
        parse(input, ParserMode::Strict),
        Ok(vec![
            Fragment::Parameter(Parameter {
                name: None,
                index: params[0].0,
                ty: params[0].1.clone(),
                hint: None,
            }),
            Fragment::Parameter(Parameter {
                name: None,
                index: params[1].0,
                ty: params[1].1.clone(),
                hint: None,
            }),
        ])
    );
}

#[rstest]
#[case::named_only("{owner}{count}", [(0, Some("owner"), Type::Format), (1, Some("count"), Type::Format)])]
#[case::named_repeated("{owner}{owner}", [(0, Some("owner"), Type::Format), (0, Some("owner"), Type::Format)])]
#[case::named_after_implicit("{}{owner}", [(0, None, Type::Format), (1, Some("owner"), Type::Format)])]
#[case::named_after_explicit("{owner}{1=u8}{0=u16}", [(2, Some("owner"), Type::Format), (1, None, Type::U8)])]
#[case::named_with_type_and_hint("{owner=u8:#x}{count=u16}", [(0, Some("owner"), Type::U8), (1, Some("count"), Type::U16)])]
fn named(#[case] input: &str, #[case] params: [(usize, Option<&str>, Type); 2]) {
    let parsed = parse(input, ParserMode::Strict).unwrap();
    let parsed_params = parsed
        .iter()
        .filter_map(|frag| match frag {
            Fragment::Parameter(param) => {
                Some((param.index, param.name.as_deref(), param.ty.clone()))
            }
            Fragment::Literal(_) => None,
        })
        .take(2)
        .collect::<Vec<_>>();
    assert_eq!(
        parsed_params,
        params
            .iter()
            .map(|(index, name, ty)| (*index, *name, ty.clone()))
            .collect::<Vec<_>>()
    );
}

#[rstest]
#[case::debug_hint("{owner:?}")]
#[case::zero_pad_hint("{owner:08}")]
fn named_with_debug_hint(#[case] input: &str) {
    // `{owner:?}` (std's Debug syntax) must parse as a named parameter of the default
    // (`Format`) type, making std format strings work unchanged.
    let parsed = parse(input, ParserMode::Strict).unwrap();
    let Fragment::Parameter(param) = &parsed[0] else {
        panic!("expected parameter, got {parsed:?}");
    };
    assert_eq!(param.index, 0);
    assert_eq!(param.name.as_deref(), Some("owner"));
    assert_eq!(param.ty, Type::Format);
}

#[rstest]
#[case("{=0..4}", 0..4)]
#[case::just_inside_128bit_range_1("{=0..128}", 0..128)]
#[case::just_inside_128bit_range_2("{=127..128}", 127..128)]
fn range(#[case] input: &str, #[case] bit_field: Range<u8>) {
    assert_eq!(
        parse(input, ParserMode::Strict),
        Ok(vec![Fragment::Parameter(Parameter {
            name: None,
            index: 0,
            ty: Type::BitField(bit_field),
            hint: None,
        })])
    );
}

#[test]
fn multiple_ranges() {
    assert_eq!(
        parse("{0=30..31}{1=0..4}{1=2..6}", ParserMode::Strict),
        Ok(vec![
            Fragment::Parameter(Parameter {
                name: None,
                index: 0,
                ty: Type::BitField(30..31),
                hint: None,
            }),
            Fragment::Parameter(Parameter {
                name: None,
                index: 1,
                ty: Type::BitField(0..4),
                hint: None,
            }),
            Fragment::Parameter(Parameter {
                name: None,
                index: 1,
                ty: Type::BitField(2..6),
                hint: None,
            }),
        ])
    );
}

#[rstest]
#[case("{=[u8; 0]}", 0)]
#[case::space_is_optional("{=[u8;42]}", 42)]
#[case::multiple_spaces_are_ok("{=[u8;    257]}", 257)]
fn arrays(#[case] input: &str, #[case] length: usize) {
    assert_eq!(
        parse(input, ParserMode::Strict),
        Ok(vec![Fragment::Parameter(Parameter {
            name: None,
            index: 0,
            ty: Type::U8Array(length),
            hint: None,
        })])
    );
}

#[rstest]
#[case::no_tabs("{=[u8; \t 3]}")]
#[case::no_linebreaks("{=[u8; \n 3]}")]
#[case::too_large("{=[u8; 9999999999999999999999999]}")]
fn arrays_err(#[case] input: &str) {
    assert!(parse(input, ParserMode::Strict).is_err());
}

#[rstest]
#[case("{=dunno}", Error::InvalidTypeSpecifier("dunno".to_string()))]
#[case("{=u8;x}", Error::InvalidTypeSpecifier("u8;x".to_string()))]
#[case("{0dunno}", Error::UnexpectedContentInFormatString("dunno".to_string()))]
#[case("{dunno.field}", Error::UnexpectedContentInFormatString(".field".to_string()))]
#[case("{_}", Error::InvalidArgumentName("_".to_string()))]
#[case("{:}", Error::MalformedFormatString)]
#[case::stray_braces_1("}string", Error::UnmatchedCloseBracket)]
#[case::stray_braces_2("{string", Error::UnmatchedOpenBracket)]
#[case::stray_braces_3("}", Error::UnmatchedCloseBracket)]
#[case::stray_braces_4("{", Error::UnmatchedOpenBracket)]
#[case::range_empty("{=0..0}", Error::InvalidTypeSpecifier("0..0".to_string()))]
#[case::range_start_gt_end("{=1..0}", Error::InvalidTypeSpecifier("1..0".to_string()))]
#[case::range_out_of_128bit_1("{=0..129}", Error::InvalidTypeSpecifier("0..129".to_string()))]
#[case::range_out_of_128bit_2("{=128..128}", Error::InvalidTypeSpecifier("128..128".to_string()))]
#[case::range_missing_parts_1("{=0..4", Error::UnmatchedOpenBracket)]
#[case::range_missing_parts_2("{=0..}", Error::InvalidTypeSpecifier("0..".to_string()))]
#[case::range_missing_parts_3("{=..4}", Error::InvalidTypeSpecifier("..4".to_string()))]
#[case::range_missing_parts_4("{=0.4}", Error::InvalidTypeSpecifier("0.4".to_string()))]
#[case::range_missing_parts_5("{=0...4}", Error::InvalidTypeSpecifier("0...4".to_string()))]
#[case::index_with_different_types(
    "{0=u8}{0=u16}",
    Error::ConflictingTypes(0, Type::U8, Type::U16)
)]
#[case::index_with_different_types_bool_is_autoassigned_index_0(
    "Hello {1=u16} {0=u8} {=bool}",
    Error::ConflictingTypes(0, Type::U8, Type::Bool)
)]
#[case::index_0_is_omitted("{1=u8}", Error::UnusedArgument(0))]
#[case::index_1_is_missing("{2=u8}{=u16}", Error::UnusedArgument(1))]
#[case::index_0_is_missing("{2=u8}{1=u16}", Error::UnusedArgument(0))]
fn error_msg(#[case] input: &str, #[case] err: Error) {
    assert_eq!(parse(input, ParserMode::Strict), Err(err));
}

#[rstest]
#[case("}}", "}")]
#[case("{{", "{")]
#[case("literal{{literal", "literal{literal")]
#[case("literal}}literal", "literal}literal")]
#[case("{{}}", "{}")]
#[case("}}{{", "}{")]
fn escaped_braces(#[case] input: &str, #[case] literal: &str) {
    assert_eq!(
        parse(input, ParserMode::Strict),
        Ok(vec![Fragment::Literal(literal.into())])
    );
}
