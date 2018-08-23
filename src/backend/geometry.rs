use std::iter::IntoIterator;

use std::collections::HashMap;
use std::ops::Deref;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct PointID(u64);
impl Deref for PointID {
    type Target = u64;
    fn deref(&self) -> &u64 {
        &self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ShapeID(u64);
impl Deref for ShapeID {
    type Target = u64;
    fn deref(&self) -> &u64 {
        &self.0
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Point {
    PrimIntersection(ShapeID, ShapeID),
    SecIntersection(ShapeID, ShapeID),
    Arbitrary((f64, f64)),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Shape {
    /// (center, point on circumference),
    Circle(PointID, PointID),
    /// Two points on the line
    Line(PointID, PointID),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResolvedShape {
    /// (center, radius),
    Circle((f64, f64), f64),
    /// y = kx + m (k, m)
    Line(f64, f64),
    /// x = k
    LineUp(f64),
}

impl ResolvedShape {
    pub fn as_eq(&self) -> String {
        match self {
            ResolvedShape::Circle((cx, cy), r) => {
                let x_part = match -*cx {
                    a if a.abs() < 1e-9 => "x".into(),
                    a if a > 0. => format!("(x+{})", a),
                    a => format!("(x-{})", -a),
                };
                let y_part = match -*cy {
                    a if a.abs() < 1e-9 => "y".into(),
                    a if a > 0. => format!("(y+{})", a),
                    a => format!("(y-{})", -a),
                };
                format!("{}^2 + {}^2 = {}^2", x_part, y_part, r)
            }
            ResolvedShape::Line(k, m) => {
                let kx_part = match *k {
                    a if a.abs() < 1e-9 => "".into(),
                    a if (a - 1.).abs() < 1e-9 => "x".into(),
                    a if (a + 1.).abs() < 1e-9 => "-x".into(),
                    a => format!("{}x", a),
                };
                let m_part = match *m {
                    a if a.abs() < 1e-9 => "".into(),
                    a if a < 0. => format!("- {:.9?}", -a),
                    a => format!("+ {:.9?}", a),
                };
                if kx_part == "" && m_part == "" {
                    "y = 0".into()
                } else {
                    format!("y = {} {}", kx_part, m_part)
                }
            }
            ResolvedShape::LineUp(x) => format!("x = {}", x),
        }
    }
}

pub struct Geometry {
    pub shapes: HashMap<ShapeID, Shape>,
    pub points: HashMap<PointID, Point>,
    last_shape: ShapeID,
    last_point: PointID,
}

impl Geometry {
    pub fn new() -> Geometry {
        Geometry {
            shapes: HashMap::new(),
            points: HashMap::new(),
            last_shape: ShapeID(0),
            last_point: PointID(0),
        }
    }

    pub fn add_shape(&mut self, shape: Shape) -> ShapeID {
        let id = self.next_shape_id();
        self.shapes.insert(id, shape);
        id
    }

    pub fn add_point(&mut self, point: Point) -> PointID {
        let id = self.next_point_id();
        self.points.insert(id, point);
        id
    }

    fn next_shape_id(&mut self) -> ShapeID {
        loop {
            self.last_shape = ShapeID(*self.last_shape + 1);
            if !self.shapes.contains_key(&self.last_shape) {
                return self.last_shape;
            }
        }
    }

    fn next_point_id(&mut self) -> PointID {
        loop {
            self.last_point = PointID(*self.last_point + 1);
            if !self.points.contains_key(&self.last_point) {
                return self.last_point;
            }
        }
    }

    pub fn resolve_point(&self, id: PointID) -> Option<(f64, f64)> {
        let point = self.points.get(&id)?;
        match point {
            Point::Arbitrary(pos) => Some(*pos),
            Point::PrimIntersection(a, b) | Point::SecIntersection(a, b) => {
                let obj_a = self.shapes.get(&a)?;
                let obj_b = self.shapes.get(&b)?;

                let intersection = match (obj_a, obj_b) {
                    (Shape::Circle(cent1_pos, circ1_pos), Shape::Circle(cent2_pos, circ2_pos)) => {
                        let (cent1, circ1, cent2, circ2) = (
                            self.resolve_point(*cent1_pos)?,
                            self.resolve_point(*circ1_pos)?,
                            self.resolve_point(*cent2_pos)?,
                            self.resolve_point(*circ2_pos)?,
                        );

                        let rad1_sq = (cent1.0 - circ1.0) * (cent1.0 - circ1.0)
                            + (cent1.1 - circ1.1) * (cent1.1 - circ1.1);
                        let rad1 = rad1_sq.sqrt();

                        let rad2_sq = (cent2.0 - circ2.0) * (cent2.0 - circ2.0)
                            + (cent2.1 - circ2.1) * (cent2.1 - circ2.1);
                        let rad2 = rad2_sq.sqrt();

                        intersect_two_circles(cent1, rad1, cent2, rad2)
                    }
                    (Shape::Circle(cent1_pos, circ1_pos), Shape::Line(point1_pos, point2_pos))
                    | (Shape::Line(point1_pos, point2_pos), Shape::Circle(cent1_pos, circ1_pos)) => {
                        let (cent1, circ1, point1, point2) = (
                            self.resolve_point(*cent1_pos)?,
                            self.resolve_point(*circ1_pos)?,
                            self.resolve_point(*point1_pos)?,
                            self.resolve_point(*point2_pos)?,
                        );

                        let rad1_sq = (cent1.0 - circ1.0) * (cent1.0 - circ1.0)
                            + (cent1.1 - circ1.1) * (cent1.1 - circ1.1);
                        let rad1 = rad1_sq.sqrt();

                        intersect_circle_line(cent1, rad1, point1, point2)
                    }
                    (Shape::Line(point1_pos, point2_pos), Shape::Line(point3_pos, point4_pos)) => {
                        let (point1, point2, point3, point4) = (
                            self.resolve_point(*point1_pos)?,
                            self.resolve_point(*point2_pos)?,
                            self.resolve_point(*point3_pos)?,
                            self.resolve_point(*point4_pos)?,
                        );
                        intersect_line_line(point1, point2, point3, point4)
                    }
                    _ => IntersectionResult::None,
                };

                match (point, intersection) {
                    (_, IntersectionResult::One(point)) => Some(point),
                    (Point::PrimIntersection(..), IntersectionResult::Two(a, _)) => Some(a),
                    (Point::SecIntersection(..), IntersectionResult::Two(_, b)) => Some(b),
                    _ => None,
                }
            }
        }
    }

    pub fn resolve_shape(&self, id: ShapeID) -> Option<ResolvedShape> {
        match self.shapes.get(&id)? {
            Shape::Circle(center_pos, cpoint_pos) => {
                let (center, cpoint) = (
                    self.resolve_point(*center_pos)?,
                    self.resolve_point(*cpoint_pos)?,
                );
                let rad_sq = (center.0 - cpoint.0) * (center.0 - cpoint.0)
                    + (center.1 - cpoint.1) * (center.1 - cpoint.1);
                let rad = rad_sq.sqrt();
                Some(ResolvedShape::Circle(center, rad))
            }
            Shape::Line(p1_pos, p2_pos) => {
                let (p1, p2) = (self.resolve_point(*p1_pos)?, self.resolve_point(*p2_pos)?);
                if p1.0 == p2.0 {
                    return Some(ResolvedShape::LineUp(p1.0));
                }
                let k = (p1.1 - p2.1) / (p1.0 - p2.0);
                // y - y0 = k(x - x0)
                // y = kx - kx0 + y0
                let m = -k * p1.0 + p1.1;
                Some(ResolvedShape::Line(k, m))
            }
        }
    }
}

#[derive(Debug)]
enum IntersectionResult {
    None,
    One((f64, f64)),
    Two((f64, f64), (f64, f64)),
}

impl PartialEq for IntersectionResult {
    fn eq(&self, other: &IntersectionResult) -> bool {
        use self::IntersectionResult::*;

        match (self, other) {
            (None, None) => true,
            (One(p1), One(p2)) => p1 == p2,
            (Two(a1, b1), Two(a2, b2)) => (a1 == a2 && b1 == b2) || (a1 == b2 && b1 == a2),
            _ => false,
        }
    }
}

use std;
impl IntoIterator for IntersectionResult {
    type Item = (f64, f64);
    type IntoIter = std::vec::IntoIter<(f64, f64)>;

    fn into_iter(self) -> Self::IntoIter {
        use self::IntersectionResult::*;

        let v = match self {
            None => vec![],
            One(a) => vec![a],
            Two(a, b) => vec![a, b],
        };
        v.into_iter()
    }
}

// Ported from https://gist.github.com/jupdike/bfe5eb23d1c395d8a0a1a4ddd94882ac
// x1,y1 is the center of the first circle, with radius r1
// x2,y2 is the center of the second ricle, with radius r2
#[allow(non_snake_case)]
fn intersect_two_circles(
    (x1, y1): (f64, f64),
    r1: f64,
    (x2, y2): (f64, f64),
    r2: f64,
) -> IntersectionResult {
    let centerdx = x1 - x2;
    let centerdy = y1 - y2;
    let R = (centerdx * centerdx + centerdy * centerdy).sqrt();
    if !((r1 - r2).abs() <= R && R <= r1 + r2) {
        // no intersection
        return IntersectionResult::None; // empty list of results
    }
    // intersection(s) should exist

    let R2 = R * R;
    let R4 = R2 * R2;
    let a = (r1 * r1 - r2 * r2) / (2. * R2);
    let r2r2 = r1 * r1 - r2 * r2;
    let c = (2. * (r1 * r1 + r2 * r2) / R2 - (r2r2 * r2r2) / R4 - 1.).sqrt();

    let fx = (x1 + x2) / 2. + a * (x2 - x1);
    let gx = c * (y2 - y1) / 2.;
    let ix1 = fx + gx;
    let ix2 = fx - gx;

    let fy = (y1 + y2) / 2. + a * (y2 - y1);
    let gy = c * (x1 - x2) / 2.;
    let iy1 = fy + gy;
    let iy2 = fy - gy;

    if gy == 0. && gx == 0. {
        return IntersectionResult::One((ix1, iy1));
    }
    // note if gy == 0 and gx == 0 then the circles are tangent and there is only one solution
    // but that one solution will just be duplicated as the code is currently written
    return IntersectionResult::Two((ix1, iy1), (ix2, iy2));
}

// Taken from http://mathworld.wolfram.com/Circle-LineIntersection.html
#[allow(non_snake_case)]
fn intersect_circle_line(
    (cx, cy): (f64, f64),
    r: f64,
    (lx1, ly1): (f64, f64),
    (lx2, ly2): (f64, f64),
) -> IntersectionResult {
    let (x1, y1) = (lx1 - cx, ly1 - cy);
    let (x2, y2) = (lx2 - cx, ly2 - cy);

    let (dx, dy) = (x2 - x1, y2 - y1);
    let dr_sq = dx * dx + dy * dy;
    let D = x1 * y2 - x2 * y1;

    let disc = r * r * dr_sq - D * D;
    if disc < 0. {
        return IntersectionResult::None;
    }

    let rx1 = cx + (D * dy + sgn(dy) * dx * disc.sqrt()) / dr_sq;
    let rx2 = cx + (D * dy - sgn(dy) * dx * disc.sqrt()) / dr_sq;
    let ry1 = cy + (-D * dx + dy.abs() * disc.sqrt()) / dr_sq;
    let ry2 = cy + (-D * dx - dy.abs() * disc.sqrt()) / dr_sq;

    if disc == 0. {
        IntersectionResult::One((rx1, ry1))
    } else {
        IntersectionResult::Two((rx1, ry1), (rx2, ry2))
    }
}

// Taken from https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection
fn intersect_line_line(
    (x1, y1): (f64, f64),
    (x2, y2): (f64, f64),
    (x3, y3): (f64, f64),
    (x4, y4): (f64, f64),
) -> IntersectionResult {
    let den = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    if den == 0. {
        return IntersectionResult::None;
    }
    let num_x = (x1 * y2 - y1 * x2) * (x3 - x4) - (x1 - x2) * (x3 * y4 - y3 * x4);
    let num_y = (x1 * y2 - y1 * x2) * (y3 - y4) - (y1 - y2) * (x3 * y4 - y3 * x4);
    IntersectionResult::One((num_x / den, num_y / den))
}

fn sgn(x: f64) -> f64 {
    if x < 0. {
        -1.
    } else {
        1.
    }
}

#[test]
fn test_intersect() {
    assert_eq!(IntersectionResult::None, IntersectionResult::None);
    assert_eq!(
        IntersectionResult::One((3., 5.)),
        IntersectionResult::One((3., 5.))
    );
    assert_ne!(
        IntersectionResult::One((1., 2.)),
        IntersectionResult::One((2., 1.))
    );
    assert_eq!(
        IntersectionResult::Two((1., 2.), (3., 4.)),
        IntersectionResult::Two((1., 2.), (3., 4.))
    );
    assert_eq!(
        IntersectionResult::Two((3., 2.), (1., 4.)),
        IntersectionResult::Two((1., 4.), (3., 2.))
    );

    assert_eq!(
        IntersectionResult::One((1., 5.)),
        intersect_two_circles((3., 5.), 2., -1., (5., 2.))
    );

    assert_eq!(
        IntersectionResult::None,
        intersect_two_circles(0., 0., 0., 1., 1., 0.),
    );

    assert_eq!(
        IntersectionResult::None,
        intersect_two_circles((0., 0.), 0., (1., 1.), 0.),
    );

    assert_eq!(
        IntersectionResult::Two((3., 3.), (7., 1.)),
        intersect_circle_line((7., 6.), 5., (5., 2.), (9., 0.)),
    );

    assert_eq!(
        IntersectionResult::One((0., 5.)),
        intersect_circle_line((0., 0.), 5., (0., 5.), (1., 5.)),
    );
}

#[test]
quickcheck! {
    fn check_intersect_two_circles(
        x1: f64,
        y1: f64,
        r1: f64,
        x2: f64,
        y2: f64,
        r2: f64
    ) -> bool {
        let res = intersect_two_circles(
            (x1, y1),
            r1,
            (x2, y2),
            r2,
        );

        for (x, y) in res.into_iter() {
            // First circle
            let delta1 = (x - x1) * (x - x1) + (y - y1) * (y - y1) - r1 * r1;
            if delta1.abs() > 1e-9 {
                return false;
            }

            let delta2 = (x - x2) * (x - x2) + (y - y2) * (y - y2) - r2 * r2;
            if delta1.abs() > 1e-9 {
                return false;
            }
        }
        true
    }

    fn check_intersect_circle_line(
        cx: f64,
        cy: f64,
        r: f64,
        lx1: f64,
        ly1: f64,
        lx2: f64,
        ly2: f64
    ) -> bool {
        let res = intersect_circle_line(
            (cx, cy),
            r,
            (lx1, ly1),
            (lx2, ly2),
        );

        for (x, y) in res.into_iter() {
            // Circle
            let delta = (x - cx) * (x - cx) + (y - cy) * (y - cy) - r * r;
            if delta.abs() > 1e-9 {
                return false;
            }

            // Line
            let delta = (y - ly1) - (x - lx1) * (ly1 - ly2) / (lx1 - lx2);
            if delta.abs() > 1e-9 {
                return false;
            }
        }
        true
    }

    fn check_intersect_two_lines(
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x3: f64,
        y3: f64,
        x4: f64,
        y4: f64
    ) -> bool {
        let res = intersect_line_line(
            (x1, y1),
            (x2, y2),
            (x3, y3),
            (x4, y4),
        );

        for (x, y) in res.into_iter() {
            // Line 1
            let delta = (y - y1) - (x - x1) * (y1 - y2) / (x1 - x2);
            if delta.abs() > 1e-9 {
                return false;
            }

            // Line 2
            let delta = (y - y3) - (x - x3) * (y3 - y4) / (x3 - x4);
            if delta.abs() > 1e-9 {
                return false;
            }
        }
        true
    }
}
