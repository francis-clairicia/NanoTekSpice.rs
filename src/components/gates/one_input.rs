use std::{cell::Cell, collections::HashMap, rc::Weak};

use crate::{
    components::{tristate::Tristate, Component, InvalidPin, PinNumber, Tick},
    pin::{PinContainer, PinSpecification},
};

pub trait GateOneInput: Component {
    const INPUT: PinNumber;
    const OUTPUT: PinNumber;
}

pub struct GateNOT {
    pins: PinContainer,
}

impl GateNOT {
    pub fn new() -> Self {
        Self { pins: PinContainer::new(2, Self::build_pins_spec()) }
    }

    #[inline]
    fn build_pins_spec() -> HashMap<PinNumber, PinSpecification> {
        HashMap::from([
            (Self::INPUT, PinSpecification::UnidirectionalInput()),
            (Self::OUTPUT, PinSpecification::UnidirectionalOutput()),
        ])
    }
}

impl Component for GateNOT {
    fn set_link(&self, pin: PinNumber, other_component: Weak<dyn Component>, other_pin: PinNumber) -> Result<(), InvalidPin> {
        self.pins.set_link_to_external_component(pin, other_component, other_pin)
    }

    fn simulate(&self, tick: Tick) {
        self.pins.simulate(tick, |output_cells| {
            let input: Tristate = self.pins.compute_input(Self::INPUT).unwrap();
            let output: &Cell<Tristate> = output_cells.get(&Self::OUTPUT).unwrap();

            output.set(!input);
        })
    }

    fn compute(&self, pin: PinNumber) -> Result<Tristate, InvalidPin> {
        self.pins.compute_for_external(pin)
    }
}

impl GateOneInput for GateNOT {
    const INPUT: PinNumber = 1;
    const OUTPUT: PinNumber = 2;
}

impl Default for GateNOT {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
