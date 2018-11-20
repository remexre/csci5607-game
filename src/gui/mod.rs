use failure::{Fallible, SyncFailure};
use glium::{
    glutin::{Api, ContextBuilder, EventsLoop, GlProfile, GlRequest, WindowBuilder},
    Display,
};

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

    /// Runs the GUI for one step.
    pub fn step(&mut self) {
        let mut frame = self.display.draw();
        unimplemented!()
    }
}
