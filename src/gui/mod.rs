mod model;

pub use crate::gui::model::{Model, Vertex};
use crate::{state::State, System};
use failure::{Fallible, SyncFailure};
use glium::{
    backend::Facade,
    glutin::{
        Api, ContextBuilder, Event, EventsLoop, GlProfile, GlRequest, WindowBuilder, WindowEvent,
    },
    Display, Surface,
};
use std::sync::Arc;

/// The GUI system.
pub struct GuiSystem {
    display: Display,
    event_loop: EventsLoop,
}

impl GuiSystem {
    /// Sets up the GUI.
    pub fn new() -> Fallible<GuiSystem> {
        let event_loop = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_dimensions((800, 600).into())
            .with_title("Game");
        let context = ContextBuilder::new()
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core)
            .with_vsync(true);
        let display = Display::new(window, context, &event_loop).map_err(SyncFailure::new)?;

        {
            let window = display.gl_window();
            if let Err(err) = window.grab_cursor(true) {
                error!("Couldn't grab cursor: {}", err);
            } else {
                window.hide_cursor(true);
            }
        }

        Ok(GuiSystem {
            display,
            event_loop,
        })
    }

    /// Gets a reference to the `Facade` wrapped by the `GuiSystem`.
    pub fn facade(&self) -> &impl Facade {
        &self.display
    }
}

impl System for GuiSystem {
    fn step(&mut self, state: &mut State, dt: u64) {
        match state {
            State::Playing(ref mut world) | State::Done(ref mut world, _) => {
                let mut frame = self.display.draw();

                frame.clear_color(
                    world.clear_color[0],
                    world.clear_color[1],
                    world.clear_color[2],
                    world.clear_color[3],
                );
                frame.finish().unwrap();
            }
            _ => {}
        }

        self.event_loop.poll_events(|event| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *state = State::Close,
                _ => trace!("Unhandled event {:#?}", event),
            },
            _ => trace!("Unhandled event {:#?}", event),
        });
    }
}

/// A graphical component.
pub struct RenderComponent {
    /// The model for the component.
    pub model: Arc<Model>,
}

impl_Component!(RenderComponent);
