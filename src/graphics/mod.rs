use std::f64::consts::PI;
use std::time::{Duration, Instant};

use super::backend::geometry;
use super::icons;
use super::transform::Transform;

use ytesrev::sdl2::event::Event;
use ytesrev::sdl2::keyboard::Keycode;
use ytesrev::sdl2::mouse::MouseButton;
use ytesrev::sdl2::pixels::Color;
use ytesrev::sdl2::rect::{Point, Rect};
use ytesrev::sdl2::render::Canvas;
use ytesrev::sdl2::video::Window;
use ytesrev::drawable::{Drawable, Position, DrawSettings, State};
use ytesrev::image::PngImage;

pub const SIZE: (usize, usize) = (1200, 800);
const FPS_PRINT_RATE: Duration = Duration::from_millis(1000);
const CIRCLE_STEP: usize = 100;
const TOOL_EDGE: u32 = 2;

lazy_static! {
    static ref TOOLS: Vec<(ToolKind, PngImage)> = vec![
        (ToolKind::Point, icons::TOOL_POINT.clone()),
        (ToolKind::Line, icons::TOOL_LINE.clone()),
        (ToolKind::Circle, icons::TOOL_CIRCLE.clone()),
        (ToolKind::Mover, icons::TOOL_MOVER.clone()),
    ];
    static ref TOOL_RECTS: Vec<Rect> = {
        let mut x = TOOL_EDGE as i32;

        let mut res = Vec::with_capacity(TOOLS.len());
        for (_, image) in TOOLS.iter() {
            res.push(Rect::new(
                x,
                TOOL_EDGE as i32,
                image.width as u32,
                image.height as u32,
            ));
            x += image.width as i32 + 10;
        }

        res
    };
    static ref TOOL_HEIGHT: usize = TOOLS
        .iter()
        .map(|(_, image)| image.width)
        .max()
        .unwrap_or(0);
}

pub struct DWindow {
    pub world: geometry::Geometry,

    pub transform: Transform,

    pub mouse_last: Point,
    pub moving_screen: bool,
    pub scrolling: f64,
    pub tool: Tool,

    time_manager: Option<TimeManager>,
}

pub struct Tool {
    kind: ToolKind,
    selected: Vec<geometry::PointID>,
}

#[derive(Clone, PartialEq)]
pub enum ToolKind {
    Point,
    Line,
    Circle,
    Mover,
}

impl ToolKind {
    fn needed_selected(&self) -> usize {
        match self {
            ToolKind::Point => 1,
            ToolKind::Line => 2,
            ToolKind::Circle => 2,
            ToolKind::Mover => 1,
        }
    }
}

impl DWindow {
    pub fn new(world: geometry::Geometry) -> DWindow {
        DWindow {
            world,
            transform: Transform::new_from_winsize((SIZE.0 as f64, SIZE.1 as f64)),
            mouse_last: Point::new(0, 0),
            moving_screen: false,
            scrolling: 0.,
            tool: Tool {
                kind: ToolKind::Point,
                selected: Vec::new(),
            },
            time_manager: None,
        }
    }

    fn draw_menu(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let width = canvas.window().size().0;

        canvas.set_draw_color(Color::RGBA(38, 62, 99, 255));
        canvas.fill_rect(Rect::new(0, 0, width, *TOOL_HEIGHT as u32 + TOOL_EDGE * 2))?;
        canvas.set_draw_color(Color::RGBA(162, 184, 219, 255));
        canvas.fill_rect(Rect::new(
            TOOL_EDGE as i32,
            TOOL_EDGE as i32,
            width - TOOL_EDGE * 2,
            *TOOL_HEIGHT as u32,
        ))?;

        for (rect, (tool, image)) in TOOL_RECTS.iter().zip(TOOLS.iter()) {
            if &self.tool.kind == tool {
                canvas.set_draw_color(Color::RGBA(140, 120, 100, 255));
                canvas.fill_rect(*rect)?;
                canvas.set_draw_color(Color::RGBA(245, 230, 230, 255));
                canvas.fill_rect(Rect::new(
                    rect.x() + 2,
                    rect.y() + 2,
                    rect.width() as u32 - 4,
                    rect.height() as u32 - 4,
                ))?;
            }
            //image.draw(canvas, Point::new(rect.x(), rect.y()))?;
        }
        Ok(())
    }

    pub fn process_event(&mut self, event: Event) -> bool {
        match event {
            Event::Quit { .. } => return false,
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                self.tool.selected.clear();
            }
            Event::KeyDown {
                keycode: Some(Keycode::L),
                ..
            } => {
                self.tool.kind = ToolKind::Line;
            }
            Event::KeyDown {
                keycode: Some(Keycode::C),
                ..
            } => {
                self.tool.kind = ToolKind::Circle;
            }
            Event::KeyDown {
                keycode: Some(Keycode::P),
                ..
            } => {
                self.tool.kind = ToolKind::Point;
            }
            Event::KeyDown {
                keycode: Some(Keycode::M),
                ..
            } => {
                self.tool.kind = ToolKind::Mover;
            }
            Event::MouseButtonDown {
                x,
                y,
                mouse_btn: MouseButton::Left,
                ..
            } => {
                if y < *TOOL_HEIGHT as i32 {
                    for (rect, (tool, _)) in TOOL_RECTS.iter().zip(TOOLS.iter()) {
                        if rect.contains_point(Point::new(x, y)) {
                            self.tool.kind = tool.clone();
                            self.tool.selected.clear();
                        }
                    }
                } else {
                    let mouse_po = self.transform.transform_px_to_po((x as f64, y as f64));
                    match self.tool.kind {
                        ToolKind::Point => {
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
                                if self.tool.kind == ToolKind::Mover {
                                    if let Some(geometry::Point::Arbitrary(_)) =
                                        self.world.points.get(&id)
                                    {
                                    } else {
                                        return true;
                                    }
                                }
                                self.tool.selected.push(id);
                                if self.tool.selected.len() >= self.tool.kind.needed_selected() {
                                    match self.tool.kind {
                                        ToolKind::Line => {
                                            self.world.add_shape(geometry::Shape::Line(
                                                self.tool.selected[0],
                                                self.tool.selected[1],
                                            ));
                                            self.tool.selected.clear();
                                        }
                                        ToolKind::Circle => {
                                            self.world.add_shape(geometry::Shape::Circle(
                                                self.tool.selected[0],
                                                self.tool.selected[1],
                                            ));
                                            self.tool.selected.clear();
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Event::MouseButtonUp { .. } => match self.tool.kind {
                ToolKind::Mover => {
                    self.tool.selected.clear();
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
                if let ToolKind::Mover = self.tool.kind {
                    if let Some(id) = self.tool.selected.get(0) {
                        let mouse_po = self.transform.transform_px_to_po((x as f64, y as f64));

                        if let Some(point) = self.world.points.get_mut(&id) {
                            *point = geometry::Point::Arbitrary(mouse_po);
                        }
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

    fn try_draw(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
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

        for (id, point) in self.world.points.iter() {
            if let Some(rpoint) = self.world.resolve_point(point) {
                let p_px = self.transform.transform_po_to_px(rpoint);

                canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
                if let ToolKind::Mover = self.tool.kind {
                    if let geometry::Point::Arbitrary(_) = point {
                        canvas.set_draw_color(Color::RGBA(0, 0, 255, 255));
                    }
                }
                if self.tool.selected.contains(id) {
                    canvas.set_draw_color(Color::RGBA(0, 255, 0, 255));
                }
                fill_circle(canvas, p_px, 5.)?;
            }
        }

        self.draw_menu(canvas)?;

        Ok(())
    }
}


impl Drawable for DWindow {
    fn content(&self) -> Vec<&Drawable> {
        Vec::new()
    }

    fn content_mut(&mut self) -> Vec<&mut Drawable> {
        Vec::new()
    }

    fn step(&mut self) {
    }

    fn state(&self) -> State {
        State::Working
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>, position: &Position, _: DrawSettings) {
        self.try_draw(canvas).expect("Can't draw");
    }

    fn update(&mut self, dt: f64) {
        self.scrolling = self.scrolling * (0.01_f64).powf(dt);
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
