use skia_safe::Point;

#[derive(PartialEq, Eq, Hash)]
pub enum MouseEventType {
    MouseDown,
    MouseUp,
    MouseMove,

    DoubleClick,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MouseEvent {
    modifiers: u32,
    global_position: Point,
    local_position: Point,
    delta_position: Point,
}

impl MouseEvent {
    pub fn new(modifiers: u32, global_position: &Point, local_position: &Point) -> Self {
        Self {
            modifiers,
            global_position: *global_position,
            local_position: *local_position,
            delta_position: Point::new(0., 0.),
        }
    }

    pub fn to_local(&self, position: &Point) -> Self {
        let mut new_event = *self;
        new_event.local_position = self.local_position - *position;
        new_event
    }

    pub fn new_with_delta(
        modifiers: u32,
        global_position: &Point,
        local_position: &Point,
        delta_position: &Point,
    ) -> Self {
        Self {
            modifiers,
            global_position: *global_position,
            local_position: *local_position,
            delta_position: *delta_position,
        }
    }

    pub fn is_control_down(&self) -> bool {
        (self.modifiers & 1) != 0
    }

    pub fn is_shift_down(&self) -> bool {
        (self.modifiers & 2) != 0
    }

    pub fn is_right_mouse(&self) -> bool {
        (self.modifiers & 4) != 0
    }

    pub fn global_position(&self) -> &Point {
        &self.global_position
    }

    pub fn local_position(&self) -> &Point {
        &self.local_position
    }

    pub fn delta_position(&self) -> &Point {
        &self.delta_position
    }
}
