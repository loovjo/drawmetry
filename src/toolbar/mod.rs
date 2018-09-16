use std::sync::{Mutex, Arc};

use graphics::*;
use icons;
use backend::geometry;

use ytesrev::drawable::{DrawSettings, Drawable, Position, State};
use ytesrev::image::PngImage;
use ytesrev::sdl2::event::Event;
use ytesrev::sdl2::keyboard::Keycode;
use ytesrev::sdl2::mouse::MouseButton;
use ytesrev::sdl2::pixels::Color;
use ytesrev::sdl2::rect::{Point, Rect};
use ytesrev::sdl2::render::Canvas;
use ytesrev::sdl2::video::Window;

pub const TOOL_EDGE: u32 = 2;

lazy_static! {
    pub static ref TOOLS: Mutex<Vec<(ToolKind, PngImage)>> = Mutex::new(vec![
        (ToolKind::Point, icons::TOOL_POINT.clone()),
        (ToolKind::Line, icons::TOOL_LINE.clone()),
        (ToolKind::Circle, icons::TOOL_CIRCLE.clone()),
        (ToolKind::Mover, icons::TOOL_MOVER.clone()),
    ]);
    pub static ref TOOL_RECTS: Vec<Rect> = {
        let mut x = TOOL_EDGE as i32;

        let mut res = Vec::new();
        for (_, image) in TOOLS.lock().unwrap().iter() {
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
    pub static ref TOOL_HEIGHT: usize = TOOLS
        .lock()
        .unwrap()
        .iter()
        .map(|(_, image)| image.width)
        .max()
        .unwrap_or(0);

}

#[derive(Debug)]
pub struct Tool {
    pub kind: ToolKind,
    pub selected: Vec<geometry::PointID>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToolKind {
    Point,
    Line,
    Circle,
    Mover,
}

impl ToolKind {
    pub fn needed_selected(&self) -> usize {
        match self {
            ToolKind::Point => 1,
            ToolKind::Line => 2,
            ToolKind::Circle => 2,
            ToolKind::Mover => 1,
        }
    }
}

pub struct ToolBar {
    pub state: Arc<Mutex<DState>>,
}

impl ToolBar {
    pub fn new(state: Arc<Mutex<DState>>) -> ToolBar {
        ToolBar { state }
    }

    pub fn mouse_down(&mut self, position: Point, button: MouseButton) {
        for (rect, (tool, _)) in TOOL_RECTS.iter().zip(TOOLS.lock().unwrap().iter()) {
            if rect.contains_point(position) {
                if let Ok(mut state) = self.state.lock() {
                    state.current_tool.kind = *tool;
                    state.current_tool.selected.clear();
                }
            }
        }
    }

    fn draw_menu(&mut self, canvas: &mut Canvas<Window>, settings: DrawSettings) -> Result<(), String> {
        let current_tool = &self.state.lock().unwrap().current_tool;

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

        for (rect, (tool, image)) in TOOL_RECTS.iter().zip(TOOLS.lock().unwrap().iter_mut()) {
            if *tool == current_tool.kind {
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

            image.draw(canvas, &Position::TopLeftCorner(Point::new(rect.x(), rect.y())), settings);
        }

        Ok(())
    }
}

impl Drawable for ToolBar {
    fn content(&self) -> Vec<&Drawable> {
        Vec::new()
    }

    fn content_mut(&mut self) -> Vec<&mut Drawable> {
        Vec::new()
    }

    fn step(&mut self) {}

    fn state(&self) -> State {
        State::Working
    }

    fn draw(&mut self, canvas: &mut Canvas<Window>, position: &Position, settings: DrawSettings) {
        self.draw_menu(canvas, settings).expect("Can't draw toolbar");
    }

    fn update(&mut self, dt: f64) {
    }

    fn event(&mut self, event: Event) {
        match event {
            _ => {}
        }
    }
}
