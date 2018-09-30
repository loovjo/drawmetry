use std::sync::{Arc, Mutex};

use super::backend::{geometry, gwrapper};
use super::graphics::*;
use super::icons;
use super::tool::ToolKind;
use super::transform::Transform;

use ytesrev::drawable::State;
use ytesrev::prelude::*;
use ytesrev::sdl2::event::Event;
use ytesrev::sdl2::keyboard::Keycode;
use ytesrev::sdl2::mouse::MouseButton;

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

    fn try_draw(
        &self,
        canvas: &mut Canvas<Window>,
        settings: DrawSettings,
    ) -> Result<(), String> {
        let state = self.state.lock().unwrap();

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

                        utils::line_aa(canvas, (start_px.0, start_px.1), (end_px.0, end_px.1));
                    }
                    geometry::ResolvedShape::LineUp(x) => {
                        let x_px = self.transform.transform_po_to_px((x, 0.)).0;
                        utils::line_aa(canvas, (x_px, 0.), (x_px, h as f64));
                    }
                }
            }
        }

        for (id, point) in state.world.points.iter() {
            if let Some(rpoint) = state.world.resolve_point(point) {
                let p_px = self.transform.transform_po_to_px(rpoint);

                let is_selected = state.current_tool.selected().contains_key(&gwrapper::ThingID::PointID(*id));
                let mover = false;//state.current_tool.kind() == ToolKind::Mover;

                let image = match (is_selected, mover) {
                    (true, false) => &*icons::CIRCLE_SELECT,
                    (false, true) => {
                        if let geometry::Point::Arbitrary(_) = point {
                            &*icons::CIRCLE_MOVER
                        } else {
                            &*icons::CIRCLE_NORMAL
                        }
                    }
                    (true, true) => &*icons::CIRCLE_MOVING,
                    (false, _) => &*icons::CIRCLE_NORMAL,
                };

                image.draw(
                    canvas,
                    &Position::Center(Point::new(p_px.0 as i32, p_px.1 as i32)),
                    settings,
                );
            }
        }

        Ok(())
    }

    pub fn mouse_down(&mut self, position: Point, _button: MouseButton) {
        let (x, y) = (position.x(), position.y());

        let mouse_po = self.transform.transform_px_to_po((x as f64, y as f64));

        let state = &mut *self.state.lock().unwrap();

        state.current_tool.click(&mut state.world, mouse_po);

        //match state.current_tool.kind() {
            //ToolKind::Point => {
                //let point = get_closest(
                    //mouse_po,
                    //state.world.get_potential_points(),
                    //|point| state.world.resolve_point(point).unwrap_or((0., 0.)),
                    //Some(100. / self.transform.scale),
                //).unwrap_or(geometry::create_arbitrary(mouse_po));

                //state.world.add_point(point);
            //}
            //_ => {
                //if let Some((&id, _)) = get_closest(
                    //mouse_po,
                    //state.world.points.iter().collect(),
                    //|(_, point)| state.world.resolve_point(point).unwrap_or((0., 0.)),
                    //Some(100. / self.transform.scale),
                //) {
                    //if state.current_tool.kind == ToolKind::Mover {
                        //if let Some(geometry::Point::Arbitrary(_)) = state.world.points.get(&id) {
                        //} else {
                            //return;
                        //}
                    //}
                    //state.current_tool.selected.push(id);
                    //if state.current_tool.selected.len()
                        //>= state.current_tool.kind.needed_selected()
                    //{
                        //match state.current_tool.kind {
                            //ToolKind::Line => {
                                //state.world.add_shape(geometry::Shape::Line(
                                    //state.current_tool.selected[0],
                                    //state.current_tool.selected[1],
                                //));
                                //state.current_tool.selected.clear();
                            //}
                            //ToolKind::Circle => {
                                //state.world.add_shape(geometry::Shape::Circle(
                                    //state.current_tool.selected[0],
                                    //state.current_tool.selected[1],
                                //));
                                //state.current_tool.selected.clear();
                            //}
                            //_ => {}
                        //}
                    //}
                //}
            //}
        //}
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

    fn draw(&self, canvas: &mut Canvas<Window>, position: &Position, settings: DrawSettings) {
        if let Position::Rect(r) = position {
            canvas.set_clip_rect(*r);
            self.try_draw(canvas, settings).expect("Can't draw");
            canvas.set_clip_rect(None);
        }
    }

    fn update(&mut self, dt: f64) {
        self.scrolling = self.scrolling * (0.01_f64).powf(dt);
        self.transform.scale *= (0.1_f64).powf(self.scrolling);
    }

    fn event(&mut self, event: Event) {
        let state = &mut *self.state.lock().unwrap();

        match event {
            Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                state.current_tool = state.current_tool.kind().into_tool();
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
}
