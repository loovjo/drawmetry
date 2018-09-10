extern crate ytesrev;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod backend;
mod drawing_board;
mod graphics;
mod icons;
mod toolbar;
mod transform;

use backend::geometry;
use ytesrev::prelude::*;
use ytesrev::window::{WindowSettings, WSETTINGS_MAIN};

fn main() {
    let world = geometry::Geometry::new();
    let all = graphics::create_layout(world);

    let mut manager = WindowManager::init_window(
        all,
        vec![(
            "Drawmetry".into(),
            WindowSettings {
                window_size: graphics::WINDOW_SIZE,
                ..WSETTINGS_MAIN
            },
        )],
    );

    manager.start();
}
