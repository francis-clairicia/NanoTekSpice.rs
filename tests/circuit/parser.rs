use std::fs::read_to_string;
use test_generator::test_resources;

use nanotekspice::{BuildErrorKind, Circuit, ParseCircuitError, SyntaxErrorKind};

#[test_resources("tests/.nts/input_output.nts")]
fn read_a_nts_file(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(content.parse::<Circuit>(), Ok(_)))
}

#[test_resources("tests/.nts/error/chipset_name_exists*.nts")]
fn chipset_name_override_error(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(
        content.parse::<Circuit>(),
        Err(ParseCircuitError::Build { line: _, kind: BuildErrorKind::ComponentNameOverride { name: _ } })
    ))
}

#[test_resources("tests/.nts/error/chipset_syntax_error*.nts")]
fn chipset_syntax_error(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(
        content.parse::<Circuit>(),
        Err(ParseCircuitError::Syntax { line: _, kind: SyntaxErrorKind::InvalidChipsetFormat })
    ))
}

#[test_resources("tests/.nts/error/chipset_type_unknown*.nts")]
fn chipset_type_unknown(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(
        content.parse::<Circuit>(),
        Err(ParseCircuitError::Build { line: _, kind: BuildErrorKind::ComponentTypeUnknown { value: _ } })
    ))
}

#[test_resources("tests/.nts/error/empty.nts")]
#[test_resources("tests/.nts/error/no_instructions.nts")]
fn empty(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(content.parse::<Circuit>(), Err(ParseCircuitError::Syntax { line: _, kind: SyntaxErrorKind::Empty })))
}

#[test_resources("tests/.nts/error/links_before_chipsets.nts")]
fn links_before_chipsets(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(
        content.parse::<Circuit>(),
        Err(ParseCircuitError::Syntax { line: _, kind: SyntaxErrorKind::FirstDeclarationMismatch })
    ))
}

#[test_resources("tests/.nts/error/links_name_unknown*.nts")]
fn links_name_unknown(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(
        content.parse::<Circuit>(),
        Err(ParseCircuitError::Build { line: _, kind: BuildErrorKind::ComponentNameUnknown { value: _ } })
    ))
}

#[test_resources("tests/.nts/error/links_pin_not_assignable*.nts")]
fn links_pin_not_assignable(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(
        content.parse::<Circuit>(),
        Err(ParseCircuitError::Build {
            line: _,
            kind: BuildErrorKind::ComponentLinkIssue { name: _, component_type: _, pin: _ }
        })
    ))
}

#[test_resources("tests/.nts/error/links_pin_not_number*.nts")]
fn links_pin_not_number(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(
        content.parse::<Circuit>(),
        Err(ParseCircuitError::Syntax { line: _, kind: SyntaxErrorKind::InvalidLinkPin { pin: _ } })
    ))
}

#[test_resources("tests/.nts/error/links_syntax_error*.nts")]
fn links_syntax_error(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(
        content.parse::<Circuit>(),
        Err(ParseCircuitError::Syntax { line: _, kind: SyntaxErrorKind::InvalidLinkFormat })
    ))
}

#[test_resources("tests/.nts/error/no_chipsets.nts")]
fn no_chipsets(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(content.parse::<Circuit>(), Err(ParseCircuitError::Build { line: _, kind: BuildErrorKind::NoChipset })))
}

#[test_resources("tests/.nts/error/redeclaration_*.nts")]
fn redeclaration(resource: &str) {
    let content = read_to_string(resource).unwrap();

    assert!(matches!(
        content.parse::<Circuit>(),
        Err(ParseCircuitError::Syntax { line: _, kind: SyntaxErrorKind::DeclarationDuplicate { declaration: _ } })
    ))
}
