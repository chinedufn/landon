precision mediump float;

// TODO: Heavily comment this shader

uniform vec3 uCameraPos;

varying vec3 vNormal;
varying vec3 vWorldSpacePos;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float specular_intensity;
};

uniform Material material;

// TODO: Array of multiple light colors and positions to support
// multiple point lights.
// TODO: Use uniforms
const vec3 lightColor = vec3(1.0, 1.0, 1.0);
const vec3 lightPos = vec3(2.0, 2.0, 2.0);

uniform vec3 baseColor;

const float PI = 3.14159265359;

uniform float roughness;
uniform float metallic;

// TODO: Comment each of these functions where we define them - describing exactly what they are
// what what they're doing.
float DistributionGGX(vec3 normal, vec3 halfwayVector, float roughness);
float GeometrySchlickGGX(float NdotV, float roughness);
float GeometrySmith(vec3 normal, vec3 toCamera, vec3 lightDir, float roughness);
vec3 fresnelSchlick(float cosTheta, vec3 F0);

// TODO: BREADCRUMB - the specular highlight is the same no matter what angle I look
// at ... so we're miscalculating something somewhere in here. Need to read over
// the shader and figure out what.

// TODO: Rename veriables from one letter names to more descriptive names
void main(void) {
    vec3 surfaceNormal = normalize(vNormal);
    vec3 toCamera = normalize(uCameraPos - vWorldSpacePos);

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, baseColor, metallic);

    // ---------- Reflectance equation --------------------------------------------------

    // Sum of the inpact of all lights (currently just one light)
    vec3 Lo = vec3(0.0);

    // Calculate the per light radiance
    vec3 lightDir = normalize(lightPos - vWorldSpacePos);
    vec3 H = normalize(toCamera + lightDir);
    float distance = length(lightPos - vWorldSpacePos);
    float attenuation = 1.0 / (distance * distance);
    vec3 radiance = lightColor * attenuation;
    radiance = lightColor;

    // cook-torrence brdf
    float NDF = DistributionGGX(surfaceNormal, H, roughness);
    float G = GeometrySmith(surfaceNormal, toCamera, lightDir, roughness);
    vec3 F = fresnelSchlick(max(dot(H, toCamera), 0.0), F0);

    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - metallic;

    vec3 numerator = NDF * G * F;
    float denominator = 4.0 *
      max(dot(surfaceNormal, toCamera), 0.0) *
      max(dot(surfaceNormal, lightDir), 0.0);

    vec3 specular = numerator / max(denominator, 0.001);

    // Add the outgoing radiance Lo
    float NdotL = max(dot(surfaceNormal, lightDir), 0.0);
    Lo += (kD * baseColor / PI + specular) * radiance * NdotL;

    // --------------------------------------------------

    vec3 ambient = vec3(0.04) * baseColor;
    vec3 color = ambient + Lo;

    color = color / (color + vec3(1.0));
    color = pow(color, vec3(1.0 / 2.2));

    gl_FragColor = vec4(color, 1.0);
}


// FIXME: Thoroughly comment
float DistributionGGX(vec3 normal, vec3 halfwayVector, float roughness) {
    float a = roughness * roughness;
    float aSquared = a * a;

    float normalDotHalfwayVec = max(dot(normal, halfwayVector), 0.0);
    float normalDotHalfwayVecSquared = normalDotHalfwayVec * normalDotHalfwayVec;

    float denom = (normalDotHalfwayVecSquared * (aSquared - 1.0) + 1.0);
    denom = PI * denom * denom;

    return aSquared / denom;
}

// FIXME: Thoroughly comment
float GeometrySmith(vec3 normal, vec3 toCamera, vec3 lightDir, float roughness) {
    float normalDotViewDir = max(dot(normal, toCamera), 0.0);
    float normalDotLightDir = max(dot(normal, lightDir), 0.0);

    float ggx2 = GeometrySchlickGGX(normalDotViewDir, roughness);
    float ggx1 = GeometrySchlickGGX(normalDotLightDir, roughness);

    return ggx1 * ggx2;
}

// FIXME: Thoroughly comment
float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float denom = NdotV * (1.0 - k) + k;

    return NdotV / denom;
}


vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}
