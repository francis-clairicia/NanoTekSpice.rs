use std::{cell::Cell, collections::HashMap, rc::Weak};

use crate::{
    components::{tristate::Tristate, Component, InvalidPin, Output, PinNumber, Tick},
    pin::{PinContainer, PinSpecification},
};

pub struct OutputComponent {
    pins: PinContainer,
    result: Cell<Tristate>,
}

impl OutputComponent {
    const PIN_INPUT: usize = 1;

    pub fn new() -> Self {
        Self {
            pins: PinContainer::new(1, Self::build_pins_spec()),
            result: Default::default(),
        }
    }

    #[inline]
    fn build_pins_spec() -> HashMap<PinNumber, PinSpecification> {
        HashMap::from([(Self::PIN_INPUT, PinSpecification::UnidirectionalInput())])
    }
}

impl Component for OutputComponent {
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
        self.pins.simulate(tick, |_| {
            let state = self.pins.compute_input(Self::PIN_INPUT).unwrap();

            self.result.set(state);
        })
    }

    fn compute(&self, pin: PinNumber) -> Result<Tristate, InvalidPin> {
        self.pins.compute_for_external(pin)
    }

    fn as_output(&self) -> Option<&dyn Output> {
        Some(self)
    }
}

impl Output for OutputComponent {
    fn get_value(&self) -> Tristate {
        self.result.get()
    }
}
