#[derive(Debug)]
pub struct Transform {
    pub win_size: (f64, f64),
    /// `S(x, y) = (sx, sy)`
    pub scale: f64,
    /// `T(x, y) = (x + x', y + y')`
    pub translation: (f64, f64),
}

impl Transform {
    pub fn new_from_winsize((width, height): (f64, f64)) -> Transform {
        Transform {
            win_size: (width, height),
            scale: (width + height) / 8.,
            translation: (0., 0.),
        }
    }

    pub fn transform_po_to_px(&self, (x, y): (f64, f64)) -> (f64, f64) {
        let (tx, ty) = (x + self.translation.0, y + self.translation.1);
        let (stx, sty) = (tx * self.scale, ty * self.scale);
        (stx + self.win_size.0 / 2., sty + self.win_size.1 / 2.)
    }

    pub fn transform_px_to_po(&self, (wstx, wsty): (f64, f64)) -> (f64, f64) {
        let (stx, sty) = (wstx - self.win_size.0 / 2., wsty - self.win_size.1 / 2.);
        let (tx, ty) = (stx / self.scale, sty / self.scale);
        (tx - self.translation.0, ty - self.translation.1)
    }
}
