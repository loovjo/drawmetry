use std::f64::consts::PI;
use std::sync::{
    mpsc::{channel, Receiver},
    Arc, Mutex,
};

use backend::gwrapper::GWrapper;
use drawing_board::{DrawingBoard, View};
use transform::Transform;
use tool::{Tool, ToolKind};
use toolbar::{default_toolbar, Button, ToolBar};
use ytesrev::drawable::KnownSize;
use ytesrev::prelude::*;
use ytesrev::sdl2::event::Event;

pub const WINDOW_SIZE: (u32, u32) = (1200, 800);

pub struct DState {
    pub world: GWrapper,
    pub current_tool: Box<dyn Tool>,
    pub view: View,
}

pub struct DScene {
    inner: Split<ToolBar, DrawingBoard>,
    state: Arc<Mutex<DState>>,
    tool_change: Receiver<Button>,
}

pub fn create_layout(world: GWrapper) -> DScene {
    let (send, recv) = channel::<Button>();
    let tool_bar = default_toolbar(send);

    let state = DState {
        world: world,
        current_tool: ToolKind::Selector.into_tool(),
        view: View {
            transform: Transform::new_from_winsize((
                WINDOW_SIZE.0 as f64,
                WINDOW_SIZE.1 as f64,
            )),
            mouse_last: Point::new(0, 0),
            moving_screen: false,
            scrolling: 0.,
            show_hidden: true,
        },
    };

    let state_arc_mutex = Arc::new(Mutex::new(state));
    let drawing_board = DrawingBoard::new(state_arc_mutex.clone());

    DScene {
        inner: Split::new_const(
            tool_bar.height() as u32,
            Orientation::Vertical,
            UpdateOrder::FirstSecond,
            tool_bar,
            drawing_board,
        ),
        state: state_arc_mutex.clone(),
        tool_change: recv,
    }
}

impl Scene for DScene {
    fn update(&mut self, dt: f64) {
        self.inner.first.update(dt);
        self.inner.second.update(dt);

        for callback in self.tool_change.try_iter() {
            if let Ok(ref mut state) = self.state.lock() {
                (*callback.function)(state);
            }
        }

        let tb_height = self.inner.first.height();
        self.inner.amount = Box::new(move |_| tb_height as u32);
    }

    fn draw(&self, canvas: &mut Canvas<Window>, settings: DrawSettings) {
        let (w, h) = canvas.window().size();
        self.inner
            .draw(canvas, &Position::Rect(Rect::new(0, 0, w, h)), settings);
    }

    fn event(&mut self, event: YEvent) {
        match event {
            YEvent::Other(Event::MouseButtonDown {
                x, y, mouse_btn, ..
            }) => {
                if y < self.inner.first.height() as i32 {
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

pub const STEPS_BY_RADIUS: f64 = 1.5;
pub const MAX_STEPS: usize = 500;

pub fn draw_circle(canvas: &mut Canvas<Window>, pos: (f64, f64), r: f64) -> Result<(), String> {
    let (width, height) = canvas.window().size();
    let (width, height) = (width as i32, height as i32);

    let points = draw_circle_points(pos, r);

    if points.len() == 0 {
        return Ok(());
    }

    let mut last = *points.last().unwrap();

    for point in points {
        if last.x() < 0 && point.x() < 0
            || last.y() < 0 && point.y() < 0
            || last.x() > width && point.x() > width
            || last.y() > height && point.y() > height
        {
            continue;
        }
        utils::line_aa(
            canvas,
            (last.x() as f64, last.y() as f64),
            (point.x() as f64, point.y() as f64),
        );
        last = point;
    }

    Ok(())
}

pub fn draw_circle_points((x, y): (f64, f64), r: f64) -> Vec<Point> {
    let steps = (STEPS_BY_RADIUS as f64 * r) as usize;
    let steps = steps.min(MAX_STEPS);

    let mut points = Vec::with_capacity(steps);

    for i in 0..steps {
        let theta = (i as f64 / steps as f64) * 2. * PI;
        let (x_, y_) = (x + r * theta.cos(), y + r * theta.sin());
        points.push(Point::new(x_ as i32, y_ as i32));
    }
    points
}

pub fn get_best<T, F: Fn(&T) -> f64>(objs: Vec<T>, f: F) -> Option<(f64, T)> {
    let mut best: Option<(f64, T)> = None;

    for obj in objs {
        let dist = f(&obj);

        if let Some((bdist, _)) = best {
            if dist < bdist {
                best = Some((dist, obj));
            }
        } else {
            best = Some((dist, obj));
        }
    }

    best
}

pub fn get_closest<T, F: Fn(&T) -> (f64, f64)>(
    to: (f64, f64),
    objs: Vec<T>,
    f: F,
    max: Option<f64>,
) -> Option<T> {
    let dist_fn = |obj: &T| -> f64 {
        let pos = f(obj);
        let (dx, dy) = (pos.0 - to.0, pos.1 - to.1);
        dx * dx + dy * dy
    };
    if let Some((dist, closest)) = get_best(objs, dist_fn) {
        if let Some(max) = max {
            if dist < max {
                Some(closest)
            } else {
                None
            }
        } else {
            Some(closest)
        }
    } else {
        None
    }
}
