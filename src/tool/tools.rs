use super::*;

use backend::{geometry, gwrapper};
use std::collections::HashMap;

pub struct PointTool {}

impl Tool for PointTool {
    fn click(&mut self, ctx: &mut gwrapper::GWrapper, at: (f64, f64)) {
        println!("Click at {:?}", at);
    }
    fn selected(&self) -> HashMap<gwrapper::ThingID, SelectedStatus> {
        HashMap::new()
    }
    fn kind(&self) -> ToolKind {
        ToolKind::Point
    }
}
