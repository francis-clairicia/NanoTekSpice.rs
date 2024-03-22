pub mod factory;
pub mod single_pin;
pub mod tristate;
pub mod types;

use std::rc::Weak;

pub type Tick = usize;
pub type PinNumber = usize;

#[derive(Debug, Clone, Copy)]
pub struct InvalidPin(pub PinNumber);

pub trait Component {
    fn simulate(&self, tick: Tick);
    fn compute(&self, pin: PinNumber) -> Result<tristate::Tristate, InvalidPin>;
    fn set_link(
        &self,
        pin: PinNumber,
        other_component: Weak<dyn Component>,
        other_pin: PinNumber,
    ) -> Result<(), InvalidPin>;

    fn as_input(&self) -> Option<&dyn Input> {
        None
    }
    fn as_output(&self) -> Option<&dyn Output> {
        None
    }
}

pub trait Input {
    fn get_current_state(&self) -> tristate::Tristate;
    fn set_state_for_next_tick(&self, state: tristate::Tristate);
}

pub trait Output {
    fn get_value(&self) -> tristate::Tristate;
}
