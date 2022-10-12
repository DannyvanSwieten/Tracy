use crate::application::Application;
use crate::application_model::ApplicationModel;
use crate::canvas_2d::Canvas2D;
use crate::constraints::BoxConstraints;
use crate::style::Theme;
use crate::window_event::MouseEvent;
use skia_safe::{Color4f, Paint, Point, Rect, Size};
use winit::event::KeyboardInput;

pub fn map_range(x: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    let slope = (d - c) / (b - a);
    c + slope * (x - a)
}

pub enum Action<Model> {
    None,
    Layout {
        nodes: Vec<&'static str>,
    },
    PopupRequest {
        request: PopupRequest<Model>,
        position: Point,
    },
    TriggerPopupMenu {
        menu: usize,
        sub_menu: usize,
    },
}

pub trait AppAction<Model> {
    fn undo(&self, _state: &mut Model);
    fn redo(&self, _state: &mut Model);
}

pub struct Properties {
    pub position: Point,
    pub size: Size,
}

#[allow(unused_variables)]
pub trait Widget<Model: ApplicationModel> {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size;
    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model);
    fn flex(&self) -> f32 {
        0f32
    }
    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        application: &mut Application<Model>,
        model: &mut Model,
    ) {
    }
    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model);
    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model);
    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model);
    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model);
    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model);

    fn keyboard_event(&mut self, event: &KeyboardInput, model: &mut Model) -> bool {
        false
    }
    fn character_received(&mut self, character: char, model: &mut Model) -> bool {
        false
    }
}

pub struct ChildSlot<Model> {
    position: Point,
    size: Size,
    widget: Box<dyn Widget<Model>>,
    has_mouse: bool,
}

impl<Model: ApplicationModel> ChildSlot<Model> {
    pub fn new(widget: impl Widget<Model> + 'static) -> Self {
        Self {
            position: Point::default(),
            size: Size::default(),
            widget: Box::new(widget),
            has_mouse: false,
        }
    }

    pub fn new_with_box(widget: Box<dyn Widget<Model>>) -> Self {
        Self {
            position: Point::default(),
            size: Size::default(),
            widget,
            has_mouse: false,
        }
    }

    pub fn set_size(&mut self, size: &Size) {
        self.size = *size
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn set_position(&mut self, position: &Point) {
        self.position = *position
    }

    pub fn position(&self) -> &Point {
        &self.position
    }

    pub fn hit_test(&mut self, point: &Point) -> bool {
        let x = point.x >= self.position.x && point.x < self.position.x + self.size.width;
        let y = point.y >= self.position.y && point.y < self.position.y + self.size.height;

        x && y
    }
}

impl<Model: ApplicationModel> Widget<Model> for ChildSlot<Model> {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        self.widget.layout(constraints, model)
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, _: &Size, model: &Model) {
        canvas.save();
        canvas.translate(self.position());
        self.widget.paint(theme, canvas, self.size(), model);
        canvas.restore();
    }

    fn flex(&self) -> f32 {
        self.widget.flex()
    }

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        _: &Properties,
        app: &mut Application<Model>,
        model: &mut Model,
    ) {
        if self.hit_test(event.local_position()) {
            let properties = Properties {
                position: *self.position(),
                size: *self.size(),
            };
            let new_event = event.to_local(self.position());
            self.widget.mouse_down(&new_event, &properties, app, model);
        }
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        if self.hit_test(event.local_position()) {
            let new_event = event.to_local(self.position());
            self.widget.mouse_up(&new_event, app, model);
        } else if self.has_mouse {
            self.has_mouse = false;
            let new_event = event.to_local(self.position());
            self.widget.mouse_left(&new_event, model);
        }
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {
        if self.hit_test(event.local_position()) {
            let new_event = event.to_local(self.position());
            self.widget.mouse_dragged(&new_event, properties, model);
        }
    }

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {
        if self.hit_test(event.local_position()) {
            let new_event = event.to_local(self.position());

            if !self.has_mouse {
                self.has_mouse = true;
                self.mouse_entered(event, model);
            }

            self.widget.mouse_moved(&new_event, model);
        } else {
            let new_event = event.to_local(self.position());
            if self.has_mouse {
                self.has_mouse = false;
                self.widget.mouse_left(event, model);
            }

            self.widget.mouse_moved(&new_event, model);
        }
    }

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        if self.hit_test(event.local_position()) {
            let new_event = event.to_local(self.position());
            self.widget.mouse_entered(&new_event, model)
        }
    }

    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        if self.hit_test(event.local_position()) {
            let new_event = event.to_local(self.position());
            self.widget.mouse_left(&new_event, model)
        }
    }

    fn keyboard_event(&mut self, event: &KeyboardInput, model: &mut Model) -> bool {
        self.widget.keyboard_event(event, model)
    }

    fn character_received(&mut self, character: char, model: &mut Model) -> bool {
        self.widget.character_received(character, model)
    }
}

pub struct Container<Model> {
    padding: f32,
    margin: f32,
    border: f32,
    width: Option<f32>,
    height: Option<f32>,
    child: ChildSlot<Model>,
    paint: Option<Paint>,
}

impl<Model: ApplicationModel> Container<Model> {
    pub fn new(child: impl Widget<Model> + 'static) -> Self {
        Self {
            padding: 0.0,
            margin: 0.0,
            border: 0.0,
            width: None,
            height: None,
            child: ChildSlot::new(child),
            paint: None,
        }
    }

    pub fn with_color(mut self, color: &Color4f) -> Self {
        self.paint = Some(Paint::new(*color, None));
        self
    }

    pub fn with_padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }
}

impl<Model: ApplicationModel> Widget<Model> for Container<Model> {
    // The container's layout strategy is to be as small as possible.
    // So shrink input constraints by border, padding and margin
    // Then return its child's size as its own size.

    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        // If the container is not given constraints from the parent check if we've been given a size
        // If not given a size we ask the child to layout without constraints
        // This might panic if the child is a flex container.
        // If given a size we ask the child to layout with that size.
        // This might still panic if the child is for example a horizontal container, but only height is given.

        // If the container is given constraints we'll shrink them by padding/margin and ask the child to layout with those constraints

        let space_around = self.padding + self.margin + self.border;
        let child_size = if constraints.max_width().is_none() || constraints.max_height().is_none()
        {
            if self.width.is_none() || self.height.is_none() {
                self.child.layout(&BoxConstraints::new(), model)
            } else {
                let mut child_constraints = BoxConstraints::new();
                if self.width.is_some() {
                    child_constraints = child_constraints.with_max_width(self.width.unwrap_or(0f32))
                }
                if self.height.is_some() {
                    child_constraints =
                        child_constraints.with_max_height(self.height.unwrap_or(0f32))
                }
                self.child
                    .layout(&child_constraints.shrunk(space_around, space_around), model)
            }
        } else {
            let child_constraints = constraints.shrunk(space_around * 2f32, space_around * 2f32);
            self.child.layout(&child_constraints, model)
        };

        self.child
            .set_position(&Point::new(space_around, space_around));
        self.child.set_size(&child_size);

        Size::new(
            (child_size.width + space_around * 2f32).max(constraints.min_width().unwrap_or(0f32)),
            (child_size.height + space_around * 2f32).max(constraints.min_height().unwrap_or(0f32)),
        )
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, size: &Size, model: &Model) {
        if let Some(paint) = &self.paint {
            canvas.draw_rect(&Rect::from_size(*size), paint);
        }

        self.child.paint(theme, canvas, self.child.size(), model);
    }

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        app: &mut Application<Model>,
        model: &mut Model,
    ) {
        self.child.mouse_down(event, properties, app, model);
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        self.child.mouse_up(event, app, model);
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {
        self.child.mouse_dragged(event, properties, model)
    }

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {
        self.child.mouse_moved(event, model)
    }

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        self.child.mouse_entered(event, model)
    }

    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        self.child.mouse_left(event, model)
    }

    fn keyboard_event(&mut self, event: &KeyboardInput, model: &mut Model) -> bool {
        self.child.keyboard_event(event, model)
    }

    fn flex(&self) -> f32 {
        0f32
    }

    fn character_received(&mut self, character: char, model: &mut Model) -> bool {
        self.child.character_received(character, model)
    }
}

pub struct Center<Model> {
    child: ChildSlot<Model>,
    size: Option<Size>,
}

impl<Model: ApplicationModel> Center<Model> {
    pub fn new<W: Widget<Model> + 'static>(child: W) -> Self {
        Self {
            child: ChildSlot::new_with_box(Box::new(child)),
            size: None,
        }
    }
}

impl<Model: ApplicationModel> Widget<Model> for Center<Model> {
    // The layout strategy for a center node: return own size if not None, otherwise as big as possible within given constraints.
    // Then center the child.
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        let my_size = if let Some(size) = &self.size {
            *size
        } else {
            // If not given a size then we need to have constraints from parent.
            Size::new(
                constraints.max_width().unwrap(),
                constraints.max_height().unwrap(),
            )
        };

        let child_size = self.child.layout(
            &BoxConstraints::new()
                .with_max_width(my_size.width)
                .with_max_height(my_size.height),
            model,
        );

        self.child.set_size(&child_size);

        let x_offset = (my_size.width - child_size.width) / 2f32;
        let y_offset = (my_size.height - child_size.height) / 2f32;
        self.child.set_position(&Point::new(x_offset, y_offset));

        my_size
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        self.child.paint(theme, canvas, rect, model)
    }

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        app: &mut Application<Model>,
        model: &mut Model,
    ) {
        self.child.mouse_down(event, properties, app, model)
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        self.child.mouse_up(event, app, model)
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {
        self.child.mouse_dragged(event, properties, model)
    }

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {
        self.child.mouse_moved(event, model)
    }

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        self.child.mouse_entered(event, model)
    }

    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        self.child.mouse_left(event, model)
    }

    fn keyboard_event(&mut self, event: &KeyboardInput, model: &mut Model) -> bool {
        self.child.keyboard_event(event, model)
    }
}

pub struct SizedBox<Model> {
    size: Size,
    child: ChildSlot<Model>,
}

impl<Model: ApplicationModel> SizedBox<Model> {
    pub fn new(size: Size, child: impl Widget<Model> + 'static) -> Self {
        Self {
            size,
            child: ChildSlot::new(child),
        }
    }
}

impl<Model: ApplicationModel> Widget<Model> for SizedBox<Model> {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        let child_constraints =
            BoxConstraints::new().with_tight_constraints(self.size.width, self.size.height);
        self.child.layout(&child_constraints, model);
        self.child.set_size(&self.size);
        self.size
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        self.child.paint(theme, canvas, rect, model);
    }

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        app: &mut Application<Model>,
        model: &mut Model,
    ) {
        self.child.mouse_down(event, properties, app, model)
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        self.child.mouse_up(event, app, model)
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {
        self.child.mouse_dragged(event, properties, model)
    }

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {
        self.child.mouse_moved(event, model)
    }

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        self.child.mouse_entered(event, model)
    }

    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        self.child.mouse_left(event, model)
    }

    fn keyboard_event(&mut self, event: &KeyboardInput, model: &mut Model) -> bool {
        self.child.keyboard_event(event, model)
    }
}

pub struct FlexBox<Model> {
    child: ChildSlot<Model>,
    flex: f32,
}

impl<Model: ApplicationModel> Widget<Model> for FlexBox<Model> {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        self.child.layout(constraints, model)
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        self.child.paint(theme, canvas, rect, model)
    }

    fn flex(&self) -> f32 {
        self.flex
    }

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        _: &mut Application<Model>,
        model: &mut Model,
    ) {
        todo!()
    }

    fn mouse_up(&mut self, event: &MouseEvent, _: &mut Application<Model>, model: &mut Model) {
        todo!()
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {
        todo!()
    }

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {
        todo!()
    }

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        todo!()
    }

    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        todo!()
    }

    fn keyboard_event(&mut self, event: &KeyboardInput, model: &mut Model) -> bool {
        self.child.keyboard_event(event, model)
    }
}

// pub struct Label {
//     text: String,
//     font: Font,
//     paint: Paint,
// }

// impl Label {
//     pub fn new(text: &str) -> Self {
//         Label {
//             text: String::from(text),
//             paint: Paint::default(),
//             font: Font::default(),
//         }
//     }
// }

// impl<Model: ApplicationModel> Widget<Model> for Label {
//     fn paint(&mut self, _: &Model, rect: &Rect, canvas: &mut dyn Canvas2D, style: &StyleSheet) {
//         assert_ne!(rect.width(), 0f32);
//         assert_ne!(rect.height(), 0f32);
//         self.paint.set_color(*style.get("bg-color").unwrap());
//         self.paint.set_anti_alias(true);
//         canvas.draw_rounded_rect(rect, 15., 15., &self.paint);
//         self.paint.set_color(Color::WHITE);
//         canvas.draw_string(&self.text, &rect.center(), &self.font, &self.paint);
//     }

//     fn calculate_size(
//         &self,
//         preferred_width: Option<f32>,
//         preferred_height: Option<f32>,
//         constraints: &Constraints,
//         children: &[Node<Model>],
//     ) -> (Size, Vec<Constraints>) {
//         let w = if let Some(preferred_width) = preferred_width {
//             let width = constraints.min_width;
//             let width = width.max(preferred_width).min(constraints.max_width);
//             width
//         } else {
//             constraints.max_width
//         };

//         let h = if let Some(preferred_height) = preferred_height {
//             let height = constraints.min_height;
//             let height = height.max(preferred_height).min(constraints.max_height);
//             height
//         } else {
//             constraints.max_height
//         };

//         (Size::new(w, h), vec![Constraints::new(0.0, w, 0.0, h)])
//     }
// }

// pub trait TableDelegate<Model> {
//     fn row_selected(&mut self, id: usize, state: &mut Model) -> Action<Model>;
//     fn row_count(&self, state: &Model) -> usize;
// }

// pub struct Table<Model> {
//     paint: Paint,
//     delegate: Box<dyn TableDelegate<Model>>,
// }

// impl<Model> Table<Model> {
//     pub fn new<D>(delegate: D) -> Self
//     where
//         D: TableDelegate<Model> + 'static,
//     {
//         Table {
//             paint: Paint::default(),
//             delegate: Box::new(delegate),
//         }
//     }
// }

// impl<Model: ApplicationModel> Widget<Model> for Table<Model> {
//     fn paint(&mut self, state: &Model, rect: &Rect, canvas: &mut dyn Canvas2D, style: &StyleSheet) {
//         assert_ne!(rect.width(), 0f32);
//         assert_ne!(rect.height(), 0f32);
//         let e_color = *style.get("even").unwrap_or(&Color::CYAN);
//         let u_color = *style.get("uneven").unwrap_or(&Color::RED);

//         let row_count = self.delegate.row_count(state);
//         let height = rect.height() / row_count as f32;

//         for i in 0..row_count {
//             if i % 2 == 0 {
//                 self.paint.set_color(e_color);
//             } else {
//                 self.paint.set_color(u_color);
//             }

//             canvas.draw_rounded_rect(
//                 &Rect::from_point_and_size(
//                     (rect.left(), rect.top() + i as f32 * height),
//                     (rect.width(), height),
//                 ),
//                 1.,
//                 1.,
//                 &self.paint,
//             )
//         }
//     }

//     fn mouse_up(&mut self, state: &mut Model, rect: &Rect, event: &MouseEvent) -> Action<Model> {
//         let row_count = self.delegate.row_count(state);
//         let y = event.global_position().y - rect.top();
//         let row_size = rect.height() / row_count as f32;
//         let row = y / row_size;

//         self.delegate.row_selected(row as usize, state)
//     }

//     fn layout(
//         &mut self,
//         _state: &Model,
//         _rect: &Rect,
//         _spacing: f32,
//         _padding: f32,
//         _children: &mut [Node<Model>],
//     ) {
//     }

//     fn calculate_size(
//         &self,
//         preferred_width: Option<f32>,
//         preferred_height: Option<f32>,
//         constraints: &Constraints,
//         _children: &[Node<Model>],
//     ) -> (Size, Vec<Constraints>) {
//         let w = if let Some(preferred_width) = preferred_width {
//             let width = constraints.min_width;
//             let width = width.max(preferred_width).min(constraints.max_width);
//             width
//         } else {
//             constraints.max_width
//         };

//         let h = if let Some(preferred_height) = preferred_height {
//             let height = constraints.min_height;
//             let height = height.max(preferred_height).min(constraints.max_height);
//             height
//         } else {
//             constraints.max_height
//         };

//         (Size::new(w, h), vec![Constraints::new(0.0, w, 0.0, h)])
//     }
// }

// // pub struct Spinner<Model> {
// //     label: String,
// //     border_paint: Paint,
// //     bg_paint: Paint,
// //     fill_paint: Paint,
// //     text_paint: Paint,
// //     font: Font,
// //     min: Option<f32>,
// //     max: Option<f32>,
// //     step_size: f32,
// //     discrete: bool,
// //     current_value: f32,
// //     value_changed: Option<Box<dyn FnMut(f32, &mut Model)>>,
// // }

// // impl<Model> Spinner<Model> {
// //     pub fn new(
// //         label: &str,
// //         min: Option<f32>,
// //         max: Option<f32>,
// //         current_value: f32,
// //         discrete: bool,
// //     ) -> Self {
// //         let mut s = Spinner {
// //             label: String::from(label),
// //             border_paint: Paint::default(),
// //             bg_paint: Paint::default(),
// //             fill_paint: Paint::default(),
// //             text_paint: Paint::default(),
// //             font: Font::default(),
// //             min,
// //             max,
// //             discrete,
// //             step_size: 0.1,
// //             current_value,
// //             value_changed: None,
// //         };

// //         if discrete {
// //             s.step_size = 1.;
// //         }

// //         s
// //     }

// //     pub fn with_handler<F>(mut self, handler: F) -> Self
// //     where
// //         F: FnMut(f32, &mut Model) + 'static,
// //     {
// //         self.value_changed = Some(Box::new(handler));
// //         self
// //     }
// // }

// // impl<Model> Widget<Model> for Spinner<Model> {
// //     fn paint(&mut self, model: &mut Model, rect: &Rect, canvas: &mut dyn Canvas2D, style: &StyleSheet) {
// //         let bg_color = style.get("bg-color");
// //         let fill_color = style.get("fill-color");
// //         let border_color = style.get("border-color");

// //         self.bg_paint
// //             .set_color(bg_color.unwrap_or(&Color::new(1., 0., 0., 1.)));
// //         self.border_paint
// //             .set_color(border_color.unwrap_or(&Color::new(1., 0., 0., 1.)));
// //         self.border_paint.set_style(PaintStyle::Stroke);
// //         self.fill_paint
// //             .set_color(fill_color.unwrap_or(&Color::new(0.2, 0.2, 0.2, 1.)));
// //         self.text_paint.set_color(&Color::new(1., 1., 1., 1.));
// //         canvas.draw_rounded_rect(
// //             rect.left(),
// //             rect.bottom(),
// //             rect.width(),
// //             rect.height(),
// //             2.,
// //             2.,
// //             &self.bg_paint,
// //         );

// //         let t = self.label.to_string() + ": " + &format!("{:.4}", &self.current_value.to_string());
// //         canvas.draw_text(
// //             &t,
// //             rect,
// //             &self.text_paint,
// //             &self.font,
// //         );
// //     }

// //     fn mouse_dragged(&mut self, state: &mut Model, _: &Rect, event: &MouseEvent) {
// //         self.current_value += -event.delta_position.y * self.step_size;

// //         if self.discrete {
// //             self.current_value = self.current_value.round();
// //         }

// //         if let Some(m) = self.min {
// //             self.current_value = self.current_value.max(m);
// //         }

// //         if let Some(m) = self.max {
// //             self.current_value = self.current_value.min(m);
// //         }
// //         if let Some(l) = &mut self.value_changed {
// //             (l)(self.current_value, state);
// //         }
// //     }
// // }

// pub struct ViewPort {
//     scroll_bar_rect: Rect,
//     orientation: Orientation,
//     paint: Paint,
//     scroll_bar_paint: Paint,
//     scroll_bar_position: f32,
//     scroll_bar_ratio: f32,
// }

// impl ViewPort {
//     pub fn new(orientation: Orientation) -> Self {
//         ViewPort {
//             scroll_bar_rect: Rect::default(),
//             orientation,
//             paint: Paint::default(),
//             scroll_bar_paint: Paint::default(),
//             scroll_bar_position: 0.,
//             scroll_bar_ratio: 0.,
//         }
//     }
// }

// // impl<Model> Widget<Model> for ViewPort {
// //     fn layout(
// //         &mut self,
// //         _state: &Model,
// //         rect: &Rect,
// //         _spacing: f32,
// //         _padding: f32,
// //         children: &mut [Node<Model>],
// //     ) {
// //         assert_eq!(1, children.len());

// //         self.scroll_bar_rect = *rect;
// //         children[0].rect.set_wh(rect.size().width, rect.size().height);
// //         children[0].rect.set_wh(children[0].constraints.size(&rect.size()));

// //         match self.orientation {
// //             Orientation::Horizontal => {
// //                 self.scroll_bar_rect.bottom = 15.;
// //                 self.scroll_bar_rect.width() = rect.width();
// //                 self.scroll_bar_rect.bottom += rect.height() - 15.;

// //                 children[0].rect.height() = rect.height() - self.scroll_bar_rect.height();

// //                 self.scroll_bar_ratio = (rect.width() / children[0].rect.width()).min(1.).max(0.);
// //             }
// //             Orientation::Vertical => {
// //                 self.scroll_bar_rect.width() = 15.;

// //                 children[0].rect.left = 15.;
// //                 children[0].rect.width() = rect.width() - self.scroll_bar_rect.width();

// //                 self.scroll_bar_ratio = (rect.height() / children[0].rect.height()).min(1.).max(0.);
// //             }
// //         }
// //     }

// //     fn paint(
// //         &mut self,
// //         _state: &Model,
// //         rect: &Rect,
// //         canvas: &mut dyn Canvas2D,
// //         _style: &StyleSheet,
// //     ) {
// //         // self.paint.set_color(&Color::from((0., 0., 0.));
// //         // canvas.draw_rect(rect, &self.paint);

// //         // self.scroll_bar_paint
// //         //     .set_color(&Color::new(0.3, 0.3, 0.3, 1.));
// //         // canvas.draw_rect(self.scroll_bar_rect, &self.scroll_bar_paint);

// //         // self.scroll_bar_paint
// //         //     .set_color(&Color::new(0.2, 0.2, 0.2, 1.));

// //         // let r = Rect::from_xywh(
// //         //     self.scroll_bar_rect.left() + 1. + self.scroll_bar_position,
// //         //     self.scroll_bar_rect.bottom() + 1.,
// //         //     self.scroll_bar_rect.width(),
// //         //     self.scroll_bar_rect.height() * self.scroll_bar_ratio,
// //         // );

// //         // canvas.draw_rect(r, &self.scroll_bar_paint);
// //     }
// // }

pub struct PopupMenu {
    id: usize,
    name: String,
    items: Vec<PopupMenu>,
}

struct PopupMenuWidget {
    active: bool,
    children: Vec<PopupMenuWidget>,
}

impl PopupMenuWidget {
    fn new(_request: PopupMenu) -> Self {
        PopupMenuWidget {
            active: true,
            children: Vec::new(),
        }
    }
}

impl<Model: ApplicationModel> Widget<Model> for PopupMenuWidget {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        todo!()
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, size: &Size, model: &Model) {
        todo!()
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        todo!()
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, properies: &Properties, model: &mut Model) {
        todo!()
    }

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {
        todo!()
    }

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        todo!()
    }

    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        todo!()
    }
}

impl PopupMenu {
    pub fn new(id: usize, name: &str) -> Self {
        PopupMenu {
            id,
            name: name.to_string(),
            items: Vec::new(),
        }
    }

    pub fn with_item(mut self, id: usize, name: &str) -> Self {
        self.items.push(PopupMenu::new(id, name));
        self
    }

    pub fn with_sub_menu(mut self, sub_menu: PopupMenu) -> Self {
        self.items.push(sub_menu);
        self
    }

    fn has_sub_menu_items(&self) -> bool {
        !self.items.is_empty()
    }
}

pub struct PopupRequest<Model> {
    menu: PopupMenu,
    pub handler: Box<dyn FnMut(usize, usize, &mut Model) -> Action<Model>>,
}

impl<Model: ApplicationModel + 'static> PopupRequest<Model> {
    pub fn new<F>(menu: PopupMenu, handler: F) -> Self
    where
        F: FnMut(usize, usize, &mut Model) -> Action<Model> + 'static,
    {
        PopupRequest {
            menu,
            handler: Box::new(handler),
        }
    }

    // pub fn build(&self) -> Box<dyn Widget<Model>> {
    //     let mut b = Node::new("menu").widget(VStack::new()).spacing(1.);

    //     for item in self.menu.items.iter() {
    //         let s = item.id;
    //         b.add_child(
    //             Node::new("btn")
    //                 .widget(Button::new(&item.name))
    //                 .with_mouse_event_callback(MouseEventType::MouseUp, move |_, _| {
    //                     Action::TriggerPopupMenu {
    //                         menu: 0,
    //                         sub_menu: s,
    //                     }
    //                 }),
    //         );
    //     }

    //     b.rect.set_wh(150., self.menu.items.len() as f32 * 28.);
    //     b
    // }
}
