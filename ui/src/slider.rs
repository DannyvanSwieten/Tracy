use skia_safe::{Font, Paint, Point, Rect, Size};

use crate::{
    application::Application,
    application_model::ApplicationModel,
    canvas_2d::Canvas2D,
    constraints::BoxConstraints,
    style::Theme,
    widget::{map_range, ChildSlot, Properties, Widget},
    window_event::MouseEvent,
};

enum SliderThumbState {
    Active,
    Inactive,
}

pub struct SliderThumb<Model> {
    state: SliderThumbState,
    marker: std::marker::PhantomData<Model>,
}

impl<Model: ApplicationModel> Widget<Model> for SliderThumb<Model> {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        let w = constraints.max_height().unwrap();
        Size::new(w, w)
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        let radius = rect.width / 2.0;
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        match self.state {
            SliderThumbState::Active => {
                paint.set_color(theme.slider.thumb.color.with_a(128));
                canvas.draw_circle(&Point::new(0f32, 0f32), radius * 1.5, &paint)
            }

            _ => (),
        }
        paint.set_color(theme.slider.thumb.color);
        canvas.draw_circle(&Point::new(0f32, 0f32), radius, &paint)
    }

    fn mouse_down(
        &mut self,
        _: &MouseEvent,
        _: &Properties,
        _: &mut Application<Model>,
        _: &mut Model,
    ) {
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {}

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {
        self.state = SliderThumbState::Active
    }

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {}

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        self.state = SliderThumbState::Active
    }

    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        self.state = SliderThumbState::Inactive
    }
}

pub struct Slider<Model> {
    thumb: ChildSlot<Model>,
    min: f32,
    max: f32,
    discrete: bool,
    current_normalized: f32,
    current_value: f32,
    last_position: f32,
    value_changed: Option<Box<dyn FnMut(f32, &mut Model)>>,
}

impl<Model: ApplicationModel + 'static> Slider<Model> {
    pub fn new() -> Self {
        Slider::new_with_min_max_and_value(0., 1., 0., false)
    }

    pub fn new_with_min_max_and_value(min: f32, max: f32, value: f32, discrete: bool) -> Self {
        Slider {
            thumb: ChildSlot::new(SliderThumb {
                state: SliderThumbState::Inactive,
                marker: std::marker::PhantomData::default(),
            }),
            min,
            max,
            discrete,
            current_normalized: value / (max - min),
            current_value: value,
            last_position: 0.,
            value_changed: None,
        }
    }

    pub fn with_handler<F>(mut self, handler: F) -> Self
    where
        F: FnMut(f32, &mut Model) + 'static,
    {
        self.value_changed = Some(Box::new(handler));
        self
    }

    pub fn set_value(&mut self, value: f32) {
        self.current_value = value.max(self.min).min(self.max);
        self.current_normalized = map_range(self.current_value, self.min, self.max, 0., 1.)
    }
}

impl<Model: ApplicationModel> Widget<Model> for Slider<Model> {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        // Boldly unwrapping here. If you have not given constraints to a slider then we don't know how big it should be.
        let my_size = Size::new(
            constraints.max_width().unwrap(),
            constraints.max_height().unwrap(),
        );

        let thumb_size = self.thumb.layout(constraints, model);
        self.thumb.set_size(&thumb_size);
        let thumb_pos = Point::new(
            self.last_position + thumb_size.width / 2.0,
            my_size.height / 2.0,
        );
        self.thumb.set_position(&thumb_pos);
        my_size
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        let mut fill_paint = Paint::default();
        fill_paint.set_anti_alias(true);

        let rounding = 4f32;

        let rect = Rect::from_size(*rect);
        let mut fill_rect = Rect::from_wh(rect.width(), rect.height() / 4.0);
        fill_rect.offset(Point::new(0f32, rect.center_y() - fill_rect.center_y()));
        let mut unfill_rect = Rect::from_wh(rect.width(), rect.height() / 5.0);
        unfill_rect.offset(Point::new(0f32, rect.center_y() - unfill_rect.center_y()));

        fill_paint.set_color(theme.slider.fill);
        fill_paint.set_stroke(true);
        canvas.draw_rounded_rect(&unfill_rect, rounding, rounding, &fill_paint);

        fill_paint.set_alpha_f(0.25);
        fill_paint.set_stroke(false);
        canvas.draw_rounded_rect(&unfill_rect, rounding, rounding, &fill_paint);

        fill_paint.set_alpha_f(1.0);
        let mut fill_rect = Rect::from_wh(self.last_position, rect.height() / 4.0);
        fill_rect.offset(Point::new(0f32, rect.center_y() - fill_rect.center_y()));
        canvas.draw_rounded_rect(&fill_rect, rounding, rounding, &fill_paint);

        self.thumb.paint(theme, canvas, &rect.size(), model)
    }

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        app: &mut Application<Model>,
        model: &mut Model,
    ) {
        self.last_position = event.local_position().x;
        self.current_normalized = (1. / properties.size.width) * self.last_position;

        self.current_value = map_range(self.current_normalized, 0., 1., self.min, self.max);
        if self.discrete {
            self.current_value = self.current_value.round();
        }
        if let Some(l) = &mut self.value_changed {
            (l)(self.current_value, model);
        }

        let mut thumb_pos = *self.thumb.position();
        thumb_pos.x = self.current_value * properties.size.width;
        self.thumb.set_position(&thumb_pos);

        self.thumb.mouse_down(event, properties, app, model)
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        self.thumb.mouse_up(event, app, model)
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {
        self.last_position = event.local_position().x;
        self.current_normalized =
            (1. / properties.size.width) * self.last_position.min(properties.size.width).max(0.);

        self.current_value = map_range(self.current_normalized, 0., 1., self.min, self.max);

        if self.discrete {
            self.current_value = self.current_value.round();
        }
        if let Some(l) = &mut self.value_changed {
            (l)(self.current_value, model);
        }

        let mut thumb_pos = *self.thumb.position();
        thumb_pos.x = self.current_value * properties.size.width;
        self.thumb.set_position(&thumb_pos);
    }

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {
        self.thumb.mouse_moved(event, model)
    }

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        self.thumb.mouse_entered(event, model)
    }

    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        self.thumb.mouse_left(event, model)
    }
}

pub struct Switch<Model> {
    thumb: ChildSlot<Model>,
    value_changed: Option<Box<dyn FnMut(bool, &mut Model)>>,
    active: bool,
}

impl<Model: ApplicationModel + 'static> Switch<Model> {
    pub fn new() -> Self {
        Self {
            thumb: ChildSlot::new(SliderThumb {
                state: SliderThumbState::Inactive,
                marker: std::marker::PhantomData::default(),
            }),
            value_changed: None,
            active: false,
        }
    }
}

impl<Model: ApplicationModel> Widget<Model> for Switch<Model> {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        // Boldly unwrapping here. If you have not given constraints to a slider then we don't know how big it should be.
        let my_size = Size::new(
            constraints.max_height().unwrap() * 2.0,
            constraints.max_height().unwrap(),
        );

        let thumb_size = self.thumb.layout(constraints, model);
        self.thumb.set_size(&thumb_size);
        let thumb_pos = if self.active {
            Point::new(my_size.width / 2.0, my_size.height / 2.0)
        } else {
            Point::new(0f32, my_size.height / 2.0)
        };
        self.thumb.set_position(&thumb_pos);
        my_size
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        let mut fill_paint = Paint::default();
        fill_paint.set_anti_alias(true);

        let rounding = 4f32;

        let rect = Rect::from_size(*rect);
        let mut fill_rect = Rect::from_wh(rect.width(), rect.height() / 4.0);
        fill_rect.offset(Point::new(0f32, rect.center_y() - fill_rect.center_y()));
        let mut unfill_rect = Rect::from_wh(rect.width(), rect.height() / 5.0);
        unfill_rect.offset(Point::new(0f32, rect.center_y() - unfill_rect.center_y()));

        fill_paint.set_color(theme.slider.fill);
        fill_paint.set_stroke(true);
        canvas.draw_rounded_rect(&unfill_rect, rounding, rounding, &fill_paint);

        fill_paint.set_alpha_f(0.25);
        fill_paint.set_stroke(false);
        canvas.draw_rounded_rect(&unfill_rect, rounding, rounding, &fill_paint);

        fill_paint.set_alpha_f(1.0);
        let mut fill_rect = Rect::from_wh(0f32, rect.height() / 4.0);
        fill_rect.offset(Point::new(0f32, rect.center_y() - fill_rect.center_y()));
        canvas.draw_rounded_rect(&fill_rect, rounding, rounding, &fill_paint);

        self.thumb.paint(theme, canvas, &rect.size(), model)
    }

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        app: &mut Application<Model>,
        model: &mut Model,
    ) {
        self.active = !self.active;
        let mut thumb_pos = *self.thumb.position();
        if self.active {
            thumb_pos.x = properties.size.width / 2.0
        } else {
            thumb_pos.x = 0f32
        }

        if let Some(l) = &mut self.value_changed {
            (l)(self.active, model);
        }

        self.thumb.set_position(&thumb_pos);

        self.thumb.mouse_down(event, properties, app, model)
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        self.thumb.mouse_up(event, app, model)
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {}

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {
        self.thumb.mouse_moved(event, model)
    }

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        self.thumb.mouse_entered(event, model)
    }

    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        self.thumb.mouse_left(event, model)
    }
}
