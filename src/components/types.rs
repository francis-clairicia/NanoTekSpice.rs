use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy)]
pub enum ParseComponentTypeError {
    InvalidValue,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ComponentType {
    /* Default components */
    Input,
    Output,
    Clock,
    True,
    False,
    /* Gates */
    C4001, // NOR
    C4011, // NAND
    C4030, // XOR
    C4069, // NOT
    C4071, // OR
    C4081, // AND
}

impl FromStr for ComponentType {
    type Err = ParseComponentTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "input" => Ok(Self::Input),
            "output" => Ok(Self::Output),
            "clock" => Ok(Self::Clock),
            "true" => Ok(Self::True),
            "false" => Ok(Self::False),
            "4001" => Ok(Self::C4001),
            "4011" => Ok(Self::C4011),
            "4030" => Ok(Self::C4030),
            "4069" => Ok(Self::C4069),
            "4071" => Ok(Self::C4071),
            "4081" => Ok(Self::C4081),
            _ => Err(Self::Err::InvalidValue),
        }
    }
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => write!(f, "input"),
            Self::Output => write!(f, "output"),
            Self::Clock => write!(f, "clock"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::C4001 => write!(f, "4001"),
            Self::C4011 => write!(f, "4011"),
            Self::C4030 => write!(f, "4030"),
            Self::C4069 => write!(f, "4069"),
            Self::C4071 => write!(f, "4071"),
            Self::C4081 => write!(f, "4081"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentType, ParseComponentTypeError};

    macro_rules! tests_suite_for_type {
        ($test_suite:ident, $component_type_name:expr, $component_type:ident) => {
            mod $test_suite {
                use super::*;

                #[test]
                fn test_string_parse() {
                    let component_type_name: &'static str = $component_type_name;

                    assert!(matches!(component_type_name.parse::<ComponentType>(), Ok(ComponentType::$component_type)));
                }

                #[test]
                fn test_to_string() {
                    let component_type = ComponentType::$component_type;
                    let component_type_name: &'static str = $component_type_name;

                    assert_eq!(component_type.to_string(), component_type_name);
                }
            }
        };
    }

    tests_suite_for_type!(input, "input", Input);

    tests_suite_for_type!(output, "output", Output);

    tests_suite_for_type!(clock, "clock", Clock);

    tests_suite_for_type!(r#true, "true", True);

    tests_suite_for_type!(r#false, "false", False);

    tests_suite_for_type!(component_4001, "4001", C4001);

    tests_suite_for_type!(component_4011, "4011", C4011);

    tests_suite_for_type!(component_4030, "4030", C4030);

    tests_suite_for_type!(component_4069, "4069", C4069);

    tests_suite_for_type!(component_4071, "4071", C4071);

    tests_suite_for_type!(component_4081, "4081", C4081);

    #[test]
    fn test_string_parse_unknown() {
        assert!(matches!("unknown".parse::<ComponentType>(), Err(ParseComponentTypeError::InvalidValue)));
    }
}
