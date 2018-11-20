use crate::{state::State, System};
use failure::{Fallible, SyncFailure};
use glium::{
    glutin::{Api, ContextBuilder, EventsLoop, GlProfile, GlRequest, WindowBuilder},
    Display,
};
use obj::{Obj, SimplePolygon};
use std::sync::Arc;

/// The GUI system.
pub struct GuiSystem {
    display: Display,
}

impl GuiSystem {
    /// Sets up the GUI.
    pub fn new() -> Fallible<GuiSystem> {
        let events_loop = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_dimensions((800, 600).into())
            .with_title("Game");
        let context = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core)
            .with_vsync(true);
        let display = Display::new(window, context, &events_loop).map_err(SyncFailure::new)?;

        Ok(GuiSystem { display })
    }
}

impl System for GuiSystem {
    fn step(&mut self, state: &mut State, dt: u64) {
        let mut frame = self.display.draw();
        //unimplemented!()
    }
}

/// A graphical component.
pub struct RenderComponent {
    /// The model for the component.
    pub model: Arc<Obj<'static, SimplePolygon>>,
}

impl_Component!(RenderComponent);
