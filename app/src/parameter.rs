pub enum Value {
    Float(f32),
    String(String),
}

pub struct Parameter {
    min: Option<Value>,
    max: Option<Value>,
    default: Option<Value>,
}
