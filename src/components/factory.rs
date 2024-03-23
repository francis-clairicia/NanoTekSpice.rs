use super::single_pin::clock_component::ClockComponent;
use super::single_pin::const_component::{FalseComponent, TrueComponent};
use super::single_pin::input_component::InputComponent;
use super::single_pin::output_component::OutputComponent;
use super::{types::ComponentType, Component};

pub trait ComponentFactory {
    type Type;

    fn create_component(&self, component_type: Self::Type) -> Box<dyn Component>;
}

pub struct DefaultComponentFactory;

impl ComponentFactory for DefaultComponentFactory {
    type Type = ComponentType;

    fn create_component(&self, component_type: ComponentType) -> Box<dyn Component> {
        match component_type {
            ComponentType::Input => Box::new(InputComponent::new()),
            ComponentType::Output => Box::new(OutputComponent::new()),
            ComponentType::Clock => Box::new(ClockComponent::new()),
            ComponentType::True => Box::new(TrueComponent::new()),
            ComponentType::False => Box::new(FalseComponent::new()),
        }
    }
}

#[cfg(test)]
pub mod mock {
    use crate::components::dummy::DummyComponent;

    use super::{Component, ComponentFactory};

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum MockComponentType {
        OnePin,
        TwelvePins,
    }

    impl std::str::FromStr for MockComponentType {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "one" => Ok(Self::OnePin),
                "twelve" => Ok(Self::TwelvePins),
                _ => Err(()),
            }
        }
    }

    pub struct MockComponentFactory;

    impl ComponentFactory for MockComponentFactory {
        type Type = MockComponentType;

        fn create_component(&self, component_type: MockComponentType) -> Box<dyn Component> {
            match component_type {
                MockComponentType::OnePin => Box::new(DummyComponent::new(1)),
                MockComponentType::TwelvePins => Box::new(DummyComponent::new(12)),
            }
        }
    }
}
