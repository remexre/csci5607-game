use cgmath::{InnerSpace, Vector3};
use crate::{
    components::{CameraComponent, CollisionComponent, LocationComponent},
    State, System,
};
use glium::glutin::{DeviceEvent, ElementState, Event, EventsLoop, WindowEvent};
use smallvec::SmallVec;

/// The control system.
pub struct ControlSystem {
    event_loop: EventsLoop,

    move_forward: f32,
    move_strafe: f32,
}

impl ControlSystem {
    /// Creates a ControlSystem around an EventsLoop.
    pub fn new(event_loop: EventsLoop) -> ControlSystem {
        ControlSystem {
            event_loop,

            move_forward: 0.0,
            move_strafe: 0.0,
        }
    }
}

impl System for ControlSystem {
    fn step(&mut self, state: &mut State, _dt: u64) {
        let mut move_pitch = 0.0;
        let mut move_yaw = 0.0;

        // Handle input events.
        let mut events = SmallVec::<[_; 4]>::new();
        self.event_loop.poll_events(|event| events.push(event));
        for event in events {
            match event {
                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::Key(event) => match event.state {
                        ElementState::Pressed => match event.scancode {
                            1 => *state = State::Close,     // Escape
                            17 => self.move_forward = 1.0,  // W
                            30 => self.move_strafe = -1.0,  // A
                            31 => self.move_forward = -1.0, // S
                            32 => self.move_strafe = 1.0,   // D
                            _ => {}
                        },
                        ElementState::Released => match event.scancode {
                            17 | 31 => self.move_forward = 0.0, // W, S
                            30 | 32 => self.move_strafe = 0.0,  // A, D
                            _ => {}
                        },
                    },
                    DeviceEvent::MouseMotion { delta: (x, y) } => {
                        move_yaw -= x as f32;
                        move_pitch += y as f32;
                    }
                    _ => {}
                },
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *state = State::Close,
                    WindowEvent::Resized(_) => {
                        // TODO: self.recompute_proj
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // Get the world.
        let world = match state {
            State::Playing(ref mut world) | State::Done(ref mut world, _) => world,
            _ => return,
        };

        // Get the camera.
        let camera = match world.iter().next() {
            Some((ent, hlist_pat![CameraComponent])) => ent,
            None => {
                warn!("No camera?");
                return;
            }
        };

        // Apply motion to the camera, if no collisions occur.
        let old_loc = *world
            .get_one::<LocationComponent>(camera)
            .expect("Camera didn't have a location?");
        let mut new_loc = old_loc.move_by(self.move_forward / 20.0, self.move_strafe / 20.0);
        let pos = Vector3::from(new_loc.xyz);

        for (_, hlist_pat![&CollisionComponent(c), &LocationComponent{xyz, scale,..}]) in
            world.iter()
        {
            if !c {
                continue;
            }

            let distance = (Vector3::from(xyz) - pos).magnitude();
            let min_distance = (scale + new_loc.scale) * 2f32.sqrt() / 2.0;
            if distance < min_distance {
                new_loc = old_loc;
                break;
            }
        }

        let camera_loc = world
            .get_mut::<LocationComponent>(camera)
            .expect("Camera didn't have a location?");
        *camera_loc = new_loc.rotate_by(move_pitch / 10.0, move_yaw / 10.0);
    }
}
