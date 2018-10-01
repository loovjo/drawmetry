mod default;
pub use self::default::default_toolbar;

use std::sync::mpsc::Sender;

use graphics::DState;
use icons;
use tool::ToolKind;

use ytesrev::drawable::{DrawSettings, Drawable, KnownSize, Position, State};
use ytesrev::prelude::*;
use ytesrev::sdl2::event::Event;
use ytesrev::sdl2::mouse::MouseButton;
use ytesrev::sdl2::rect::Rect;

pub const TOOL_EDGE: u32 = 2;

pub struct Button {
    pub function: Box<Fn(&mut DState)>,
    pub select: bool,
    pub subtoolbar: Option<ToolBar>,
}

unsafe impl Send for Button {}
unsafe impl Sync for Button {}

pub struct MakeButton(Box<Fn() -> Button>);

unsafe impl Send for MakeButton {}
unsafe impl Sync for MakeButton {}

pub struct ToolBar {
    pub tools: Vec<(MakeButton, PngImage)>,
    pub send_tool: Sender<Button>,
    pub selected: Option<usize>,
}

impl ToolBar {
    pub fn mouse_down(&mut self, position: Point, _button: MouseButton) {
        for (i, (rect, (tool, _))) in self.tool_rects().iter().zip(self.tools.iter()).enumerate() {
            if rect.contains_point(position) {
                let callback = (*tool.0)();

                if callback.select {
                    self.selected = Some(i);
                }

                self.send_tool.send(callback).expect("Couldn't send tool!");
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

    fn draw_menu(
        &self,
        canvas: &mut Canvas<Window>,
        at: Rect,
        settings: DrawSettings,
    ) -> Result<(), String> {
        canvas.set_draw_color(Color::RGBA(38, 62, 99, 255));
        canvas.fill_rect(at)?;
        canvas.set_draw_color(Color::RGBA(162, 184, 219, 255));
        canvas.fill_rect(Rect::new(
            at.left() + TOOL_EDGE as i32,
            at.top() + TOOL_EDGE as i32,
            at.width() - TOOL_EDGE * 2,
            at.height() - TOOL_EDGE * 2,
        ))?;

        for (i, (rect, (_, image))) in self.tool_rects().iter().zip(self.tools.iter()).enumerate() {
            let rect = Rect::new(
                at.left() + rect.left(),
                at.top() + rect.top(),
                rect.width(),
                rect.height(),
            );

            if Some(i) == self.selected {
                canvas.set_draw_color(Color::RGBA(140, 120, 100, 255));
                canvas.fill_rect(rect)?;
                canvas.set_draw_color(Color::RGBA(245, 230, 230, 255));
                canvas.fill_rect(Rect::new(
                    rect.x() + TOOL_EDGE as i32,
                    rect.y() + TOOL_EDGE as i32,
                    rect.width() as u32 - 4,
                    rect.height() as u32 - 4,
                ))?;
            }

            image.draw(
                canvas,
                &Position::TopLeftCorner(rect.top_left()),
                settings,
            );
        }

        Ok(())
    }

    fn content_height(&self) -> usize {
        self.tool_rects()
            .iter()
            .map(|x| x.bottom())
            .max()
            .unwrap_or(0) as usize
            + TOOL_EDGE as usize
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

    fn draw(&self, canvas: &mut Canvas<Window>, position: &Position, settings: DrawSettings) {
        let rect = Rect::new(0, 0, canvas.window().size().0, self.content_height() as u32);

        self.draw_menu(canvas, rect, settings)
            .expect("Can't draw toolbar");

        if let Some((tool, _)) = self.selected.and_then(|x| self.tools.get(x)) {
            let button = (*tool.0)();
            if let Some(subbar) = button.subtoolbar {
                subbar
                    .draw_menu(
                        canvas,
                        Rect::new(
                            rect.left(),
                            rect.top() + rect.height() as i32,
                            rect.width(),
                            rect.height(),
                        ),
                        settings,
                    ).expect("Can't draw toolbar");
            }
        }
    }

    fn update(&mut self, _dt: f64) {}

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
        if let Some((tool, _)) = self.selected.and_then(|x| self.tools.get(x)) {
            let button = (*tool.0)();
            if let Some(subbar) = button.subtoolbar {
                return self.content_height() + subbar.height();
            }
        }
        self.content_height()
    }
}
