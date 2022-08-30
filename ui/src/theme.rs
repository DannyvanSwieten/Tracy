use skia_safe::Color4f;

pub struct Theme {
    primary: Color4f,
    secondary: Color4f,

    bg_color: Color4f,
    text_color: Color4f,

    header_sizes: [f32; 8],
}
