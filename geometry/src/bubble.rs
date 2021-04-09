use crate::debug_util::is_finite;
use crate::{Aabb, Boundable, Container, Geometry, Intersectable, Intersection, Ray, Sphere};
use definitions::{Float, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Bubble {
    pub inner: Sphere,
    pub outer: Sphere,
}

impl Bubble {
    pub fn new(
        center: Vector3,
        outer_radius: Float,
        thickness: Float,
        inner_offset: Vector3,
    ) -> Self {
        debug_assert!(is_finite(&center));
        debug_assert!(outer_radius.is_finite());
        debug_assert!(outer_radius > 0.0);
        debug_assert!(thickness.is_finite());
        debug_assert!(thickness > 0.0);
        debug_assert!(outer_radius - thickness > 0.0);
        debug_assert!(is_finite(&inner_offset));
        debug_assert!(inner_offset.component_max() < thickness);

        let inner = Sphere::new(center + inner_offset, outer_radius - thickness);
        let outer = Sphere::new(center, outer_radius);

        Self { inner, outer }
    }
}

impl Boundable for Bubble {
    fn bounds(&self) -> Aabb {
        self.outer.bounds()
    }
}

impl Container for Bubble {
    fn contains(&self, point: &Vector3) -> bool {
        self.outer.contains(point) && !self.inner.contains(point)
    }
}

impl Intersectable for Bubble {
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        let outer = self.outer.intersect(ray);
        let inner = self.inner.intersect(ray);

        if let Some(o) = outer {
            if let Some(mut i) = inner {
                if o.t < i.t {
                    Some(o)
                } else {
                    // invert the "outer normal", as we hit the bubble from the inside
                    i.normal = -i.normal;
                    Some(i)
                }
            } else {
                Some(o)
            }
        } else {
            inner
        }
    }

    fn intersects(&self, ray: &Ray) -> bool {
        self.outer.intersects(ray) || self.inner.intersects(ray)
    }
}

#[typetag::serde]
impl Geometry for Bubble {}
