use std::{
    cell::Cell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::components::{tristate::Tristate, Component, InvalidPin, PinNumber, Tick};

use super::pin::{InputPin, Pin, PinMode, UnidirectionalInputPin};

type ComputationCallback = Box<dyn Fn() -> Tristate>;
type BidirectionalPin = super::pin::BidirectionalPin<ComputationCallback>;
type UnidirectionalOutputPin = super::pin::UnidirectionalOutputPin<ComputationCallback>;

pub enum PinSpecification {
    UnidirectionalInput(),
    UnidirectionalOutput(),
    Bidirectional(PinMode),
}

#[derive(Debug, Clone)]
pub enum SwitchPinModeError {
    InvalidPin(PinNumber),
    NotBidirectional(PinNumber),
}

impl From<InvalidPin> for SwitchPinModeError {
    fn from(value: InvalidPin) -> Self {
        Self::InvalidPin(value.0)
    }
}

#[derive(Debug, Clone)]
pub enum InputPinError {
    InvalidPin(PinNumber),
    NotAnInput(PinNumber),
}

impl From<InvalidPin> for InputPinError {
    fn from(value: InvalidPin) -> Self {
        Self::InvalidPin(value.0)
    }
}

enum PinRef {
    UnidirectionalInput(Rc<UnidirectionalInputPin>),
    UnidirectionalOutput(Rc<UnidirectionalOutputPin>),
    Bidirectional(Rc<BidirectionalPin>),
}

impl PinRef {
    pub fn as_pin(&self) -> Rc<dyn Pin> {
        match self {
            Self::UnidirectionalInput(pin) => pin.clone(),
            Self::UnidirectionalOutput(pin) => pin.clone(),
            Self::Bidirectional(pin) => pin.clone(),
        }
    }

    pub fn as_input_pin(&self) -> Option<Rc<dyn InputPin>> {
        match self {
            Self::UnidirectionalInput(pin) => Some(pin.clone()),
            Self::Bidirectional(pin) => Some(pin.clone()),
            Self::UnidirectionalOutput(_) => None,
        }
    }

    pub fn current_pin_mode(&self) -> PinMode {
        match self {
            Self::UnidirectionalInput(_) => PinMode::Input,
            Self::UnidirectionalOutput(_) => PinMode::Output,
            Self::Bidirectional(pin) => pin.current_mode(),
        }
    }
}

pub struct PinContainer {
    all_pins: HashMap<PinNumber, PinRef>,
    output_values: HashMap<PinNumber, Rc<Cell<Tristate>>>,
    state: Cell<PinContainerState>,
}

impl PinContainer {
    pub fn new(nb_pins: usize, mut pins_spec: HashMap<PinNumber, PinSpecification>) -> Self {
        if nb_pins < pins_spec.len() {
            panic!("More pin definition than given number of pins")
        }

        let mut output_values: HashMap<PinNumber, Rc<Cell<Tristate>>> = HashMap::new();
        let mut all_pins: HashMap<PinNumber, PinRef> = HashMap::new();

        for pin_number in 1..(nb_pins + 1) {
            let pin: PinRef = match pins_spec.remove(&pin_number) {
                Some(PinSpecification::UnidirectionalInput()) => {
                    PinRef::UnidirectionalInput(Rc::new(UnidirectionalInputPin::new()))
                }
                Some(PinSpecification::UnidirectionalOutput()) => {
                    let output_cell: Rc<Cell<Tristate>> = Rc::new(Default::default());

                    output_values.insert(pin_number, output_cell.clone());
                    PinRef::UnidirectionalOutput(Rc::new(UnidirectionalOutputPin::new(Box::new(
                        move || output_cell.get(),
                    ))))
                }
                Some(PinSpecification::Bidirectional(default_mode)) => {
                    let output_cell: Rc<Cell<Tristate>> = Rc::new(Default::default());

                    output_values.insert(pin_number, output_cell.clone());
                    PinRef::Bidirectional(Rc::new(BidirectionalPin::new(
                        Box::new(move || output_cell.get()),
                        default_mode,
                    )))
                }
                None => PinRef::UnidirectionalOutput(Rc::new(UnidirectionalOutputPin::new(
                    Box::new(|| Tristate::Undefined),
                ))),
            };
            all_pins.insert(pin_number, pin);
        }

        if !pins_spec.is_empty() {
            panic!("Invalid pin number in definition")
        }

        Self {
            all_pins,
            output_values,
            state: Default::default(),
        }
    }

    pub fn simulate<F>(&self, tick: Tick, simulate_fn: F)
    where
        F: FnOnce(&HashMap<PinNumber, &Cell<Tristate>>) -> (),
    {
        if let PinContainerState::Available(current_tick) = self.state.get() {
            if current_tick == tick {
                return;
            }
        } else if let PinContainerState::Computing(current_tick) = self.state.get() {
            if current_tick == tick {
                return;
            }
            panic!("Cyclic pin simulation with different tick ({current_tick} != {tick})");
        }

        self.state.set(PinContainerState::Computing(tick));
        for (_, pin_ref) in self.all_pins.iter() {
            if let Some(pin_ref) = pin_ref.as_input_pin() {
                pin_ref.simulate(tick);
            }
        }

        let output_values_for_simulation: HashMap<PinNumber, &Cell<Tristate>> = self
            .output_values
            .iter()
            .map(|(pin, cell)| (*pin, cell.as_ref()))
            .collect();

        simulate_fn(&output_values_for_simulation);

        self.state.set(PinContainerState::Available(tick));
    }

    pub fn compute_for_external(&self, pin: PinNumber) -> Result<Tristate, InvalidPin> {
        let pin = self.get_pin(pin)?;

        Ok(pin.compute_for_external())
    }

    pub fn set_link_to_external(
        &self,
        pin: PinNumber,
        other_component: Weak<dyn Component>,
        other_pin: PinNumber,
    ) -> Result<(), InvalidPin> {
        if let Some(input_pin) = self.get_pin_ref(pin)?.as_input_pin() {
            input_pin.link_to(other_component, other_pin);
        }

        Ok(())
    }

    pub fn compute_input(&self, pin: PinNumber) -> Result<Tristate, InputPinError> {
        let pin = self
            .get_pin_ref(pin)?
            .as_input_pin()
            .ok_or(InputPinError::NotAnInput(pin))?;

        if let PinContainerState::Computing(tick) = self.state.get() {
            // Make sure pin.simulate() is called first
            pin.simulate(tick);
        }

        Ok(pin.compute_input())
    }

    pub fn current_pin_mode(&self, pin: PinNumber) -> Result<PinMode, InvalidPin> {
        Ok(self.get_pin_ref(pin)?.current_pin_mode())
    }

    pub fn switch_pin_to_mode(
        &self,
        pin: PinNumber,
        mode: PinMode,
    ) -> Result<(), SwitchPinModeError> {
        if let PinRef::Bidirectional(pin) = self.get_pin_ref(pin)? {
            pin.switch_to_mode(mode);
            Ok(())
        } else {
            Err(SwitchPinModeError::NotBidirectional(pin))
        }
    }

    #[inline]
    fn get_pin_ref(&self, pin: PinNumber) -> Result<&PinRef, InvalidPin> {
        self.all_pins.get(&pin).ok_or(InvalidPin(pin))
    }

    #[inline]
    fn get_pin(&self, pin: PinNumber) -> Result<Rc<dyn Pin>, InvalidPin> {
        Ok(self.get_pin_ref(pin)?.as_pin())
    }
}

#[derive(Clone, Copy)]
enum PinContainerState {
    NeverComputed,
    Available(Tick),
    Computing(Tick),
}

impl Default for PinContainerState {
    #[inline]
    fn default() -> Self {
        Self::NeverComputed
    }
}
