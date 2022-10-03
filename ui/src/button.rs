use skia_safe::{font::Edging, Font, Paint, Rect, Size};

use crate::{
    application::Application,
    application_model::ApplicationModel,
    canvas_2d::Canvas2D,
    constraints::BoxConstraints,
    style::Theme,
    widget::{Properties, Widget},
    window_event::MouseEvent,
};

enum ButtonState {
    Inactive,
    Active,
    Hover,
}

pub enum ButtonStyle {
    Text,
    Outline,
    Fill,
}

pub struct TextButton<Model: ApplicationModel> {
    state: ButtonState,
    style: ButtonStyle,
    text: String,
    font: Font,
    on_click: Option<Box<dyn Fn(&mut Application<Model>, &mut Model)>>,
}

impl<Model: ApplicationModel> TextButton<Model> {
    pub fn new(text: &str, font_size: f32) -> Self {
        let mut font = Font::new(
            skia_safe::typeface::Typeface::new("arial black", skia_safe::FontStyle::normal())
                .unwrap(),
            font_size,
        );
        font.set_edging(Edging::SubpixelAntiAlias);
        font.set_subpixel(true);
        let mut bg_paint = Paint::default();
        bg_paint.set_anti_alias(true);
        bg_paint.set_color4f(skia_safe::Color4f::new(0.25, 0.25, 0.25, 1.0), None);
        let mut text_paint = Paint::default();
        text_paint.set_anti_alias(true);
        text_paint.set_color4f(skia_safe::Color4f::new(1f32, 1f32, 1f32, 1f32), None);
        Self {
            state: ButtonState::Inactive,
            text: text.to_string(),
            font,
            on_click: None,
            style: ButtonStyle::Text,
        }
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
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
        let width = constraints.max_width().unwrap_or(size.width);
        let height = constraints.max_height().unwrap_or(size.height);
        Size::new(width, height)
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, size: &Size, _: &Model) {
        let mut text_paint = Paint::default();
        text_paint.set_anti_alias(true);

        match self.style {
            ButtonStyle::Fill => {
                let mut bg_paint = Paint::default();
                bg_paint.set_anti_alias(true);
                match self.state {
                    ButtonState::Inactive => bg_paint.set_color(theme.primary.with_a(200)),
                    ButtonState::Active => bg_paint.set_color(theme.primary),
                    ButtonState::Hover => bg_paint.set_color(theme.primary.with_a(230)),
                };
                canvas.draw_rounded_rect(
                    &Rect::from_wh(size.width, size.height),
                    4f32,
                    4f32,
                    &bg_paint,
                );
                text_paint.set_color(theme.text);
                canvas.draw_string(&Rect::from_size(*size), &self.text, &self.font, &text_paint);
            }
            ButtonStyle::Outline => {
                let mut bg_paint = Paint::default();
                bg_paint.set_anti_alias(true);

                bg_paint.set_color(theme.primary);
                bg_paint.set_stroke(true);
                canvas.draw_rounded_rect(
                    &Rect::from_wh(size.width, size.height),
                    4f32,
                    4f32,
                    &bg_paint,
                );

                match self.state {
                    ButtonState::Inactive => (),
                    ButtonState::Active => {
                        bg_paint.set_color(theme.primary.with_a(100));
                        bg_paint.set_stroke(false);
                        canvas.draw_rounded_rect(
                            &Rect::from_wh(size.width, size.height),
                            4f32,
                            4f32,
                            &bg_paint,
                        );
                    }
                    ButtonState::Hover => {
                        bg_paint.set_color(theme.primary.with_a(50));
                        bg_paint.set_stroke(false);
                        canvas.draw_rounded_rect(
                            &Rect::from_wh(size.width, size.height),
                            4f32,
                            4f32,
                            &bg_paint,
                        );
                    }
                }

                text_paint.set_color(theme.primary);

                canvas.draw_string(&Rect::from_size(*size), &self.text, &self.font, &text_paint);
            }
            ButtonStyle::Text => {
                text_paint.set_color(theme.primary);

                let mut bg_paint = Paint::default();
                bg_paint.set_anti_alias(true);
                match self.state {
                    ButtonState::Inactive => (),
                    ButtonState::Active => {
                        bg_paint.set_color(theme.primary.with_a(100));
                        bg_paint.set_stroke(false);
                        canvas.draw_rounded_rect(
                            &Rect::from_wh(size.width, size.height),
                            4f32,
                            4f32,
                            &bg_paint,
                        );
                    }
                    ButtonState::Hover => {
                        bg_paint.set_color(theme.primary.with_a(50));
                        bg_paint.set_stroke(false);
                        canvas.draw_rounded_rect(
                            &Rect::from_wh(size.width, size.height),
                            4f32,
                            4f32,
                            &bg_paint,
                        );
                    }
                }

                canvas.draw_string(&Rect::from_size(*size), &self.text, &self.font, &text_paint);
            }
        }
    }

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        _: &mut Application<Model>,
        model: &mut Model,
    ) {
        self.state = ButtonState::Active
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        if let Some(handler) = &self.on_click {
            handler(app, model)
        }

        self.state = ButtonState::Hover
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {}

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {}

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        self.state = ButtonState::Hover
    }
    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        self.state = ButtonState::Inactive
    }
}
