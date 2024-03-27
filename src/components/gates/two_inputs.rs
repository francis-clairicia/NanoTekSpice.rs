use std::{cell::Cell, collections::HashMap, rc::Weak};

use crate::{
    components::{tristate::Tristate, Component, InvalidPin, PinNumber, Tick},
    pin::{PinContainer, PinSpecification},
};

pub trait GateTwoInputs: Component {
    const INPUT_LEFT: PinNumber;
    const INPUT_RIGHT: PinNumber;
    const OUTPUT: PinNumber;
}

macro_rules! gate_two_inputs_impl {
    ($name:ident, $operation:expr) => {
        pub struct $name {
            pins: PinContainer,
        }

        impl $name {
            pub fn new() -> Self {
                Self { pins: PinContainer::new(3, Self::build_pins_spec()) }
            }

            #[inline]
            fn build_pins_spec() -> HashMap<PinNumber, PinSpecification> {
                HashMap::from([
                    (Self::INPUT_LEFT, PinSpecification::UnidirectionalInput()),
                    (Self::INPUT_RIGHT, PinSpecification::UnidirectionalInput()),
                    (Self::OUTPUT, PinSpecification::UnidirectionalOutput()),
                ])
            }
        }

        impl Component for $name {
            fn set_link(
                &self,
                pin: PinNumber,
                other_component: Weak<dyn Component>,
                other_pin: PinNumber,
            ) -> Result<(), InvalidPin> {
                self.pins.set_link_to_external_component(pin, other_component, other_pin)
            }

            fn simulate(&self, tick: Tick) {
                static OPERATION: fn(Tristate, Tristate) -> Tristate = $operation;

                self.pins.simulate(tick, |output_cells| {
                    let input_left: Tristate = self.pins.compute_input(Self::INPUT_LEFT).unwrap();
                    let input_right: Tristate = self.pins.compute_input(Self::INPUT_RIGHT).unwrap();
                    let output_cell: &Cell<Tristate> = output_cells.get(&Self::OUTPUT).unwrap();

                    output_cell.set(OPERATION(input_left, input_right));
                })
            }

            fn compute(&self, pin: PinNumber) -> Result<Tristate, InvalidPin> {
                self.pins.compute_for_external(pin)
            }
        }

        impl GateTwoInputs for $name {
            const INPUT_LEFT: PinNumber = 1;
            const INPUT_RIGHT: PinNumber = 2;
            const OUTPUT: PinNumber = 3;
        }

        impl Default for $name {
            #[inline]
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

gate_two_inputs_impl!(GateAND, |left, right| left & right);

gate_two_inputs_impl!(GateOR, |left, right| left | right);

gate_two_inputs_impl!(GateXOR, |left, right| left ^ right);

gate_two_inputs_impl!(GateNAND, |left, right| !(left & right));

gate_two_inputs_impl!(GateNOR, |left, right| !(left | right));
