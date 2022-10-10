#include "constants.glsl"

mat3 getNormalSpace(in vec3 normal) {
   vec3 someVec = vec3(1.0, 0.0, 0.0);
   float dd = dot(someVec, normal);
   vec3 tangent = vec3(0.0, 1.0, 0.0);
   if(1.0 - abs(dd) > 1e-6) {
     tangent = normalize(cross(someVec, normal));
   }
   vec3 bitangent = cross(normal, tangent);
   return mat3(tangent, bitangent, normal);
}

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
  return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
} 

float D_GGX(float NoH, float roughness) {
  float alpha = roughness * roughness;
  float alpha2 = alpha * alpha;
  float NoH2 = NoH * NoH;
  float b = (NoH2 * (alpha2 - 1.0) + 1.0);
  return alpha2 / (M_PI * b * b);
}

float G1_GGX_Schlick(float NdotV, float roughness) {
  float r = roughness; // original
  //float r = 0.5 + 0.5 * roughness; // Disney remapM_PIng
  float k = (r * r) / 2.0;
  float denom = NdotV * (1.0 - k) + k;
  return NdotV / denom;
}

float G_Smith(float NoV, float NoL, float roughness) {
  float g1_l = G1_GGX_Schlick(NoL, roughness);
  float g1_v = G1_GGX_Schlick(NoV, roughness);
  return g1_l * g1_v;
}

// vec3 microfacetBRDF(in vec3 L, in vec3 V, in vec3 N, 
//               in vec3 baseColor, in float metallicness, 
//               in float fresnelReflect, in float roughness,
//               in float transmission, in float ior) {
     
//   vec3 H = normalize(V + L); // half vector

//   // all required dot products
//   float NoV = clamp(dot(N, V), 0.0, 1.0);
//   float NoL = clamp(dot(N, L), 0.0, 1.0);
//   float NoH = clamp(dot(N, H), 0.0, 1.0);
//   float VoH = clamp(dot(V, H), 0.0, 1.0);     
  
//   // F0 for dielectics in range [0.0, 0.16] 
//   // default FO is (0.16 * 0.5^2) = 0.04
//   vec3 f0 = vec3(0.16 * (fresnelReflect * fresnelReflect)); 
//   // in case of metals, baseColor contains F0
//   f0 = mix(f0, baseColor, metallicness);

//   // specular microfacet (cook-torrance) BRDF
//   vec3 F = fresnelSchlick(VoH, f0);
//   float D = D_GGX(NoH, roughness);
//   float G = G_Smith(NoV, NoL, roughness);
//   vec3 spec = (D * G * F) / max(4.0 * NoV * NoL, 0.001);
  
//   // diffuse
//   vec3 notSpec = vec3(1.0) - F; // if not specular, use as diffuse
//   notSpec *= (1.0 - metallicness) * (1.0 - transmission); // no diffuse for metals
//   vec3 diff = notSpec * baseColor / M_PI; 
  
//   return diff + spec;
// }

vec3 sampleMicrofacetBRDF(in vec3 V, in vec3 N, in vec3 baseColor, in float metallicness, 
              in float fresnelReflect, in float roughness, in float transmission, 
              in float ior, in vec3 random, out vec3 nextFactor) {
  
  if(random.z < 0.5) { // non-specular light
    if(2.0 * random.z < transmission) { // transmitted light
      vec3 forwardNormal = N;
      float frontFacing = dot(V, N);
      float eta =1.0 / ior;
      if(frontFacing < 0.0) {
         forwardNormal = -N;
         eta = ior;
      } 
      
      // important sample GGX
      // pdf = D * cos(theta) * sin(theta)
      float a = roughness * roughness;
      float theta = acos(sqrt((1.0 - random.y) / (1.0 + (a * a - 1.0) * random.y)));
      float phi = 2.0 * M_PI * random.x;
      
      vec3 localH = vec3(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta));
      vec3 H = getNormalSpace(forwardNormal) * localH;  
      
      // compute L from sampled H
      vec3 L = refract(-V, H, eta);
      
      // all required dot products
      float NoV = clamp(dot(forwardNormal, V), 0.0, 1.0);
      float NoL = clamp(dot(-forwardNormal, L), 0.0, 1.0); // reverse normal
      float NoH = clamp(dot(forwardNormal, H), 0.0, 1.0);
      float VoH = clamp(dot(V, H), 0.0, 1.0);     
      
      // F0 for dielectics in range [0.0, 0.16] 
      // default FO is (0.16 * 0.5^2) = 0.04
      vec3 f0 = vec3(0.16 * (fresnelReflect * fresnelReflect)); 
      // in case of metals, baseColor contains F0
      f0 = mix(f0, baseColor, metallicness);
    
      vec3 F = fresnelSchlick(VoH, f0);
      float D = D_GGX(NoH, roughness);
      float G = G_Smith(NoV, NoL, roughness);
      nextFactor = baseColor * (vec3(1.0) - F) * G * VoH / max((NoH * NoV), 0.001);
    
      nextFactor *= 2.0; // compensate for splitting diffuse and specular
      return L;
      
    } else { // diffuse light
      
      // important sampling diffuse
      // pdf = cos(theta) * sin(theta) / M_PI
      float theta = asin(sqrt(random.y));
      float phi = 2.0 * M_PI * random.x;
      // sampled indirect diffuse direction in normal space
      vec3 localDiffuseDir = vec3(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta));
      vec3 L = getNormalSpace(N) * localDiffuseDir;  
      
       // half vector
      vec3 H = normalize(V + L);
      float VoH = clamp(dot(V, H), 0.0, 1.0);     
      
      // F0 for dielectics in range [0.0, 0.16] 
      // default FO is (0.16 * 0.5^2) = 0.04
      vec3 f0 = vec3(0.16 * (fresnelReflect * fresnelReflect)); 
      // in case of metals, baseColor contains F0
      f0 = mix(f0, baseColor, metallicness);    
      vec3 F = fresnelSchlick(VoH, f0);
      
      vec3 notSpec = vec3(1.0) - F; // if not specular, use as diffuse
      notSpec *= (1.0 - metallicness); // no diffuse for metals
    
      nextFactor = notSpec * baseColor;
      nextFactor *= 2.0; // compensate for splitting diffuse and specular
      return L;
    }
  } else {// specular light
    
    // important sample GGX
    // pdf = D * cos(theta) * sin(theta)
    float a = roughness * roughness;
    float theta = acos(sqrt((1.0 - random.y) / (1.0 + (a * a - 1.0) * random.y)));
    float phi = 2.0 * M_PI * random.x;
    
    vec3 localH = vec3(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta));
    vec3 H = getNormalSpace(N) * localH;  
    vec3 L = reflect(-V, H);

    // all required dot products
    float NoV = clamp(dot(N, V), 0.0, 1.0);
    float NoL = clamp(dot(N, L), 0.0, 1.0);
    float NoH = clamp(dot(N, H), 0.0, 1.0);
    float VoH = clamp(dot(V, H), 0.0, 1.0);     
    
    // F0 for dielectics in range [0.0, 0.16] 
    // default FO is (0.16 * 0.5^2) = 0.04
    vec3 f0 = vec3(0.16 * (fresnelReflect * fresnelReflect)); 
    // in case of metals, baseColor contains F0
    f0 = mix(f0, baseColor, metallicness);
  
    // specular microfacet (cook-torrance) BRDF
    vec3 F = fresnelSchlick(VoH, f0);
    float D = D_GGX(NoH, roughness);
    float G = G_Smith(NoV, NoL, roughness);
    nextFactor =  F * G * VoH / max((NoH * NoV), 0.001);
    
    nextFactor *= 2.0; // compensate for splitting diffuse and specular
    return L;
  } 
  
}
