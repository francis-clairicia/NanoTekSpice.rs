use std::{fmt, str::FromStr};

#[derive(Debug, Clone)]
pub enum ParseComponentTypeError {
    UnknownComponentType(String),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ComponentType {
    /* Default components */
    Input,
    Output,
    Clock,
    True,
    False,
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
            _ => Err(Self::Err::UnknownComponentType(s.to_owned())),
        }
    }
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Input => write!(f, "input"),
            Self::Output => write!(f, "output"),
            Self::Clock => write!(f, "clock"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod test_input_type {
        use super::*;

        #[test]
        fn test_string_parse() {
            assert!(matches!(
                "input".parse::<ComponentType>(),
                Ok(ComponentType::Input)
            ));
        }

        #[test]
        fn test_to_string() {
            let component_type = ComponentType::Input;

            assert_eq!(component_type.to_string(), "input");
        }
    }

    mod test_output_type {
        use super::*;

        #[test]
        fn test_string_parse() {
            assert!(matches!(
                "output".parse::<ComponentType>(),
                Ok(ComponentType::Output)
            ));
        }

        #[test]
        fn test_to_string() {
            let component_type = ComponentType::Output;

            assert_eq!(component_type.to_string(), "output");
        }
    }

    mod test_clock_type {
        use super::*;

        #[test]
        fn test_string_parse() {
            assert!(matches!(
                "clock".parse::<ComponentType>(),
                Ok(ComponentType::Clock)
            ));
        }

        #[test]
        fn test_to_string() {
            let component_type = ComponentType::Clock;

            assert_eq!(component_type.to_string(), "clock");
        }
    }

    mod test_true_type {
        use super::*;

        #[test]
        fn test_string_parse() {
            assert!(matches!(
                "true".parse::<ComponentType>(),
                Ok(ComponentType::True)
            ));
        }

        #[test]
        fn test_to_string() {
            let component_type = ComponentType::True;

            assert_eq!(component_type.to_string(), "true");
        }
    }

    mod test_false_type {
        use super::*;

        #[test]
        fn test_string_parse() {
            assert!(matches!(
                "false".parse::<ComponentType>(),
                Ok(ComponentType::False)
            ));
        }

        #[test]
        fn test_to_string() {
            let component_type = ComponentType::False;

            assert_eq!(component_type.to_string(), "false");
        }
    }

    #[test]
    fn test_string_parse_unknown() {
        assert!(
            matches!("unknown".parse::<ComponentType>(), Err(ParseComponentTypeError::UnknownComponentType(given)) if given == "unknown")
        );
    }
}
