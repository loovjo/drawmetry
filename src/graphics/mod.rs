use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

use backend::geometry;
use drawing_board::DrawingBoard;
use toolbar::{Tool, ToolBar, ToolKind, TOOL_HEIGHT};
use ytesrev::image::PngImage;
use ytesrev::prelude::*;
use ytesrev::sdl2::event::Event;
use ytesrev::sdl2::keyboard::Keycode;
use ytesrev::sdl2::mouse::MouseButton;
use ytesrev::sdl2::pixels::Color;
use ytesrev::sdl2::rect::{Point, Rect};
use ytesrev::sdl2::render::Canvas;
use ytesrev::sdl2::video::Window;

pub const WINDOW_SIZE: (u32, u32) = (1200, 800);

pub struct DState {
    pub world: geometry::Geometry,
    pub current_tool: Tool,
}

pub fn create_layout(world: geometry::Geometry) -> impl Drawable {
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


    Split::new_const(
        *TOOL_HEIGHT as u32,
        Orientation::Vertical,
        UpdateOrder::FirstSecond,
        tool_bar,
        drawing_board,
    )
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

pub fn fill_circle(canvas: &mut Canvas<Window>, pos: (f64, f64), r: f64) -> Result<(), String> {
    canvas.draw_lines(&*fill_circle_points(pos, r))
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
