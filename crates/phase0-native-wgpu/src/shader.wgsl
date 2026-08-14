struct Uniforms {
  view_proj: mat4x4<f32>,
  model: mat4x4<f32>,
  light_dir: vec4<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var material_texture: texture_2d<f32>;
@group(0) @binding(2) var material_sampler: sampler;

struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
};
struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) normal: vec3<f32>,
  @location(1) uv: vec2<f32>,
};

@vertex fn vs_main(input: VertexInput) -> VertexOutput {
  var output: VertexOutput;
  let world = uniforms.model * vec4<f32>(input.position, 1.0);
  output.position = uniforms.view_proj * world;
  output.normal = normalize((uniforms.model * vec4<f32>(input.normal, 0.0)).xyz);
  output.uv = input.uv;
  return output;
}

@fragment fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
  let base = textureSample(material_texture, material_sampler, input.uv).rgb;
  let diffuse = max(dot(normalize(input.normal), normalize(-uniforms.light_dir.xyz)), 0.0);
  let lighting = 0.18 + diffuse * 0.82;
  return vec4<f32>(base * lighting, 1.0);
}

