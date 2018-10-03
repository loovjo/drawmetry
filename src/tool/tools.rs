use std::f64;

use super::*;

use backend::{geometry, gwrapper};
use graphics::{get_best, get_closest};
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
    fn selected(&self, _ctx: &gwrapper::GWrapper) -> HashMap<gwrapper::ThingID, SelectedStatus> {
        HashMap::new()
    }
    fn kind(&self) -> ToolKind {
        ToolKind::Point
    }
}

pub struct CircleTool {
    pub center: Option<geometry::PointID>,
}

impl Tool for CircleTool {
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
    fn selected(&self, _ctx: &gwrapper::GWrapper) -> HashMap<gwrapper::ThingID, SelectedStatus> {
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

pub struct LineTool {
    pub edge: Option<geometry::PointID>,
}

impl Tool for LineTool {
    fn click(&mut self, ctx: &mut gwrapper::GWrapper, view: &mut View, at: (f64, f64)) {
        if let Some((&id, _)) = get_closest(
            at,
            ctx.geometry.points.iter().collect(),
            |(_, point)| ctx.geometry.resolve_point(point).unwrap_or((0., 0.)),
            Some(100. / view.transform.scale),
        ) {
            if let Some(edge) = self.edge {
                ctx.geometry.add_shape(geometry::Shape::Line(edge, id));
                self.edge = None;
            } else {
                self.edge = Some(id);
            }
        }
    }
    fn selected(&self, _ctx: &gwrapper::GWrapper) -> HashMap<gwrapper::ThingID, SelectedStatus> {
        let mut res = HashMap::new();
        if let Some(edge) = self.edge {
            res.insert(gwrapper::ThingID::PointID(edge), SelectedStatus::Primary);
        }
        res
    }
    fn kind(&self) -> ToolKind {
        ToolKind::Line
    }
}

pub struct MoverTool {
    pub moving: Option<geometry::PointID>,
}

impl Tool for MoverTool {
    fn click(&mut self, ctx: &mut gwrapper::GWrapper, _view: &mut View, at: (f64, f64)) {
        if let Some((&id, _)) = get_closest(
            at,
            ctx.geometry
                .points
                .iter()
                .filter(|(_, point)| {
                    if let geometry::Point::Arbitrary(_) = point {
                        true
                    } else {
                        false
                    }
                }).collect(),
            |(_, point)| ctx.geometry.resolve_point(point).unwrap_or((0., 0.)),
            None,
        ) {
            self.moving = Some(id);
        }
    }
    fn selected(&self, ctx: &gwrapper::GWrapper) -> HashMap<gwrapper::ThingID, SelectedStatus> {
        let mut res = HashMap::new();
        for (id, point) in &ctx.geometry.points {
            if let geometry::Point::Arbitrary(_) = point {
                res.insert(gwrapper::ThingID::PointID(*id), SelectedStatus::Active);
            }
        }

        if let Some(moving) = self.moving {
            res.insert(gwrapper::ThingID::PointID(moving), SelectedStatus::Primary);
        }
        res
    }
    fn kind(&self) -> ToolKind {
        ToolKind::Mover
    }
}

pub struct Selector {
    pub selected: Vec<gwrapper::ThingID>,
}

impl Tool for Selector {
    fn click(&mut self, ctx: &mut gwrapper::GWrapper, view: &mut View, at: (f64, f64)) {
        let mut objects = Vec::<(gwrapper::ThingID, gwrapper::Thing)>::new();

        for (id, point) in &ctx.geometry.points {
            objects.push((
                gwrapper::ThingID::PointID(*id),
                gwrapper::Thing::Point(*point),
            ));
        }
        for (id, shape) in &ctx.geometry.shapes {
            objects.push((
                gwrapper::ThingID::ShapeID(*id),
                gwrapper::Thing::Shape(*shape),
            ));
        }

        let point_bonus = 25. / view.transform.scale;

        let dist_fn = |(_, obj): &(gwrapper::ThingID, gwrapper::Thing)| match obj {
            gwrapper::Thing::Point(p) => {
                if let Some(pos) = ctx.geometry.resolve_point(&p) {
                    let (dx, dy) = (pos.0 - at.0, pos.1 - at.1);
                    (dx * dx + dy * dy).sqrt()
                } else {
                    f64::MAX
                }
            }
            gwrapper::Thing::Shape(s) => {
                let shape = ctx.geometry.resolve_shape(&s);
                match shape {
                    Some(geometry::ResolvedShape::Circle(pos, rad)) => {
                        let (dx, dy) = (pos.0 - at.0, pos.1 - at.1);
                        let dist = (dx * dx + dy * dy).sqrt();
                        (dist - rad).abs() + point_bonus
                    }
                    Some(geometry::ResolvedShape::Line(k, m)) => {
                        let k_ = -1. / k;
                        let m_ = at.1 - k_ * at.0;

                        let int_x = (m - m_) / (k_ - k);
                        let int_y = k * int_x + m;

                        let (dx, dy) = (int_x - at.0, int_y - at.1);

                        (dx * dx + dy * dy).sqrt() + point_bonus
                    }
                    Some(geometry::ResolvedShape::LineUp(k)) => (k - at.0).abs() + point_bonus,
                    None => f64::MAX,
                }
            }
        };

        if let Some((_, (id, _))) = get_best(objects, dist_fn) {
            println!("Best: {:?}", id);
            self.selected.push(id);
        }
    }
    fn selected(&self, _ctx: &gwrapper::GWrapper) -> HashMap<gwrapper::ThingID, SelectedStatus> {
        let mut res = HashMap::new();

        for x in &self.selected {
            res.insert(*x, SelectedStatus::Active);
        }

        res
    }
    fn kind(&self) -> ToolKind {
        ToolKind::Selector
    }
}
