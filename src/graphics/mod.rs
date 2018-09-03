extern crate sdl2;

use std::f64::consts::PI;
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

pub const SIZE: (usize, usize) = (1200, 800);
const FPS_PRINT_RATE: Duration = Duration::from_millis(1000);
const CIRCLE_STEP: usize = 100;

pub struct DWindow {
    pub world: geometry::Geometry,

    pub transform: Transform,

    pub mouse_last: Point,
    pub moving_screen: bool,
    pub scrolling: f64,
    pub tool: Tool,

    time_manager: Option<TimeManager>,
}

#[derive(Clone)]
pub enum Tool {
    Point,
    Line(LineState),
    Circle(CircleState),
    Mover(MoverState),
}

#[derive(Clone)]
pub enum LineState {
    Nothing,
    OnePoint(geometry::PointID),
}

#[derive(Clone)]
pub enum CircleState {
    Nothing,
    Centered(geometry::PointID),
}

#[derive(Clone)]
pub enum MoverState {
    Nothing,
    Moving(geometry::PointID),
}

impl DWindow {
    pub fn new(world: geometry::Geometry) -> DWindow {
        DWindow {
            world,
            transform: Transform::new_from_winsize((SIZE.0 as f64, SIZE.1 as f64)),
            mouse_last: Point::new(0, 0),
            moving_screen: false,
            scrolling: 0.,
            tool: Tool::Point,
            time_manager: None,
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
        canvas.clear();

        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let (w, h) = canvas.window().size();

        for obj in self.world.shapes.values() {
            if let Some(ro) = self.world.resolve_shape(obj) {
                match ro {
                    geometry::ResolvedShape::Circle(center, rad) => {
                        let center_px = self.transform.transform_po_to_px(center);
                        draw_circle(canvas, center_px, rad * self.transform.scale)?;
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

                        canvas.draw_line(
                            Point::new(start_px.0 as i32, start_px.1 as i32),
                            Point::new(end_px.0 as i32, end_px.1 as i32),
                        )?;
                    }
                    geometry::ResolvedShape::LineUp(x) => {
                        let x_px = self.transform.transform_po_to_px((x, 0.)).0;
                        canvas.draw_line(
                            Point::new(x_px as i32, 0),
                            Point::new(x_px as i32, h as i32),
                        )?;
                    }
                }
            }
        }

        for point in self.world.points.values() {
            if let Some(rpoint) = self.world.resolve_point(point) {
                let p_px = self.transform.transform_po_to_px(rpoint);

                canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
                if let Tool::Mover(_) = self.tool {
                    if let geometry::Point::Arbitrary(_) = point {
                        canvas.set_draw_color(Color::RGBA(0, 0, 255, 255));
                    }
                }
                fill_circle(canvas, p_px, 5.)?;
            }
        }

        canvas.present();

        Ok(())
    }

    pub fn process_event(&mut self, event: Event) -> bool {
        match event {
            Event::Quit { .. } => return false,
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                self.tool = match self.tool.clone() {
                    Tool::Line(LineState::OnePoint(_)) => Tool::Line(LineState::Nothing),
                    Tool::Circle(CircleState::Centered(_)) => Tool::Circle(CircleState::Nothing),
                    x => x,
                };
            }
            Event::KeyDown {
                keycode: Some(Keycode::L),
                ..
            } => {
                self.tool = Tool::Line(LineState::Nothing);
            }
            Event::KeyDown {
                keycode: Some(Keycode::C),
                ..
            } => {
                self.tool = Tool::Circle(CircleState::Nothing);
            }
            Event::KeyDown {
                keycode: Some(Keycode::P),
                ..
            } => {
                self.tool = Tool::Point;
            }
            Event::KeyDown {
                keycode: Some(Keycode::M),
                ..
            } => {
                self.tool = Tool::Mover(MoverState::Nothing);
            }
            Event::MouseButtonDown {
                x,
                y,
                mouse_btn: MouseButton::Left,
                ..
            } => {
                let mouse_po = self.transform.transform_px_to_po((x as f64, y as f64));

                match self.tool {
                    Tool::Point => {
                        let point = get_closest(
                            mouse_po,
                            self.world.get_potential_points(),
                            |point| self.world.resolve_point(point).unwrap_or((0., 0.)),
                            Some(100. / self.transform.scale),
                        ).unwrap_or(geometry::Point::Arbitrary(mouse_po));

                        self.world.add_point(point);
                    }
                    _ => {
                        if let Some((&id, _)) = get_closest(
                            mouse_po,
                            self.world.points.iter().collect(),
                            |(_, point)| self.world.resolve_point(point).unwrap_or((0., 0.)),
                            Some(100. / self.transform.scale),
                        ) {
                            match self.tool {
                                Tool::Point => unreachable!(),
                                Tool::Mover(_) => {
                                    if let Some(geometry::Point::Arbitrary(_)) =
                                        self.world.points.get(&id)
                                    {
                                        self.tool = Tool::Mover(MoverState::Moving(id));
                                    }
                                }
                                Tool::Line(LineState::Nothing) => {
                                    self.tool = Tool::Line(LineState::OnePoint(id));
                                }
                                Tool::Line(LineState::OnePoint(id1)) => {
                                    self.world.add_shape(geometry::Shape::Line(id, id1));
                                    self.tool = Tool::Line(LineState::Nothing);
                                }
                                Tool::Circle(CircleState::Nothing) => {
                                    self.tool = Tool::Circle(CircleState::Centered(id));
                                }
                                Tool::Circle(CircleState::Centered(cent)) => {
                                    self.world.add_shape(geometry::Shape::Circle(cent, id));
                                    self.tool = Tool::Circle(CircleState::Nothing);
                                }
                            }
                        }
                    }
                }
            }
            Event::MouseButtonUp { .. } => match self.tool {
                Tool::Mover(MoverState::Moving(_)) => {
                    self.tool = Tool::Mover(MoverState::Nothing);
                }
                _ => {}
            },
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
                if let Tool::Mover(MoverState::Moving(id)) = self.tool {
                    let mouse_po = self.transform.transform_px_to_po((x as f64, y as f64));

                    if let Some(point) = self.world.points.get_mut(&id) {
                        *point = geometry::Point::Arbitrary(mouse_po);
                    }
                }
                self.mouse_last = Point::new(x, y);
            }
            Event::MouseWheel { y, .. } => {
                self.scrolling += -y as f64 / 300.;
            }
            _ => {}
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
}

fn draw_circle(canvas: &mut Canvas<Window>, pos: (f64, f64), r: f64) -> Result<(), String> {
    canvas.draw_lines(&*draw_circle_points(pos, r))
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

fn fill_circle(canvas: &mut Canvas<Window>, pos: (f64, f64), r: f64) -> Result<(), String> {
    canvas.draw_lines(&*fill_circle_points(pos, r))
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

fn get_closest<T, F: Fn(&T) -> (f64, f64)>(
    to: (f64, f64),
    objs: Vec<T>,
    f: F,
    max: Option<f64>,
) -> Option<T> {
    let mut best: Option<(f64, T)> = None;
    for obj in objs {
        let pos = f(&obj);

        let (dx, dy) = (pos.0 - to.0, pos.1 - to.1);
        let dist_sq = dx * dx + dy * dy;

        if max.map(|x| dist_sq > x * x).unwrap_or(false) {
            continue;
        }
        if let Some((cur_dist, _)) = best {
            if dist_sq < cur_dist {
                best = Some((dist_sq, obj));
            }
        } else {
            best = Some((dist_sq, obj));
        }
    }

    best.map(|x| x.1)
}
