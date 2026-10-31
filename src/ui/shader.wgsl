struct ScreenUniform {
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> u_screen: ScreenUniform;

@group(0) @binding(1)
var t_font: texture_2d<f32>;

@group(0) @binding(2)
var s_font: sampler;

@group(0) @binding(3)
var t_blocks: texture_2d_array<f32>;

@group(0) @binding(4)
var s_blocks: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) mode: u32,
    @location(4) tex_layer: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) mode: u32,
    @location(3) @interpolate(flat) tex_layer: u32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Map screen pixel coords (0..width, 0..height) to NDC (-1..1, 1..-1)
    let ndc_x = (in.position.x / u_screen.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (in.position.y / u_screen.screen_size.y) * 2.0;

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    out.mode = in.mode;
    out.tex_layer = in.tex_layer;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.mode == 0u) {
        // Solid colored quad / border
        return in.color;
    } else if (in.mode == 1u) {
        // Font glyph (alpha masked)
        let font_sample = textureSample(t_font, s_font, in.uv);
        if (font_sample.a < 0.1) {
            discard;
        }
        return vec4<f32>(in.color.rgb, font_sample.a * in.color.a);
    } else if (in.mode == 2u) {
        // Block texture from 2D array
        let block_sample = textureSample(t_blocks, s_blocks, in.uv, in.tex_layer);
        if (block_sample.a < 0.1) {
            discard;
        }
        return vec4<f32>(block_sample.rgb * in.color.rgb, block_sample.a * in.color.a);
    }
    return in.color;
}
