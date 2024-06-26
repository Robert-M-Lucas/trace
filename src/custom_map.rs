use ratatui::prelude::Color;
use ratatui::widgets::canvas::{MapResolution, Painter, Shape};
use crate::DATA_TYPE;

pub struct CMap {
    pub data: DATA_TYPE,
    pub zoom: f32,
    pub pos: (f32, f32)
}

impl Shape for CMap {
    fn draw(&self, painter: &mut Painter) {
        for (x, y) in self.data.iter() {
            let x = (*x - self.pos.0) * self.zoom + 0.5;
            let y = (*y - self.pos.1) * self.zoom + 0.5;
            if let Some((x, y)) = painter.get_point(x as f64, 1.0 - y as f64) {
                painter.paint(x, y, Color::White);
            }
        }
    }
}