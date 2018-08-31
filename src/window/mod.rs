extern crate sdl2;

use std::thread::sleep;
use std::time::Duration;

use self::sdl2::render::Canvas;
use self::sdl2::video::Window;
use self::sdl2::EventPump;
use graphics::{DWindow, SIZE};

pub struct WindowContainer {
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,

    pub inner: DWindow,
}

impl WindowContainer {
    pub fn new(inner: DWindow) -> WindowContainer {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let main_window = video_subsystem
            .window("drawometry", SIZE.0 as u32, SIZE.1 as u32)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let canvas = main_window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();

        WindowContainer {
            canvas,
            event_pump,
            inner,
        }
    }

    pub fn start(&mut self) {
        'outer: loop {
            self.inner.draw(&mut self.canvas);
            self.inner.update();
            for event in self.event_pump.poll_iter() {
                if !self.inner.process_event(event) {
                    break 'outer;
                }
            }

            sleep(Duration::from_millis(5));
        }
    }
}
