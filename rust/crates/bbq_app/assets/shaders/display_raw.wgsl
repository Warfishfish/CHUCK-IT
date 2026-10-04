// Makes Bevy show the colour numbers as they are, like the browser game does.
//
// The browser game (three.js r149) adds up light straight on the hex colour numbers and shows
// the result with no sRGB step. Bevy always encodes to sRGB at the very end. So just before
// that, this undoes the encoding: it clamps the result to 0..1 (three does too) and turns it
// into the "linear" number that the GPU's sRGB step will turn back into the same number.
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct Settings {
    on: f32,
}
@group(0) @binding(2) var<uniform> settings: Settings;

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(hi, lo, c <= vec3<f32>(0.04045));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let c = textureSample(screen_texture, texture_sampler, in.uv);
    if settings.on < 0.5 {
        return c;
    }
    return vec4<f32>(srgb_to_linear(clamp(c.rgb, vec3<f32>(0.0), vec3<f32>(1.0))), c.a);
}
