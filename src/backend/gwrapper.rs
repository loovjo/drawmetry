// TODO: REMOVE!
#![allow(unused)]

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
    pub visibility: HashMap<ThingID, Visibility>,
}

impl Deref for GWrapper {
    type Target = geometry::Geometry;
    fn deref(&self) -> &Self::Target {
        &self.geometry
    }
}

impl DerefMut for GWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.geometry
    }
}

impl GWrapper {
    pub fn new(geometry: geometry::Geometry) -> GWrapper {
        GWrapper {
            geometry,
            visibility: HashMap::new(),
        }
    }
}
