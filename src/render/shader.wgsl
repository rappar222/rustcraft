struct CameraUniform {
    view_proj: mat4x4<f32>,
    cam_pos: vec4<f32>,
    fog_color: vec4<f32>,
    fog_info: vec4<f32>, // [min_dist, max_dist, daylight_factor, is_underwater]
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var t_array: texture_2d_array<f32>;

@group(0) @binding(2)
var s_array: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal_idx: u32,
    @location(2) uv: vec2<f32>,
    @location(3) tex_layer: u32,
    @location(4) ao: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) tex_layer: u32,
    @location(2) light_factor: f32,
    @location(3) world_dist: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = in.position;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    out.uv = in.uv;
    out.tex_layer = in.tex_layer;

    // Directional face shading (Minecraft classic directional brightness)
    var face_light: f32 = 0.8;
    if (in.normal_idx == 0u) {
        face_light = 0.85; // +X
    } else if (in.normal_idx == 1u) {
        face_light = 0.75; // -X
    } else if (in.normal_idx == 2u) {
        face_light = 1.00; // +Y (Top)
    } else if (in.normal_idx == 3u) {
        face_light = 0.50; // -Y (Bottom)
    } else if (in.normal_idx == 4u) {
        face_light = 0.80; // +Z
    } else if (in.normal_idx == 5u) {
        face_light = 0.70; // -Z
    }

    let daylight = camera.fog_info.z;
    // Emissive / torch-lit vertices have ao > 1.0, keeping caves & night builds illuminated!
    let emit_light = clamp((in.ao - 0.85) * 1.4, 0.0, 1.15);
    let env_light = max(daylight, emit_light);
    let base_ao = min(in.ao, 1.05);

    out.light_factor = face_light * base_ao * env_light;
    out.world_dist = distance(world_pos, camera.cam_pos.xyz);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_array, s_array, in.uv, in.tex_layer);

    // Alpha cutout for transparent blocks/items (leaves, glass, tools, torches)
    if (tex_color.a < 0.1) {
        discard;
    }

    let lit_color = tex_color.rgb * in.light_factor;

    // Distance fog blending
    let fog_min = camera.fog_info.x;
    let fog_max = camera.fog_info.y;
    let fog_factor = clamp((in.world_dist - fog_min) / (fog_max - fog_min), 0.0, 1.0);

    let final_color = mix(lit_color, camera.fog_color.rgb, fog_factor);
    return vec4<f32>(final_color, tex_color.a);
}
