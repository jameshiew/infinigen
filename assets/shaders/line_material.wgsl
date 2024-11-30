// From lines example of Bevy - https://github.com/bevyengine/bevy/blob/release-0.14.2/assets/shaders/line_material.wgsl 
#import bevy_pbr::forward_io::VertexOutput

struct LineMaterial {
    color: vec4<f32>,
};

@group(2) @binding(0) var<uniform> material: LineMaterial;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return material.color;
}
