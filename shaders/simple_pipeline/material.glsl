struct Material
{
    vec4 base_color;
    vec4 emission;
    // float roughness;
    // float metallic;
    // float sheen;
    // float clear_coat;
    vec4 properties;
    // int base_color_texture;
    // int metallic_roughness_texture;
    // int normal_texture;
    // int emission_texture;
    vec2 transmission;
    ivec4 maps;
};