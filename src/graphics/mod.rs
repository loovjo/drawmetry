extern crate sdl2;

use std::f64::consts::PI;
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::backend::geometry;
use super::transform::Transform;

use self::sdl2::event::Event;
use self::sdl2::keyboard::Keycode;
use self::sdl2::mouse::MouseButton;
use self::sdl2::pixels::Color;
use self::sdl2::rect::Point;
use self::sdl2::render::Canvas;
use self::sdl2::video::Window;
use self::sdl2::EventPump;

const FPS_PRINT_RATE: Duration = Duration::from_millis(1000);
const SIZE: (usize, usize) = (1200, 800);
const CIRCLE_STEP: usize = 100;

pub struct DWindow {
    pub world: geometry::Geometry,

    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,

    pub transform: Transform,

    pub mouse_last: Point,
    pub moving_screen: bool,
    pub scrolling: f64,

    time_manager: Option<TimeManager>,
    tick: usize,
}

impl DWindow {
    pub fn new(world: geometry::Geometry) -> DWindow {
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

        DWindow {
            world,
            canvas,
            event_pump,
            time_manager: None,
            transform: Transform::new_from_winsize((SIZE.0 as f64, SIZE.1 as f64)),
            mouse_last: Point::new(0, 0),
            moving_screen: false,
            scrolling: 0.,
            tick: 0,
        }
    }

    fn draw(&mut self) {
        self.canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
        self.canvas.clear();

        self.canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let (w, h) = self.canvas.window().size();

        for obj in self.world.shapes.values() {
            if let Some(ro) = self.world.resolve_shape(obj) {
                match ro {
                    geometry::ResolvedShape::Circle(center, rad) => {
                        let center_px = self.transform.transform_po_to_px(center);
                        draw_circle(&mut self.canvas, center_px, rad * self.transform.scale);
                    }
                    geometry::ResolvedShape::Line(k, m) => {
                        let start_x = self.transform.transform_px_to_po((0., 0.)).0;
                        let start_y = k * start_x + m;
                        let start_point = (start_x, start_y);

                        let end_x = self.transform.transform_px_to_po((w as f64, 0.)).0;
                        let end_y = k * end_x + m;
                        let end_point = (end_x, end_y);

                        let start_px = self.transform.transform_po_to_px(start_point);
                        let end_px = self.transform.transform_po_to_px(end_point);

                        self.canvas.draw_line(
                            Point::new(start_px.0 as i32, start_px.1 as i32),
                            Point::new(end_px.0 as i32, end_px.1 as i32),
                        );
                    }
                    geometry::ResolvedShape::LineUp(x) => {
                        let x_px = self.transform.transform_po_to_px((x, 0.)).0;
                        self.canvas.draw_line(
                            Point::new(x_px as i32, 0),
                            Point::new(x_px as i32, h as i32),
                        );
                    }
                }
            }
        }

        for point in self.world.points.values() {
            if let Some(rpoint) = self.world.resolve_point(point) {
                let p_px = self.transform.transform_po_to_px(rpoint);

                fill_circle(&mut self.canvas, p_px, 5.);
            }
        }

        self.canvas.present();
    }

    fn process_events(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return false,
                //Event::KeyDown { keycode: Some(Keycode::L), .. } => {
                //self.placing = Some(Placing::Line(self.get_closest()));
                //}
                Event::MouseButtonDown {
                    x,
                    y,
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    let mouse_po = self.transform.transform_px_to_po((x as f64, y as f64));

                    if let Some((best, dist)) = self.world.get_closest_point(mouse_po) {
                        if self.transform.scale * dist.sqrt() > 100. {
                            self.world.add_point(geometry::Point::Arbitrary(mouse_po));
                        } else {
                            self.world.add_point(best);
                        }
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    self.moving_screen = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    self.moving_screen = false;
                }
                Event::MouseMotion { x, y, .. } => {
                    if self.moving_screen {
                        let (dx, dy) = (x - self.mouse_last.x, y - self.mouse_last.y);
                        let (dtx, dty) = (
                            dx as f64 / self.transform.scale,
                            dy as f64 / self.transform.scale,
                        );

                        self.transform.translation = (
                            self.transform.translation.0 + dtx as f64,
                            self.transform.translation.1 + dty as f64,
                        );
                    }
                    self.mouse_last = Point::new(x, y);
                }
                Event::MouseWheel { y, .. } => {
                    self.scrolling += -y as f64 / 300.;
                }
                _ => {}
            }
        }

        true
    }

    pub fn update(&mut self) {
        let dt = if let Some(ref mut tm) = self.time_manager {
            let dt = tm.dt();
            dt
        } else {
            self.time_manager = Some(TimeManager::new());
            return;
        };

        self.scrolling = self.scrolling * (0.1_f64).powf(dt);
        self.transform.scale *= (0.1_f64).powf(self.scrolling);
    }

    pub fn start(&mut self) {
        loop {
            self.tick += 1;
            self.draw();
            self.update();
            if !self.process_events() {
                break;
            }

            sleep(Duration::from_millis(5));
        }
    }
}

fn draw_circle(canvas: &mut Canvas<Window>, pos: (f64, f64), r: f64) {
    canvas.draw_lines(&*draw_circle_points(pos, r));
}

fn draw_circle_points((x, y): (f64, f64), r: f64) -> Vec<Point> {
    let mut points = Vec::with_capacity(CIRCLE_STEP);

    for i in 0..CIRCLE_STEP + 1 {
        let theta = (i as f64 / CIRCLE_STEP as f64) * 2. * PI;
        let (x_, y_) = (x + r * theta.cos(), y + r * theta.sin());
        points.push(Point::new(x_ as i32, y_ as i32));
    }
    points
}

fn fill_circle(canvas: &mut Canvas<Window>, pos: (f64, f64), r: f64) {
    canvas.draw_lines(&*fill_circle_points(pos, r));
}

fn fill_circle_points((x, y): (f64, f64), r: f64) -> Vec<Point> {
    let mut points = Vec::with_capacity(r as usize * 4);
    for x_ in -r as i32..r as i32 {
        let y_ = (r * r - (x_ * x_) as f64).sqrt();
        points.push(Point::new(x_ + x as i32, (y_ + y) as i32));
        points.push(Point::new(x_ + x as i32, (-y_ + y) as i32));
    }
    points
}

struct TimeManager {
    last_time: Instant,

    last_fps_print: Instant,
    durs: Vec<Duration>,
}

impl TimeManager {
    fn new() -> TimeManager {
        TimeManager {
            last_time: Instant::now(),
            last_fps_print: Instant::now(),
            durs: Vec::new(),
        }
    }

    fn dt(&mut self) -> f64 {
        let now = Instant::now();

        let diff = now - self.last_time;
        self.last_time = now;

        self.durs.push(diff);
        if now - self.last_fps_print > FPS_PRINT_RATE {
            let num_dur = self.durs.len() as u32;

            let avg_dur: Duration = self.durs.drain(..).sum::<Duration>() / num_dur;

            let fps = 1. / (avg_dur.as_secs() as f64 + avg_dur.subsec_millis() as f64 / 1000.);

            eprintln!("FPS: {:.2}", fps);

            self.last_fps_print = now;
        }

        diff.as_secs() as f64 + diff.subsec_millis() as f64 / 1000.
    }
}
