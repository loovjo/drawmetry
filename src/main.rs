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
    let world = geometry::Geometry::new();

    let window = graphics::DWindow::new(world);
    let mut container = window::WindowContainer::new(window);

    container.start();
}
