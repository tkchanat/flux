// Vertex shader
struct CameraUniform {
  view: mat4x4<f32>,
  projection: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct PushConstant {
  model: mat4x4<f32>,
}
var<push_constant> pc: PushConstant;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) texcoord: vec2<f32>,
}

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) normal: vec3<f32>,
}

@vertex
fn vs_main(
  model: VertexInput,
) -> VertexOutput {
  var out: VertexOutput;
  out.clip_position = camera.projection * camera.view * pc.model * vec4<f32>(model.position, 1.0);
  out.normal = normalize(model.position);
  return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
  let color = max(dot(light_dir, in.normal), 0.0);
  return vec4<f32>(color, color, color, 1.0);
}