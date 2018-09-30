use super::*;

use backend::{geometry, gwrapper};
use graphics::get_closest;
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

pub struct PointCircle {
    pub center: Option<geometry::PointID>,
}

impl Tool for PointCircle {
    fn click(&mut self, ctx: &mut gwrapper::GWrapper, view: &mut View, at: (f64, f64)) {
        if let Some((&id, _)) = get_closest(
            at,
            ctx.geometry.points.iter().collect(),
            |(_, point)| ctx.geometry.resolve_point(point).unwrap_or((0., 0.)),
            Some(100. / view.transform.scale),
        ) {
            if let Some(center) = self.center {
                ctx.geometry.add_shape(geometry::Shape::Circle(center, id));
                self.center = None;
            } else {
                self.center = Some(id);
            }
        }
    }
    fn selected(&self) -> HashMap<gwrapper::ThingID, SelectedStatus> {
        let mut res = HashMap::new();
        if let Some(center) = self.center {
            res.insert(gwrapper::ThingID::PointID(center), SelectedStatus::Primary);
        }
        res
    }
    fn kind(&self) -> ToolKind {
        ToolKind::Circle
    }
}
