use std::collections::{HashMap, HashSet};

use crate::components::PinNumber;

use super::builder::{CircuitBuildError, CircuitBuilder};
use super::Circuit;

static CHIPSET_DECLARATION: &str = ".chipsets:";
static LINK_DECLARATION: &str = ".links:";

#[derive(Debug, Clone)]
pub enum ParseCircuitError {
    Syntax { line: usize, kind: SyntaxErrorKind },
    Build { line: usize, kind: BuildErrorKind },
}

#[derive(Debug, Clone)]
pub enum SyntaxErrorKind {
    InvalidChipsetFormat,
    InvalidLinkFormat,
    InvalidLinkPin { pin: String },
    FirstDeclarationMismatch,
    DeclarationDuplicate { declaration: String },
    Empty,
}

#[derive(Debug, Clone)]
pub enum BuildErrorKind {
    NoChipset,
    ComponentNameOverride {
        name: String,
    },
    ComponentNameUnknown {
        value: String,
    },
    ComponentTypeUnknown {
        value: String,
    },
    ComponentLinkIssue {
        name: String,
        component_type: String,
        pin: PinNumber,
    },
}

impl std::fmt::Display for ParseCircuitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax { line, kind } => write!(f, "line {line}: Syntax error: {kind}"),
            Self::Build { line, kind } => write!(f, "line {line}: Build error: {kind}"),
        }
    }
}

impl std::fmt::Display for SyntaxErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidChipsetFormat => {
                write!(f, "Chipset declaration must respect this form: type name")
            }
            Self::InvalidLinkFormat => write!(
                f,
                "Link declaration must respect this form: name1:pin1 name2:pin2"
            ),
            Self::InvalidLinkPin { pin } => {
                write!(f, "\"{pin}\" is not a valid pin number")
            }
            Self::FirstDeclarationMismatch => {
                write!(f, "The first instruction must be the chipsets declaration")
            }
            Self::DeclarationDuplicate { declaration } => {
                write!(f, "Redeclaration of \"{declaration}\"")
            }
            Self::Empty => write!(f, "There is no instructions inside content"),
        }
    }
}

impl std::fmt::Display for BuildErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoChipset => write!(f, "No chipset in the circuit."),
            Self::ComponentNameOverride { name } => {
                write!(f, "A component with name \"{name}\" already exists.")
            }
            Self::ComponentNameUnknown { value } => {
                write!(f, "Unknown component name \"{value}\".")
            }
            Self::ComponentTypeUnknown { value } => {
                write!(f, "Unknown component type \"{value}\".")
            }
            Self::ComponentLinkIssue {
                name,
                component_type,
                pin,
            } => {
                write!(
                    f,
                    "\"{name}\": {component_type} component does not have pin {pin}."
                )
            }
        }
    }
}

impl<Type> From<CircuitBuildError<'_, Type>> for BuildErrorKind
where
    Type: std::fmt::Debug + Clone + ToString,
{
    fn from(value: CircuitBuildError<'_, Type>) -> Self {
        match value {
            CircuitBuildError::NoChipset => BuildErrorKind::NoChipset,
            CircuitBuildError::ComponentNameOverride(name) => {
                BuildErrorKind::ComponentNameOverride {
                    name: name.to_owned(),
                }
            }
            CircuitBuildError::ComponentNameUnknown(value) => {
                BuildErrorKind::ComponentNameUnknown {
                    value: value.to_owned(),
                }
            }
            CircuitBuildError::ComponentTypeUnknown(value) => {
                BuildErrorKind::ComponentTypeUnknown {
                    value: value.to_owned(),
                }
            }
            CircuitBuildError::ComponentLinkIssue(name, component_type, pin) => {
                BuildErrorKind::ComponentLinkIssue {
                    name: name.to_owned(),
                    component_type: component_type.to_string(),
                    pin,
                }
            }
        }
    }
}

pub struct Parser;

impl Parser {
    pub fn read(input: &str) -> Result<Circuit, ParseCircuitError> {
        let lines = Self::parse_lines(input)
            .map_err(|(line, kind)| ParseCircuitError::Syntax { line, kind })?;

        let mut builder = CircuitBuilder::default();

        for line in lines.into_iter() {
            let build_result = match line.instruction {
                Instruction::AddComponent {
                    name,
                    component_type,
                } => builder.add_component(component_type, name),
                Instruction::LinkComponents {
                    left_name,
                    left_pin,
                    right_name,
                    right_pin,
                } => builder.link_components(left_name, left_pin, right_name, right_pin),
            };

            builder = build_result.map_err(|err| ParseCircuitError::Build {
                line: line.index,
                kind: err.into(),
            })?;
        }

        builder.build().map_err(|err| ParseCircuitError::Build {
            line: 0,
            kind: err.into(),
        })
    }

    fn parse_lines<'a>(input: &'a str) -> Result<Vec<Line<'a>>, (usize, SyntaxErrorKind)> {
        let mut output: Vec<Line<'a>> = Vec::new();

        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        enum Declaration {
            Chipsets,
            Links,
        }

        let mut current_declaration: Option<Declaration> = None;
        let mut already_declared: HashSet<Declaration> = HashSet::new();

        let initializers: HashMap<&str, Declaration> = HashMap::from([
            (CHIPSET_DECLARATION, Declaration::Chipsets),
            (LINK_DECLARATION, Declaration::Links),
        ]);

        for (index, content) in input.lines().enumerate() {
            let index = index + 1;
            let content = if let Some(comment_idx) = content.find('#') {
                &content[..comment_idx]
            } else {
                content
            };
            let content = content.trim();
            if content.is_empty() {
                continue;
            }

            if let Some(&declaration) = initializers.get(content) {
                if !already_declared.insert(declaration) {
                    return Err((
                        index,
                        SyntaxErrorKind::DeclarationDuplicate {
                            declaration: content.to_owned(),
                        },
                    ));
                }
                if current_declaration.is_none() && declaration != Declaration::Chipsets {
                    return Err((index, SyntaxErrorKind::FirstDeclarationMismatch));
                }
                current_declaration = Some(declaration);
            } else {
                let instruction: Result<Instruction<'a>, SyntaxErrorKind> =
                    match current_declaration {
                        Some(Declaration::Chipsets) => Self::parse_chipset_line(content),
                        Some(Declaration::Links) => Self::parse_link_line(content),
                        None => Err(SyntaxErrorKind::FirstDeclarationMismatch),
                    };

                let instruction = instruction.map_err(|kind| (index, kind))?;

                output.push(Line { index, instruction })
            }
        }

        if already_declared.is_empty() {
            return Err((0, SyntaxErrorKind::Empty));
        }

        Ok(output)
    }

    fn parse_chipset_line<'a>(content: &'a str) -> Result<Instruction<'a>, SyntaxErrorKind> {
        let content: Vec<&str> = content.split_whitespace().collect();

        if let [component_type, component_name] = content[..] {
            Ok(Instruction::AddComponent {
                name: component_name,
                component_type,
            })
        } else {
            Err(SyntaxErrorKind::InvalidChipsetFormat)
        }
    }

    fn parse_link_line<'a>(content: &'a str) -> Result<Instruction<'a>, SyntaxErrorKind> {
        let content: Vec<&str> = content.split_whitespace().collect();
        if let [left_component_link, right_component_link] = content[..] {
            fn parse_simple_link<'a>(
                content: &'a str,
            ) -> Result<(&'a str, PinNumber), SyntaxErrorKind> {
                let content: Vec<&str> = content.split(':').collect();

                if let [component_name, component_pin] = content[..] {
                    Ok((
                        component_name,
                        component_pin.parse::<PinNumber>().map_err(|_| {
                            SyntaxErrorKind::InvalidLinkPin {
                                pin: component_pin.to_owned(),
                            }
                        })?,
                    ))
                } else {
                    Err(SyntaxErrorKind::InvalidLinkFormat)
                }
            }

            let (left_name, left_pin) = parse_simple_link(left_component_link)?;
            let (right_name, right_pin) = parse_simple_link(right_component_link)?;

            Ok(Instruction::LinkComponents {
                left_name,
                left_pin,
                right_name,
                right_pin,
            })
        } else {
            Err(SyntaxErrorKind::InvalidLinkFormat)
        }
    }
}

struct Line<'a> {
    pub index: usize,
    pub instruction: Instruction<'a>,
}

enum Instruction<'a> {
    AddComponent {
        name: &'a str,
        component_type: &'a str,
    },
    LinkComponents {
        left_name: &'a str,
        left_pin: PinNumber,
        right_name: &'a str,
        right_pin: PinNumber,
    },
}
