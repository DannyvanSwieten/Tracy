use skia_safe::{
    font::Edging,
    shaper::{BiDiRunIterator, TextBlobBuilderRunHandler},
    Font, FontMgr, FourByteTag, Paint, Point, Shaper, TextBlobBuilder,
};
use winit::event::VirtualKeyCode;

use crate::{application_model::ApplicationModel, widget::Widget};
#[derive(Default)]
struct EditorState {
    text: String,
    caret_position: usize,
}

pub struct TextBox {
    state: EditorState,
}

impl TextBox {
    pub fn new() -> Self {
        Self {
            state: EditorState::default(),
        }
    }
}

impl<Model: ApplicationModel> Widget<Model> for TextBox {
    fn layout(
        &mut self,
        constraints: &crate::constraints::BoxConstraints,
        model: &Model,
    ) -> skia_safe::Size {
        todo!()
    }

    fn paint(
        &self,
        theme: &crate::style::Theme,
        canvas: &mut dyn crate::canvas_2d::Canvas2D,
        rect: &skia_safe::Size,
        model: &Model,
    ) {
        Shaper::purge_caches();
        let mut builder = TextBlobBuilderRunHandler::new(&self.state.text, Point::new(0f32, 0f32));
        let mut font = Font::default();
        font.set_size(14f32);
        font.set_edging(Edging::SubpixelAntiAlias);
        font.set_subpixel(true);

        let mut font_iterator = Shaper::new_font_mgr_run_iterator(&self.state.text, &font, None);
        let run_iterator = Shaper::new_bidi_run_iterator(&self.state.text, 0);
        let lang_iterator = Shaper::new_std_language_run_iterator(&self.state.text);
        let tag = FourByteTag::from_chars('Z', 'y', 'y', 'y');
        let mut script_iterator = Shaper::new_script_run_iterator(&self.state.text, tag);
        let shaper = Shaper::new(FontMgr::new());
        shaper.shape_with_iterators(
            &self.state.text,
            &mut font_iterator,
            &mut run_iterator.unwrap(),
            &mut script_iterator,
            &mut lang_iterator.unwrap(),
            rect.width,
            &mut builder,
        );

        let blob = builder.make_blob();
        let mut text_paint = Paint::default();
        text_paint.set_anti_alias(true);
        text_paint.set_color(theme.text);
        if let Some(blob) = blob {
            canvas.draw_text_blob(&Point::new(0f32, 0f32), &blob, &text_paint);
        }
    }

    fn mouse_up(
        &mut self,
        event: &crate::window_event::MouseEvent,
        app: &mut crate::application::Application<Model>,
        model: &mut Model,
    ) {
        todo!()
    }

    fn mouse_dragged(
        &mut self,
        event: &crate::window_event::MouseEvent,
        properties: &crate::widget::Properties,
        model: &mut Model,
    ) {
        todo!()
    }

    fn mouse_moved(&mut self, event: &crate::window_event::MouseEvent, model: &mut Model) {
        todo!()
    }

    fn mouse_entered(&mut self, event: &crate::window_event::MouseEvent, model: &mut Model) {
        todo!()
    }

    fn mouse_left(&mut self, event: &crate::window_event::MouseEvent, model: &mut Model) {
        todo!()
    }

    fn keyboard_event(&mut self, event: &winit::event::KeyboardInput, model: &mut Model) {
        if let Some(keycode) = event.virtual_keycode {
            match keycode {
                VirtualKeyCode::Left => self.state.caret_position -= 1,
                VirtualKeyCode::Right => self.state.caret_position += 1,
                _ => (),
            }
        }
    }
}
