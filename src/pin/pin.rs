use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    hash::Hash,
    rc::Weak,
};

use crate::components::{tristate::Tristate, Component, PinNumber, Tick};

pub trait Pin {
    fn compute_for_external(&self) -> Tristate;
}

pub trait InputPin: Pin {
    fn simulate(&self, tick: Tick);
    fn link_to(&self, component: Weak<dyn Component>, pin: PinNumber);
    fn compute_input(&self) -> Tristate;
}

pub struct UnidirectionalOutputPin<F>
where
    F: Fn() -> Tristate,
{
    compute_output_fn: F,
}

impl<F> UnidirectionalOutputPin<F>
where
    F: Fn() -> Tristate,
{
    pub fn new(compute_output_fn: F) -> Self {
        Self { compute_output_fn }
    }
}

impl<F> Pin for UnidirectionalOutputPin<F>
where
    F: Fn() -> Tristate,
{
    fn compute_for_external(&self) -> Tristate {
        let compute_output_fn = &self.compute_output_fn;

        compute_output_fn()
    }
}

pub struct UnidirectionalInputPin {
    input_value: Cell<Tristate>,
    input_state: Cell<PinState>,
    links: RefCell<HashSet<PinLink>>,
}

impl UnidirectionalInputPin {
    pub fn new() -> Self {
        Self {
            input_value: Default::default(),
            input_state: Default::default(),
            links: HashSet::new().into(),
        }
    }

    fn recompute_input_cache(&self, tick: Tick) {
        self.input_state.set(PinState::Computing(tick));

        let mut state: Tristate = false.into();
        for link in self.links.borrow().iter() {
            state |= link.compute(tick);
        }

        self.input_value.set(state);
        self.input_state.set(PinState::Available(tick));
    }
}

impl Pin for UnidirectionalInputPin {
    fn compute_for_external(&self) -> Tristate {
        false.into()
    }
}

impl InputPin for UnidirectionalInputPin {
    fn simulate(&self, tick: Tick) {
        match self.input_state.get() {
            PinState::NeverComputed => self.recompute_input_cache(tick),
            PinState::Available(current_tick) => {
                if current_tick != tick {
                    self.recompute_input_cache(tick);
                }
            }
            PinState::Computing(current_tick) => {
                if current_tick != tick {
                    panic!("Cyclic pin simulation with different tick ({current_tick} != {tick})");
                }
            }
        }
    }

    fn link_to(&self, component: Weak<dyn Component>, pin: PinNumber) {
        self.links.borrow_mut().insert(PinLink::new(component, pin));
    }

    fn compute_input(&self) -> Tristate {
        self.input_value.get()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PinMode {
    Input,
    Output,
}

pub struct BidirectionalPin<F>
where
    F: Fn() -> Tristate,
{
    input_pin: UnidirectionalInputPin,
    output_pin: UnidirectionalOutputPin<F>,
    mode: Cell<PinMode>,
}

impl<F> BidirectionalPin<F>
where
    F: Fn() -> Tristate,
{
    pub fn new(compute_output_fn: F, default_mode: PinMode) -> Self {
        Self {
            input_pin: UnidirectionalInputPin::new(),
            output_pin: UnidirectionalOutputPin::new(compute_output_fn),
            mode: Cell::new(default_mode),
        }
    }

    pub fn switch_to_mode(&self, mode: PinMode) {
        self.mode.set(mode);
    }

    pub fn current_mode(&self) -> PinMode {
        self.mode.get()
    }
}

impl<F> Pin for BidirectionalPin<F>
where
    F: Fn() -> Tristate,
{
    fn compute_for_external(&self) -> Tristate {
        match self.mode.get() {
            PinMode::Input => self.input_pin.compute_for_external(),
            PinMode::Output => self.output_pin.compute_for_external(),
        }
    }
}

impl<F> InputPin for BidirectionalPin<F>
where
    F: Fn() -> Tristate,
{
    fn simulate(&self, tick: Tick) {
        self.input_pin.simulate(tick);
    }

    fn link_to(&self, component: Weak<dyn Component>, pin: PinNumber) {
        self.input_pin.link_to(component, pin)
    }

    fn compute_input(&self) -> Tristate {
        match self.mode.get() {
            PinMode::Input => self.input_pin.compute_input(),
            PinMode::Output => false.into(),
        }
    }
}

/* Private helpers */

#[derive(Clone, Copy)]
enum PinState {
    NeverComputed,
    Available(Tick),
    Computing(Tick),
}

impl Default for PinState {
    #[inline]
    fn default() -> Self {
        Self::NeverComputed
    }
}

struct PinLink {
    component: Weak<dyn Component>,
    pin: PinNumber,
}

impl PinLink {
    pub fn new(component: Weak<dyn Component>, pin: PinNumber) -> Self {
        Self { component, pin }
    }

    pub fn compute(&self, tick: Tick) -> Tristate {
        let component = self.component.upgrade().expect("Weak reference lost");

        component.simulate(tick);
        component
            .compute(self.pin)
            .expect("Broken link to a pin of a component")
    }
}

impl PartialEq for PinLink {
    fn eq(&self, other: &Self) -> bool {
        self.component.upgrade().is_some()
            && self.component.ptr_eq(&other.component)
            && self.pin == other.pin
    }
}

impl Eq for PinLink {}

impl Hash for PinLink {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        ((self.component.as_ptr() as *const Self as usize), self.pin).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_output_pin {
        use super::*;

        #[test]
        fn test_default() {
            let pin = UnidirectionalOutputPin::new(|| true.into());

            assert_eq!(pin.compute_for_external(), true.into());
        }
    }
}
