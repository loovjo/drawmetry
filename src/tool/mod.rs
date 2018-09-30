pub mod tools;

use backend::{geometry, gwrapper};
use std::collections::HashMap;

pub trait Tool: Send {
    fn click(&mut self, ctx: &mut gwrapper::GWrapper, at: (f64, f64));
    fn selected(&self) -> HashMap<gwrapper::ThingID, SelectedStatus>;
    fn kind(&self) -> ToolKind;
}

pub enum SelectedStatus {
    Primary,
    Active,
}

#[derive(Clone)]
pub enum ToolKind {
    Point,
    //Circle,
    //Line,
    //Mover,
}

impl ToolKind {
    pub fn into_tool(self) -> Box<dyn Tool> {
        match self {
            ToolKind::Point => Box::new(tools::PointTool {}),
            //ToolKind::Circle => {}
            //ToolKind::Line => {}
            //ToolKind::Mover => {}
        }
    }
}
