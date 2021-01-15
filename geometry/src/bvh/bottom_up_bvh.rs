use crate::{Aabb, Boundable, Intersectable, Intersection, Ray};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

pub struct BottomUpBVH<T> {
    pub aabb: Aabb,
    pub children: Vec<Arc<BottomUpBVH<T>>>,
    pub objects: Vec<T>,
}

impl<T> Default for BottomUpBVH<T> {
    fn default() -> Self {
        Self::new(Aabb::empty(), vec![], vec![])
    }
}

impl<T> BottomUpBVH<T> {
    pub fn new(aabb: Aabb, children: Vec<Arc<BottomUpBVH<T>>>, objects: Vec<T>) -> Self {
        Self {
            aabb,
            children,
            objects,
        }
    }
}

impl<T> BottomUpBVH<T>
where
    T: Boundable,
{
    pub fn create_from_vec(objects: Vec<T>) -> Arc<Self> {
        Self::create(objects.into_iter().enumerate().collect())
    }

    pub fn create(mut objects: HashMap<usize, T>) -> Arc<Self> {
        if objects.is_empty() {
            return Arc::new(Self::default());
        } else if objects.len() == 1 {
            let object = objects.drain().next().unwrap();
            let aabb = object.1.bounds();

            return Arc::new(Self::new(aabb, vec![], vec![object.1]));
        } else if objects.len() == 2 {
            let mut drain = objects.drain();
            let o1 = drain.next().unwrap();
            let o2 = drain.next().unwrap();
            let aabb = o1.1.bounds().join(&o2.1.bounds());

            return Arc::new(Self::new(aabb, vec![], vec![o1.1, o2.1]));
        }

        let mut nodes: HashMap<usize, Arc<Self>> = HashMap::default();
        let mut node_counter = 0;

        // create tree by closest bounding box center distances.
        while !objects.is_empty() || nodes.len() > 1 {
            let mut oo = None;
            let mut on = None;
            let mut nn = None;

            let mut distance = f32::INFINITY;

            objects.iter().for_each(|first| {
                objects.iter().for_each(|second| {
                    if first.0 != second.0 {
                        let d = (first.1.bounds().center() - second.1.bounds().center()).mag();
                        if d < distance {
                            distance = d;
                            oo = Some((*first.0, *second.0));
                            on = None;
                            nn = None;
                        }
                    }
                });

                nodes.iter_mut().for_each(|second| {
                    let d = (first.1.bounds().center() - second.1.bounds().center()).mag();
                    if d < distance {
                        distance = d;
                        oo = None;
                        on = Some((*first.0, *second.0));
                        nn = None;
                    }
                })
            });

            nodes.iter().for_each(|first| {
                nodes.iter().for_each(|second| {
                    if first.0 != second.0 {
                        let d = (first.1.bounds().center() - second.1.bounds().center()).mag();
                        if d < distance {
                            distance = d;
                            oo = None;
                            on = None;
                            nn = Some((*first.0, *second.0));
                        }
                    }
                })
            });

            let (children, objects) = if let Some(oo) = oo {
                let o1 = objects
                    .remove(&oo.0)
                    .expect("Key was not in objects map anymore");
                let o2 = objects
                    .remove(&oo.1)
                    .expect("Key was not in objects map anymore");

                (vec![], vec![o1, o2])
            } else if let Some(on) = on {
                let o = objects
                    .remove(&on.0)
                    .expect("Key was not in objects map anymore");
                let n = nodes
                    .remove(&on.1)
                    .expect("Key was not in nodes map anymore");

                (vec![n], vec![o])
            } else if let Some(nn) = nn {
                let n1 = nodes
                    .remove(&nn.0)
                    .expect("Key was not in nodes map anymore");
                let n2 = nodes
                    .remove(&nn.1)
                    .expect("Key was not in nodes map anymore");

                (vec![n1, n2], vec![])
            } else {
                unreachable!("Unreachable. Is a cube infinite?");
            };

            let cube = children
                .iter()
                .map(|c| c.bounds())
                .chain(objects.iter().map(|o| o.bounds()))
                .fold(Aabb::empty(), |acc, next| acc.join(&next));

            let key = node_counter;
            node_counter += 1;

            let new_node = Self::new(cube, children, objects);
            nodes.insert(key, Arc::new(new_node));
        }

        assert_eq!(nodes.len(), 1);

        let super_node = nodes.drain().next().unwrap();
        super_node.1
    }

    fn build_tree() {}

    fn combine_clusters() {}
}

impl<T> Boundable for BottomUpBVH<T>
where
    T: Boundable,
{
    fn bounds(&self) -> Aabb {
        self.aabb
    }
}

impl<T> Intersectable for BottomUpBVH<T>
where
    T: Intersectable,
{
    fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        if !self.aabb.intersects(ray) {
            return None;
        }

        let mut pruned_ray = *ray;

        self.objects
            .iter()
            .map(|o| {
                let i: &dyn Intersectable = o;
                i
            })
            .chain(self.children.iter().map(|c| {
                let i: &dyn Intersectable = c.deref();
                i
            }))
            .filter_map(|o| {
                if let Some(i) = o.intersect(&pruned_ray) {
                    pruned_ray.t_end = i.t;
                    Some(i)
                } else {
                    None
                }
            })
            .next()
    }

    fn intersects(&self, ray: &Ray) -> bool {
        self.objects.iter().any(|o| o.intersects(ray))
            || self.children.iter().any(|c| c.intersects(ray))
    }
}
