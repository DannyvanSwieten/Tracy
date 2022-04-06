use crate::material::Material;

pub struct Instance{
    transform: glm::Mat4,
    material: Material,
}