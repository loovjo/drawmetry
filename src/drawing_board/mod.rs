use std::f64::consts::PI;
use std::sync::{Mutex, Arc};
use std::time::{Duration, Instant};

use super::backend::geometry;
use super::graphics::*;
use super::icons;
use super::toolbar::{Tool, ToolKind};
use super::transform::Transform;

use ytesrev::drawable::{DrawSettings, Drawable, Position, State};
use ytesrev::image::PngImage;
use ytesrev::sdl2::event::Event;
use ytesrev::sdl2::keyboard::Keycode;
use ytesrev::sdl2::mouse::MouseButton;
use ytesrev::sdl2::pixels::Color;
use ytesrev::sdl2::rect::{Point, Rect};
use ytesrev::sdl2::render::Canvas;
use ytesrev::sdl2::video::Window;

pub struct DrawingBoard {
    pub state: Arc<Mutex<DState>>,

    pub transform: Transform,

    pub mouse_last: Point,
    pub moving_screen: bool,
    pub scrolling: f64,
}

impl DrawingBoard {
    pub fn new(state: Arc<Mutex<DState>>) -> DrawingBoard {
        DrawingBoard {
            state,
            transform: Transform::new_from_winsize((WINDOW_SIZE.0 as f64, WINDOW_SIZE.1 as f64)),
            mouse_last: Point::new(0, 0),
            moving_screen: false,
            scrolling: 0.,
        }
    }

    fn try_draw(&mut self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let state = self.state.lock().unwrap();

        //canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
        //canvas.clear();

        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let (w, h) = canvas.window().size();

        for obj in state.world.shapes.values() {
            if let Some(ro) = state.world.resolve_shape(obj) {
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

        for (id, point) in state.world.points.iter() {
            if let Some(rpoint) = state.world.resolve_point(point) {
                let p_px = self.transform.transform_po_to_px(rpoint);

                canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
                if let ToolKind::Mover = state.current_tool.kind {
                    if let geometry::Point::Arbitrary(_) = point {
                        canvas.set_draw_color(Color::RGBA(0, 0, 255, 255));
                    }
                }
                if state.current_tool.selected.contains(id) {
                    canvas.set_draw_color(Color::RGBA(0, 255, 0, 255));
                }
                fill_circle(canvas, p_px, 5.)?;
            }
        }

        Ok(())
    }
}

impl Drawable for DrawingBoard {
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

    fn draw(&mut self, canvas: &mut Canvas<Window>, position: &Position, _: DrawSettings) {
        if let Position::Rect(r) = position {
            canvas.set_clip_rect(*r);
            self.try_draw(canvas).expect("Can't draw");
            canvas.set_clip_rect(None);
        }
    }

    fn update(&mut self, dt: f64) {
        self.scrolling = self.scrolling * (0.01_f64).powf(dt);
        self.transform.scale *= (0.1_f64).powf(self.scrolling);
    }

    fn event(&mut self, event: Event) {
        //match event {
            //Event::KeyDown {
                //keycode: Some(Keycode::Escape),
                //..
            //} => {
                //self.tool.selected.clear();
            //}
            //Event::KeyDown {
                //keycode: Some(Keycode::L),
                //..
            //} => {
                //self.tool.kind = ToolKind::Line;
            //}
            //Event::KeyDown {
                //keycode: Some(Keycode::C),
                //..
            //} => {
                //self.tool.kind = ToolKind::Circle;
            //}
            //Event::KeyDown {
                //keycode: Some(Keycode::P),
                //..
            //} => {
                //self.tool.kind = ToolKind::Point;
            //}
            //Event::KeyDown {
                //keycode: Some(Keycode::M),
                //..
            //} => {
                //self.tool.kind = ToolKind::Mover;
            //}
            //Event::MouseButtonDown {
                //x,
                //y,
                //mouse_btn: MouseButton::Left,
                //..
            //} => {
                //let mouse_po = self.transform.transform_px_to_po((x as f64, y as f64));
                //match self.tool.kind {
                    //ToolKind::Point => {
                        //let point = get_closest(
                            //mouse_po,
                            //self.world.get_potential_points(),
                            //|point| self.world.resolve_point(point).unwrap_or((0., 0.)),
                            //Some(100. / self.transform.scale),
                        //).unwrap_or(geometry::Point::Arbitrary(mouse_po));

                        //self.world.add_point(point);
                    //}
                    //_ => {
                        //if let Some((&id, _)) = get_closest(
                            //mouse_po,
                            //self.world.points.iter().collect(),
                            //|(_, point)| self.world.resolve_point(point).unwrap_or((0., 0.)),
                            //Some(100. / self.transform.scale),
                        //) {
                            //if self.tool.kind == ToolKind::Mover {
                                //if let Some(geometry::Point::Arbitrary(_)) =
                                    //self.world.points.get(&id)
                                //{
                                //} else {
                                    //return;
                                //}
                            //}
                            //self.tool.selected.push(id);
                            //if self.tool.selected.len() >= self.tool.kind.needed_selected() {
                                //match self.tool.kind {
                                    //ToolKind::Line => {
                                        //self.world.add_shape(geometry::Shape::Line(
                                            //self.tool.selected[0],
                                            //self.tool.selected[1],
                                        //));
                                        //self.tool.selected.clear();
                                    //}
                                    //ToolKind::Circle => {
                                        //self.world.add_shape(geometry::Shape::Circle(
                                            //self.tool.selected[0],
                                            //self.tool.selected[1],
                                        //));
                                        //self.tool.selected.clear();
                                    //}
                                    //_ => {}
                                //}
                            //}
                        //}
                    //}
                //}
            //}
            //Event::MouseButtonUp { .. } => match self.tool.kind {
                //ToolKind::Mover => {
                    //self.tool.selected.clear();
                //}
                //_ => {}
            //},
            //Event::KeyDown {
                //keycode: Some(Keycode::Space),
                //..
            //} => {
                //self.moving_screen = true;
            //}
            //Event::KeyUp {
                //keycode: Some(Keycode::Space),
                //..
            //} => {
                //self.moving_screen = false;
            //}
            //Event::MouseMotion { x, y, .. } => {
                //if self.moving_screen {
                    //let (dx, dy) = (x - self.mouse_last.x, y - self.mouse_last.y);
                    //let (dtx, dty) = (
                        //dx as f64 / self.transform.scale,
                        //dy as f64 / self.transform.scale,
                    //);

                    //self.transform.translation = (
                        //self.transform.translation.0 + dtx as f64,
                        //self.transform.translation.1 + dty as f64,
                    //);
                //}
                //if let ToolKind::Mover = self.tool.kind {
                    //if let Some(id) = self.tool.selected.get(0) {
                        //let mouse_po = self.transform.transform_px_to_po((x as f64, y as f64));

                        //if let Some(point) = self.world.points.get_mut(&id) {
                            //*point = geometry::Point::Arbitrary(mouse_po);
                        //}
                    //}
                //}
                //self.mouse_last = Point::new(x, y);
            //}
            //Event::MouseWheel { y, .. } => {
                //self.scrolling += -y as f64 / 300.;
            //}
            //_ => {}
        //}
    }
}

