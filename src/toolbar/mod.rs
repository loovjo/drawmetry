use std::sync::mpsc::Sender;

use tool::ToolKind;

use icons;
use backend::geometry;

use ytesrev::drawable::{DrawSettings, Drawable, Position, State, KnownSize};
use ytesrev::prelude::*;
use ytesrev::sdl2::event::Event;
use ytesrev::sdl2::mouse::MouseButton;

pub const TOOL_EDGE: u32 = 2;

pub struct ToolBar {
    pub tools: Vec<(ToolKind, PngImage)>,
    pub send_tool: Sender<ToolKind>,
    pub selected: Option<usize>,
}

impl ToolBar {
    pub fn mouse_down(&mut self, position: Point, _button: MouseButton) {
        for (i, (rect, (tool, _))) in self.tool_rects().iter().zip(self.tools.iter()).enumerate() {
            if rect.contains_point(position) {
                self.send_tool.send(tool.clone()).expect("Couldn't send tool!");
                self.selected = Some(i);
            }
        }
    }

    pub fn tool_rects(&self) -> Vec<Rect> {
        let mut x = TOOL_EDGE as i32;

        let mut res = Vec::new();
        for (_, image) in self.tools.iter() {
            res.push(Rect::new(
                x,
                TOOL_EDGE as i32,
                image.width() as u32,
                image.height() as u32,
            ));

            x += image.width() as i32 + 10;
        }

        res
    }

    fn draw_menu(&self, canvas: &mut Canvas<Window>, settings: DrawSettings) -> Result<(), String> {
        let width = canvas.window().size().0;

        canvas.set_draw_color(Color::RGBA(38, 62, 99, 255));
        canvas.fill_rect(Rect::new(0, 0, width, self.height() as u32))?;
        canvas.set_draw_color(Color::RGBA(162, 184, 219, 255));
        canvas.fill_rect(Rect::new(
            TOOL_EDGE as i32,
            TOOL_EDGE as i32,
            width - TOOL_EDGE * 2,
            self.height() as u32 - TOOL_EDGE * 2,
        ))?;

        for (i, (rect, (_, image))) in self.tool_rects().iter().zip(self.tools.iter()).enumerate() {
            if Some(i) == self.selected {
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

    fn draw(&self, canvas: &mut Canvas<Window>, _position: &Position, settings: DrawSettings) {
        self.draw_menu(canvas, settings).expect("Can't draw toolbar");
    }

    fn update(&mut self, _dt: f64) {
    }

    fn event(&mut self, event: Event) {
        match event {
            _ => {}
        }
    }
}

impl KnownSize for ToolBar {
    fn width(&self) -> usize {
        self.tool_rects().last().map(|r| r.right()).unwrap_or(0) as usize
    }

    fn height(&self) -> usize {
        self.tool_rects().iter().map(|x| x.bottom()).max().unwrap_or(0) as usize + TOOL_EDGE as usize
    }
}

