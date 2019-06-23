precision mediump float;

uniform vec3 uCameraPos;

varying vec3 vNormal;
varying vec3 vWorldSpacePos;

uniform vec3 lightColor;
uniform vec3 lightPos;

uniform vec3 baseColor;

const float PI = 3.14159265359;

uniform float roughness;
uniform float metallic;

vec3 calculateLighting(vec3 lightPos, vec3 lightColor);

vec3 calculateF0(vec3 baseColor, float metallic);

float TrowbridgeReitzGGX(vec3 normal, vec3 halfwayVector, float roughness);

float GeometrySchlickGGX(float NdotV, float roughness);
float GeometrySmith(vec3 normal, vec3 toCamera, vec3 lightDir, float roughness);

vec3 fresnelSchlick(vec3 F0, float vDotH);

void main(void) {

    // ---------- Reflectance equation --------------------------------------------------

    // Sum of the inpact of all lights (TODO: currently just one light)
    vec3 Lo = vec3(0.0);

    // Calculate the per light radiance
    // TODO: We only have one light right now - in future iterate through multiple lights
    Lo += calculateLighting(lightPos, lightColor);

    // --------------------------------------------------

    vec3 ambient = vec3(0.03) * baseColor;
    vec3 color = ambient + Lo;

    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));

    gl_FragColor = vec4(color, 1.0);
}


vec3 calculateLighting (vec3 lightPos, vec3 lightColor) {
    vec3 surfaceNormal = normalize(vNormal);

    vec3 fragToCamera = normalize(uCameraPos - vWorldSpacePos);
    vec3 fragToLight = normalize(lightPos - vWorldSpacePos);

    vec3 halfwayVec = normalize(fragToCamera + fragToLight);

    vec3 F0 = calculateF0(baseColor, metallic);

    // Radiance of this point light
    float distance = length(lightPos - vWorldSpacePos);
    float attenuation = 1.0 / (distance * distance);
    vec3 radiance = lightColor * attenuation;

    // Cool torrence BRDF
    //
    // f(l, v) = D(h) * F(v, h) * G(l, v, h)   <----- numerator
    //           ---------------------------
    //            4 * (n · l) * (n · v)        <----- denominator

    float D = TrowbridgeReitzGGX(surfaceNormal, halfwayVec, roughness);
    vec3 F = fresnelSchlick(F0, dot(fragToCamera, halfwayVec));
    float G = GeometrySmith(surfaceNormal, fragToCamera, fragToLight, roughness);

    // We used the fresenel-shlick approxmimation to approximate the percentage of incoming light this is
    // reflected (specular)
    // Since energy is conserved, the absored lighting (diffuse) + the specular lighting should equal `1.0`
    vec3 kD = vec3(1.0) - F;

    // Metallic surfaces don't have diffuse lighting, only specular.
    // So if this surface is metallic we'll multiply the diffuse component `kD` by zero.
    kD *= 1.0 - metallic;

    vec3 numerator = D * F * G;

    float denominator = 4.0 *
    max(dot(surfaceNormal, fragToCamera), 0.0) *
    max(dot(surfaceNormal, fragToLight), 0.0);

    vec3 specular = numerator / max(denominator, 0.001);

    float NdotL = max(dot(surfaceNormal, fragToLight), 0.0);

    return (kD * baseColor / PI + specular) * radiance * NdotL;
}

// The amount that the surface reflects light when looking directly at it.
// (Known as the the surface reflection as incidence zero)
//
// For non metallics we use a constant approximation of 0.04
// For metallics we use the base color of the fragment
vec3 calculateF0 (vec3 baseColor, float metallic) {
    return mix(vec3(0.04), baseColor, metallic);
}

float TrowbridgeReitzGGX(vec3 normal, vec3 halfwayVector, float roughness) {
    float a = roughness * roughness;
    float aSquared = a * a;

    float normalDotHalfwayVec = max(dot(normal, halfwayVector), 0.0);
    float normalDotHalfwayVecSquared = normalDotHalfwayVec * normalDotHalfwayVec;

    float denom = (normalDotHalfwayVecSquared * (aSquared - 1.0) + 1.0);
    denom = PI * denom * denom;

    return aSquared / denom;
}

float GeometrySmith(vec3 normal, vec3 toCamera, vec3 lightDir, float roughness) {
    float normalDotViewDir = max(dot(normal, toCamera), 0.0);
    float normalDotLightDir = max(dot(normal, lightDir), 0.0);

    float ggx2 = GeometrySchlickGGX(normalDotViewDir, roughness);
    float ggx1 = GeometrySchlickGGX(normalDotLightDir, roughness);

    return ggx1 * ggx2;
}

float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float denom = NdotV * (1.0 - k) + k;

    return NdotV / denom;
}

vec3 fresnelSchlick(vec3 F0, float nDotH) {
    float sphericalGaussian = (-5.55473 * nDotH - 6.98316) * nDotH;

    return F0 + (1.0 - F0) * pow(2.0, sphericalGaussian);
}
