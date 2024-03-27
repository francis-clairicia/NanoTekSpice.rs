use std::{
    cell::{Cell, RefCell},
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
    output_values: RefCell<HashMap<PinNumber, OutputComputationMethod>>,
    state: Cell<PinContainerState>,
    internal_component_proxy: RefCell<Option<Rc<dyn Component>>>,
}

impl PinContainer {
    pub fn new(nb_pins: usize, mut pins_spec: HashMap<PinNumber, PinSpecification>) -> Self {
        if nb_pins < pins_spec.len() {
            panic!("More pin definition than given number of pins")
        }

        let mut output_values: HashMap<PinNumber, OutputComputationMethod> = HashMap::new();
        let mut all_pins: HashMap<PinNumber, PinRef> = HashMap::new();

        for pin_number in 1..(nb_pins + 1) {
            let pin: PinRef = match pins_spec.remove(&pin_number) {
                Some(PinSpecification::UnidirectionalInput()) => {
                    PinRef::UnidirectionalInput(Rc::new(UnidirectionalInputPin::new()))
                }
                Some(PinSpecification::UnidirectionalOutput()) => {
                    let output_cell: Rc<Cell<Tristate>> = Rc::new(Default::default());

                    output_values.insert(pin_number, OutputComputationMethod::Manual(output_cell.clone()));
                    PinRef::UnidirectionalOutput(Rc::new(UnidirectionalOutputPin::new(Box::new(move || output_cell.get()))))
                }
                Some(PinSpecification::Bidirectional(default_mode)) => {
                    let output_cell: Rc<Cell<Tristate>> = Rc::new(Default::default());

                    output_values.insert(pin_number, OutputComputationMethod::Manual(output_cell.clone()));
                    PinRef::Bidirectional(Rc::new(BidirectionalPin::new(Box::new(move || output_cell.get()), default_mode)))
                }
                None => PinRef::UnidirectionalOutput(Rc::new(UnidirectionalOutputPin::new(Box::new(|| Tristate::Undefined)))),
            };
            all_pins.insert(pin_number, pin);
        }

        if !pins_spec.is_empty() {
            panic!("Invalid pin number in definition")
        }

        Self {
            all_pins,
            output_values: RefCell::new(output_values),
            state: Default::default(),
            internal_component_proxy: RefCell::new(Default::default()),
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
        self.simulate_all_inputs(tick);

        let output_values = self.output_values.borrow();
        let output_values_for_simulation: HashMap<PinNumber, &Cell<Tristate>> = output_values
            .iter()
            .filter_map(|(pin, method)| {
                match method {
                    OutputComputationMethod::Manual(cell) => Some((*pin, cell.as_ref())),
                    OutputComputationMethod::Automatic(output) => {
                        // Okay we must call simulate() now.
                        output.simulate(tick);
                        None
                    }
                }
            })
            .collect();

        simulate_fn(&output_values_for_simulation);

        self.state.set(PinContainerState::Available(tick));
    }

    pub fn simulate_no_manual_outputs(&self, tick: Tick) {
        self.simulate(tick, |outputs| {
            debug_assert!(outputs.is_empty(), "There is manually computed output pins!");
        })
    }

    pub fn compute_for_external(&self, pin: PinNumber) -> Result<Tristate, InvalidPin> {
        let pin_number = pin;
        let pin = self.get_pin(pin)?;

        if let PinContainerState::Computing(tick) = self.state.get() {
            if let Some(OutputComputationMethod::Automatic(output)) = self.output_values.borrow().get(&pin_number) {
                // Make sure output.simulate() is called first
                output.simulate(tick);
            }
        }

        Ok(pin.compute_for_external())
    }

    pub fn set_link_to_external_component(
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

    pub fn link_internal_component<C: Component + 'static>(
        self: &Rc<Self>,
        pin: PinNumber,
        other_component: Weak<C>,
        other_pin: PinNumber,
    ) {
        let other_component: Rc<dyn Component> = other_component.upgrade().unwrap();
        let proxy: Rc<dyn Component> = {
            let mut internal_component_proxy = self.internal_component_proxy.borrow_mut();

            if let Some(ref component) = *internal_component_proxy {
                component.clone()
            } else {
                let component: Rc<dyn Component> = Rc::new(InternalComponentProxy::new(Rc::downgrade(&self)));

                *internal_component_proxy = Some(component.clone());
                component
            }
        };

        proxy.set_link(pin, Rc::downgrade(&other_component), other_pin).unwrap();
        other_component.set_link(other_pin, Rc::downgrade(&proxy), pin).unwrap();
    }

    pub fn compute_input(&self, pin: PinNumber) -> Result<Tristate, InputPinError> {
        let pin = self.get_pin_ref(pin)?.as_input_pin().ok_or(InputPinError::NotAnInput(pin))?;

        if let PinContainerState::Computing(tick) = self.state.get() {
            // Make sure pin.simulate() is called first
            pin.simulate(tick);
        }

        Ok(pin.compute_input())
    }

    pub fn current_pin_mode(&self, pin: PinNumber) -> Result<PinMode, InvalidPin> {
        Ok(self.get_pin_ref(pin)?.current_pin_mode())
    }

    pub fn switch_pin_to_mode(&self, pin: PinNumber, mode: PinMode) -> Result<(), SwitchPinModeError> {
        if let PinRef::Bidirectional(pin) = self.get_pin_ref(pin)? {
            pin.switch_to_mode(mode);
            Ok(())
        } else {
            Err(SwitchPinModeError::NotBidirectional(pin))
        }
    }

    pub fn check(&self, pin: PinNumber) -> Result<(), InvalidPin> {
        self.get_pin_ref(pin).map(|_| ())
    }

    #[inline]
    fn get_pin_ref(&self, pin: PinNumber) -> Result<&PinRef, InvalidPin> {
        self.all_pins.get(&pin).ok_or(InvalidPin(pin))
    }

    #[inline]
    fn get_pin(&self, pin: PinNumber) -> Result<Rc<dyn Pin>, InvalidPin> {
        Ok(self.get_pin_ref(pin)?.as_pin())
    }

    fn simulate_all_inputs(&self, tick: Tick) {
        for (_, pin_ref) in self.all_pins.iter() {
            if let Some(pin_ref) = pin_ref.as_input_pin() {
                pin_ref.simulate(tick);
            }
        }
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

enum OutputComputationMethod {
    Manual(Rc<Cell<Tristate>>),
    Automatic(OutputFromInternalComponents),
}

struct OutputFromInternalComponents {
    result: Rc<Cell<Tristate>>,
    input: UnidirectionalInputPin,
}

impl OutputFromInternalComponents {
    pub fn new(result: Rc<Cell<Tristate>>) -> Self {
        Self { result, input: UnidirectionalInputPin::new() }
    }

    pub fn link_to(&mut self, component: Weak<dyn Component>, pin: PinNumber) {
        self.input.link_to(component, pin);
    }

    pub fn simulate(&self, tick: Tick) {
        self.input.simulate(tick);
        self.result.set(self.input.compute_input());
    }
}

struct InternalComponentProxy {
    container_wr: Weak<PinContainer>,
}

impl InternalComponentProxy {
    pub fn new(container_wr: Weak<PinContainer>) -> Self {
        Self { container_wr }
    }

    #[inline]
    fn container(&self) -> Rc<PinContainer> {
        self.container_wr.upgrade().unwrap()
    }
}

impl Component for InternalComponentProxy {
    fn set_link(&self, pin: PinNumber, other_component: Weak<dyn Component>, other_pin: PinNumber) -> Result<(), InvalidPin> {
        let container = self.container();

        container.check(pin)?;
        let mut output_values = container.output_values.borrow_mut();

        match output_values.get_mut(&pin) {
            Some(OutputComputationMethod::Manual(cell_rc)) => {
                let cell_rc = cell_rc.clone();
                let mut output = OutputFromInternalComponents::new(cell_rc);

                output.link_to(other_component, other_pin);
                output_values.insert(pin, OutputComputationMethod::Automatic(output));
            }
            Some(OutputComputationMethod::Automatic(output)) => {
                output.link_to(other_component, other_pin);
            }
            None => (),
        };

        Ok(())
    }

    fn simulate(&self, tick: Tick) {
        self.container().simulate_all_inputs(tick)
    }

    fn compute(&self, pin: PinNumber) -> Result<Tristate, InvalidPin> {
        match self.container().compute_input(pin) {
            Ok(result) => Ok(result),
            Err(InputPinError::NotAnInput(_)) => Ok(Tristate::Undefined),
            Err(InputPinError::InvalidPin(pin)) => Err(InvalidPin(pin)),
        }
    }
}
