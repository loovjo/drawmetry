extern crate sdl2;
extern crate png;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod backend;
mod graphics;
mod transform;
mod window;
mod image;
use backend::geometry;

fn main() {
    let world = geometry::Geometry::new();

    let window = graphics::DWindow::new(world);
    let mut container = window::WindowContainer::new(window);

    container.start();
}
