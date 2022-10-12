use skia_safe::{Point, Size};

use crate::{
    application::Application,
    application_model::ApplicationModel,
    canvas_2d::Canvas2D,
    constraints::BoxConstraints,
    style::Theme,
    widget::{ChildSlot, Properties, Widget},
    window_event::MouseEvent,
};

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

    pub fn push<W>(mut self, child: W) -> Self
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
        // This is not a scrollable view. It needs constraints
        assert!(constraints.max_width().is_some() && constraints.max_height().is_some());
        // Start without child constraints
        let child_constraints = BoxConstraints::new();
        let constrained_sizes: Vec<Size> = self
            .children
            .iter_mut()
            .flat_map(|child| {
                if child.flex() == 0f32 {
                    let child_size = child.layout(&child_constraints, model);
                    child.set_size(&child_size);
                    Some(child_size)
                } else {
                    None
                }
            })
            .collect();

        let constrained_size =
            constrained_sizes
                .iter()
                .fold(Size::new(0f32, 0f32), |mut acc, child_size| {
                    acc.width += child_size.width + self.spacing;
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
                    let child_constraints = BoxConstraints::new()
                        .with_max_width(flex_factor * child.flex())
                        .with_max_height(constraints.max_height().unwrap());
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

        Size::new(
            constraints.max_width().unwrap(),
            constraints.max_height().unwrap(),
        )
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        for child in &self.children {
            child.paint(theme, canvas, rect, model)
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

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {
        for child in &mut self.children {
            child.mouse_dragged(event, properties, model)
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

    fn flex(&self) -> f32 {
        0f32
    }

    fn keyboard_event(&mut self, event: &winit::event::KeyboardInput, model: &mut Model) -> bool {
        for child in &mut self.children {
            if child.keyboard_event(event, model) {
                return true;
            }
        }

        false
    }

    fn character_received(&mut self, character: char, model: &mut Model) -> bool {
        for child in &mut self.children {
            if child.character_received(character, model) {
                return true;
            }
        }

        false
    }
}

pub struct Column<Model> {
    children: Vec<ChildSlot<Model>>,
    spacing: f32,
}

impl<Model: ApplicationModel> Column<Model> {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            spacing: 0f32,
        }
    }

    pub fn push<W>(mut self, child: W) -> Self
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

impl<Model: ApplicationModel> Widget<Model> for Column<Model> {
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        // It needs constraints
        assert!(constraints.max_width().is_some() && constraints.max_height().is_some());
        let total_spacing = (self.children.len() as f32 - 1.0) * self.spacing;
        // Start with no constraints
        let child_constraints = BoxConstraints::new();
        let constrained_sizes: Vec<Size> = self
            .children
            .iter_mut()
            .flat_map(|child| {
                if child.flex() == 0f32 {
                    let child_size = child.layout(&child_constraints, model);
                    child.set_size(&child_size);
                    Some(child_size)
                } else {
                    None
                }
            })
            .collect();

        let constrained_size =
            constrained_sizes
                .iter()
                .fold(Size::new(0f32, 0f32), |mut acc, child_size| {
                    acc.height += child_size.height + self.spacing;
                    acc.width = acc.width.max(child_size.width);
                    acc
                });

        let total_flex = self
            .children
            .iter()
            .fold(0f32, |acc, child| acc + child.flex());

        if total_flex > 0f32 {
            let height = constraints.max_height().unwrap();
            let unconstraint_height = height - total_spacing - constrained_size.height;
            let flex_factor = unconstraint_height / total_flex;
            for child in &mut self.children {
                if child.flex() != 0f32 {
                    let child_constraints = BoxConstraints::new()
                        .with_max_height(flex_factor * child.flex())
                        .with_max_width(constraints.max_width().unwrap());
                    let child_size = child.layout(&child_constraints, model);
                    child.set_size(&child_size);
                }
            }
        }

        let mut position = Point::new(0f32, 0f32);
        for child in &mut self.children {
            child.set_position(&position);
            position.y += child.size().height + self.spacing;
        }

        Size::new(
            constraints.max_width().unwrap(),
            constraints.max_height().unwrap(),
        )
    }

    fn paint(&self, theme: &Theme, canvas: &mut dyn Canvas2D, rect: &Size, model: &Model) {
        for child in &self.children {
            child.paint(theme, canvas, rect, model)
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

    fn mouse_dragged(&mut self, event: &MouseEvent, properties: &Properties, model: &mut Model) {
        for child in &mut self.children {
            child.mouse_dragged(event, properties, model)
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

    fn flex(&self) -> f32 {
        0f32
    }

    fn keyboard_event(&mut self, event: &winit::event::KeyboardInput, model: &mut Model) -> bool {
        for child in &mut self.children {
            if child.keyboard_event(event, model) {
                return true;
            }
        }

        false
    }

    fn character_received(&mut self, character: char, model: &mut Model) -> bool {
        for child in &mut self.children {
            if child.character_received(character, model) {
                return true;
            }
        }

        false
    }
}

pub struct Expanded<Model> {
    child: ChildSlot<Model>,
    width: Option<f32>,
    height: Option<f32>,
    flex: f32,
}

impl<Model: ApplicationModel> Expanded<Model> {
    pub fn new(child: impl Widget<Model> + 'static) -> Self {
        Self {
            child: ChildSlot::new(child),
            width: None,
            height: None,
            flex: 1f32,
        }
    }

    pub fn with_flex(mut self, flex: f32) -> Self {
        self.flex = flex;
        self
    }

    pub fn with_width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    pub fn with_height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }
}

impl<Model: ApplicationModel> Widget<Model> for Expanded<Model> {
    // If given to a flex container it will expand based on it's flex parameter in the dominant layout direction.
    // If for example you add it to a row it will expand in the horizontal direction. Therefor you should provide a height.
    fn layout(&mut self, constraints: &BoxConstraints, model: &Model) -> Size {
        let size = Size::new(
            self.width
                .unwrap_or_else(|| constraints.max_width().unwrap()),
            self.height
                .unwrap_or_else(|| constraints.max_height().unwrap()),
        );

        let child_size = self.child.layout(
            &BoxConstraints::new().with_tight_constraints(size.width, size.height),
            model,
        );

        self.child.set_size(&child_size);
        size
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

    fn mouse_entered(&mut self, _event: &MouseEvent, _model: &mut Model) {}

    fn mouse_left(&mut self, _event: &MouseEvent, _model: &mut Model) {}

    fn keyboard_event(&mut self, event: &winit::event::KeyboardInput, model: &mut Model) -> bool {
        self.child.keyboard_event(event, model)
    }

    fn character_received(&mut self, character: char, model: &mut Model) -> bool {
        self.child.character_received(character, model)
    }
}
