mod model;

pub use crate::gui::model::{Model, Vertex};
use crate::{State, System, World};
use failure::{Fallible, SyncFailure};
use glium::{
    backend::Facade,
    glutin::{
        Api, ContextBuilder, Event, EventsLoop, GlProfile, GlRequest, WindowBuilder, WindowEvent,
    },
    Display, Program, Surface,
};
use std::sync::Arc;

/// The GUI system.
pub struct GuiSystem<T> {
    display: Display,
    event_loop: EventsLoop,
    data: T,
}

impl<T> GuiSystem<T> {
    /// Gets a reference to the `Facade` wrapped by the `GuiSystem`.
    pub fn facade(&self) -> &impl Facade {
        &self.display
    }
}

impl GuiSystem<()> {
    /// Sets up the GUI.
    pub fn new() -> Fallible<GuiSystem<()>> {
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
            data: (),
        })
    }

    /// Adds `RenderData` to a `GuiSystem`.
    pub fn add_render_data(self, data: RenderData) -> GuiSystem<RenderData> {
        GuiSystem {
            display: self.display,
            event_loop: self.event_loop,
            data,
        }
    }
}

impl GuiSystem<RenderData> {
    /// Does the work of rendering a frame.
    fn render(&mut self, world: &mut World, frame: &mut impl Surface) {
        unimplemented!()
    }
}

impl System for GuiSystem<RenderData> {
    fn step(&mut self, state: &mut State, _dt: u64) {
        match state {
            State::Playing(ref mut world) | State::Done(ref mut world, _) => {
                let mut frame = self.display.draw();

                frame.clear_color(
                    self.data.clear_color[0],
                    self.data.clear_color[1],
                    self.data.clear_color[2],
                    self.data.clear_color[3],
                );
                self.render(world, &mut frame);
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

/// The data required to render a world.
pub struct RenderData {
    /// The clear color.
    pub clear_color: [f32; 4],

    /// The GLSL program.
    pub program: Program,
}

/// A graphical component.
pub struct RenderComponent {
    /// The model for the component.
    pub model: Arc<Model>,
}

impl_Component!(RenderComponent);
