extern crate cgmath;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate frunk;
#[macro_use]
extern crate glium;
extern crate image;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate obj;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate typemap;

#[macro_use]
pub mod util;

mod components;
mod gui;
mod map;
mod state;

pub use crate::{
    components::LocationComponent,
    gui::{GuiSystem, Material, Model, RenderComponent, RenderData, Vertex},
    map::{Map, Tile},
    state::{State, World},
};
use frunk::{FuncMut, PolyMut};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

/// An entity.
#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
pub struct Entity(usize);

impl Debug for Entity {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "Entity(n={:?})", self.0)
    }
}

impl Display for Entity {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:?}", self)
    }
}

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
