#[macro_use]
extern crate failure;
#[macro_use]
extern crate frunk;
extern crate glium;
#[macro_use]
extern crate log;
extern crate obj;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate typemap;

#[macro_use]
pub mod util;

mod components;
mod gui;
mod map;
mod state;

pub use crate::{
    components::LocationComponent,
    gui::{GuiSystem, RenderComponent},
    map::{Map, Tile},
    state::{State, World},
};
use frunk::{FuncMut, PolyMut};

/// An entity.
pub type Entity = usize;

/// The trait for a system.
pub trait System {
    /// Runs a single step.
    fn step(&mut self, state: &mut State, dt: u64);
}

/// A helper for stepping through a system with Frunk.
pub struct SystemStepper<'a>(pub &'a mut State, pub u64);

impl<'a> SystemStepper<'a> {
    /// Creates a `PolyMut` function for calling `System::step` with the given arguments.
    pub fn with_args(state: &'a mut State, dt: u64) -> PolyMut<SystemStepper<'a>> {
        PolyMut(SystemStepper(state, dt))
    }
}

impl<'a, 'b, S: System> FuncMut<&'b mut S> for SystemStepper<'a> {
    type Output = ();

    fn call(&mut self, system: &'b mut S) {
        system.step(self.0, self.1)
    }
}
