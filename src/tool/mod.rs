pub mod tools;

use backend::gwrapper;
use drawing_board::View;
use std::collections::HashMap;

pub trait Tool: Send {
    fn click(&mut self, ctx: &mut gwrapper::GWrapper, view: &mut View, at: (f64, f64));
    fn selected(&self, ctx: &gwrapper::GWrapper) -> HashMap<gwrapper::ThingID, SelectedStatus>;
    fn kind(&self) -> ToolKind;
}

#[derive(Clone, PartialEq)]
pub enum SelectedStatus {
    Primary,
    Active,
}

#[derive(Clone, PartialEq)]
pub enum ToolKind {
    Point,
    Circle,
    Line,
    Mover,
}

impl ToolKind {
    pub fn into_tool(self) -> Box<dyn Tool> {
        match self {
            ToolKind::Point => Box::new(tools::PointTool {}),
            ToolKind::Circle => Box::new(tools::CircleTool { center: None }),
            ToolKind::Line => Box::new(tools::LineTool { edge: None }),
            ToolKind::Mover => Box::new(tools::MoverTool { moving: None }),
        }
    }
}
