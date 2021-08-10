use skia_safe::{Color, Font, Image, Paint, Point, Rect};

pub trait Canvas2D {
    fn clear(&mut self, color: &Color);
    fn save(&mut self);
    fn restore(&mut self);
    fn draw_rect(&mut self, rect: &Rect, paint: &Paint);
    fn draw_rounded_rect(&mut self, rect: &Rect, rx: f32, ry: f32, paint: &Paint);
    fn draw_string(&mut self, text: &str, center: &Point, font: &Font, paint: &Paint);
    fn flush(&mut self) -> (ash::vk::Image, ash::vk::ImageView);
}
