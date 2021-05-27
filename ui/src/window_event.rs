use skia_safe::Point;

#[derive(PartialEq, Eq, Hash)]
pub enum MouseEventType {
    MouseDown,
    MouseUp,
    MouseMove,

    DoubleClick,
}

#[repr(C)]
pub struct MouseEvent {
    modifiers: u32,
    global_position: Point,
    local_position: Point,
}

impl MouseEvent {
    pub fn new(modifiers: u32, global_position: &Point, local_position: &Point) -> Self {
        Self {
            modifiers,
            global_position: *global_position,
            local_position: *local_position,
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
}
