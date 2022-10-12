use std::ops::Range;

use skia_safe::{
    font::Edging,
    shaper::{BiDiRunIterator, TextBlobBuilderRunHandler},
    textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
    Color, Color4f, Font, FontMgr, FourByteTag, Paint, Point, Rect, Shaper, Size, TextBlobBuilder,
};
use winit::event::{ElementState, VirtualKeyCode};

use crate::{application_model::ApplicationModel, widget::Widget};
#[derive(Default)]
struct EditorState {
    text: String,
    caret_position: usize,
    selection: Range<usize>,
}

pub struct TextBox {
    state: EditorState,
    placeholder: String,
    style: ParagraphStyle,
}

impl TextBox {
    pub fn new(placeholder: &str) -> Self {
        Self {
            state: EditorState::default(),
            placeholder: placeholder.to_string(),
            style: ParagraphStyle::new(),
        }
    }
}

impl<Model: ApplicationModel> Widget<Model> for TextBox {
    fn layout(&mut self, constraints: &crate::constraints::BoxConstraints, model: &Model) -> Size {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        let mut paragraph_builder = ParagraphBuilder::new(&self.style, font_collection);
        let mut ts = TextStyle::new();
        ts.set_font_size(18f32);
        ts.set_foreground_color(Paint::default());
        paragraph_builder.push_style(&ts);
        paragraph_builder.add_text(&self.placeholder);
        let mut paragraph = paragraph_builder.build();
        paragraph.layout(constraints.max_width().unwrap() - 4f32);
        Size::new(constraints.max_width().unwrap(), paragraph.height())
    }

    fn paint(
        &self,
        theme: &crate::style::Theme,
        canvas: &mut dyn crate::canvas_2d::Canvas2D,
        rect: &skia_safe::Size,
        model: &Model,
    ) {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        let mut paragraph_builder = ParagraphBuilder::new(&self.style, font_collection);
        let mut ts = TextStyle::new();
        ts.set_font_size(18f32);
        if !self.state.text.is_empty() {
            ts.set_foreground_color(Paint::default());
            paragraph_builder.push_style(&ts);
            paragraph_builder.add_text(&self.state.text);
        } else {
            ts.set_foreground_color(Paint::new(Color4f::new(0.0, 0.0, 0.0, 0.5), None));
            paragraph_builder.push_style(&ts);
            paragraph_builder.add_text(&self.placeholder);
        }

        let mut paragraph = paragraph_builder.build();
        paragraph.layout(rect.width - 4f32);
        let selection_boxes = paragraph.get_rects_for_range(
            self.state.selection.clone(),
            skia_safe::textlayout::RectHeightStyle::IncludeLineSpacingBottom,
            skia_safe::textlayout::RectWidthStyle::Tight,
        );

        let mut selected_rect = Rect::default();
        for textbox in &selection_boxes {
            selected_rect.join(textbox.rect)
        }

        let mut border_paint = Paint::default();
        border_paint.set_color(Color::from_rgb(255, 255, 255));
        canvas.draw_rect(&Rect::from_size(*rect), &border_paint);
        if selected_rect.width() > 0f32 {
            border_paint.set_color(Color::from_rgb(0, 0, 255));
            canvas.draw_rect(&selected_rect, &border_paint);
        }
        border_paint.set_stroke(true);
        border_paint.set_color(Color::from_rgb(0, 0, 0));
        canvas.draw_rect(&Rect::from_size(*rect), &border_paint);
        canvas.draw_paragraph(&Point::new(2f32, 0f32), &paragraph)
    }

    fn mouse_up(
        &mut self,
        event: &crate::window_event::MouseEvent,
        app: &mut crate::application::Application<Model>,
        model: &mut Model,
    ) {
    }

    fn mouse_dragged(
        &mut self,
        event: &crate::window_event::MouseEvent,
        properties: &crate::widget::Properties,
        model: &mut Model,
    ) {
    }

    fn mouse_moved(&mut self, event: &crate::window_event::MouseEvent, model: &mut Model) {}

    fn mouse_entered(&mut self, event: &crate::window_event::MouseEvent, model: &mut Model) {}

    fn mouse_left(&mut self, event: &crate::window_event::MouseEvent, model: &mut Model) {}

    fn keyboard_event(&mut self, event: &winit::event::KeyboardInput, model: &mut Model) -> bool {
        if let Some(keycode) = event.virtual_keycode {
            if event.state == ElementState::Pressed {
                match keycode {
                    VirtualKeyCode::Left => self.state.caret_position -= 1,
                    VirtualKeyCode::Right => self.state.caret_position += 1,
                    VirtualKeyCode::Back => {
                        if self.state.caret_position > 0 {
                            self.state.text.remove(self.state.caret_position - 1);
                            self.state.caret_position -= 1;
                        }
                    }
                    _ => (),
                }
            }
        }

        true
    }

    fn flex(&self) -> f32 {
        0f32
    }

    fn mouse_down(
        &mut self,
        event: &crate::window_event::MouseEvent,
        _: &crate::widget::Properties,
        _: &mut crate::application::Application<Model>,
        model: &mut Model,
    ) {
    }

    fn character_received(&mut self, character: char, model: &mut Model) -> bool {
        if !character.is_ascii_control() {
            self.state.text.push(character);
            self.state.caret_position += 1;
        }

        true
    }
}
