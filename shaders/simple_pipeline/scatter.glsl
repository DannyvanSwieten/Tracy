#include "constants.glsl"

float hash12(vec2 p) {
    vec3 p3  = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 19.19);
    return fract((p3.x + p3.y) * p3.z);
}

mat3 make_rotation_matrix_z(const vec3 z)
{
	const vec3 ref = abs(dot(z, vec3(0, 1, 0))) > 0.99f ? vec3(0, 0, 1) : vec3(0, 1, 0);

	const vec3 x = normalize(cross(ref, z));
	const vec3 y = cross(z, x);

	return mat3(x, y, z);
}

bool same_hemisphere(const vec3 wo, const vec3 wi, const vec3 N) {
    return dot(wo, N) * dot(wi, N) > 0.0;
}

vec3 sample_lambert_brdf(const vec3 N, inout vec3 L, const vec3 V, const vec2 Xi)
{
	const float phi = 2 * M_PI * Xi.y;
	const float sinTheta = sqrt(Xi.x);
	const float cosTheta = sqrt(1 - Xi.x);

	L = make_rotation_matrix_z(N) * vec3(sinTheta * cos(phi), sinTheta * sin(phi), cosTheta);

	return vec3(M_1_PI);
}

float schlick_weight(float cos_theta) {
    float m = clamp(1. - cos_theta, 0., 1.);
    return (m * m) * (m * m) * m;
}

float gtr2(float NdotH, float a) {
    float a2 = a*a;
    float t = 1. + (a2 - 1.) * NdotH*NdotH;
    return a2 / (M_PI * t*t);
}

float pow2(float x) {
	return x*x;
}

float gtr2_aniso(float NdotH, float HdotX, float HdotY, float ax, float ay) {
    return 1. / (M_PI * ax*ay * pow2( pow2(HdotX/ax) + pow2(HdotY/ay) + pow2(NdotH) ));
}

float smithG_GGX(float NdotV, float alphaG) {
    const float a = alphaG * alphaG;
    const float b = NdotV * NdotV;
    return 1. / max(0.0001, (abs(NdotV) + max(sqrt(a + b - a*b), 0.0001)));
}

float smithG_GGX_aniso(float NdotV, float VdotX, float VdotY, float ax, float ay) {
    return 1. / (NdotV + sqrt( pow2(VdotX*ax) + pow2(VdotY*ay) + pow2(NdotV) ));
}

vec3 evaluate_disney_anisotropic_specular(
	float NdotL, float NdotV, float NdotH, float LdotH, 
	const vec3 L, const vec3 V, const vec3 H, 
	const vec3 X, const vec3 Y, 
	vec3 base_color, float roughness, float metal, float anisotropy
	) {
    
    float Cdlum = 0.3 * base_color.r + 0.6 * base_color.g + 0.1 * base_color.b; // luminance approx.

    vec3 Ctint = Cdlum > 0. ? base_color / Cdlum : vec3(1.); // normalize lum. to isolate hue+sat
	const float specular = 1.0;
	const float specular_tint = 0.0;
	const vec3 tint = mix(vec3(1.), Ctint, specular_tint);
    const vec3 Cspec0 = mix(specular *.08 * tint, base_color, metal);

	float roughness_2 = roughness * roughness;
    
    float aspect = sqrt(1. - anisotropy * .9);
    float ax = max(.001, roughness_2 / aspect);
    float ay = max(.001, roughness_2 * aspect);
    float Ds = gtr2_aniso(NdotH, dot(H, X), dot(H, Y), ax, ay);
    float FH = schlick_weight(LdotH);
    vec3 Fs = mix(Cspec0, vec3(1), FH);
    float Gs = smithG_GGX_aniso(NdotL, dot(L, X), dot(L, Y), ax, ay) * smithG_GGX_aniso(NdotV, dot(V, X), dot(V, Y), ax, ay);
    
    return Gs*Fs*Ds;
}

vec3 evaluate_disney_diffuse(const float NdotL, const float NdotV, const float LdotH, const vec3 base_color, const float roughness) {
    
    const float FL = schlick_weight(NdotL);
	const float FV = schlick_weight(NdotV);
    
    float Fd90 = 0.5 + 2. * LdotH*LdotH * roughness;
    float Fd = mix(1.0, Fd90, FL) * mix(1.0, Fd90, FV);
    
    return (1.0 / M_PI) * Fd * base_color;
}

vec3 evaluate_disney_bsdf(const vec3 wi, const vec3 wo, const vec3 N, const vec3 X, const vec3 Y, vec3 base_color, float roughness, float metal, float anisotropy)
{
	if( !same_hemisphere(wo, wi, N) )
        return vec3(0.);

	float NdotL = dot(N, wo);
    float NdotV = dot(N, wi);

	if (NdotL < 0. || NdotV < 0.) return vec3(0.);

    vec3 H = normalize(wo+wi);
    float NdotH = dot(N,H);
    float LdotH = dot(wo,H);

	vec3 diffuse = evaluate_disney_diffuse(NdotL, NdotV, LdotH, base_color, roughness);
	vec3 specular = evaluate_disney_anisotropic_specular(NdotL, NdotV, NdotH, LdotH, wi, wo, H, X, Y, base_color, roughness, metal, anisotropy);
	vec3 color = diffuse * (1 - metal) + specular;
	return color;
}

vec3 sample_disney_micro_facet_anisotropic(const vec3 wo, const vec3 N, const vec3 X, const vec3 Y, const vec2 Xi, const float roughness, const float anisotropy) {
	float roughness_2 = roughness * roughness;
    
    float aspect = sqrt(1. - anisotropy * .9);
    float alphax = max(0.001, roughness_2 / aspect);
    float alphay = max(0.001, roughness_2 * aspect);
    
    float phi = atan(alphay / alphax * tan(2. * M_PI * Xi.y + .5 * M_PI));
	if (Xi.y > .5f) {
		phi += M_PI;
	}

    float sin_phi = sin(phi);
	float cos_phi = cos(phi);
    float alphax2 = alphax * alphax;
	float alphay2 = alphay * alphay;
    float alpha2 = 1. / max(0.001, (cos_phi * cos_phi / alphax2 + sin_phi * sin_phi / alphay2));
    float tan_theta2 = alpha2 * Xi.x / (1. - Xi.x);
    float cos_theta = 1. / sqrt(1. + tan_theta2);
	float sin_theta = sqrt(max(0., 1. - cos_theta * cos_theta));
	
    vec3 wh_local = vec3(sin_theta * cos(phi), sin_theta * sin(phi), cos_theta);
	vec3 wh = wh_local.x * X + wh_local.y * Y + wh_local.z * N;
	if(!same_hemisphere(wo, wh, N)) {
       wh *= -1.;
    }
            
    return normalize(reflect(-wo, wh));
}

float pdf_disney_diffuse(const vec3 wi, const vec3 wo, const vec3 N) {
	return same_hemisphere(wo, wi, N) ? abs(dot(N, wi)) / M_PI : 0.;
}

float pdf_disney_microfacet_anisotropic(const vec3 wi, const vec3 wo, const vec3 N, const vec3 X, const vec3 Y, float roughness, float anisotropy){
	if (!same_hemisphere(wo, wi, N)) return 0.;

	vec3 wh = normalize(wi + wo);
	float alpha_2 = pow2(roughness);
	float aspect = sqrt(1.0 - anisotropy * 0.9);
	float alpha_x = max(0.001, alpha_2 / aspect);
	float alpha_x_2 = alpha_x * alpha_x;
	float alpha_y = max(0.001, alpha_2 * aspect);
	float alpha_y_2 = alpha_y * alpha_y;

	float hDotX = dot(wh, X);
    float hDotY = dot(wh, Y);
    float NdotH = dot(N, wh);

	float denom = hDotX * hDotX / alpha_x_2 + hDotY * hDotY / alpha_y_2 + NdotH * NdotH;
	if( denom == 0. ) return 0.;

	float pdf_distribution = NdotH / (M_PI * alpha_x * alpha_y * denom * denom);
	return pdf_distribution / (4. * dot(wo, wh));
}

float pdf_disney_bsdf(const vec3 wi, const vec3 wo, const vec3 N, const vec3 X, const vec3 Y, float roughness, float metal, float anisotropy) {
	float diffuse = pdf_disney_diffuse(wi, wo, N);
	float specular = pdf_disney_microfacet_anisotropic(wi, wo, N, X, Y, roughness, anisotropy);
	return diffuse * .5 + specular * (1 - .5);
}

vec3 sample_disney_bsdf(const vec2 Xi, out vec3 wi, const vec3 wo, const vec3 N, const vec3 X, const vec3 Y, vec3 base_color, float roughness, float metal, float anisotropy, out float pdf)
{
	//generate random number to choose scatter
	float r = hash12(Xi);

	if(r < .5) {
		wi = sample_disney_micro_facet_anisotropic(wo, N, X, Y, Xi, roughness, anisotropy);
	} else {
		sample_lambert_brdf(N, wi, wo, Xi);
	}

	pdf = pdf_disney_bsdf(wi, wo, N, X, Y, roughness, metal, anisotropy);
	if(pdf < 0.001)
		return vec3(0);
	else
		return evaluate_disney_bsdf(wi, wo, N, X, Y, base_color, roughness, metal, anisotropy) * abs(dot(wi, N));
}