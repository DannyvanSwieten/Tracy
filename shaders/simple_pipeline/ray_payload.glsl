
struct RayPayload
{
    vec4 color;
    vec4 emission;
    vec3 point;
    vec3 w_out;
    vec3 normal;
    bool hit;

    uint seed;
};