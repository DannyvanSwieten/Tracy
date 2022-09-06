use skia_safe::{Color, Font};
use std::collections::HashMap;
#[derive(Default)]
pub struct SliderThumbStyle {
    pub color: Color,
    pub size: f32,
}
#[derive(Default)]
pub struct SliderStyle {
    pub background: Color,
    pub fill: Color,
    pub thumb: SliderThumbStyle,
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
    pub text: Color,

    pub button: TextButtonStyle,
    pub slider: SliderStyle,
}

impl Theme {
    pub fn default_light() -> Self {
        Self {
            background: Color::new(0xFFFFFFFF),
            primary: Color::new(0xFF766AC8),
            secondary: Color::new(0xFF73C8A6),
            text: Color::new(0xFFFFFFFF),
            button: TextButtonStyle::default(),
            slider: SliderStyle {
                background: Color::new(0xFFFFFFFF),
                fill: Color::new(0xFF766AC8),
                thumb: SliderThumbStyle {
                    color: Color::new(0xFF73C8A6),
                    size: 0.125,
                },
            },
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
