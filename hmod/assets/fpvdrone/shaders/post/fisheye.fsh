#version 330
#extension GL_ARB_separate_shader_objects : require

// Barrel distortion: the look of an FPV camera's wide fisheye lens over the world's
// rectilinear frame. The middle of the picture is magnified and the edges squeezed, with the
// frame's corners kept on its corners, so nothing outside the frame is ever sampled.

uniform sampler2D InSampler;

layout(std140) uniform SamplerInfo {
    vec2 OutSize;
    vec2 InSize;
};

layout(std140) uniform FisheyeConfig {
    // How much the middle is magnified against the corners (0 = none).
    float Strength;
    // Darkening at the corners, as a cheap lens's light falloff (0 = none).
    float Vignette;
};

layout(location = 0) in vec2 texCoord;

layout(location = 0) out vec4 fragColor;

void main() {
    vec2 aspect = vec2(OutSize.x / max(OutSize.y, 1.0), 1.0);
    vec2 p = texCoord - 0.5;
    vec2 q = p * aspect;
    // Radius squared, with the frame's corner at 1.
    float r2 = dot(q, q) / dot(0.5 * aspect, 0.5 * aspect);
    vec2 src = 0.5 + p * (1.0 + Strength * r2) / (1.0 + Strength);
    vec3 color = texture(InSampler, clamp(src, 0.0, 1.0)).rgb;
    color *= 1.0 - Vignette * r2 * r2;
    fragColor = vec4(color, 1.0);
}
