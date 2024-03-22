use std::fmt;
use std::ops;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Tristate {
    State(bool),
    Undefined,
}

#[derive(Debug)]
pub enum ParseTristateError {
    UnknownValue(String),
}

impl FromStr for Tristate {
    type Err = ParseTristateError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::State(false)),
            "1" => Ok(Self::State(true)),
            "U" => Ok(Self::Undefined),
            _ => Err(Self::Err::UnknownValue(s.to_owned())),
        }
    }
}

impl From<bool> for Tristate {
    #[inline]
    fn from(value: bool) -> Self {
        Self::State(value)
    }
}

impl Default for Tristate {
    #[inline]
    fn default() -> Self {
        Self::Undefined
    }
}

impl fmt::Display for Tristate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::State(false) => write!(f, "0"),
            Self::State(true) => write!(f, "1"),
            Self::Undefined => write!(f, "U"),
        }
    }
}

impl ops::BitOr for Tristate {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::State(lhs), Self::State(rhs)) => Self::State(lhs | rhs),
            (Self::State(true), _) | (_, Self::State(true)) => Self::State(true),
            _ => Self::Undefined,
        }
    }
}

impl ops::BitOrAssign for Tristate {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl ops::BitAnd for Tristate {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::State(lhs), Self::State(rhs)) => Self::State(lhs & rhs),
            (Self::State(false), _) | (_, Self::State(false)) => Self::State(false),
            _ => Self::Undefined,
        }
    }
}

impl ops::BitAndAssign for Tristate {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl ops::BitXor for Tristate {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::State(lhs), Self::State(rhs)) => Self::State(lhs ^ rhs),
            _ => Self::Undefined,
        }
    }
}

impl ops::BitXorAssign for Tristate {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs
    }
}

impl ops::Not for Tristate {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        if let Self::State(state) = self {
            Self::State(!state)
        } else {
            Self::Undefined
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod test_false_state {
        use super::*;

        #[test]
        fn test_string_parse() {
            assert!(matches!(
                "0".parse::<Tristate>(),
                Ok(Tristate::State(false))
            ));
        }

        #[test]
        fn test_to_string() {
            let state = Tristate::State(false);

            assert_eq!(state.to_string(), "0");
        }

        #[test]
        fn test_true_into() {
            let state: Tristate = false.into();

            assert_eq!(state, Tristate::State(false))
        }
    }

    mod test_true_state {
        use super::*;

        #[test]
        fn test_string_parse() {
            assert!(matches!("1".parse::<Tristate>(), Ok(Tristate::State(true))));
        }

        #[test]
        fn test_to_string() {
            let state = Tristate::State(true);

            assert_eq!(state.to_string(), "1");
        }

        #[test]
        fn test_false_into() {
            let state: Tristate = true.into();

            assert_eq!(state, Tristate::State(true))
        }
    }

    mod test_undefined_state {
        use super::*;

        #[test]
        fn test_string_parse() {
            assert!(matches!("U".parse::<Tristate>(), Ok(Tristate::Undefined)));
        }

        #[test]
        fn test_to_string() {
            let state = Tristate::Undefined;

            assert_eq!(state.to_string(), "U");
        }
    }

    mod test_default {
        use super::*;

        #[test]
        fn test_default() {
            let state: Tristate = Default::default();

            assert_eq!(state, Tristate::Undefined);
        }
    }

    mod test_string_parse_error {
        use super::*;

        #[test]
        fn test_string_parse_bad_value() {
            assert!(matches!(
                "unknown".parse::<Tristate>(),
                Err(ParseTristateError::UnknownValue(given)) if given == "unknown"
            ));
        }
    }

    mod test_bitwise_operators {
        use super::*;

        #[test]
        fn test_bitor_operator_truth_table() {
            assert_eq!(
                Tristate::State(false) | Tristate::State(false),
                Tristate::State(false)
            );
            assert_eq!(
                Tristate::State(false) | Tristate::State(true),
                Tristate::State(true)
            );
            assert_eq!(
                Tristate::State(true) | Tristate::State(false),
                Tristate::State(true)
            );
            assert_eq!(
                Tristate::State(true) | Tristate::State(true),
                Tristate::State(true)
            );
        }

        #[test]
        fn test_bitor_operator_handle_undefined_state() {
            assert_eq!(
                Tristate::Undefined | Tristate::Undefined,
                Tristate::Undefined
            );
            assert_eq!(
                Tristate::State(false) | Tristate::Undefined,
                Tristate::Undefined
            );
            assert_eq!(
                Tristate::Undefined | Tristate::State(false),
                Tristate::Undefined
            );
            assert_eq!(
                Tristate::State(true) | Tristate::Undefined,
                Tristate::State(true)
            );
            assert_eq!(
                Tristate::Undefined | Tristate::State(true),
                Tristate::State(true)
            );
        }

        #[test]
        fn test_bitand_operator_truth_table() {
            assert_eq!(
                Tristate::State(false) & Tristate::State(false),
                Tristate::State(false)
            );
            assert_eq!(
                Tristate::State(false) & Tristate::State(true),
                Tristate::State(false)
            );
            assert_eq!(
                Tristate::State(true) & Tristate::State(false),
                Tristate::State(false)
            );
            assert_eq!(
                Tristate::State(true) & Tristate::State(true),
                Tristate::State(true)
            );
        }

        #[test]
        fn test_bitand_operator_handle_undefined_state() {
            assert_eq!(
                Tristate::Undefined & Tristate::Undefined,
                Tristate::Undefined
            );
            assert_eq!(
                Tristate::State(false) & Tristate::Undefined,
                Tristate::State(false)
            );
            assert_eq!(
                Tristate::Undefined & Tristate::State(false),
                Tristate::State(false)
            );
            assert_eq!(
                Tristate::State(true) & Tristate::Undefined,
                Tristate::Undefined
            );
            assert_eq!(
                Tristate::Undefined & Tristate::State(true),
                Tristate::Undefined
            );
        }

        #[test]
        fn test_bitxor_operator_truth_table() {
            assert_eq!(
                Tristate::State(false) ^ Tristate::State(false),
                Tristate::State(false)
            );
            assert_eq!(
                Tristate::State(false) ^ Tristate::State(true),
                Tristate::State(true)
            );
            assert_eq!(
                Tristate::State(true) ^ Tristate::State(false),
                Tristate::State(true)
            );
            assert_eq!(
                Tristate::State(true) ^ Tristate::State(true),
                Tristate::State(false)
            );
        }

        #[test]
        fn test_bitxor_operator_handle_undefined_state() {
            assert_eq!(
                Tristate::Undefined ^ Tristate::Undefined,
                Tristate::Undefined
            );
            assert_eq!(
                Tristate::State(false) ^ Tristate::Undefined,
                Tristate::Undefined
            );
            assert_eq!(
                Tristate::Undefined ^ Tristate::State(false),
                Tristate::Undefined
            );
            assert_eq!(
                Tristate::State(true) ^ Tristate::Undefined,
                Tristate::Undefined
            );
            assert_eq!(
                Tristate::Undefined ^ Tristate::State(true),
                Tristate::Undefined
            );
        }

        #[test]
        fn test_not_operator_truth_table() {
            assert_eq!(!Tristate::State(false), Tristate::State(true));
            assert_eq!(!Tristate::State(true), Tristate::State(false));
        }

        #[test]
        fn test_not_operator_handle_undefined_state() {
            assert_eq!(!Tristate::Undefined, Tristate::Undefined);
        }
    }
}
