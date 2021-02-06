#[derive(Clone, PartialEq)]
pub enum Plane {
    X(f32),
    Y(f32),
    Z(f32),
}

impl Plane {
    pub fn val(&self) -> f32 {
        match self {
            Plane::X(v) => *v,
            Plane::Y(v) => *v,
            Plane::Z(v) => *v,
        }
    }
}
