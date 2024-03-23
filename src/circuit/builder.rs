use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::components::factory::ComponentFactory;
use crate::components::{Component, InvalidPin, PinNumber, Tick};

use super::Circuit;

#[derive(Debug, Clone)]
pub enum CircuitBuildError<'a, Type: std::fmt::Debug + Clone> {
    NoChipset,
    ComponentNameExists(&'a str),
    ComponentNameUnknown(&'a str),
    ComponentTypeUnknown(&'a str),
    ComponentLinkIssue(&'a str, Type, PinNumber),
}

pub struct CircuitBuilder<Factory: ComponentFactory> {
    components: HashMap<String, (Factory::Type, Rc<dyn Component>)>,
    factory: Factory,
}

impl<Factory> CircuitBuilder<Factory>
where
    Factory: ComponentFactory,
    Factory::Type: std::str::FromStr + std::fmt::Debug + Copy,
{
    pub fn new(factory: Factory) -> Self {
        Self {
            components: HashMap::new(),
            factory,
        }
    }

    pub fn build(self) -> Result<Circuit, CircuitBuildError<'static, Factory::Type>> {
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

    pub fn add_component<'a>(
        mut self,
        component_type: &'a str,
        name: &'a str,
    ) -> Result<Self, CircuitBuildError<'a, Factory::Type>> {
        let component_type: Factory::Type = match component_type.parse() {
            Ok(t) => t,
            Err(_) => {
                return Err(CircuitBuildError::ComponentTypeUnknown(component_type));
            }
        };

        use std::collections::hash_map::Entry;

        match self.components.entry(name.to_owned()) {
            Entry::Vacant(v) => {
                let component = self.factory.create_component(component_type);
                v.insert((component_type, component.into()));
                Ok(self)
            }
            Entry::Occupied(_) => Err(CircuitBuildError::ComponentNameExists(name)),
        }
    }

    pub fn link_components<'a>(
        self,
        left_component_name: &'a str,
        left_component_pin: PinNumber,
        right_component_name: &'a str,
        right_component_pin: PinNumber,
    ) -> Result<Self, CircuitBuildError<'a, Factory::Type>> {
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

    fn get_component<'a>(
        &self,
        name: &'a str,
    ) -> Result<(Factory::Type, Rc<dyn Component>), CircuitBuildError<'a, Factory::Type>> {
        let component_pair = self
            .components
            .get(name)
            .ok_or(CircuitBuildError::ComponentNameUnknown(name))?;

        let component_type = component_pair.0;
        let component = component_pair.1.clone();

        Ok((component_type, component))
    }

    #[inline(always)]
    fn unidirectional_link<'a>(
        name: &'a str,
        component: &dyn Component,
        component_type: Factory::Type,
        component_pin: PinNumber,
        other_component: Weak<dyn Component>,
        other_component_pin: PinNumber,
    ) -> Result<(), CircuitBuildError<'a, Factory::Type>> {
        match component.set_link(component_pin, other_component, other_component_pin) {
            Ok(_) => Ok(()),
            Err(InvalidPin(pin)) => Err(CircuitBuildError::ComponentLinkIssue(
                name,
                component_type,
                pin,
            )),
        }
    }
}

impl Default for CircuitBuilder<crate::components::factory::DefaultComponentFactory> {
    fn default() -> Self {
        CircuitBuilder::new(crate::components::factory::DefaultComponentFactory)
    }
}

#[cfg(test)]
mod tests {
    use super::{CircuitBuildError, CircuitBuilder};
    use crate::components::factory::mock::{MockComponentFactory, MockComponentType};

    #[test]
    fn test_create_circuit() {
        let builder = CircuitBuilder::new(MockComponentFactory);
        let builder = builder.add_component("one", "dummy_one").unwrap();
        let builder = builder.add_component("twelve", "dummy_twelve").unwrap();

        let circuit = builder.build().unwrap();

        assert_eq!(circuit.components.len(), 2);
        assert!(circuit.has_component("dummy_one"));
        assert!(circuit.has_component("dummy_twelve"));
    }

    #[test]
    fn test_empty_circuit() {
        assert!(matches!(
            CircuitBuilder::new(MockComponentFactory).build(),
            Err(CircuitBuildError::NoChipset)
        ));
    }

    #[test]
    fn test_add_component_error_unknown_type() {
        let builder = CircuitBuilder::new(MockComponentFactory);

        assert!(matches!(
            builder.add_component("dezkdmpk", "name"),
            Err(CircuitBuildError::ComponentTypeUnknown("dezkdmpk"))
        ))
    }

    #[test]
    fn test_add_component_error_name_duplicate() {
        let builder = CircuitBuilder::new(MockComponentFactory);
        let builder = builder.add_component("one", "dummy").unwrap();

        assert!(matches!(
            builder.add_component("twelve", "dummy"),
            Err(CircuitBuildError::ComponentNameExists("dummy")),
        ))
    }

    #[test]
    fn test_link_components_error_unknown_left_component() {
        let builder = CircuitBuilder::new(MockComponentFactory);
        let builder = builder.add_component("one", "dummy").unwrap();

        assert!(matches!(
            builder.link_components("unknown_left", 1, "dummy", 1),
            Err(CircuitBuildError::ComponentNameUnknown("unknown_left")),
        ));
    }

    #[test]
    fn test_link_components_error_unknown_right_component() {
        let builder = CircuitBuilder::new(MockComponentFactory);
        let builder = builder.add_component("one", "dummy").unwrap();

        assert!(matches!(
            builder.link_components("dummy", 1, "unknown_right", 1),
            Err(CircuitBuildError::ComponentNameUnknown("unknown_right")),
        ));
    }

    #[test]
    fn test_link_components_error_invalid_pin_for_left_component() {
        let builder = CircuitBuilder::new(MockComponentFactory);
        let builder = builder.add_component("one", "dummy_left").unwrap();
        let builder = builder.add_component("twelve", "dummy_right").unwrap();

        assert!(matches!(
            builder.link_components("dummy_left", 42, "dummy_right", 1),
            Err(CircuitBuildError::ComponentLinkIssue(
                "dummy_left",
                MockComponentType::OnePin,
                42
            )),
        ));
    }

    #[test]
    fn test_link_components_error_invalid_pin_for_right_component() {
        let builder = CircuitBuilder::new(MockComponentFactory);
        let builder = builder.add_component("one", "dummy_left").unwrap();
        let builder = builder.add_component("twelve", "dummy_right").unwrap();

        assert!(matches!(
            builder.link_components("dummy_left", 1, "dummy_right", 42),
            Err(CircuitBuildError::ComponentLinkIssue(
                "dummy_right",
                MockComponentType::TwelvePins,
                42
            )),
        ));
    }
}
