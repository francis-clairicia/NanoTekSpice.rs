use std::rc::Weak;

use crate::components::{tristate::Tristate, Component, InvalidPin, PinNumber, Tick};

pub struct ConstStateComponent<const STATE: bool>;

pub type FalseComponent = ConstStateComponent<false>;
pub type TrueComponent = ConstStateComponent<true>;

impl<const STATE: bool> ConstStateComponent<STATE> {
    const OUTPUT: PinNumber = 1;

    pub fn new() -> Self {
        Self {}
    }
}

impl<const STATE: bool> Component for ConstStateComponent<STATE> {
    fn set_link(&self, pin: PinNumber, _other_component: Weak<dyn Component>, _other_pin: PinNumber) -> Result<(), InvalidPin> {
        match pin {
            Self::OUTPUT => Ok(()),
            _ => Err(InvalidPin(pin)),
        }
    }

    fn simulate(&self, _tick: Tick) {}

    fn compute(&self, pin: PinNumber) -> Result<Tristate, InvalidPin> {
        match pin {
            Self::OUTPUT => Ok(STATE.into()),
            _ => Err(InvalidPin(pin)),
        }
    }
}
