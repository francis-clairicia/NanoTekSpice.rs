use super::single_pin::clock_component::ClockComponent;
use super::single_pin::const_component::{FalseComponent, TrueComponent};
use super::single_pin::input_component::InputComponent;
use super::single_pin::output_component::OutputComponent;
use super::{types::ComponentType, Component};

pub struct ComponentFactory;

impl ComponentFactory {
    pub fn create_component(component_type: ComponentType) -> Box<dyn Component> {
        match component_type {
            ComponentType::Input => Box::new(InputComponent::new()),
            ComponentType::Output => Box::new(OutputComponent::new()),
            ComponentType::Clock => Box::new(ClockComponent::new()),
            ComponentType::True => Box::new(TrueComponent::new()),
            ComponentType::False => Box::new(FalseComponent::new()),
        }
    }
}
