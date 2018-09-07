extern crate ytesrev;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod backend;
mod graphics;
mod icons;
mod transform;

use backend::geometry;
use ytesrev::prelude::*;
use ytesrev::window::WSETTINGS_MAIN;

fn main() {
    let world = geometry::Geometry::new();
    let window = graphics::DWindow::new(world);

    let mut manager = WindowManager::init_window(
        DrawableWrapper(window),
        vec![("Drawmetry".into(), WSETTINGS_MAIN)],
    );

    manager.start();
}
