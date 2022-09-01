use crate::application::Application;
use crate::application_model::ApplicationModel;
use crate::canvas_2d::Canvas2D;
// use crate::node::*;
use crate::window_event::{MouseEvent, MouseEventType};
use skia_safe::{Color, Color4f, Font, Paint, PaintStyle, Point, Rect, Size};

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

#[derive(Default)]
pub struct BoxConstraints {
    min_width: Option<f32>,
    min_height: Option<f32>,
    max_width: Option<f32>,
    max_height: Option<f32>,
}

impl BoxConstraints {
    pub fn new() -> Self {
        Self {
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }

    pub fn with_min_width(mut self, min_width: f32) -> Self {
        self.min_width = Some(min_width);
        self
    }

    pub fn with_max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn with_min_height(mut self, min_height: f32) -> Self {
        self.min_height = Some(min_height);
        self
    }

    pub fn with_max_height(mut self, max_height: f32) -> Self {
        self.max_height = Some(max_height);
        self
    }

    pub fn with_tight_constraints(mut self, width: f32, height: f32) -> Self {
        self.min_width = Some(width);
        self.max_width = Some(width);
        self.min_height = Some(height);
        self.max_height = Some(height);
        self
    }

    pub fn shrunk(&self, dw: f32, dh: f32) -> Self {
        let width = if let Some(width) = self.max_width {
            Some(width - dw)
        } else {
            None
        };

        let height = if let Some(height) = self.max_height {
            Some(height - dh)
        } else {
            None
        };

        Self {
            min_width: self.min_width,
            min_height: self.min_height,
            max_width: width,
            max_height: height,
        }
    }

    pub fn min_width(&self) -> Option<f32> {
        self.min_width
    }
    pub fn max_width(&self) -> Option<f32> {
        self.max_width
    }

    pub fn min_height(&self) -> Option<f32> {
        self.min_height
    }

    pub fn max_height(&self) -> Option<f32> {
        self.max_height
    }
}

pub struct DragSource<Model> {
    child: ChildSlot<Model>,
    on_drag_start: Box<Fn() -> Box<dyn Widget<Model>>>,
}

pub struct DragContext<Model> {
    dragged_sources: Vec<Box<dyn Widget<Model>>>,
}

pub struct Contexts<Model> {
    drag_context: DragContext<Model>,
}

pub struct Properties {
    pub position: Point,
    pub size: Size,
}

pub trait Widget<Model: ApplicationModel> {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size;
    fn paint(&self, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model);
    fn flex(&self) -> f32 {
        0f32
    }
    fn mouse_down(
        &mut self,
        _: &MouseEvent,
        _: &Properties,
        _: &mut Application<Model>,
        _: &mut Model,
    ) {
    }
    fn mouse_up(&mut self, _: &MouseEvent, _: &mut Application<Model>, _: &mut Model) {}
    fn mouse_dragged(&mut self, _: &MouseEvent, _: &mut Model) {}
    fn mouse_moved(&mut self, _: &MouseEvent, _: &mut Model) {}
    fn mouse_entered(&mut self, _: &MouseEvent, _: &mut Model) {}
    fn mouse_left(&mut self, _: &MouseEvent, _: &mut Model) {}
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

    fn paint(&self, canvas: &mut dyn Canvas2D, _: &Size, model: &Model) {
        canvas.save();
        canvas.translate(self.position());
        self.widget.paint(canvas, self.size(), model);
        canvas.restore();
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
        } else {
            if self.has_mouse {
                self.has_mouse = false;
                let new_event = event.to_local(self.position());
                self.widget.mouse_left(&new_event, model);
            }
        }
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, model: &mut Model) {
        if self.hit_test(event.local_position()) {
            let new_event = event.to_local(self.position());
            self.widget.mouse_dragged(&new_event, model);
        }
    }

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {
        if self.hit_test(event.local_position()) {
            let new_event = event.to_local(self.position());

            if !self.has_mouse {
                self.has_mouse = true;
                self.mouse_entered(&event, model);
            }

            self.widget.mouse_moved(&new_event, model);
        } else {
            let new_event = event.to_local(self.position());
            if self.has_mouse {
                self.has_mouse = false;
                self.widget.mouse_left(&event, model);
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
}

pub struct Container<Model> {
    padding: f32,
    margin: f32,
    border: f32,
    child: ChildSlot<Model>,
    paint: Option<Paint>,
}

impl<Model: ApplicationModel> Container<Model> {
    pub fn new(child: impl Widget<Model> + 'static) -> Self {
        Self {
            padding: 0.0,
            margin: 0.0,
            border: 0.0,
            child: ChildSlot::new(child),
            paint: None,
        }
    }

    pub fn with_bg_color(mut self, color: &Color4f) -> Self {
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
        let space_around = self.padding + self.margin + self.border;
        self.child
            .set_position(&Point::new(space_around, space_around));
        let space_around = space_around * 2f32;
        let child_constraints = constraints.shrunk(space_around, space_around);
        let child_size = self.child.layout(&child_constraints, model);
        self.child.set_size(&child_size);

        Size::new(
            child_size.width + space_around,
            child_size.height + space_around,
        )
    }

    fn paint(&self, canvas: &mut dyn Canvas2D, size: &Size, model: &Model) {
        if let Some(paint) = &self.paint {
            let margin_rect =
                Rect::from_point_and_size(Point::new(self.margin, self.margin), *size);
            canvas.draw_rect(&margin_rect, paint);
        }

        self.child.paint(canvas, self.child.size(), model);
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

    fn mouse_dragged(&mut self, _: &MouseEvent, _: &mut Model) {
        todo!()
    }

    fn mouse_moved(&mut self, _: &MouseEvent, _: &mut Model) {
        todo!()
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
            assert!(constraints.max_width().is_some());
            assert!(constraints.max_height().is_some());

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

    fn paint(&self, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        self.child.paint(canvas, rect, model)
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

    fn mouse_dragged(&mut self, event: &MouseEvent, model: &mut Model) {
        self.child.mouse_dragged(event, model)
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
}

pub struct Row<Model> {
    children: Vec<ChildSlot<Model>>,
    spacing: f32,
}

impl<Model: ApplicationModel> Row<Model> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            spacing: 0f32,
        }
    }

    pub fn with_child<W>(mut self, child: W) -> Self
    where
        W: Widget<Model> + 'static,
    {
        self.children.push(ChildSlot::new(child));
        self
    }

    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl<Model: ApplicationModel> Widget<Model> for Row<Model> {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        let constrained_sizes: Vec<Size> = self
            .children
            .iter_mut()
            .flat_map(|child| {
                if child.flex() == 0f32 {
                    let child_size = child.layout(constraints, model);
                    child.set_size(&child_size);
                    Some(child_size)
                } else {
                    None
                }
            })
            .collect();

        if constrained_sizes.len() != self.children.len() {
            // If there are flex children in this Row but there are no horizontal constraints, we are screwed.
            // If you hit this assert, make sure you wrap this row inside a flexbox.
            assert!(constraints.max_width().is_some());
        }

        let constrained_size =
            constrained_sizes
                .iter()
                .fold(Size::new(0f32, 0f32), |mut acc, child_size| {
                    acc.width += child_size.width;
                    acc.height = acc.height.max(child_size.height);
                    acc
                });

        let total_flex = self
            .children
            .iter()
            .fold(0f32, |acc, child| acc + child.flex());

        if total_flex > 0f32 {
            let width = constraints.max_width().unwrap();
            let unconstraint_width = width - constrained_size.width;
            let flex_factor = unconstraint_width / total_flex;
            for child in &mut self.children {
                if child.flex() != 0f32 {
                    let child_constraints =
                        BoxConstraints::new().with_max_width(flex_factor * child.flex());
                    let child_size = child.layout(&child_constraints, model);
                    child.set_size(&child_size);
                }
            }
        }

        let mut position = Point::new(0f32, 0f32);
        for child in &mut self.children {
            child.set_position(&position);
            position.x += child.size().width + self.spacing;
        }

        let height = self
            .children
            .iter()
            .fold(0f32, |result, child| result.max(child.size().height));

        Size::new(
            constraints
                .min_width()
                .unwrap_or(0f32)
                .max(position.x)
                .min(constraints.max_height.unwrap_or(std::f32::MAX)),
            constraints.min_height().unwrap_or(height),
        )
    }

    fn paint(&self, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        for child in &self.children {
            child.paint(canvas, rect, model)
        }
    }

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        app: &mut Application<Model>,
        model: &mut Model,
    ) {
        for child in &mut self.children {
            child.mouse_down(event, properties, app, model)
        }
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        for child in &mut self.children {
            child.mouse_up(event, app, model)
        }
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, model: &mut Model) {
        for child in &mut self.children {
            child.mouse_dragged(event, model)
        }
    }

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {
        for child in &mut self.children {
            child.mouse_moved(event, model)
        }
    }

    fn mouse_entered(&mut self, event: &MouseEvent, model: &mut Model) {
        for child in &mut self.children {
            child.mouse_entered(event, model)
        }
    }

    fn mouse_left(&mut self, event: &MouseEvent, model: &mut Model) {
        for child in &mut self.children {
            child.mouse_left(event, model)
        }
    }
}

pub struct Column<Model> {
    children: Vec<ChildSlot<Model>>,
}

impl<Model: ApplicationModel> Column<Model> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn with_child<W>(mut self, child: W) -> Self
    where
        W: Widget<Model> + 'static,
    {
        self.children.push(ChildSlot::new(child));
        self
    }
}

impl<Model: ApplicationModel> Widget<Model> for Column<Model> {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        let child_sizes: Vec<Size> = self
            .children
            .iter_mut()
            .map(|child| child.layout(constraints, model))
            .collect();

        let unconstrained_children: Vec<bool> = child_sizes
            .iter()
            .map(|size| size.width == 0f32 && size.height == 0f32)
            .collect();

        let size = child_sizes
            .iter()
            .fold(Size::new(0f32, 0f32), |mut acc, child_size| {
                acc.width = acc.width.max(child_size.width);
                acc.height += child_size.height;
                acc
            });

        let mut position = Point::new(0f32, 0f32);
        for (index, size) in child_sizes.iter().enumerate() {
            self.children[index].set_position(&position);
            self.children[index].set_size(&child_sizes[index]);
            position.y += size.height;
        }

        size
    }

    fn paint(&self, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        for child in &self.children {
            child.paint(canvas, rect, model)
        }
    }

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        app: &mut Application<Model>,
        model: &mut Model,
    ) {
        for child in &mut self.children {
            child.mouse_down(event, properties, app, model)
        }
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        for child in &mut self.children {
            child.mouse_up(event, app, model)
        }
    }

    fn mouse_dragged(&mut self, event: &MouseEvent, model: &mut Model) {
        for child in &mut self.children {
            child.mouse_dragged(event, model)
        }
    }

    fn mouse_moved(&mut self, event: &MouseEvent, model: &mut Model) {
        for child in &mut self.children {
            child.mouse_moved(event, model)
        }
    }
}

pub struct SizedBox<Model> {
    child: ChildSlot<Model>,
    size: Size,
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
    fn layout(&mut self, _: &BoxConstraints, _: &Model) -> Size {
        self.child.set_size(&self.size);
        self.size
    }

    fn paint(&self, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        self.child.paint(canvas, rect, model);
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

    fn mouse_up(&mut self, _: &MouseEvent, _: &mut Application<Model>, _: &mut Model) {
        todo!()
    }

    fn mouse_dragged(&mut self, _: &MouseEvent, _: &mut Model) {
        todo!()
    }

    fn mouse_moved(&mut self, _: &MouseEvent, _: &mut Model) {
        todo!()
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

    fn paint(&self, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        self.child.paint(canvas, rect, model)
    }

    fn flex(&self) -> f32 {
        self.flex
    }

    fn mouse_down(
        &mut self,
        _: &MouseEvent,
        properties: &Properties,
        _: &mut Application<Model>,
        _: &mut Model,
    ) {
        todo!()
    }

    fn mouse_up(&mut self, _: &MouseEvent, _: &mut Application<Model>, _: &mut Model) {
        todo!()
    }

    fn mouse_dragged(&mut self, _: &MouseEvent, _: &mut Model) {
        todo!()
    }

    fn mouse_moved(&mut self, _: &MouseEvent, _: &mut Model) {
        todo!()
    }
}
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

    fn paint(&self, canvas: &mut dyn Canvas2D, size: &Size, _: &Model) {
        canvas.draw_rounded_rect(&Rect::from_size(*size), 3f32, 3f32, &self.bg_paint);
        canvas.draw_string(&self.text, &self.font, &self.text_paint);
    }

    fn mouse_up(&mut self, event: &MouseEvent, app: &mut Application<Model>, model: &mut Model) {
        if let Some(handler) = &self.on_click {
            handler(app, model)
        }

        self.bg_paint
            .set_color4f(skia_safe::Color4f::new(0.35, 0.35, 0.35, 1.0), None);
    }

    fn mouse_down(
        &mut self,
        _: &MouseEvent,
        properties: &Properties,
        _: &mut Application<Model>,
        _: &mut Model,
    ) {
        self.bg_paint
            .set_color4f(skia_safe::Color4f::new(0.45, 0.45, 0.45, 1.0), None);
    }

    fn mouse_entered(&mut self, _: &MouseEvent, _: &mut Model) {
        self.bg_paint
            .set_color4f(skia_safe::Color4f::new(0.35, 0.35, 0.35, 1.0), None);
    }

    fn mouse_left(&mut self, _: &MouseEvent, _: &mut Model) {
        self.bg_paint
            .set_color4f(skia_safe::Color4f::new(0.25, 0.25, 0.25, 1.0), None);
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

pub struct Slider<Model> {
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
    value_changed: Option<Box<dyn FnMut(f32, &mut Model)>>,
}

impl<Model> Slider<Model> {
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
        Size::new(
            constraints.max_width().unwrap(),
            constraints.max_height().unwrap(),
        )
    }

    // fn mouse_dragged(&mut self, state: &mut Model, rect: &Rect, event: &MouseEvent) {
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

    fn mouse_down(
        &mut self,
        event: &MouseEvent,
        properties: &Properties,
        app: &mut Application<Model>,
        model: &mut Model,
    ) {
        let x = event.global_position().x - properties.position.x;
        self.current_normalized = (1. / properties.size.width) * x;

        self.current_value = map_range(self.current_normalized, 0., 1., self.min, self.max);
        if self.discrete {
            self.current_value = self.current_value.round();
        }
        if let Some(l) = &mut self.value_changed {
            (l)(self.current_value, model);
        }

        self.last_position = x;
    }

    fn paint(&self, canvas: &mut dyn Canvas2D, rect: &Size, _: &Model) {
        let bg_paint = Paint::new(Color4f::new(0.5, 0.5, 0.5, 1.0), None);
        let fill_paint = Paint::new(Color4f::new(0.6, 0.6, 0.6, 1.0), None);
        let border_paint = Paint::new(Color4f::new(0.5, 0.5, 0.5, 1.0), None);
        let text_paint = Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None);

        let rect = Rect::from_size(*rect);
        canvas.draw_rounded_rect(&rect, 2., 2., &bg_paint);
        canvas.draw_rounded_rect(&rect, 2., 2., &border_paint);
        let mut fill_rect = Rect::from_xywh(
            rect.left(),
            rect.top(),
            rect.width() * self.current_normalized,
            rect.height(),
        );
        fill_rect.inset((2, 2));

        canvas.draw_rounded_rect(&fill_rect, 0., 0., &fill_paint);

        let t = self.label.to_string() + ": " + &format!("{:.4}", &self.current_value.to_string());
        canvas.draw_string(&t, &self.font, &text_paint);
    }
}

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
// //     fn paint(&mut self, _: &mut Model, rect: &Rect, canvas: &mut dyn Canvas2D, style: &StyleSheet) {
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

impl<Model: ApplicationModel> Widget<Model> for PopupMenuWidget {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        todo!()
    }

    fn paint(&self, canvas: &mut dyn Canvas2D, size: &Size, model: &Model) {
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
        self.items.len() != 0
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
