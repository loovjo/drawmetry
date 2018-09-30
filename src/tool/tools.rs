use super::*;

use graphics::get_closest;
use backend::{geometry, gwrapper};
use std::collections::HashMap;

pub struct PointTool {}

impl Tool for PointTool {
    fn click(&mut self, ctx: &mut gwrapper::GWrapper, view: &mut View, at: (f64, f64)) {
        let point = get_closest(
            at,
            ctx.geometry.get_potential_points(),
            |point| ctx.geometry.resolve_point(point).unwrap_or((0., 0.)),
            Some(100. / view.transform.scale),
        ).unwrap_or(geometry::create_arbitrary(at));

        ctx.geometry.add_point(point);
    }
    fn selected(&self) -> HashMap<gwrapper::ThingID, SelectedStatus> {
        HashMap::new()
    }
    fn kind(&self) -> ToolKind {
        ToolKind::Point
    }
}
