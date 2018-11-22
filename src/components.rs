//! Common components.

use cgmath::{Deg, InnerSpace, Matrix3, Matrix4, Point3, Vector3, Vector4};
pub use crate::gui::{DecalComponent, RenderComponent};

/// A component for an object having a location.
#[derive(Copy, Clone, Debug)]
pub struct LocationComponent {
    /// The location of the object.
    pub xyz: Point3<f32>,

    /// The rotation of the object.
    pub rotation: Vector3<f32>,

    /// The amount to scale by.
    pub scale: f32,
}

impl LocationComponent {
    /// Creates a location component with no rotation and no scale.
    pub fn pos(x: f32, y: f32, z: f32) -> LocationComponent {
        LocationComponent {
            xyz: Point3::new(x, y, z),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: 1.0,
        }
    }

    /// Moves by the given amount forwards and sideways, adjusted for the rotation matrix.
    pub fn move_by(mut self, forward: f32, strafe: f32) -> LocationComponent {
        let front = Matrix3::from_angle_y(Deg(self.rotation[1])) * Vector3::unit_z();
        self.xyz += forward * front;
        self.xyz += strafe * front.cross(Vector3::unit_y());
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

    /// Returns a forward direction.
    pub fn forward(&self) -> Vector3<f32> {
        (Matrix4::from_angle_z(Deg(self.rotation[2]))
            * Matrix4::from_angle_y(Deg(self.rotation[1]))
            * Matrix4::from_angle_x(Deg(self.rotation[0]))
            * Vector4::unit_z()).truncate()
    }

    /// Computes the model matrix.
    pub fn model(&self) -> Matrix4<f32> {
        Matrix4::from_translation(Vector3::new(self.xyz.x, self.xyz.y, self.xyz.z))
            * Matrix4::from_angle_z(Deg(self.rotation[2]))
            * Matrix4::from_angle_y(Deg(self.rotation[1]))
            * Matrix4::from_angle_x(Deg(self.rotation[0]))
            * Matrix4::from_scale(self.scale)
    }

    /// Computes the view matrix.
    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_dir(self.xyz, self.forward(), Vector3::new(0.0, 1.0, 0.0))
    }

    /// Returns whether the two objects collide.
    pub fn collides(&self, other: &LocationComponent) -> bool {
        let distance = (self.xyz - other.xyz).magnitude();
        let min_distance = (self.scale + other.scale) * 2f32.sqrt() / 2.0;
        distance < min_distance
    }
}

impl Default for LocationComponent {
    fn default() -> LocationComponent {
        LocationComponent {
            xyz: Point3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
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

/// A component for the goal location.
#[derive(Copy, Clone, Debug, Default)]
pub struct GoalComponent;

impl_Component!(GoalComponent);

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
