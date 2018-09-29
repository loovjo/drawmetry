extern crate ordered_float;
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

use backend::{geometry, gwrapper};
use ytesrev::prelude::*;
use ytesrev::window::{WindowSettings, WSETTINGS_MAIN};
use ytesrev::sdl2::event::Event;

fn main() {
    let world = gwrapper::GWrapper::new(geometry::Geometry::new());
    let all = graphics::create_layout(world);

    let mut manager = WindowManager::init_window(
        all,
        WindowManagerSettings {
            windows: vec![(
                "Drawmetry".into(),
                WindowSettings {
                    window_size: graphics::WINDOW_SIZE,
                    ..WSETTINGS_MAIN
                },
            )],
            event_step_rule: Box::new(|_| false),
            quit_rule: Box::new(|event| if let Event::Quit{..} = event { true } else { false } ),
        },
    );

    manager.start();
}
