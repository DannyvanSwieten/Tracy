use crate::node::*;
use crate::window::MouseEvent;
use crate::window::MouseEventType;
use skia_safe::canvas::Canvas;
use skia_safe::Color;
use skia_safe::Color4f;
use skia_safe::Font;
use skia_safe::Paint;
use skia_safe::PaintStyle;
use skia_safe::Point;
use skia_safe::Rect;
use skia_safe::Size;

use std::collections::HashMap;

pub fn map_range(x: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    let slope = (d - c) / (b - a);
    c + slope * (x - a)
}

pub type StyleSheet = HashMap<String, Color4f>;

#[derive(Clone)]
pub struct Material {
    data: HashMap<String, StyleSheet>,
}

impl Material {
    pub fn new() -> Self {
        let mut body = StyleSheet::new();
        body.insert(String::from("bg-color"), Color4f::new(0.0, 0.0, 0.0, 1.0));
        body.insert(
            String::from("border-color"),
            Color4f::new(0.0, 0.0, 0.0, 1.0),
        );

        let mut div = StyleSheet::new();
        div.insert(
            String::from("bg-color"),
            Color4f::new(0.08, 0.08, 0.08, 1.0),
        );
        div.insert(
            String::from("border-color"),
            Color4f::new(0.35, 0.35, 0.35, 1.),
        );

        let mut btn = StyleSheet::new();
        btn.insert(String::from("bg-color"), Color4f::new(0.2, 0.2, 0.2, 1.));
        btn.insert(String::from("hovered"), Color4f::new(0.35, 0.35, 0.35, 1.));
        btn.insert(String::from("pressed"), Color4f::new(0.3, 0.3, 0.3, 1.));

        let mut label = StyleSheet::new();
        label.insert(String::from("bg-color"), Color4f::new(0.15, 0.15, 0.15, 1.));
        label.insert(
            String::from("border-color"),
            Color4f::new(0.0, 0.0, 0.0, 0.0),
        );

        let mut slider = StyleSheet::new();
        slider.insert(String::from("bg-color"), Color4f::new(0.2, 0.2, 0.2, 1.0));
        slider.insert(String::from("fill-color"), Color4f::new(0.4, 0.4, 0.4, 1.0));
        slider.insert(
            String::from("border-color"),
            Color4f::new(0.35, 0.35, 0.35, 1.),
        );

        let mut menu = StyleSheet::new();
        menu.insert(String::from("bg-color"), Color4f::new(0.0, 0.0, 0.0, 1.0));
        menu.insert(
            String::from("border-color"),
            Color4f::new(0.0, 0.0, 0.0, 1.0),
        );

        let mut data = HashMap::new();
        data.insert(String::from("body"), body);
        data.insert(String::from("div"), div);
        data.insert(String::from("btn"), btn);
        data.insert(String::from("slider"), slider);
        data.insert(String::from("label"), label);
        data.insert(String::from("menu"), menu);

        Material { data: data }
    }

    pub fn create_child(&mut self, tag: &str) -> Option<&mut StyleSheet> {
        self.data.insert(tag.to_string(), StyleSheet::new());
        self.data.get_mut(tag)
    }

    pub fn get_child(&self, key: &str) -> Option<&StyleSheet> {
        self.data.get(key)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum UnitType {
    Relative,
    Absolute,
}

#[derive(Clone)]
pub struct Constraints {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub flex: f32,
    pub unit_type: UnitType,
}

impl Constraints {
    pub fn default() -> Self {
        let c = Constraints {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            flex: 1.0,
            unit_type: UnitType::Absolute,
        };
        c
    }

    pub fn new(
        min_width: Option<f32>,
        max_width: Option<f32>,
        min_height: Option<f32>,
        max_height: Option<f32>,
        flex: f32,
        unit_type: UnitType,
    ) -> Self {
        let c = Constraints {
            min_width,
            max_width,
            min_height,
            max_height,
            flex,
            unit_type,
        };
        c
    }

    pub fn is_height_constraint(&self) -> bool {
        match self.max_height {
            Some(_) => true,
            None => false,
        }
    }

    pub fn is_width_constraint(&self) -> bool {
        match self.max_width {
            Some(_) => true,
            None => false,
        }
    }

    pub fn size(&self, input: &Size) -> Size {
        let mut width_out = input.width;
        if let Some(mh) = self.max_width {
            if UnitType::Relative == self.unit_type {
                width_out = mh * 0.01 * input.width;
            } else {
                width_out = std::cmp::min(input.width as i32, mh as i32) as f32;
            }
        }

        let mut height_out = input.height;
        if let Some(mh) = self.max_height {
            if UnitType::Relative == self.unit_type {
                height_out = mh * 0.01 * input.height;
            } else {
                height_out = std::cmp::min(input.height as i32, mh as i32) as f32;
            }
        }

        if let Some(min_h) = self.min_height {
            height_out = height_out.max(min_h);
        }

        let s = Size::new(width_out, height_out);
        s
    }
}

pub enum Orientation {
    Horizontal,
    Vertical,
}

pub enum HorizontalJustification {
    Left,
    Center,
    Right,
}

pub enum VerticalJustification {
    Top,
    Center,
    Bottom,
}

pub enum Action<DataModel> {
    None,
    Layout {
        nodes: Vec<&'static str>,
    },
    PopupRequest {
        request: PopupRequest<DataModel>,
        position: Point,
    },
    TriggerPopupMenu {
        menu: usize,
        sub_menu: usize,
    },
}

pub trait AppAction<DataModel> {
    fn perform(&self, _state: &mut DataModel);
    fn undo(&self, _state: &mut DataModel);
    fn redo(&self, _state: &mut DataModel);
}

pub trait Widget<DataModel> {
    fn paint(
        &mut self,
        _state: &mut DataModel,
        _rect: &Rect,
        _canvas: &mut Canvas,
        _style: &StyleSheet,
    ) {
    }
    fn layout(
        &mut self,
        _state: &mut DataModel,
        _rect: &Rect,
        _spacing: f32,
        _padding: f32,
        _children: &mut [Node<DataModel>],
    ) {
    }
    fn mouse_down(&mut self, _state: &mut DataModel, _rect: &Rect, _event: &MouseEvent) {
        println!("Mouse Down");
    }
    fn mouse_up(
        &mut self,
        _state: &mut DataModel,
        _window_id: usize,
        _rect: &Rect,
        _event: &MouseEvent,
    ) -> Action<DataModel> {
        println!("Mouse Up");
        return Action::None;
    }
    fn double_click(
        &mut self,
        _state: &mut DataModel,
        _rect: &Rect,
        _event: &MouseEvent,
    ) -> Action<DataModel> {
        println!("Mouse Double Click");
        return Action::None;
    }
    fn mouse_drag(&mut self, _state: &mut DataModel, _rect: &Rect, _event: &MouseEvent) {
        println!("Mouse Drag");
    }
    fn mouse_moved(&mut self, _state: &mut DataModel, _rect: &Rect, _event: &MouseEvent) {
        println!("Mouse Moved");
    }
    fn mouse_enter(&mut self, _state: &mut DataModel, _rect: &Rect, _event: &MouseEvent) {
        println!("Mouse Enter");
    }
    fn mouse_leave(&mut self, _state: &mut DataModel, _rect: &Rect, _event: &MouseEvent) {
        println!("Mouse Leave");
    }
    fn needs_gpu(&self) -> bool {
        false
    }
}

#[derive(Default)]
pub struct Container {
    margin: f32,
    paint: Paint,
}

impl Container {
    pub fn new(margin: f32) -> Self {
        let c = Container {
            margin,
            paint: Paint::default(),
        };
        c
    }
}

impl<DataModel> Widget<DataModel> for Container {
    fn paint(&mut self, _: &mut DataModel, rect: &Rect, canvas: &mut Canvas, style: &StyleSheet) {
        self.paint.set_color(Color::WHITE);
        let rect = skia_safe::Rect::from_xywh(
            rect.left() + self.margin,
            rect.bottom() + self.margin,
            rect.width() - self.margin * 2.,
            rect.height() - self.margin * 2.,
        );

        canvas.draw_round_rect(rect, 5., 5., &self.paint);
    }
}

// pub struct Stack<DataModel> {
//     orientation: Orientation,
//     horizontal_justification: HorizontalJustification,
//     vertical_justification: VerticalJustification,
//     paint: Paint,
//     border_paint: Paint,
//     phantom: std::marker::PhantomData<DataModel>,
// }

// impl<DataModel> Stack<DataModel> {
//     pub fn new(orientation: Orientation) -> Self {
//         Stack {
//             orientation,
//             horizontal_justification: HorizontalJustification::Center,
//             vertical_justification: VerticalJustification::Center,
//             paint: Paint::default(),
//             border_paint: Paint::default(),
//             phantom: std::marker::PhantomData {},
//         }
//     }

//     pub fn layout_horizontally(
//         &self,
//         rect: &Rect,
//         children: &mut [Node<DataModel>],
//         spacing: f32,
//         padding: f32,
//     ) {
//         let mut available_width =
//             rect.width() - spacing * (children.len() as f32 - 1.) - (padding * 2.);

//         let padding_compensation = (padding * 2.) / children.len() as f32;

//         let mut unconstrained_children = children.len();
//         for child in children.iter_mut() {
//             if child.constraints.is_width_constraint() {
//                 let s = child.constraints.size(&rect.size());
//                 available_width -= s.width + padding_compensation;
//                 child.rect.size = s;
//                 child.rect.bottom -= padding;
//                 unconstrained_children = unconstrained_children - 1;
//             }
//         }

//         let child_width = available_width / unconstrained_children as f32;
//         let mut child_pos = rect.left() + padding;

//         for child in children.iter_mut() {
//             if !child.constraints.is_width_constraint() {
//                 child.rect.width() = child_width - padding_compensation;
//                 child.rect.height() = rect.height() - padding * 2.;
//             }

//             child.rect.left = child_pos;
//             child.rect.bottom = rect.bottom() + padding;
//             let s = child.rect.size();

//             // match self.vertical_justification {
//             //     VerticalJustification::Center => {
//             //         child.rect.bottom() += (rect.height() - self.margin * 2. - s.height) / 2.;
//             //     },
//             //     VerticalJustification::Bottom => {
//             //         child.rect.bottom() += rect.height() - s.height;
//             //     },
//             //     VerticalJustification::Top => {

//             //     }
//             // }

//             child_pos += s.width + spacing;
//         }
//     }

//     pub fn layout_vertically(
//         &self,
//         rect: &Rect,
//         children: &mut [Node<DataModel>],
//         spacing: f32,
//         padding: f32,
//     ) {
//         let mut available_height =
//             rect.height() - spacing * (children.len() as f32 - 1.) - (padding * 2.);
//         let mut unconstrained_children = children.len();

//         let padding_compensation = (padding * 2.) / children.len() as f32;

//         for child in children.iter_mut() {
//             if child.constraints.is_height_constraint() {
//                 let s = child.constraints.size(&rect.size());
//                 available_height -= s.height + padding_compensation;
//                 child.rect.size = s;
//                 child.rect.width() -= padding * 2.;
//                 unconstrained_children = unconstrained_children - 1;
//             }
//         }

//         let child_height = available_height / unconstrained_children as f32;
//         let mut child_pos = rect.bottom() + padding;

//         for child in children.iter_mut() {
//             if !child.constraints.is_height_constraint() {
//                 child.rect.height() = child_height - padding_compensation;
//                 child.rect.width() = rect.width() - padding * 2.;
//             }

//             child.rect.bottom = child_pos;
//             child.rect.left = rect.left() + padding;
//             child_pos += child.rect.height() + spacing;
//         }
//     }
// }

// impl<DataModel> Widget<DataModel> for Stack<DataModel> {
//     fn paint(&mut self, _: &mut DataModel, rect: &Rect, canvas: &mut Canvas, style: &StyleSheet) {
//         self.paint.set_color(style.get("bg-color").unwrap());
//         self.border_paint
//             .set_color(style.get("border-color").unwrap());
//         self.border_paint.set_style(PaintStyle::Stroke);
//         canvas.draw_rounded_rect(
//             rect.left(),
//             rect.bottom(),
//             rect.width(),
//             rect.height(),
//             0.,
//             0.,
//             &self.paint,
//         );

//         canvas.draw_rounded_rect(
//             rect.left(),
//             rect.bottom(),
//             rect.width(),
//             rect.height(),
//             0.,
//             0.,
//             &self.border_paint,
//         );
//     }

//     fn layout(
//         &mut self,
//         _: &mut DataModel,
//         rect: &Rect,
//         spacing: f32,
//         padding: f32,
//         children: &mut [Node<DataModel>],
//     ) {
//         match self.orientation {
//             Orientation::Horizontal => self.layout_horizontally(rect, children, spacing, padding),
//             Orientation::Vertical => self.layout_vertically(rect, children, spacing, padding),
//         }
//     }
// }

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
//             font: Font::new(12., "Arial"),
//         }
//     }
// }

// impl<DataModel> Widget<DataModel> for Label {
//     fn paint(&mut self, _: &mut DataModel, rect: &Rect, canvas: &mut Canvas, style: &StyleSheet) {
//         self.paint.set_color(style.get("bg-color").unwrap());
//         canvas.draw_rounded_rect(
//             rect.left(),
//             rect.bottom(),
//             rect.width(),
//             rect.height(),
//             1.,
//             1.,
//             &self.paint,
//         );
//         self.paint.set_color(&Color4f::new(1., 1., 1., 1.));
//         canvas.draw_text(
//             &self.text,
//             rect.left(),
//             rect.bottom(),
//             rect.width(),
//             rect.height(),
//             &self.paint,
//             &self.font,
//         );
//     }
// }

// pub struct Button<DataModel> {
//     pressed: bool,
//     hovered: bool,
//     text: String,
//     paint: Paint,
//     font: Font,
//     on_click: Option<Box<dyn FnMut(usize, &mut DataModel) -> Action<DataModel>>>,
//     phantom: std::marker::PhantomData<DataModel>,
// }

// impl<DataModel> Button<DataModel> {
//     pub fn new(text: &str) -> Self {
//         let b = Button {
//             pressed: false,
//             hovered: false,
//             text: String::from(text),
//             paint: Paint::default(),
//             font: Font::default(),
//             on_click: None,
//             phantom: std::marker::PhantomData,
//         };
//         b
//     }

//     pub fn get_text(&self) -> &str {
//         &self.text
//     }
// }

// impl<DataModel> Widget<DataModel> for Button<DataModel> {
//     fn mouse_down(&mut self, _: &mut DataModel, _: &Rect, _: &MouseEvent) {
//         self.pressed = true;
//     }

//     fn mouse_up(
//         &mut self,
//         state: &mut DataModel,
//         window_id: usize,
//         _: &Rect,
//         _: &MouseEvent,
//     ) -> Action<DataModel> {
//         self.pressed = false;
//         if let Some(handler) = self.on_click.as_mut() {
//             return (*handler)(window_id, state);
//         } else {
//             Action::None
//         }
//     }

//     fn mouse_enter(&mut self, _: &mut DataModel, _: &Rect, _: &MouseEvent) {
//         self.hovered = true;
//     }

//     fn mouse_leave(&mut self, _: &mut DataModel, _: &Rect, _: &MouseEvent) {
//         self.hovered = false;
//     }

//     fn paint(&mut self, _: &mut DataModel, rect: &Rect, canvas: &mut Canvas, style: &StyleSheet) {
//         if self.hovered {
//             if let Some(color) = style.get("hovered") {
//                 self.paint.set_color(color);
//             }
//         } else {
//             if let Some(color) = style.get("bg-color") {
//                 self.paint.set_color(color);
//             }
//         }

//         if self.pressed {
//             if let Some(color) = style.get("pressed") {
//                 self.paint.set_color(color);
//             }
//         }

//         canvas.draw_rounded_rect(
//             rect.left(),
//             rect.bottom(),
//             rect.width(),
//             rect.height(),
//             1.,
//             1.,
//             &self.paint,
//         );
//         self.paint.set_color(&Color4f::new(1., 1., 1., 1.));
//         canvas.draw_text(
//             &self.text,
//             rect,
//             &self.paint,
//             &self.font,
//         );
//     }
// }

// pub trait TableDelegate {
//     fn row_selected(&mut self, id: u32);
// }

// pub struct Table<'a> {
//     row_count: usize,
//     paint: Paint,
//     delegate: Option<&'a mut dyn TableDelegate>,
// }

// impl<'a> Table<'a> {
//     pub fn new(row_count: usize, delegate: Option<&'a mut dyn TableDelegate>) -> Self {
//         let w = Table {
//             row_count,
//             paint: Paint::default(),
//             delegate,
//         };
//         w
//     }
// }

// impl<'a, DataModel> Widget<DataModel> for Table<'a> {
//     fn paint(&mut self, _: &mut DataModel, rect: &Rect, canvas: &mut Canvas, style: &StyleSheet) {
//         let e_color = style.get("even").unwrap();
//         let u_color = style.get("uneven").unwrap();

//         let height = rect.height() / self.row_count as f32;

//         for i in 0..self.row_count {
//             if i % 2 == 0 {
//                 self.paint.set_color(e_color);
//             } else {
//                 self.paint.set_color(u_color);
//             }

//             canvas.draw_rounded_rect(
//                 rect.left(),
//                 rect.bottom() + height * i as f32,
//                 rect.width(),
//                 height,
//                 0.,
//                 0.,
//                 &self.paint,
//             )
//         }
//     }

//     fn mouse_down(&mut self, _: &mut DataModel, rect: &Rect, event: &MouseEvent) {
//         let y = (event.global_position.y - rect.bottom()) / self.row_count as f32;
//         let row_size = rect.height() / self.row_count as f32;
//         let row = y / row_size;

//         if let Some(d) = &mut self.delegate {
//             d.row_selected(row as u32)
//         }
//     }
// }

// pub struct Slider<DataModel> {
//     label: String,
//     border_paint: Paint,
//     bg_paint: Paint,
//     fill_paint: Paint,
//     text_paint: Paint,
//     font: Font,
//     min: f32,
//     max: f32,
//     discrete: bool,
//     current_normalized: f32,
//     current_value: f32,
//     last_position: f32,
//     value_changed: Option<Box<dyn FnMut(f32, &mut DataModel)>>,
// }

// impl<DataModel> Slider<DataModel> {
//     pub fn new(label: &str) -> Self {
//         Slider::new_with_min_max_and_value(label, 0., 1., 0., false)
//     }

//     pub fn new_with_min_max_and_value(
//         label: &str,
//         min: f32,
//         max: f32,
//         value: f32,
//         discrete: bool,
//     ) -> Self {
//         Slider {
//             label: label.to_string(),
//             bg_paint: Paint::default(),
//             fill_paint: Paint::default(),
//             text_paint: Paint::default(),
//             border_paint: Paint::default(),
//             font: Font::default(),
//             min,
//             max,
//             discrete,
//             current_normalized: value / (max - min),
//             current_value: value,
//             last_position: 0.,
//             value_changed: None,
//         }
//     }

//     pub fn with_handler<F>(mut self, handler: F) -> Self
//     where
//         F: FnMut(f32, &mut DataModel) + 'static,
//     {
//         self.value_changed = Some(Box::new(handler));
//         self
//     }

//     pub fn set_value(&mut self, value: f32) {
//         self.current_value = value.max(self.min).min(self.max);
//         self.current_normalized = map_range(self.current_value, self.min, self.max, 0., 1.)
//     }
// }

// impl<DataModel> Widget<DataModel> for Slider<DataModel> {
//     fn paint(&mut self, _: &mut DataModel, rect: &Rect, canvas: &mut Canvas, style: &StyleSheet) {
//         let bg_color = style.get("bg-color");
//         let fill_color = style.get("fill-color");
//         let border_color = style.get("border-color");

//         self.bg_paint
//             .set_color(bg_color.unwrap_or(&Color4f::new(1., 0., 0., 1.)));
//         self.border_paint
//             .set_color(border_color.unwrap_or(&Color4f::new(1., 0., 0., 1.)));
//         self.border_paint.set_style(PaintStyle::Stroke);
//         self.fill_paint
//             .set_color(fill_color.unwrap_or(&Color4f::new(0.2, 0.2, 0.2, 1.)));
//         self.text_paint.set_color(&Color4f::new(1., 1., 1., 1.));
//         canvas.draw_rounded_rect(
//             rect.left(),
//             rect.bottom(),
//             rect.width(),
//             rect.height(),
//             2.,
//             2.,
//             &self.bg_paint,
//         );
//         canvas.draw_rounded_rect(
//             rect.left(),
//             rect.bottom(),
//             rect.width(),
//             rect.height(),
//             2.,
//             2.,
//             &self.border_paint,
//         );
//         canvas.draw_rounded_rect(
//             rect.left(),
//             rect.bottom() + 2.,
//             (rect.width()) * self.current_normalized,
//             rect.height() - 4.,
//             0.,
//             0.,
//             &self.fill_paint,
//         );

//         let t = self.label.to_string() + ": " + &format!("{:.4}", &self.current_value.to_string());
//         canvas.draw_text(
//             &t,
//             rect,
//             &self.text_paint,
//             &self.font,
//         );
//     }

//     fn mouse_down(&mut self, state: &mut DataModel, rect: &Rect, event: &MouseEvent) {
//         let x = event.global_position.x - rect.left();
//         self.current_normalized = (1. / rect.width()) * x;

//         self.current_value = map_range(self.current_normalized, 0., 1., self.min, self.max);
//         if self.discrete {
//             self.current_value = self.current_value.round();
//         }
//         if let Some(l) = &mut self.value_changed {
//             (l)(self.current_value, state);
//         }

//         self.last_position = x;
//     }

//     fn mouse_drag(&mut self, state: &mut DataModel, rect: &Rect, event: &MouseEvent) {
//         self.last_position += event.delta_position.x;
//         self.current_normalized =
//             (1. / rect.width()) * self.last_position.min(rect.width()).max(0.);

//         self.current_value = map_range(self.current_normalized, 0., 1., self.min, self.max);

//         if self.discrete {
//             self.current_value = self.current_value.round();
//         }
//         if let Some(l) = &mut self.value_changed {
//             (l)(self.current_value, state);
//         }
//     }
// }

// pub struct Spinner<DataModel> {
//     label: String,
//     border_paint: Paint,
//     bg_paint: Paint,
//     fill_paint: Paint,
//     text_paint: Paint,
//     font: Font,
//     min: Option<f32>,
//     max: Option<f32>,
//     step_size: f32,
//     discrete: bool,
//     current_value: f32,
//     value_changed: Option<Box<dyn FnMut(f32, &mut DataModel)>>,
// }

// impl<DataModel> Spinner<DataModel> {
//     pub fn new(
//         label: &str,
//         min: Option<f32>,
//         max: Option<f32>,
//         current_value: f32,
//         discrete: bool,
//     ) -> Self {
//         let mut s = Spinner {
//             label: String::from(label),
//             border_paint: Paint::default(),
//             bg_paint: Paint::default(),
//             fill_paint: Paint::default(),
//             text_paint: Paint::default(),
//             font: Font::default(),
//             min,
//             max,
//             discrete,
//             step_size: 0.1,
//             current_value,
//             value_changed: None,
//         };

//         if discrete {
//             s.step_size = 1.;
//         }

//         s
//     }

//     pub fn with_handler<F>(mut self, handler: F) -> Self
//     where
//         F: FnMut(f32, &mut DataModel) + 'static,
//     {
//         self.value_changed = Some(Box::new(handler));
//         self
//     }
// }

// impl<DataModel> Widget<DataModel> for Spinner<DataModel> {
//     fn paint(&mut self, _: &mut DataModel, rect: &Rect, canvas: &mut Canvas, style: &StyleSheet) {
//         let bg_color = style.get("bg-color");
//         let fill_color = style.get("fill-color");
//         let border_color = style.get("border-color");

//         self.bg_paint
//             .set_color(bg_color.unwrap_or(&Color4f::new(1., 0., 0., 1.)));
//         self.border_paint
//             .set_color(border_color.unwrap_or(&Color4f::new(1., 0., 0., 1.)));
//         self.border_paint.set_style(PaintStyle::Stroke);
//         self.fill_paint
//             .set_color(fill_color.unwrap_or(&Color4f::new(0.2, 0.2, 0.2, 1.)));
//         self.text_paint.set_color(&Color4f::new(1., 1., 1., 1.));
//         canvas.draw_rounded_rect(
//             rect.left(),
//             rect.bottom(),
//             rect.width(),
//             rect.height(),
//             2.,
//             2.,
//             &self.bg_paint,
//         );

//         let t = self.label.to_string() + ": " + &format!("{:.4}", &self.current_value.to_string());
//         canvas.draw_text(
//             &t,
//             rect,
//             &self.text_paint,
//             &self.font,
//         );
//     }

//     fn mouse_drag(&mut self, state: &mut DataModel, _: &Rect, event: &MouseEvent) {
//         self.current_value += -event.delta_position.y * self.step_size;

//         if self.discrete {
//             self.current_value = self.current_value.round();
//         }

//         if let Some(m) = self.min {
//             self.current_value = self.current_value.max(m);
//         }

//         if let Some(m) = self.max {
//             self.current_value = self.current_value.min(m);
//         }
//         if let Some(l) = &mut self.value_changed {
//             (l)(self.current_value, state);
//         }
//     }
// }

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
//             paint: Paint::new(),
//             scroll_bar_paint: Paint::new(),
//             scroll_bar_position: 0.,
//             scroll_bar_ratio: 0.,
//         }
//     }
// }

// impl<DataModel> Widget<DataModel> for ViewPort {
//     fn layout(
//         &mut self,
//         _state: &mut DataModel,
//         rect: &Rect,
//         _spacing: f32,
//         _padding: f32,
//         children: &mut [Node<DataModel>],
//     ) {
//         assert_eq!(1, children.len());

//         self.scroll_bar_rect = *rect;
//         children[0].rect.set_wh(rect.size().width, rect.size.height);
//         children[0].rect.size = children[0].constraints.size(&rect.size);

//         match self.orientation {
//             Orientation::Horizontal => {
//                 self.scroll_bar_rect.bottom = 15.;
//                 self.scroll_bar_rect.width() = rect.width();
//                 self.scroll_bar_rect.bottom += rect.height() - 15.;

//                 children[0].rect.height() = rect.height() - self.scroll_bar_rect.height();

//                 self.scroll_bar_ratio = (rect.width() / children[0].rect.width()).min(1.).max(0.);
//             }
//             Orientation::Vertical => {
//                 self.scroll_bar_rect.width() = 15.;

//                 children[0].rect.left = 15.;
//                 children[0].rect.width() = rect.width() - self.scroll_bar_rect.width();

//                 self.scroll_bar_ratio = (rect.height() / children[0].rect.height()).min(1.).max(0.);
//             }
//         }
//     }

//     fn paint(
//         &mut self,
//         _state: &mut DataModel,
//         rect: &Rect,
//         canvas: &mut Canvas,
//         _style: &StyleSheet,
//     ) {
//         self.paint.set_color(&Color4f::from((0., 0., 0.));
//         canvas.draw_rect(rect, &self.paint);

//         self.scroll_bar_paint
//             .set_color(&Color4f::new(0.3, 0.3, 0.3, 1.));
//         canvas.draw_rect(self.scroll_bar_rect, &self.scroll_bar_paint);

//         self.scroll_bar_paint
//             .set_color(&Color4f::new(0.2, 0.2, 0.2, 1.));

//         let r = Rect::from_xywh(
//             self.scroll_bar_rect.left() + 1. + self.scroll_bar_position,
//             self.scroll_bar_rect.bottom() + 1.,
//             self.scroll_bar_rect.width(),
//             self.scroll_bar_rect.height() * self.scroll_bar_ratio,
//         );

//         canvas.draw_rect(r, &self.scroll_bar_paint);
//     }
// }

pub struct PopupMenu {
    id: usize,
    name: String,
    items: Vec<PopupMenu>,
}

struct PopupMenuWidget {
    active: bool,
    children: Vec<Box<PopupMenuWidget>>,
}

impl PopupMenuWidget {
    fn new(request: PopupMenu) -> Self {
        PopupMenuWidget {
            active: true,
            children: Vec::new(),
        }
    }
}

impl<DataModel> Widget<DataModel> for PopupMenuWidget {}

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
        self.items.len() != 0
    }
}

pub struct PopupRequest<DataModel> {
    menu: PopupMenu,
    pub handler: Box<dyn FnMut(usize, usize, &mut DataModel) -> Action<DataModel>>,
}

impl<DataModel: 'static> PopupRequest<DataModel> {
    pub fn new<F>(menu: PopupMenu, handler: F) -> Self
    where
        F: FnMut(usize, usize, &mut DataModel) -> Action<DataModel> + 'static,
    {
        PopupRequest {
            menu,
            handler: Box::new(handler),
        }
    }

    pub fn build(&self) -> Node<DataModel> {
        let mut b = Node::new("menu");
        // .with_widget(Stack::new(Orientation::Vertical))
        // .with_spacing(1.);

        for item in self.menu.items.iter() {
            let s = item.id;
            // b.add_child(
            //     Node::new("btn")
            //         .with_widget(Button::new(&item.name))
            //         .with_event_callback(MouseEventType::MouseUp, move |_, _| {
            //             Action::TriggerPopupMenu {
            //                 menu: 0,
            //                 sub_menu: s,
            //             }
            //         }),
            // );
        }

        b.rect.set_wh(150., self.menu.items.len() as f32 * 28.);
        b
    }
}
