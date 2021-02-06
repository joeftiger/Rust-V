use crate::{Aabb, Boundable};
use std::collections::HashSet;
use std::sync::Arc;

/// An item consists of a primitive [`Boundable](Boundable) and an id to be hashed.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Item<T>
where
    T: Boundable,
{
    pub value: Arc<T>,
    pub id: u32,
}

impl<T> Item<T>
where
    T: Boundable,
{
    pub fn new(value: T, id: u32) -> Self {
        Self {
            value: Arc::new(value),
            id,
        }
    }
}

pub type Items<T> = Vec<Arc<Item<T>>>;

#[derive(Clone, Debug)]
pub struct InternalNode<T>
where
    T: Boundable,
{
    left_bounds: Aabb,
    left_node: KDtreeNode<T>,
    right_bounds: Aabb,
    right_node: KDtreeNode<T>,
}

/// a tree node
#[derive(Clone, Debug)]
pub enum KDtreeNode<T>
where
    T: Boundable,
{
    Leaf { items: HashSet<Arc<Item<T>>> },
}
