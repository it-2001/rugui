

struct VertexInput {
    @builtin(vertex_index) index: u32
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@group(0)@binding(0) var<uniform> screen_size: vec2<f32>;

@group(1)@binding(0) var<uniform> center: vec2<f32>; 
@group(1)@binding(1) var<uniform> size: vec2<f32>;
@group(1)@binding(2) var<uniform> rotation: f32;

@group(2)@binding(0) var<uniform> color: vec4<f32>;


@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    var position = vertex_position(in.index);
    var scale = vec2(size.x * position.x, size.y * position.y);
    var cosin = vec2(cos(rotation), sin(rotation));
    var rotation = vec2(
        scale.x * cosin.r - scale.y * cosin.g,
        scale.x * cosin.g + scale.y * cosin.r
    );
    var new_position = center + rotation;
    var screen_space = new_position / screen_size * 2.0 - 1.0;
    var invert_y = vec2(screen_space.x, -screen_space.y);
    
    out.position = vec4<f32>(invert_y, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0)vec4<f32> {
    return color;
}



fn vertex_position(vertex_index: u32) -> vec2<f32> {
    // x: + + - - - +
    // y: + - - - + +
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u))-0.5;
}