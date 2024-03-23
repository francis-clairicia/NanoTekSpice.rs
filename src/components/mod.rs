pub mod factory;
pub mod tristate;
pub mod types;

/* Components implementations */
pub mod single_pin;
/* -------------------------- */

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

#[cfg(test)]
pub mod dummy {
    use std::collections::HashMap;

    use crate::pin::PinContainer;

    use super::{tristate::Tristate, Component, InvalidPin, PinNumber, Tick};

    pub struct DummyComponent {
        pins: PinContainer,
    }

    impl DummyComponent {
        pub fn new(nb_pins: usize) -> Self {
            Self {
                pins: PinContainer::new(nb_pins, HashMap::new()),
            }
        }
    }

    impl Component for DummyComponent {
        fn simulate(&self, tick: Tick) {
            self.pins.simulate(tick, |_| ())
        }

        fn compute(&self, pin: PinNumber) -> Result<Tristate, InvalidPin> {
            self.pins.compute_for_external(pin)
        }

        fn set_link(
            &self,
            pin: PinNumber,
            other_component: std::rc::Weak<dyn Component>,
            other_pin: PinNumber,
        ) -> Result<(), InvalidPin> {
            self.pins
                .set_link_to_external(pin, other_component, other_pin)
        }
    }
}
