extern crate sdl2;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod backend;
mod graphics;
mod transform;
mod window;
use backend::geometry;

fn main() {
    let mut world = geometry::Geometry::new();

    let po_o = world.add_point(geometry::Point::Arbitrary((0., 0.)));
    let po_1_0 = world.add_point(geometry::Point::Arbitrary((1., 0.)));

    let li_0 = world.add_shape(geometry::Shape::Line(po_o, po_1_0));
    let ci_o = world.add_shape(geometry::Shape::Circle(po_o, po_1_0));

    let po_n1_0 = world.add_point(geometry::Point::SecIntersection(li_0, ci_o));

    let ci_1_0 = world.add_shape(geometry::Shape::Circle(po_1_0, po_n1_0));
    let ci_n1_0 = world.add_shape(geometry::Shape::Circle(po_n1_0, po_1_0));

    let po_y_0 = world.add_point(geometry::Point::PrimIntersection(ci_1_0, ci_n1_0));
    let po_y_1 = world.add_point(geometry::Point::SecIntersection(ci_1_0, ci_n1_0));

    let li_y = world.add_shape(geometry::Shape::Line(po_y_0, po_y_1));

    let po_0_1 = world.add_point(geometry::Point::PrimIntersection(li_y, ci_o));
    let po_0_n1 = world.add_point(geometry::Point::SecIntersection(li_y, ci_o));

    world.add_shape(geometry::Shape::Line(po_0_n1, po_1_0));
    world.add_shape(geometry::Shape::Line(po_1_0, po_0_1));
    world.add_shape(geometry::Shape::Line(po_0_1, po_n1_0));
    world.add_shape(geometry::Shape::Line(po_n1_0, po_0_n1));

    let window = graphics::DWindow::new(world);
    let mut container = window::WindowContainer::new(window);

    container.start();
}
