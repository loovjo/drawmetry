use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use super::geometry;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Thing {
    Point(geometry::Point),
    Shape(geometry::Shape),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum ThingID {
    PointID(geometry::PointID),
    ShapeID(geometry::ShapeID),
}

pub const DEFAULT_GROUP: Group = Group(0);

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct Group(u64);

impl Deref for Group {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Group {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum Visibility {
    Visible,
    Hidden,
}

pub struct GWrapper {
    pub geometry: geometry::Geometry,
    pub groups: HashMap<ThingID, Group>,
    pub visibility: HashMap<Group, Visibility>,
}

impl GWrapper {
    fn new(geometry: geometry::Geometry) -> GWrapper {
        GWrapper {
            geometry,
            groups: HashMap::new(),
            visibility: HashMap::new(),
        }
    }

    fn get_group(&self, id: &ThingID) -> Group {
        *self.groups.get(id).unwrap_or(&DEFAULT_GROUP)
    }

    fn get_visibility(&self, id: &ThingID) -> Visibility {
        *self
            .visibility
            .get(&self.get_group(id))
            .unwrap_or(&Visibility::Visible)
    }
}
