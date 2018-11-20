/// A component for an object having a location.
#[derive(Copy, Clone, Debug)]
pub struct LocationComponent(pub f32, pub f32, pub f32);

impl_Component!(LocationComponent);
