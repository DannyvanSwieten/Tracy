use skia_safe::{Color, Font};
use std::collections::HashMap;
#[derive(Default)]
pub struct SliderThumbStyle {
    color: Color,
    size: f32,
}
#[derive(Default)]
pub struct SliderStyle {
    background: Color,
    label_font: Font,
    thumb: SliderThumbStyle,
}
#[derive(Default)]
pub struct TextButtonStyle {
    inactive: Color,
    active: Color,
    hoverd: Color,
    text: Color,
    font: Font,
}
#[derive(Default)]
pub struct Theme {
    pub background: Color,
    pub primary: Color,
    pub secondary: Color,

    pub button: TextButtonStyle,
    pub slider: SliderStyle,
}

impl Theme {
    pub fn default_light() -> Self {
        Self {
            background: Color::new(0xFFFFFFFF),
            primary: Color::new(0xFF766AC8),
            secondary: Color::new(0xFF73C8A6),
            button: TextButtonStyle::default(),
            slider: SliderStyle::default(),
        }
    }
}

pub struct StyleContext {
    themes: HashMap<String, Theme>,
}

impl StyleContext {
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        themes.insert("light".to_string(), Theme::default_light());
        Self { themes }
    }

    pub fn theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }
}
