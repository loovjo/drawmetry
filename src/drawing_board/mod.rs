use std::sync::{Arc, Mutex};

use super::backend::{geometry, gwrapper};
use super::graphics::*;
use super::icons;
use super::tool::{SelectedStatus, ToolKind};
use super::transform::Transform;

use ytesrev::drawable::State;
use ytesrev::prelude::*;
use ytesrev::sdl2::event::Event;
use ytesrev::sdl2::keyboard::Keycode;
use ytesrev::sdl2::mouse::MouseButton;

pub struct DrawingBoard {
    pub state: Arc<Mutex<DState>>,
}

pub struct View {
    pub transform: Transform,
    pub mouse_last: Point,
    pub moving_screen: bool,
    pub scrolling: f64,
    pub show_hidden: bool,
}

impl DrawingBoard {
    pub fn new(state: Arc<Mutex<DState>>) -> DrawingBoard {
        DrawingBoard {
            state,
        }
    }

    fn try_draw(&self, canvas: &mut Canvas<Window>, settings: DrawSettings) -> Result<(), String> {
        let state = self.state.lock().unwrap();

        let (w, h) = canvas.window().size();

        for (id, obj) in &state.world.shapes {
            let mut alpha = 255;
            if let Some(gwrapper::Visibility::Hidden) =
                state.world.visibility.get(&gwrapper::ThingID::ShapeID(*id))
            {
                if !state.view.show_hidden {
                    continue;
                }
                alpha = 64;
            }

            canvas.set_draw_color(Color::RGBA(0, 0, 0, alpha));
            if let Some(selected) = state
                .current_tool
                .selected(&state.world)
                .get(&gwrapper::ThingID::ShapeID(*id))
            {
                let col = match selected {
                    SelectedStatus::Primary => (0, 255, 0),
                    SelectedStatus::Active => (128, 195, 255),
                };
                canvas.set_draw_color(Color::RGBA(col.0, col.1, col.2, alpha));
            }


            if let Some(ro) = state.world.resolve_shape(obj) {
                match ro {
                    geometry::ResolvedShape::Circle(center, rad) => {
                        let center_px = state.view.transform.transform_po_to_px(center);
                        draw_circle(canvas, center_px, rad * state.view.transform.scale)?;
                    }
                    geometry::ResolvedShape::Line(k, m) => {
                        let start_x = state.view.transform.transform_px_to_po((0., 0.)).0;
                        let start_y = k * start_x + m;
                        let start_point = (start_x, start_y);

                        let end_x = state.view.transform.transform_px_to_po((w as f64, 0.)).0;
                        let end_y = k * end_x + m;
                        let end_point = (end_x, end_y);

                        let start_px = state.view.transform.transform_po_to_px(start_point);
                        let end_px = state.view.transform.transform_po_to_px(end_point);

                        utils::line_aa(canvas, (start_px.0, start_px.1), (end_px.0, end_px.1));
                    }
                    geometry::ResolvedShape::LineUp(x) => {
                        let x_px = state.view.transform.transform_po_to_px((x, 0.)).0;
                        utils::line_aa(canvas, (x_px, 0.), (x_px, h as f64));
                    }
                }
            }
        }

        for (id, point) in state.world.points.iter() {
            if let Some(rpoint) = state.world.resolve_point(point) {
                let p_px = state.view.transform.transform_po_to_px(rpoint);

                let mut image = &*icons::CIRCLE_NORMAL;

                if let Some(selected) = state
                    .current_tool
                    .selected(&state.world)
                    .get(&gwrapper::ThingID::PointID(*id))
                {
                    image = match selected {
                        SelectedStatus::Primary => &*icons::CIRCLE_PRIMARY,
                        SelectedStatus::Active => &*icons::CIRCLE_ACTIVE,
                    }
                }

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

        let state = &mut *self.state.lock().unwrap();

        let mouse_po = state.view.transform.transform_px_to_po((x as f64, y as f64));

        state
            .current_tool
            .click(&mut state.world, &mut state.view, mouse_po);
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
        let state = &mut *self.state.lock().unwrap();

        state.view.scrolling = state.view.scrolling * (0.01_f64).powf(dt);
        state.view.transform.scale *= (0.1_f64).powf(state.view.scrolling);
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
                state.view.moving_screen = true;
            }
            Event::KeyUp {
                keycode: Some(Keycode::Space),
                ..
            } => {
                state.view.moving_screen = false;
            }
            Event::MouseButtonUp { .. } => {
                if state.current_tool.kind() == ToolKind::Mover {
                    state.current_tool = state.current_tool.kind().into_tool();
                }
            }
            Event::MouseMotion { x, y, .. } => {
                if state.view.moving_screen {
                    let (dx, dy) = (x - state.view.mouse_last.x, y - state.view.mouse_last.y);
                    let (dtx, dty) = (
                        dx as f64 / state.view.transform.scale,
                        dy as f64 / state.view.transform.scale,
                    );

                    state.view.transform.translation = (
                        state.view.transform.translation.0 + dtx as f64,
                        state.view.transform.translation.1 + dty as f64,
                    );
                }
                if state.current_tool.kind() == ToolKind::Mover {
                    // Get the active point by inspecting Tool::selected
                    let selected = state.current_tool.selected(&state.world);
                    if let Some((gwrapper::ThingID::PointID(id), _)) = selected
                        .iter()
                        .find(|(_, x)| x == &&SelectedStatus::Primary)
                    {
                        if let Some(point) = state.world.geometry.points.get_mut(id) {
                            *point = geometry::create_arbitrary(
                                state.view.transform.transform_px_to_po((x as f64, y as f64)),
                            );
                        }
                    }
                }
                state.view.mouse_last = Point::new(x, y);
            }
            Event::MouseWheel { y, .. } => {
                state.view.scrolling += -y as f64 / 300.;
            }
            _ => {}
        }
    }
}
