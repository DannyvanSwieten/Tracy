use skia_safe::{Font, Paint, Rect, Size};

use crate::{
    application::Application,
    application_model::ApplicationModel,
    canvas_2d::Canvas2D,
    constraints::BoxConstraints,
    style::Theme,
    widget::{Contexts, Properties, Widget},
    window_event::MouseEvent,
};

pub struct TextButton<Model: ApplicationModel> {
    text: String,
    font: Font,
    on_click: Option<Box<dyn Fn(&mut Application<Model>, &mut Model)>>,
    bg_paint: Paint,
    text_paint: Paint,
}

impl<Model: ApplicationModel> TextButton<Model> {
    pub fn new(text: &str, font_size: f32) -> Self {
        let font = Font::new(
            skia_safe::typeface::Typeface::new("arial", skia_safe::FontStyle::normal()).unwrap(),
            font_size,
        );
        let mut bg_paint = Paint::default();
        bg_paint.set_anti_alias(true);
        bg_paint.set_color4f(skia_safe::Color4f::new(0.25, 0.25, 0.25, 1.0), None);
        let mut text_paint = Paint::default();
        text_paint.set_anti_alias(true);
        text_paint.set_color4f(skia_safe::Color4f::new(1f32, 1f32, 1f32, 1f32), None);
        Self {
            text: text.to_string(),
            font,
            on_click: None,
            bg_paint,
            text_paint,
        }
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&mut Application<Model>, &mut Model) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl<Model: ApplicationModel> Widget<Model> for TextButton<Model> {
    fn layout(&mut self, constraints: &BoxConstraints, _: &Model) -> Size {
        let blob = skia_safe::TextBlob::from_str(&self.text, &self.font);
        let size = blob.unwrap().bounds().size();
        let width = size
            .width
            .min(constraints.max_width().unwrap_or(size.width));
        let height = size
            .height
            .min(constraints.max_height().unwrap_or(size.height));
        Size::new(width, height)
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, size: &Size, _: &Model) {
        canvas.draw_rounded_rect(&Rect::from_size(*size), 3f32, 3f32, &self.bg_paint);
        canvas.draw_string(&self.text, &self.font, &self.text_paint);
    }

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        _: &mut Application<Model>,
        model: &mut Model,
    ) {
        self.bg_paint
            .set_color4f(skia_safe::Color4f::new(0.45, 0.45, 0.45, 1.0), None);
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        if let Some(handler) = &self.on_click {
            handler(app, model)
        }

        self.bg_paint
            .set_color4f(skia_safe::Color4f::new(0.35, 0.35, 0.35, 1.0), None);
    }

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        self.bg_paint
            .set_color4f(skia_safe::Color4f::new(0.35, 0.35, 0.35, 1.0), None);
    }

    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        self.bg_paint
            .set_color4f(skia_safe::Color4f::new(0.25, 0.25, 0.25, 1.0), None);
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {}

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {}
}
