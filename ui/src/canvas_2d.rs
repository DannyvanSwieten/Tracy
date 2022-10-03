use crate::skia_vulkan_canvas::SkiaCanvasImage;
use skia_safe::{textlayout::Paragraph, Color, Font, Paint, Point, Rect, TextBlob};

pub trait Canvas2D {
    fn clear(&mut self, color: &Color);

    fn save(&mut self);
    fn restore(&mut self);
    fn translate(&mut self, point: &Point);

    fn draw_rect(&mut self, rect: &Rect, paint: &Paint);
    fn draw_rounded_rect(&mut self, rect: &Rect, rx: f32, ry: f32, paint: &Paint);

    fn draw_circle(&mut self, center: &Point, radius: f32, paint: &Paint);

    fn draw_string(&mut self, rect: &Rect, text: &str, font: &Font, paint: &Paint);
    fn draw_text_blob(&mut self, pos: &Point, blob: &TextBlob, paint: &Paint);
    fn draw_paragraph(&mut self, pos: &Point, paragraph: &Paragraph);

    fn draw_vk_image(&mut self, image: &ash::vk::Image, width: u32, height: u32);
    fn draw_vk_image_rect(&mut self, src_rect: &Rect, dst_rect: &Rect, image: &ash::vk::Image);
    fn flush(&mut self) -> (SkiaCanvasImage, ash::vk::ImageView);
}
