use std::{
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    components::{
        gates::{
            one_input::{GateNOT, GateOneInput},
            two_inputs::{GateAND, GateNAND, GateNOR, GateOR, GateTwoInputs, GateXOR},
        },
        tristate::Tristate,
        Component, InvalidPin, PinNumber, Tick,
    },
    pin::{PinContainer, PinSpecification},
};

/* Final Components Declaration */
pub type Component4001 = ParallelGatesTwoInputs<GateNOR>;
pub type Component4011 = ParallelGatesTwoInputs<GateNAND>;
pub type Component4030 = ParallelGatesTwoInputs<GateXOR>;
pub type Component4069 = ParallelGatesOneInput<GateNOT>;
pub type Component4071 = ParallelGatesTwoInputs<GateOR>;
pub type Component4081 = ParallelGatesTwoInputs<GateAND>;
/* ---------------------------- */

/* -----------
GATE ONE INPUT
------------*/

pub struct ParallelGatesOneInput<G: GateOneInput + 'static> {
    pins: Rc<PinContainer>,
    components: [Rc<G>; 6],
}

impl<G> ParallelGatesOneInput<G>
where
    G: GateOneInput + Default + 'static,
{
    const INPUT_1: PinNumber = 1;
    const OUTPUT_1: PinNumber = 2;
    const INPUT_2: PinNumber = 3;
    const OUTPUT_2: PinNumber = 4;
    const INPUT_3: PinNumber = 5;
    const OUTPUT_3: PinNumber = 6;
    const OUTPUT_4: PinNumber = 8;
    const INPUT_4: PinNumber = 9;
    const OUTPUT_5: PinNumber = 10;
    const INPUT_5: PinNumber = 11;
    const OUTPUT_6: PinNumber = 12;
    const INPUT_6: PinNumber = 13;

    const PER_GATES: [(PinNumber, PinNumber); 6] = [
        (Self::INPUT_1, Self::OUTPUT_1),
        (Self::INPUT_2, Self::OUTPUT_2),
        (Self::INPUT_3, Self::OUTPUT_3),
        (Self::INPUT_4, Self::OUTPUT_4),
        (Self::INPUT_5, Self::OUTPUT_5),
        (Self::INPUT_6, Self::OUTPUT_6),
    ];

    pub fn new() -> Self {
        let this = Self { pins: Rc::new(PinContainer::new(14, Self::build_pins_spec())), components: Default::default() };

        debug_assert_eq!(this.components.len(), Self::PER_GATES.len());

        for (idx, (input_pin, output_pin)) in Self::PER_GATES.into_iter().enumerate() {
            let component = Rc::downgrade(&this.components[idx]);

            this.pins.link_internal_component(input_pin, component.clone(), G::INPUT);
            this.pins.link_internal_component(output_pin, component.clone(), G::OUTPUT);
        }

        this
    }

    fn build_pins_spec() -> HashMap<PinNumber, PinSpecification> {
        let mut spec: HashMap<PinNumber, PinSpecification> = Default::default();

        for (input_pin, output_pin) in Self::PER_GATES {
            spec.extend([
                (input_pin, PinSpecification::UnidirectionalInput()),
                (output_pin, PinSpecification::UnidirectionalOutput()),
            ]);
        }

        spec
    }
}

impl<G> Component for ParallelGatesOneInput<G>
where
    G: GateOneInput + 'static,
{
    fn simulate(&self, tick: Tick) {
        self.pins.simulate_no_manual_outputs(tick);
    }

    fn compute(&self, pin: PinNumber) -> Result<Tristate, InvalidPin> {
        self.pins.compute_for_external(pin)
    }

    fn set_link(&self, pin: PinNumber, other_component: Weak<dyn Component>, other_pin: PinNumber) -> Result<(), InvalidPin> {
        self.pins.set_link_to_external_component(pin, other_component, other_pin)
    }
}

/* ------------
GATE TWO INPUTS
-------------*/

pub struct ParallelGatesTwoInputs<G: GateTwoInputs + 'static> {
    pins: Rc<PinContainer>,
    components: [Rc<G>; 4],
}

impl<G> ParallelGatesTwoInputs<G>
where
    G: GateTwoInputs + Default + 'static,
{
    const INPUT_1_LEFT: PinNumber = 1;
    const INPUT_1_RIGHT: PinNumber = 2;
    const OUTPUT_1: PinNumber = 3;
    const OUTPUT_2: PinNumber = 4;
    const INPUT_2_LEFT: PinNumber = 5;
    const INPUT_2_RIGHT: PinNumber = 6;
    const INPUT_3_LEFT: PinNumber = 8;
    const INPUT_3_RIGHT: PinNumber = 9;
    const OUTPUT_3: PinNumber = 10;
    const OUTPUT_4: PinNumber = 11;
    const INPUT_4_LEFT: PinNumber = 12;
    const INPUT_4_RIGHT: PinNumber = 13;

    const PER_GATES: [(PinNumber, PinNumber, PinNumber); 4] = [
        (Self::INPUT_1_LEFT, Self::INPUT_1_RIGHT, Self::OUTPUT_1),
        (Self::INPUT_2_LEFT, Self::INPUT_2_RIGHT, Self::OUTPUT_2),
        (Self::INPUT_3_LEFT, Self::INPUT_3_RIGHT, Self::OUTPUT_3),
        (Self::INPUT_4_LEFT, Self::INPUT_4_RIGHT, Self::OUTPUT_4),
    ];

    pub fn new() -> Self {
        let this = Self { pins: Rc::new(PinContainer::new(14, Self::build_pins_spec())), components: Default::default() };

        debug_assert_eq!(this.components.len(), Self::PER_GATES.len());

        for (idx, (input_left_pin, input_right_pin, output_pin)) in Self::PER_GATES.into_iter().enumerate() {
            let component = Rc::downgrade(&this.components[idx]);

            this.pins.link_internal_component(input_left_pin, component.clone(), G::INPUT_LEFT);
            this.pins.link_internal_component(input_right_pin, component.clone(), G::INPUT_RIGHT);
            this.pins.link_internal_component(output_pin, component.clone(), G::OUTPUT);
        }

        this
    }

    fn build_pins_spec() -> HashMap<PinNumber, PinSpecification> {
        let mut spec: HashMap<PinNumber, PinSpecification> = Default::default();

        for (input_left_pin, input_right_pin, output_pin) in Self::PER_GATES {
            spec.extend([
                (input_left_pin, PinSpecification::UnidirectionalInput()),
                (input_right_pin, PinSpecification::UnidirectionalInput()),
                (output_pin, PinSpecification::UnidirectionalOutput()),
            ]);
        }

        spec
    }
}

impl<G> Component for ParallelGatesTwoInputs<G>
where
    G: GateTwoInputs + 'static,
{
    fn simulate(&self, tick: Tick) {
        self.pins.simulate_no_manual_outputs(tick);
    }

    fn compute(&self, pin: PinNumber) -> Result<Tristate, InvalidPin> {
        self.pins.compute_for_external(pin)
    }

    fn set_link(&self, pin: PinNumber, other_component: Weak<dyn Component>, other_pin: PinNumber) -> Result<(), InvalidPin> {
        self.pins.set_link_to_external_component(pin, other_component, other_pin)
    }
}
