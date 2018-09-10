use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

use backend::geometry;
use drawing_board::DrawingBoard;
use toolbar::{Tool, ToolBar, ToolKind, TOOL_HEIGHT};
use ytesrev::drawable::{DrawSettings, Position};
use ytesrev::prelude::*;
use ytesrev::scene::Action;
use ytesrev::sdl2::event::Event;
use ytesrev::sdl2::rect::Point;
use ytesrev::sdl2::render::Canvas;
use ytesrev::sdl2::video::Window;
use ytesrev::window::YEvent;

pub const WINDOW_SIZE: (u32, u32) = (1200, 800);

pub struct DState {
    pub world: geometry::Geometry,
    pub current_tool: Tool,
}

pub struct DScene {
    inner: Split<ToolBar, DrawingBoard>,
}

pub fn create_layout(world: geometry::Geometry) -> DScene {
    let state = DState {
        world: world,
        current_tool: Tool {
            kind: ToolKind::Point,
            selected: Vec::new(),
        },
    };

    let state_arc_mutex = Arc::new(Mutex::new(state));
    let tool_bar = ToolBar::new(state_arc_mutex.clone());
    let drawing_board = DrawingBoard::new(state_arc_mutex.clone());

    DScene {
        inner: Split::new_const(
            *TOOL_HEIGHT as u32,
            Orientation::Vertical,
            UpdateOrder::FirstSecond,
            tool_bar,
            drawing_board,
        ),
    }
}

impl Scene for DScene {
    fn update(&mut self, dt: f64) {
        self.inner.first.update(dt);
        self.inner.second.update(dt);
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>, settings: DrawSettings) {
        let (w, h) = canvas.window().size();
        self.inner.draw(canvas, &Position::Rect(Rect::new(0, 0, w, h)), settings);
    }

    fn event(&mut self, event: YEvent) {
        match event {
            YEvent::Other(Event::MouseButtonDown { x, y, mouse_btn, .. }) => {
                if y < *TOOL_HEIGHT as i32 {
                    self.inner.first.mouse_down(Point::new(x, y), mouse_btn);
                } else {
                    self.inner.second.mouse_down(Point::new(x, y), mouse_btn);
                }
            }
            YEvent::Other(e) => {
                self.inner.first.event(e.clone());
                self.inner.second.event(e.clone());
            }
            _ => {}
        }
    }

    fn action(&self) -> Action {
        Action::Continue
    }

    fn register(&mut self) {
        self.inner.first.register();
        self.inner.second.register();
    }

    fn load(&mut self) {
        self.inner.first.load();
        self.inner.second.load();
    }
}

pub const CIRCLE_STEP: usize = 100;

pub fn draw_circle(canvas: &mut Canvas<Window>, pos: (f64, f64), r: f64) -> Result<(), String> {
    canvas.draw_lines(&*draw_circle_points(pos, r))
}

pub fn draw_circle_points((x, y): (f64, f64), r: f64) -> Vec<Point> {
    let mut points = Vec::with_capacity(CIRCLE_STEP);

    for i in 0..CIRCLE_STEP + 1 {
        let theta = (i as f64 / CIRCLE_STEP as f64) * 2. * PI;
        let (x_, y_) = (x + r * theta.cos(), y + r * theta.sin());
        points.push(Point::new(x_ as i32, y_ as i32));
    }
    points
}

pub fn fill_circle_points((x, y): (f64, f64), r: f64) -> Vec<Point> {
    let mut points = Vec::with_capacity(r as usize * 4);
    for x_ in -r as i32..r as i32 {
        let y_ = (r * r - (x_ * x_) as f64).sqrt();
        points.push(Point::new(x_ + x as i32, (y_ + y) as i32));
        points.push(Point::new(x_ + x as i32, (-y_ + y) as i32));
    }
    points
}

pub fn get_closest<T, F: Fn(&T) -> (f64, f64)>(
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
