use crate::Vector2;
use serde::{Deserialize, Serialize};
use ultraviolet::UVec2;

pub type UBounds2 = Bound<UVec2>;
pub type Bounds2 = Bound<Vector2>;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Bound<T> {
    pub min: T,
    pub max: T,
}

impl<T> Bound<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<T: Default> From<T> for Bound<T> {
    fn from(max: T) -> Self {
        Self::new(T::default(), max)
    }
}

fn in_range<T: PartialOrd>(t: T, min: T, max: T) -> bool {
    min <= t && t <= max
}

impl Bound<Vector2> {
    pub fn contains(&self, p: Vector2) -> bool {
        in_range(p.x, self.min.x, self.max.x) && in_range(p.y, self.min.y, self.max.y)
    }
}

impl Default for Bounds2 {
    fn default() -> Self {
        Self {
            min: Default::default(),
            max: Default::default(),
        }
    }
}

impl Bound<UVec2> {
    pub fn to_range(&self) -> UVec2 {
        self.max - self.min
    }

    pub fn contains(&self, p: UVec2) -> bool {
        in_range(p.x, self.min.x, self.max.x) && in_range(p.y, self.min.y, self.max.y)
    }

    pub fn area(&self) -> usize {
        let diff = self.to_range();
        (diff.x * diff.y) as usize
    }
}

impl Default for UBounds2 {
    fn default() -> Self {
        Self {
            min: Default::default(),
            max: Default::default(),
        }
    }
}
