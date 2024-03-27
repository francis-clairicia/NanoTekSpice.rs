use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use super::components::{tristate::Tristate, Component, Tick};

mod builder;
mod parser;

pub use parser::{BuildErrorKind, ParseCircuitError, SyntaxErrorKind};

#[derive(Debug, Clone)]
pub enum SetInputError<'a> {
    UnknownName(&'a str),
    NotAnInput(&'a str),
    ValueParseError(&'a str),
}

pub struct Circuit {
    current_tick: Tick,
    components: HashMap<String, Rc<dyn Component>>,
}

impl Circuit {
    pub fn simulate(&mut self) {
        self.current_tick += 1;

        for (_, component) in self.components.iter() {
            component.simulate(self.current_tick);
        }
    }

    pub fn set_value<'a>(&self, name: &'a str, value: &'a str) -> Result<(), SetInputError<'a>> {
        let value: Tristate = match value.parse() {
            Ok(value) => value,
            Err(_) => {
                return Err(SetInputError::ValueParseError(value));
            }
        };

        self.components
            .get(&name.to_owned())
            .ok_or(SetInputError::UnknownName(name))?
            .as_input()
            .ok_or(SetInputError::NotAnInput(name))?
            .set_state_for_next_tick(value);

        Ok(())
    }

    pub fn get_input(&self, name: &str) -> Option<String> {
        Some(self.components.get(&name.to_owned())?.as_input()?.get_current_state().to_string())
    }

    pub fn get_output(&self, name: &str) -> Option<String> {
        Some(self.components.get(&name.to_owned())?.as_output()?.get_value().to_string())
    }
    /* Helpers for unit tests */
    #[cfg(test)]
    pub(super) fn has_component(&self, name: &str) -> bool {
        self.components.contains_key(&name.to_owned())
    }
}

impl fmt::Display for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "tick: {}", self.current_tick)?;

        let mut components: Vec<(&String, &Rc<dyn Component>)> = self.components.iter().collect();
        components.sort_by_key(|(name, _)| *name);

        writeln!(f, "input(s):")?;
        for (name, component) in components.iter() {
            if let Some(component) = component.as_input() {
                writeln!(f, "  {}: {}", name, component.get_current_state())?
            }
        }

        writeln!(f, "output(s):")?;
        for (name, component) in components.iter() {
            if let Some(component) = component.as_output() {
                writeln!(f, "  {}: {}", name, component.get_value())?
            }
        }

        Ok(())
    }
}

impl std::str::FromStr for Circuit {
    type Err = ParseCircuitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::Parser::read(s)
    }
}

#[cfg(test)]
mod tests {
    use super::builder::CircuitBuilder;
    use super::Circuit;
    use super::SetInputError;

    #[test]
    fn test_create_and_handle_nanotekspice_circuit() {
        let mut circuit: Circuit = CircuitBuilder::default()
            .add_component("input", "in")
            .unwrap()
            .add_component("output", "out")
            .unwrap()
            .link_components("in", 1, "out", 1)
            .unwrap()
            .build()
            .unwrap();

        assert!(circuit.has_component("in"));
        assert!(circuit.has_component("out"));

        assert_eq!(circuit.get_input("in").unwrap(), "U");
        assert_eq!(circuit.get_output("out").unwrap(), "U");
        assert_eq!(circuit.current_tick, 0);

        circuit.set_value("in", "1").unwrap();
        circuit.simulate();

        assert_eq!(circuit.current_tick, 1);
        assert_eq!(circuit.get_input("in").unwrap(), "1");
        assert_eq!(circuit.get_output("out").unwrap(), "1");

        circuit.set_value("in", "0").unwrap();
        circuit.simulate();

        assert_eq!(circuit.current_tick, 2);
        assert_eq!(circuit.get_input("in").unwrap(), "0");
        assert_eq!(circuit.get_output("out").unwrap(), "0");
    }

    #[test]
    fn test_error_for_non_existing_component_name() {
        let circuit: Circuit = CircuitBuilder::default().add_component("input", "in").unwrap().build().unwrap();

        assert!(!circuit.has_component("unknown"));
        assert!(matches!(circuit.set_value("unknown", "1"), Err(SetInputError::UnknownName("unknown"))))
    }

    #[test]
    fn test_error_for_non_input_component_name() {
        let circuit: Circuit = CircuitBuilder::default().add_component("output", "out").unwrap().build().unwrap();

        assert!(matches!(circuit.set_value("out", "1"), Err(SetInputError::NotAnInput("out"))))
    }
}
