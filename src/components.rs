//! Common components.

use cgmath::{Deg, Matrix3, Matrix4, Point3, Vector3, Vector4};
pub use crate::gui::RenderComponent;

/// A component for an object having a location.
#[derive(Copy, Clone, Debug)]
pub struct LocationComponent {
    /// The location of the object.
    pub xyz: [f32; 3],

    /// The rotation of the object.
    pub rotation: [f32; 3],

    /// The amount to scale by.
    pub scale: f32,
}

impl LocationComponent {
    /// Creates a location component with no rotation and no scale.
    pub fn pos(x: f32, y: f32, z: f32) -> LocationComponent {
        LocationComponent {
            xyz: [x, y, z],
            rotation: [0.0, 0.0, 0.0],
            scale: 1.0,
        }
    }

    /// Moves by the given amount forwards and sideways, adjusted for the rotation matrix.
    pub fn move_by(mut self, forward: f32, strafe: f32) -> LocationComponent {
        let front = Matrix3::from_angle_y(Deg(self.rotation[1])) * Vector3::unit_z();
        let delta: [f32; 3] = (forward * front).into();
        for i in 0..3 {
            self.xyz[i] += delta[i];
        }

        let delta: [f32; 3] = (strafe * front.cross(Vector3::unit_y())).into();
        for i in 0..3 {
            self.xyz[i] += delta[i];
        }

        self
    }

    /// Rotates by the given pitch and yaw.
    pub fn rotate_by(mut self, pitch: f32, yaw: f32) -> LocationComponent {
        // TODO: IMPLEMENT THIS CORRECTLY!!!
        self.rotation[0] += pitch;
        if self.rotation[0] > 85.0 {
            self.rotation[0] = 85.0
        } else if self.rotation[0] < -85.0 {
            self.rotation[0] = -85.0
        }

        self.rotation[1] = (self.rotation[1] + yaw) % 360.0;

        self
    }

    /// Computes the model matrix.
    pub fn model(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.xyz.into())
            * Matrix4::from_angle_z(Deg(self.rotation[2]))
            * Matrix4::from_angle_y(Deg(self.rotation[1]))
            * Matrix4::from_angle_x(Deg(self.rotation[0]))
            * Matrix4::from_scale(self.scale)
    }

    /// Computes the view matrix.
    pub fn view(&self) -> Matrix4<f32> {
        let direction = Matrix4::from_angle_z(Deg(self.rotation[2]))
            * Matrix4::from_angle_y(Deg(self.rotation[1]))
            * Matrix4::from_angle_x(Deg(self.rotation[0]))
            * Vector4::new(0.0, 0.0, 1.0, 0.0);

        Matrix4::look_at_dir(
            Point3::from(self.xyz),
            direction.truncate(),
            Vector3::new(0.0, 1.0, 0.0),
        )
    }
}

impl Default for LocationComponent {
    fn default() -> LocationComponent {
        LocationComponent {
            xyz: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: 1.0,
        }
    }
}

impl_Component!(LocationComponent);

/// A component for the camera.
#[derive(Copy, Clone, Debug, Default)]
pub struct CameraComponent;

impl_Component!(CameraComponent);

/// A component for a door.
#[derive(Copy, Clone, Debug, Default)]
pub struct DoorComponent(pub char);

impl_Component!(DoorComponent);

/// A component for a key.
#[derive(Copy, Clone, Debug, Default)]
pub struct KeyComponent {
    /// The letter of the key.
    pub letter: char,

    /// Whether the key is held by the player.
    pub held: bool,
}

impl_Component!(KeyComponent);

/// A component to allow checking for collisions, based on the radius of the object's
/// LocationComponent. The boolean is whether the object obstructs movement.
#[derive(Copy, Clone, Debug, Default)]
pub struct CollisionComponent(pub bool);

impl_Component!(CollisionComponent);
