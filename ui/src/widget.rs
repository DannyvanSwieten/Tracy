use crate::canvas_2d::Canvas2D;
use crate::node::*;
use crate::window_event::{MouseEvent, MouseEventType};
use skia_safe::{Color, Font, Paint, PaintStyle, Point, Rect, Size};

use std::collections::HashMap;

pub fn map_range(x: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    let slope = (d - c) / (b - a);
    c + slope * (x - a)
}

pub type StyleSheet = HashMap<String, Color>;

#[derive(Clone)]
pub struct Material {
    data: HashMap<String, StyleSheet>,
}

impl Material {
    pub fn new() -> Self {
        let mut body = StyleSheet::new();
        body.insert(String::from("bg-color"), Color::BLACK);

        body.insert(String::from("border-color"), Color::BLACK);

        let mut div = StyleSheet::new();
        div.insert(String::from("bg-color"), Color::DARK_GRAY);
        div.insert(String::from("border-color"), Color::LIGHT_GRAY);

        let mut btn = StyleSheet::new();
        btn.insert(String::from("bg-color"), Color::from_rgb(51, 51, 51));
        btn.insert(String::from("hovered"), Color::from_rgb(89, 89, 89));
        btn.insert(String::from("pressed"), Color::from_rgb(77, 77, 77));

        let mut label = StyleSheet::new();
        label.insert(String::from("bg-color"), Color::from_rgb(38, 38, 38));
        label.insert(String::from("border-color"), Color::TRANSPARENT);

        let mut slider = StyleSheet::new();
        slider.insert(String::from("bg-color"), Color::from_rgb(51, 51, 51));
        slider.insert(String::from("fill-color"), Color::from_rgb(102, 102, 102));
        slider.insert(String::from("border-color"), Color::from_rgb(89, 89, 89));

        let mut menu = StyleSheet::new();
        menu.insert(String::from("bg-color"), Color::BLACK);
        menu.insert(String::from("border-color"), Color::BLACK);

        let mut table = StyleSheet::new();
        table.insert(String::from("bg-color"), Color::BLACK);
        table.insert(String::from("even"), Color::from_rgb(89, 89, 89));
        table.insert(String::from("uneven"), Color::from_rgb(77, 77, 77));

        let mut data = HashMap::new();
        data.insert(String::from("body"), body);
        data.insert(String::from("div"), div);
        data.insert(String::from("btn"), btn);
        data.insert(String::from("slider"), slider);
        data.insert(String::from("label"), label);
        data.insert(String::from("menu"), menu);
        data.insert(String::from("table"), table);

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
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Constraints {
    pub fn default() -> Self {
        Constraints {
            min_width: 0.0,
            max_width: 0.0,
            min_height: 0.0,
            max_height: 0.0,
        }
    }

    pub fn new(min_width: f32, max_width: f32, min_height: f32, max_height: f32) -> Self {
        Constraints {
            min_width,
            max_width,
            min_height,
            max_height,
        }
    }

    pub fn is_height_constraint(&self) -> bool {
        self.min_height == self.max_height
    }

    pub fn is_width_constraint(&self) -> bool {
        self.min_width == self.max_width
    }

    pub fn size(&self, input: &Size) -> Size {
        *input
    }
}

pub enum Orientation {
    Horizontal,
    Vertical,
}

pub enum HorizontalAlignment {
    Leading,
    Center,
    Trailing,
}

pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

pub enum Action<AppState> {
    None,
    Layout {
        nodes: Vec<&'static str>,
    },
    PopupRequest {
        request: PopupRequest<AppState>,
        position: Point,
    },
    TriggerPopupMenu {
        menu: usize,
        sub_menu: usize,
    },
}

pub trait AppAction<AppState> {
    fn undo(&self, _state: &mut AppState);
    fn redo(&self, _state: &mut AppState);
}

pub trait Widget<AppState> {
    fn paint(
        &mut self,
        _state: &AppState,
        _rect: &Rect,
        _canvas: &mut dyn Canvas2D,
        _style: &StyleSheet,
    ) {
    }
    fn paint_3d(&mut self, _state: &AppState, _rect: &Rect) {}

    fn resized(&mut self, _state: &mut AppState, _rect: &Rect) {}

    fn calculate_size(
        &self,
        preferred_width: Option<f32>,
        preferred_height: Option<f32>,
        constraints: &Constraints,
        _children: &[Node<AppState>],
    ) -> (Size, Vec<Constraints>) {
        let w = if let Some(preferred_width) = preferred_width {
            let width = constraints.min_width;
            let width = width.max(preferred_width).min(constraints.max_width);
            width
        } else {
            constraints.max_width
        };

        let h = if let Some(preferred_height) = preferred_height {
            let height = constraints.min_height;
            let height = height.max(preferred_height).min(constraints.max_height);
            height
        } else {
            constraints.max_height
        };

        (Size::new(w, h), vec![Constraints::new(0.0, w, 0.0, h)])
    }

    fn layout(
        &mut self,
        _state: &AppState,
        _rect: &Rect,
        _spacing: f32,
        _padding: f32,
        _children: &mut [Node<AppState>],
    ) {
    }
    fn mouse_down(&mut self, _state: &mut AppState, _rect: &Rect, _event: &MouseEvent) {}
    fn mouse_up(
        &mut self,
        _state: &mut AppState,
        _rect: &Rect,
        _event: &MouseEvent,
    ) -> Action<AppState> {
        return Action::None;
    }
    fn double_click(
        &mut self,
        _state: &mut AppState,
        _rect: &Rect,
        _event: &MouseEvent,
    ) -> Action<AppState> {
        return Action::None;
    }
    fn mouse_drag(&mut self, _state: &mut AppState, _rect: &Rect, _event: &MouseEvent) {}
    fn mouse_moved(&mut self, _state: &mut AppState, _rect: &Rect, _event: &MouseEvent) {}
    fn mouse_enter(&mut self, _state: &mut AppState, _rect: &Rect, _event: &MouseEvent) {}
    fn mouse_leave(&mut self, _state: &mut AppState, _rect: &Rect, _event: &MouseEvent) {}
}

#[derive(Default)]
pub struct Container {
    paint: Paint,
}

impl Container {
    pub fn new() -> Self {
        let mut c = Container {
            paint: Paint::default(),
        };
        c.paint.set_anti_alias(true);
        c
    }
}

impl<AppState> Widget<AppState> for Container {
    fn paint(&mut self, _: &AppState, rect: &Rect, canvas: &mut dyn Canvas2D, style: &StyleSheet) {
        assert_ne!(rect.width(), 0f32);
        assert_ne!(rect.height(), 0f32);
        let bg = style.get("bg-color").unwrap_or(&Color::RED);
        self.paint.set_color(*bg);
        canvas.draw_rounded_rect(rect, 5., 5., &self.paint);
    }

    fn layout(
        &mut self,
        _state: &AppState,
        rect: &Rect,
        _spacing: f32,
        _padding: f32,
        children: &mut [Node<AppState>],
    ) {
        assert_eq!(children.len() < 2, true);
        if children.len() != 0 {
            let x = (rect.size().width - children[0].rect.width()) / 2.0;
            let y = (rect.size().height - children[0].rect.height()) / 2.0;
            children[0].rect.offset((x, y))
        }
    }

    fn calculate_size(
        &self,
        preferred_width: Option<f32>,
        preferred_height: Option<f32>,
        constraints: &Constraints,
        children: &[Node<AppState>],
    ) -> (Size, Vec<Constraints>) {
        assert_eq!(children.len() < 2, true);
        let w = if let Some(preferred_width) = preferred_width {
            let width = constraints.min_width;
            let width = width.max(preferred_width).min(constraints.max_width);
            width
        } else {
            constraints.max_width
        };

        let h = if let Some(preferred_height) = preferred_height {
            let height = constraints.min_height;
            let height = height.max(preferred_height).min(constraints.max_height);
            height
        } else {
            constraints.max_height
        };

        (Size::new(w, h), vec![Constraints::new(0.0, w, 0.0, h)])
    }
}

pub struct HStack<AppState> {
    vertical_alignment: VerticalAlignment,
    paint: Paint,
    border_paint: Paint,
    phantom: std::marker::PhantomData<AppState>,
}

impl<AppState> HStack<AppState> {
    pub fn new() -> Self {
        HStack {
            vertical_alignment: VerticalAlignment::Center,
            paint: Paint::default(),
            border_paint: Paint::default(),
            phantom: std::marker::PhantomData {},
        }
    }

    pub fn layout_horizontally(
        &self,
        rect: &Rect,
        children: &mut [Node<AppState>],
        spacing: f32,
        padding: f32,
    ) {
        let total_spacing = spacing * (children.len() as f32 - 1.);
        let total_padding = padding * 2.;
        //let mut available_width = rect.width() - total_spacing - total_padding;

        //let padding_compensation = (padding * 2.) / children.len() as f32;

        //let mut unconstrained_children = children.len();
        // for child in children.iter_mut() {
        //     if child.constraints.is_width_constraint() {
        //         let s = child.constraints.size(&rect.size());
        //         available_width -= s.width + padding_compensation;
        //         child.rect.set_xywh(rect.left, rect.top, s.width, s.height);
        //         // child.rect.inset((0., padding));
        //         unconstrained_children = unconstrained_children - 1;
        //     }
        // }

        //let child_width = available_width / unconstrained_children as f32;
        let mut child_pos = rect.left() + padding;

        for child in children.iter_mut() {
            // if !child.constraints.is_width_constraint() {
            //     let w = child_width; // - padding_compensation;
            //     let h = rect.height() - total_padding;
            //     child.rect.set_xywh(rect.left, rect.top, w, h);
            // }

            child.rect.offset((child_pos, padding + rect.top()));
            child_pos += child.rect.width() + spacing;
        }
    }
}

pub struct VStack<AppState> {
    horizontal_alignment: HorizontalAlignment,
    paint: Paint,
    border_paint: Paint,
    phantom: std::marker::PhantomData<AppState>,
}

impl<AppState> VStack<AppState> {
    pub fn new() -> Self {
        VStack {
            horizontal_alignment: HorizontalAlignment::Center,
            paint: Paint::default(),
            border_paint: Paint::default(),
            phantom: std::marker::PhantomData {},
        }
    }

    pub fn layout_vertically(
        &self,
        rect: &Rect,
        children: &mut [Node<AppState>],
        spacing: f32,
        padding: f32,
    ) {
        // let mut available_height =
        //     rect.height() - spacing * (children.len() as f32 - 1.) - (padding * 2.);
        // let mut unconstrained_children = children.len();

        // let padding_compensation = (padding * 2.) / children.len() as f32;

        // for child in children.iter_mut() {
        //     if child.constraints.is_height_constraint() {
        //         let s = child.constraints.size(&rect.size());
        //         available_height -= s.height + padding_compensation;
        //         child.rect.set_wh(s.width - padding * 2., s.height);
        //         unconstrained_children = unconstrained_children - 1;
        //     }
        // }

        // let child_height = available_height / unconstrained_children as f32;
        let mut child_pos = rect.top() + padding;

        for child in children.iter_mut() {
            // if !child.constraints.is_height_constraint() {
            //     let h = child_height - padding_compensation;
            //     let w = rect.width() - padding * 2.;
            //     child.rect.set_wh(w, h);
            // }

            child
                .rect
                .offset((rect.left() + padding, child_pos + rect.top()));
            child_pos += child.rect.height() + spacing;
        }
    }
}

impl<AppState> Widget<AppState> for HStack<AppState> {
    fn paint(&mut self, _: &AppState, rect: &Rect, canvas: &mut dyn Canvas2D, style: &StyleSheet) {
        assert_ne!(rect.width(), 0f32);
        assert_ne!(rect.height(), 0f32);

        self.paint.set_anti_alias(true);
        self.paint.set_color(*style.get("bg-color").unwrap());
        self.border_paint
            .set_color(*style.get("border-color").unwrap());
        self.border_paint.set_style(PaintStyle::Stroke);
        canvas.draw_rounded_rect(rect, 1., 1., &self.paint);

        canvas.draw_rounded_rect(rect, 1., 1., &self.border_paint);
    }

    fn calculate_size(
        &self,
        preferred_width: Option<f32>,
        preferred_height: Option<f32>,
        constraints: &Constraints,
        children: &[Node<AppState>],
    ) -> (Size, Vec<Constraints>) {
        let w = if let Some(preferred_width) = preferred_width {
            let width = constraints.min_width;
            let width = width.max(preferred_width).min(constraints.max_width);
            width
        } else {
            constraints.max_width
        };

        let h = if let Some(preferred_height) = preferred_height {
            let height = constraints.min_height;
            let height = height.max(preferred_height).min(constraints.max_height);
            height
        } else {
            constraints.max_height
        };

        let remaining_space = w - children.iter().fold(0.0, |acc, child| {
            acc + child.preferred_width().unwrap_or(0.)
        });

        let width_per_flex = remaining_space
            / children
                .iter()
                .fold(0.0, |acc, child| acc + child.flex().unwrap_or(0.));

        let mut children_constraints = Vec::new();
        for child in children {
            if let Some(flex) = child.flex() {
                children_constraints.push(Constraints::new(
                    width_per_flex * flex,
                    width_per_flex * flex,
                    0.0,
                    constraints.max_height,
                ))
            } else {
                children_constraints.push(Constraints::new(
                    0.0,
                    child.preferred_width().unwrap_or(constraints.max_width),
                    0.0,
                    child.preferred_height().unwrap_or(constraints.max_height),
                ))
            }
        }

        (Size::new(w, h), children_constraints)
    }

    fn layout(
        &mut self,
        _: &AppState,
        rect: &Rect,
        spacing: f32,
        padding: f32,
        children: &mut [Node<AppState>],
    ) {
        self.layout_horizontally(rect, children, spacing, padding)
    }
}

impl<AppState> Widget<AppState> for VStack<AppState> {
    fn paint(&mut self, _: &AppState, rect: &Rect, canvas: &mut dyn Canvas2D, style: &StyleSheet) {
        assert_ne!(rect.width(), 0f32);
        assert_ne!(rect.height(), 0f32);
        self.paint.set_anti_alias(true);
        self.paint.set_color(*style.get("bg-color").unwrap());
        self.border_paint
            .set_color(*style.get("border-color").unwrap());
        self.border_paint.set_style(PaintStyle::Stroke);
        canvas.draw_rounded_rect(rect, 1., 1., &self.paint);

        canvas.draw_rounded_rect(rect, 1., 1., &self.border_paint);
    }

    fn calculate_size(
        &self,
        preferred_width: Option<f32>,
        preferred_height: Option<f32>,
        constraints: &Constraints,
        children: &[Node<AppState>],
    ) -> (Size, Vec<Constraints>) {
        let w = if let Some(preferred_width) = preferred_width {
            let width = constraints.min_width;
            let width = width.max(preferred_width).min(constraints.max_width);
            width
        } else {
            constraints.max_width
        };

        let h = if let Some(preferred_height) = preferred_height {
            let height = constraints.min_height;
            let height = height.max(preferred_height).min(constraints.max_height);
            height
        } else {
            constraints.max_height
        };

        let remaining_space = h - children.iter().fold(0.0, |acc, child| {
            acc + child.preferred_height().unwrap_or(0.)
        });

        let height_per_flex = remaining_space
            / children
                .iter()
                .fold(0.0, |acc, child| acc + child.flex().unwrap_or(0.));

        let mut children_constraints = Vec::new();
        for child in children {
            if let Some(flex) = child.flex() {
                children_constraints.push(Constraints::new(
                    0.0,
                    child.preferred_width().unwrap_or(w),
                    height_per_flex * flex,
                    height_per_flex * flex,
                ))
            } else {
                children_constraints.push(Constraints::new(
                    0.0,
                    child.preferred_width().unwrap_or(w),
                    0.0,
                    child.preferred_height().unwrap_or(h),
                ))
            }
        }

        (Size::new(w, h), children_constraints)
    }

    fn layout(
        &mut self,
        _: &AppState,
        rect: &Rect,
        spacing: f32,
        padding: f32,
        children: &mut [Node<AppState>],
    ) {
        self.layout_vertically(rect, children, spacing, padding)
    }
}

pub struct Label {
    text: String,
    font: Font,
    paint: Paint,
}

impl Label {
    pub fn new(text: &str) -> Self {
        Label {
            text: String::from(text),
            paint: Paint::default(),
            font: Font::default(),
        }
    }
}

impl<AppState> Widget<AppState> for Label {
    fn paint(&mut self, _: &AppState, rect: &Rect, canvas: &mut dyn Canvas2D, style: &StyleSheet) {
        assert_ne!(rect.width(), 0f32);
        assert_ne!(rect.height(), 0f32);
        self.paint.set_color(*style.get("bg-color").unwrap());
        self.paint.set_anti_alias(true);
        canvas.draw_rounded_rect(rect, 15., 15., &self.paint);
        self.paint.set_color(Color::WHITE);
        canvas.draw_string(&self.text, &rect.center(), &self.font, &self.paint);
    }

    fn calculate_size(
        &self,
        preferred_width: Option<f32>,
        preferred_height: Option<f32>,
        constraints: &Constraints,
        children: &[Node<AppState>],
    ) -> (Size, Vec<Constraints>) {
        let w = if let Some(preferred_width) = preferred_width {
            let width = constraints.min_width;
            let width = width.max(preferred_width).min(constraints.max_width);
            width
        } else {
            constraints.max_width
        };

        let h = if let Some(preferred_height) = preferred_height {
            let height = constraints.min_height;
            let height = height.max(preferred_height).min(constraints.max_height);
            height
        } else {
            constraints.max_height
        };

        (Size::new(w, h), vec![Constraints::new(0.0, w, 0.0, h)])
    }
}

pub struct Button<AppState> {
    pressed: bool,
    hovered: bool,
    text: String,
    paint: Paint,
    font: Font,
    on_click: Option<Box<dyn FnMut(&mut AppState) -> Action<AppState>>>,
    phantom: std::marker::PhantomData<AppState>,
}

impl<AppState> Button<AppState> {
    pub fn new(text: &str) -> Self {
        let b = Button {
            pressed: false,
            hovered: false,
            text: String::from(text),
            paint: Paint::default(),
            font: Font::default(),
            on_click: None,
            phantom: std::marker::PhantomData,
        };
        b
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }
}

impl<AppState> Widget<AppState> for Button<AppState> {
    fn calculate_size(
        &self,
        preferred_width: Option<f32>,
        preferred_height: Option<f32>,
        constraints: &Constraints,
        _children: &[Node<AppState>],
    ) -> (Size, Vec<Constraints>) {
        let w = if let Some(preferred_width) = preferred_width {
            let width = constraints.min_width;
            let width = width.max(preferred_width).min(constraints.max_width);
            width
        } else {
            constraints.max_width
        };

        let h = if let Some(preferred_height) = preferred_height {
            let height = constraints.min_height;
            let height = height.max(preferred_height).min(constraints.max_height);
            height
        } else {
            constraints.max_height
        };

        (Size::new(w, h), vec![Constraints::new(0.0, w, 0.0, h)])
    }

    fn mouse_down(&mut self, _: &mut AppState, _: &Rect, _: &MouseEvent) {
        self.pressed = true;
    }

    fn mouse_up(&mut self, state: &mut AppState, _: &Rect, _: &MouseEvent) -> Action<AppState> {
        self.pressed = false;
        if let Some(handler) = self.on_click.as_mut() {
            return (*handler)(state);
        } else {
            Action::None
        }
    }

    fn mouse_enter(&mut self, _: &mut AppState, _: &Rect, _: &MouseEvent) {
        self.hovered = true;
    }

    fn mouse_leave(&mut self, _: &mut AppState, _: &Rect, _: &MouseEvent) {
        self.hovered = false;
    }

    fn paint(&mut self, _: &AppState, rect: &Rect, canvas: &mut dyn Canvas2D, style: &StyleSheet) {
        assert_ne!(rect.width(), 0f32);
        assert_ne!(rect.height(), 0f32);
        self.paint.set_anti_alias(true);
        if self.hovered {
            if let Some(color) = style.get("hovered") {
                self.paint.set_color(*color);
            }
        } else {
            if let Some(color) = style.get("bg-color") {
                self.paint.set_color(*color);
            }
        }

        if self.pressed {
            if let Some(color) = style.get("pressed") {
                self.paint.set_color(*color);
            }
        }

        canvas.draw_rounded_rect(rect, 1., 1., &self.paint);
        self.paint.set_color(Color::WHITE);
        canvas.draw_string(&self.text, &rect.center(), &self.font, &self.paint);
    }
}

pub trait TableDelegate<AppState> {
    fn row_selected(&mut self, id: usize, state: &mut AppState) -> Action<AppState>;
    fn row_count(&self, state: &AppState) -> usize;
}

pub struct Table<AppState> {
    paint: Paint,
    delegate: Box<dyn TableDelegate<AppState>>,
}

impl<AppState> Table<AppState> {
    pub fn new<D>(delegate: D) -> Self
    where
        D: TableDelegate<AppState> + 'static,
    {
        Table {
            paint: Paint::default(),
            delegate: Box::new(delegate),
        }
    }
}

impl<AppState> Widget<AppState> for Table<AppState> {
    fn paint(
        &mut self,
        state: &AppState,
        rect: &Rect,
        canvas: &mut dyn Canvas2D,
        style: &StyleSheet,
    ) {
        assert_ne!(rect.width(), 0f32);
        assert_ne!(rect.height(), 0f32);
        let e_color = *style.get("even").unwrap_or(&Color::CYAN);
        let u_color = *style.get("uneven").unwrap_or(&Color::RED);

        let row_count = self.delegate.row_count(state);
        let height = rect.height() / row_count as f32;

        for i in 0..row_count {
            if i % 2 == 0 {
                self.paint.set_color(e_color);
            } else {
                self.paint.set_color(u_color);
            }

            canvas.draw_rounded_rect(
                &Rect::from_point_and_size(
                    (rect.left(), rect.top() + i as f32 * height),
                    (rect.width(), height),
                ),
                1.,
                1.,
                &self.paint,
            )
        }
    }

    fn mouse_up(
        &mut self,
        state: &mut AppState,
        rect: &Rect,
        event: &MouseEvent,
    ) -> Action<AppState> {
        let row_count = self.delegate.row_count(state);
        let y = event.global_position().y - rect.top();
        let row_size = rect.height() / row_count as f32;
        let row = y / row_size;

        self.delegate.row_selected(row as usize, state)
    }

    fn layout(
        &mut self,
        _state: &AppState,
        _rect: &Rect,
        _spacing: f32,
        _padding: f32,
        _children: &mut [Node<AppState>],
    ) {
    }

    fn calculate_size(
        &self,
        preferred_width: Option<f32>,
        preferred_height: Option<f32>,
        constraints: &Constraints,
        _children: &[Node<AppState>],
    ) -> (Size, Vec<Constraints>) {
        let w = if let Some(preferred_width) = preferred_width {
            let width = constraints.min_width;
            let width = width.max(preferred_width).min(constraints.max_width);
            width
        } else {
            constraints.max_width
        };

        let h = if let Some(preferred_height) = preferred_height {
            let height = constraints.min_height;
            let height = height.max(preferred_height).min(constraints.max_height);
            height
        } else {
            constraints.max_height
        };

        (Size::new(w, h), vec![Constraints::new(0.0, w, 0.0, h)])
    }
}

pub struct Slider<AppState> {
    label: String,
    border_paint: Paint,
    bg_paint: Paint,
    fill_paint: Paint,
    text_paint: Paint,
    font: Font,
    min: f32,
    max: f32,
    discrete: bool,
    current_normalized: f32,
    current_value: f32,
    last_position: f32,
    value_changed: Option<Box<dyn FnMut(f32, &mut AppState)>>,
}

impl<AppState> Slider<AppState> {
    pub fn new(label: &str) -> Self {
        Slider::new_with_min_max_and_value(label, 0., 1., 0., false)
    }

    pub fn new_with_min_max_and_value(
        label: &str,
        min: f32,
        max: f32,
        value: f32,
        discrete: bool,
    ) -> Self {
        Slider {
            label: label.to_string(),
            bg_paint: Paint::default(),
            fill_paint: Paint::default(),
            text_paint: Paint::default(),
            border_paint: Paint::default(),
            font: Font::default(),
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
        F: FnMut(f32, &mut AppState) + 'static,
    {
        self.value_changed = Some(Box::new(handler));
        self
    }

    pub fn set_value(&mut self, value: f32) {
        self.current_value = value.max(self.min).min(self.max);
        self.current_normalized = map_range(self.current_value, self.min, self.max, 0., 1.)
    }
}

impl<AppState> Widget<AppState> for Slider<AppState> {
    fn paint(&mut self, _: &AppState, rect: &Rect, canvas: &mut dyn Canvas2D, style: &StyleSheet) {
        assert_ne!(rect.width(), 0f32);
        assert_ne!(rect.height(), 0f32);

        let bg_color = style.get("bg-color");
        let fill_color = style.get("fill-color");
        let border_color = style.get("border-color");

        self.bg_paint
            .set_color(*bg_color.unwrap_or(&Color::new(128)));
        self.border_paint
            .set_color(*border_color.unwrap_or(&Color::new(128)));
        self.border_paint.set_style(PaintStyle::Stroke);
        self.fill_paint
            .set_color(*fill_color.unwrap_or(&Color::new(128)));
        self.text_paint.set_color(Color::new(255));
        canvas.draw_rounded_rect(rect, 2., 2., &self.bg_paint);
        canvas.draw_rounded_rect(rect, 2., 2., &self.border_paint);
        let mut fill_rect = Rect::from_xywh(
            rect.left(),
            rect.top(),
            rect.width() * self.current_normalized,
            rect.height(),
        );
        fill_rect.inset((2, 2));

        canvas.draw_rounded_rect(&fill_rect, 0., 0., &self.fill_paint);

        let t = self.label.to_string() + ": " + &format!("{:.4}", &self.current_value.to_string());
        canvas.draw_string(&t, &rect.center(), &self.font, &self.text_paint);
    }

    fn mouse_down(&mut self, state: &mut AppState, rect: &Rect, event: &MouseEvent) {
        let x = event.global_position().x - rect.left();
        self.current_normalized = (1. / rect.width()) * x;

        self.current_value = map_range(self.current_normalized, 0., 1., self.min, self.max);
        if self.discrete {
            self.current_value = self.current_value.round();
        }
        if let Some(l) = &mut self.value_changed {
            (l)(self.current_value, state);
        }

        self.last_position = x;
    }

    fn calculate_size(
        &self,
        preferred_width: Option<f32>,
        preferred_height: Option<f32>,
        constraints: &Constraints,
        _children: &[Node<AppState>],
    ) -> (Size, Vec<Constraints>) {
        let w = if let Some(preferred_width) = preferred_width {
            let width = constraints.min_width;
            let width = width.max(preferred_width).min(constraints.max_width);
            width
        } else {
            constraints.max_width
        };

        let h = if let Some(preferred_height) = preferred_height {
            let height = constraints.min_height;
            let height = height.max(preferred_height).min(constraints.max_height);
            height
        } else {
            constraints.max_height
        };

        (Size::new(w, h), vec![Constraints::new(0.0, w, 0.0, h)])
    }

    // fn mouse_drag(&mut self, state: &mut AppState, rect: &Rect, event: &MouseEvent) {
    //     self.last_position += event.delta_position.x;
    //     self.current_normalized =
    //         (1. / rect.width()) * self.last_position.min(rect.width()).max(0.);

    //     self.current_value = map_range(self.current_normalized, 0., 1., self.min, self.max);

    //     if self.discrete {
    //         self.current_value = self.current_value.round();
    //     }
    //     if let Some(l) = &mut self.value_changed {
    //         (l)(self.current_value, state);
    //     }
    // }
}

// pub struct Spinner<AppState> {
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
//     value_changed: Option<Box<dyn FnMut(f32, &mut AppState)>>,
// }

// impl<AppState> Spinner<AppState> {
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
//         F: FnMut(f32, &mut AppState) + 'static,
//     {
//         self.value_changed = Some(Box::new(handler));
//         self
//     }
// }

// impl<AppState> Widget<AppState> for Spinner<AppState> {
//     fn paint(&mut self, _: &mut AppState, rect: &Rect, canvas: &mut dyn Canvas2D, style: &StyleSheet) {
//         let bg_color = style.get("bg-color");
//         let fill_color = style.get("fill-color");
//         let border_color = style.get("border-color");

//         self.bg_paint
//             .set_color(bg_color.unwrap_or(&Color::new(1., 0., 0., 1.)));
//         self.border_paint
//             .set_color(border_color.unwrap_or(&Color::new(1., 0., 0., 1.)));
//         self.border_paint.set_style(PaintStyle::Stroke);
//         self.fill_paint
//             .set_color(fill_color.unwrap_or(&Color::new(0.2, 0.2, 0.2, 1.)));
//         self.text_paint.set_color(&Color::new(1., 1., 1., 1.));
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

//     fn mouse_drag(&mut self, state: &mut AppState, _: &Rect, event: &MouseEvent) {
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

pub struct ViewPort {
    scroll_bar_rect: Rect,
    orientation: Orientation,
    paint: Paint,
    scroll_bar_paint: Paint,
    scroll_bar_position: f32,
    scroll_bar_ratio: f32,
}

impl ViewPort {
    pub fn new(orientation: Orientation) -> Self {
        ViewPort {
            scroll_bar_rect: Rect::default(),
            orientation,
            paint: Paint::default(),
            scroll_bar_paint: Paint::default(),
            scroll_bar_position: 0.,
            scroll_bar_ratio: 0.,
        }
    }
}

// impl<AppState> Widget<AppState> for ViewPort {
//     fn layout(
//         &mut self,
//         _state: &AppState,
//         rect: &Rect,
//         _spacing: f32,
//         _padding: f32,
//         children: &mut [Node<AppState>],
//     ) {
//         assert_eq!(1, children.len());

//         self.scroll_bar_rect = *rect;
//         children[0].rect.set_wh(rect.size().width, rect.size().height);
//         children[0].rect.set_wh(children[0].constraints.size(&rect.size()));

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
//         _state: &AppState,
//         rect: &Rect,
//         canvas: &mut dyn Canvas2D,
//         _style: &StyleSheet,
//     ) {
//         // self.paint.set_color(&Color::from((0., 0., 0.));
//         // canvas.draw_rect(rect, &self.paint);

//         // self.scroll_bar_paint
//         //     .set_color(&Color::new(0.3, 0.3, 0.3, 1.));
//         // canvas.draw_rect(self.scroll_bar_rect, &self.scroll_bar_paint);

//         // self.scroll_bar_paint
//         //     .set_color(&Color::new(0.2, 0.2, 0.2, 1.));

//         // let r = Rect::from_xywh(
//         //     self.scroll_bar_rect.left() + 1. + self.scroll_bar_position,
//         //     self.scroll_bar_rect.bottom() + 1.,
//         //     self.scroll_bar_rect.width(),
//         //     self.scroll_bar_rect.height() * self.scroll_bar_ratio,
//         // );

//         // canvas.draw_rect(r, &self.scroll_bar_paint);
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
    fn new(_request: PopupMenu) -> Self {
        PopupMenuWidget {
            active: true,
            children: Vec::new(),
        }
    }
}

impl<AppState> Widget<AppState> for PopupMenuWidget {}

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

pub struct PopupRequest<AppState> {
    menu: PopupMenu,
    pub handler: Box<dyn FnMut(usize, usize, &mut AppState) -> Action<AppState>>,
}

impl<AppState: 'static> PopupRequest<AppState> {
    pub fn new<F>(menu: PopupMenu, handler: F) -> Self
    where
        F: FnMut(usize, usize, &mut AppState) -> Action<AppState> + 'static,
    {
        PopupRequest {
            menu,
            handler: Box::new(handler),
        }
    }

    pub fn build(&self) -> Node<AppState> {
        let mut b = Node::new("menu").widget(VStack::new()).spacing(1.);

        for item in self.menu.items.iter() {
            let s = item.id;
            b.add_child(
                Node::new("btn")
                    .widget(Button::new(&item.name))
                    .with_mouse_event_callback(MouseEventType::MouseUp, move |_, _| {
                        Action::TriggerPopupMenu {
                            menu: 0,
                            sub_menu: s,
                        }
                    }),
            );
        }

        b.rect.set_wh(150., self.menu.items.len() as f32 * 28.);
        b
    }
}
