extern crate sdl2;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod graphics;
mod transform;
mod backend;
use backend::geometry;


fn main() {
    let mut world = geometry::Geometry::new();
    let po_o = world.add_object(geometry::Object::Point(geometry::Position::Arbitrary(
        0., 0.,
    )));
    let po_1 = world.add_object(geometry::Object::Point(geometry::Position::Arbitrary(
        1., 0.,
    )));

    let ci_o = world.add_object(geometry::Object::Circle(
        geometry::Position::AtPoint(po_o),
        geometry::Position::AtPoint(po_1),
    ));

    let li_o_1 = world.add_object(geometry::Object::Line(
        geometry::Position::AtPoint(po_o),
        geometry::Position::AtPoint(po_1),
    ));

    let po_n1 = world.add_object(geometry::Object::Point(
        geometry::Position::SecIntersection(ci_o, li_o_1),
    ));

    let ci_1 = world.add_object(geometry::Object::Circle(
        geometry::Position::AtPoint(po_1),
        geometry::Position::AtPoint(po_n1),
    ));

    let ci_n1 = world.add_object(geometry::Object::Circle(
        geometry::Position::AtPoint(po_n1),
        geometry::Position::AtPoint(po_1),
    ));

    let li_v = world.add_object(geometry::Object::Line(
        geometry::Position::PrimIntersection(ci_1, ci_n1),
        geometry::Position::SecIntersection(ci_1, ci_n1),
    ));

    world.add_object(geometry::Object::Line(
        geometry::Position::PrimIntersection(ci_o, li_v),
        geometry::Position::AtPoint(po_1),
    ));

    world.add_object(geometry::Object::Line(
        geometry::Position::AtPoint(po_1),
        geometry::Position::SecIntersection(ci_o, li_v),
    ));

    world.add_object(geometry::Object::Line(
        geometry::Position::SecIntersection(ci_o, li_v),
        geometry::Position::AtPoint(po_n1),
    ));

    world.add_object(geometry::Object::Line(
        geometry::Position::AtPoint(po_n1),
        geometry::Position::PrimIntersection(ci_o, li_v),
    ));


    let mut window = graphics::DWindow::new(world);
    window.start();
}
