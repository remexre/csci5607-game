use cgmath::{Deg, Matrix4};

/// A component for an object having a location.
#[derive(Copy, Clone, Debug, Default)]
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

    /// Computes the model matrix.
    pub fn model(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.xyz.into())
            * Matrix4::from_angle_z(Deg(self.rotation[2]))
            * Matrix4::from_angle_y(Deg(self.rotation[1]))
            * Matrix4::from_angle_x(Deg(self.rotation[0]))
            * Matrix4::from_scale(self.scale)
    }
}

impl_Component!(LocationComponent);
