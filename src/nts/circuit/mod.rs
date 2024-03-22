use std::collections::HashMap;
use std::fmt;
use std::rc::{Rc, Weak};

use super::components::factory::ComponentFactory;
use super::components::{
    tristate::Tristate, types::ComponentType, Component, InvalidPin, PinNumber, Tick,
};

#[derive(Debug, Clone)]
pub enum CircuitBuildError {
    NoChipset,
    ComponentNameExists(String),
    ComponentNameUnknown(String),
    ComponentLinkIssue(String, ComponentType, PinNumber),
}

pub struct CircuitBuilder {
    components: HashMap<String, (ComponentType, Rc<dyn Component>)>,
}

impl CircuitBuilder {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn build(self) -> Result<Circuit, CircuitBuildError> {
        let components: HashMap<String, Rc<dyn Component>> = self
            .components
            .into_iter()
            .map(|(name, (_, component))| (name, component))
            .collect();

        if components.is_empty() {
            return Err(CircuitBuildError::NoChipset);
        }

        let current_tick: Tick = 0;

        for (_, component) in components.iter() {
            component.simulate(current_tick);
        }

        Ok(Circuit {
            current_tick,
            components,
        })
    }

    pub fn add_component(
        mut self,
        component_type: ComponentType,
        name: String,
    ) -> Result<Self, CircuitBuildError> {
        use std::collections::hash_map::Entry;

        match self.components.entry(name) {
            Entry::Vacant(v) => {
                let component = ComponentFactory::create_component(component_type);
                v.insert((component_type, component.into()));
                Ok(self)
            }
            Entry::Occupied(o) => Err(CircuitBuildError::ComponentNameExists(o.key().clone())),
        }
    }

    pub fn link_components(
        self,
        left_component_name: &String,
        left_component_pin: PinNumber,
        right_component_name: &String,
        right_component_pin: PinNumber,
    ) -> Result<Self, CircuitBuildError> {
        let (left_component_type, left_component) = self.get_component(left_component_name)?;
        let (right_component_type, right_component) = self.get_component(right_component_name)?;

        Self::unidirectional_link(
            left_component_name,
            left_component.as_ref(),
            left_component_type,
            left_component_pin,
            Rc::downgrade(&right_component),
            right_component_pin,
        )?;
        Self::unidirectional_link(
            right_component_name,
            right_component.as_ref(),
            right_component_type,
            right_component_pin,
            Rc::downgrade(&left_component),
            left_component_pin,
        )?;
        Ok(self)
    }

    fn get_component(
        &self,
        name: &String,
    ) -> Result<(ComponentType, Rc<dyn Component>), CircuitBuildError> {
        let component_pair = self
            .components
            .get(name)
            .ok_or_else(|| CircuitBuildError::ComponentNameUnknown(name.clone()))?;

        let component_type = component_pair.0;
        let component = component_pair.1.clone();

        Ok((component_type, component))
    }

    #[inline(always)]
    fn unidirectional_link(
        name: &String,
        component: &dyn Component,
        component_type: ComponentType,
        component_pin: PinNumber,
        other_component: Weak<dyn Component>,
        other_component_pin: PinNumber,
    ) -> Result<(), CircuitBuildError> {
        match component.set_link(component_pin, other_component, other_component_pin) {
            Ok(_) => Ok(()),
            Err(InvalidPin(pin)) => Err(CircuitBuildError::ComponentLinkIssue(
                name.clone(),
                component_type,
                pin,
            )),
        }
    }
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

    pub fn set_value(&self, name: &String, value: Tristate) -> Result<(), String> {
        self.components
            .get(name)
            .ok_or_else(|| name.clone())?
            .as_input()
            .ok_or_else(|| name.clone())?
            .set_state_for_next_tick(value);

        Ok(())
    }

    /* Helpers for unit tests */

    #[cfg(test)]
    fn has_component(&self, name: &String) -> bool {
        self.components.get(name).is_some()
    }

    #[cfg(test)]
    fn get_tick(&self) -> Tick {
        self.current_tick
    }

    #[cfg(test)]
    fn get_input(&self, name: &String) -> Option<Tristate> {
        Some(self.components.get(name)?.as_input()?.get_current_state())
    }

    #[cfg(test)]
    fn get_output(&self, name: &String) -> Option<Tristate> {
        Some(self.components.get(name)?.as_output()?.get_value())
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

#[cfg(test)]
mod test {
    use super::{Circuit, CircuitBuilder};
    use crate::nts::components::tristate::Tristate;

    #[test]
    fn test_create_and_handle_nanotekspice_circuit() {
        let mut circuit: Circuit = CircuitBuilder::new()
            .add_component("input".parse().unwrap(), "in".to_owned())
            .unwrap()
            .add_component("output".parse().unwrap(), "out".to_owned())
            .unwrap()
            .link_components(&"in".to_owned(), 1, &"out".to_owned(), 1)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(
            circuit.get_input(&"in".to_owned()).unwrap(),
            Tristate::Undefined
        );
        assert_eq!(
            circuit.get_output(&"out".to_owned()).unwrap(),
            Tristate::Undefined
        );
        assert_eq!(circuit.get_tick(), 0);

        circuit.set_value(&"in".to_owned(), true.into()).unwrap();
        circuit.simulate();

        assert_eq!(circuit.get_tick(), 1);
        assert_eq!(circuit.get_input(&"in".to_owned()).unwrap(), true.into());
        assert_eq!(circuit.get_output(&"out".to_owned()).unwrap(), true.into());

        circuit.set_value(&"in".to_owned(), false.into()).unwrap();
        circuit.simulate();

        assert_eq!(circuit.get_tick(), 2);
        assert_eq!(circuit.get_input(&"in".to_owned()).unwrap(), false.into());
        assert_eq!(circuit.get_output(&"out".to_owned()).unwrap(), false.into());
    }

    #[test]
    fn test_error_for_non_existing_component_name() {
        let circuit: Circuit = CircuitBuilder::new()
            .add_component("input".parse().unwrap(), "in".to_owned())
            .unwrap()
            .build()
            .unwrap();

        assert!(
            matches!(circuit.set_value(&"unknown".to_owned(), true.into()), Err(invalid_name) if invalid_name == "unknown")
        )
    }
}
