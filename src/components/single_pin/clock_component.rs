use std::{cell::Cell, collections::HashMap, rc::Weak};

use crate::{
    components::{tristate::Tristate, Component, Input, InvalidPin, PinNumber, Tick},
    pin::{PinContainer, PinSpecification},
};

pub struct ClockComponent {
    pins: PinContainer,
    value_for_next_tick: Cell<Option<Tristate>>,
}

impl ClockComponent {
    const PIN_OUTPUT: usize = 1;

    pub fn new() -> Self {
        Self {
            pins: PinContainer::new(1, Self::build_pins_spec()),
            value_for_next_tick: Default::default(),
        }
    }

    #[inline]
    fn build_pins_spec() -> HashMap<PinNumber, PinSpecification> {
        HashMap::from([(Self::PIN_OUTPUT, PinSpecification::UnidirectionalOutput())])
    }
}

impl Component for ClockComponent {
    fn set_link(
        &self,
        pin: PinNumber,
        other_component: Weak<dyn Component>,
        other_pin: PinNumber,
    ) -> Result<(), InvalidPin> {
        self.pins
            .set_link_to_external(pin, other_component, other_pin)
    }

    fn simulate(&self, tick: Tick) {
        self.pins.simulate(tick, |outputs| {
            let output = outputs.get(&Self::PIN_OUTPUT).unwrap();

            if let Some(state) = self.value_for_next_tick.replace(None) {
                output.set(state);
            } else {
                output.set(!output.get());
            }
        })
    }

    fn compute(&self, pin: PinNumber) -> Result<Tristate, InvalidPin> {
        self.pins.compute_for_external(pin)
    }

    fn as_input(&self) -> Option<&dyn Input> {
        Some(self)
    }
}

impl Input for ClockComponent {
    fn get_current_state(&self) -> Tristate {
        self.compute(Self::PIN_OUTPUT).unwrap()
    }

    fn set_state_for_next_tick(&self, state: Tristate) {
        self.value_for_next_tick.set(Some(state));
    }
}
