mod controls;
mod model;
mod render;

pub use crate::gui::{
    controls::ControlSystem,
    model::{Material, Model, Vertex},
    render::{RenderComponent, RenderData},
};
use crate::{State, System};
use failure::{Fallible, SyncFailure};
use glium::{
    backend::Facade,
    draw_parameters::{DepthTest, DrawParameters},
    glutin::{
        dpi::LogicalPosition, Api, ContextBuilder, EventsLoop, GlProfile, GlRequest, WindowBuilder,
    },
    Depth, Display, Surface,
};

/// The GUI system.
pub struct GuiSystem<T> {
    display: Display,
    grab_mouse: bool,
    params: DrawParameters<'static>,
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
    pub fn new(grab_mouse: bool) -> Fallible<(ControlSystem, GuiSystem<()>)> {
        let event_loop = EventsLoop::new();
        let window = WindowBuilder::new()
            .with_dimensions((800, 600).into())
            .with_title("Game");
        let context = ContextBuilder::new()
            .with_depth_buffer(24)
            .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
            .with_gl_profile(GlProfile::Core)
            .with_vsync(true);
        let display = Display::new(window, context, &event_loop).map_err(SyncFailure::new)?;

        if grab_mouse {
            display.gl_window().hide_cursor(true);
        }

        let params = DrawParameters {
            depth: Depth {
                test: DepthTest::IfLess,
                write: true,
                ..Depth::default()
            },
            backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..DrawParameters::default()
        };

        Ok((
            ControlSystem::new(event_loop),
            GuiSystem {
                display,
                grab_mouse,
                params,
                data: (),
            },
        ))
    }

    /// Adds `RenderData` to a `GuiSystem`.
    pub fn add_render_data(self, data: RenderData) -> GuiSystem<RenderData> {
        let mut system = GuiSystem {
            display: self.display,
            grab_mouse: self.grab_mouse,
            params: self.params,
            data,
        };
        system.recompute_proj();
        system
    }
}

impl System for GuiSystem<RenderData> {
    fn step(&mut self, state: &mut State, _dt: u64) {
        // Get the world.
        let world = match state {
            State::Playing(ref mut world) | State::Done(ref mut world, _) => world,
            _ => return,
        };

        // Render the frame.
        let mut frame = self.display.draw();
        frame.clear_color_and_depth(
            (
                self.data.clear_color[0],
                self.data.clear_color[1],
                self.data.clear_color[2],
                self.data.clear_color[3],
            ),
            1.0,
        );
        self.render(world, &mut frame);
        frame.finish().unwrap();

        // Move the mouse.
        if self.grab_mouse {
            self.display
                .gl_window()
                .set_cursor_position(LogicalPosition {
                    x: self.data.dims.width / 2.0,
                    y: self.data.dims.height / 2.0,
                }).ok();
        }
    }
}
