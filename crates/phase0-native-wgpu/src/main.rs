use anyhow::{bail, Context, Result};
use bytemuck::{Pod, Zeroable};
use clap::{Parser, ValueEnum};
use glam::{Mat4, Vec3};
use image::{ImageBuffer, Rgba};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::time::Instant;
use wgpu::util::DeviceExt;

#[derive(Parser)]
struct Args {
    #[arg(long = "case")]
    case_id: String,
    #[arg(long, value_enum)]
    profile: Profile,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, value_enum, default_value_t = FrameOrder::Sequential)]
    frame_order: FrameOrder,
}

#[derive(Clone, Copy, ValueEnum)]
enum Profile {
    Smoke,
    Full,
}

#[derive(Clone, Copy, ValueEnum)]
enum FrameOrder {
    Sequential,
    Shuffled,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    light_dir: [f32; 4],
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
        normal: [0.0, 0.0, 1.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0],
        normal: [0.0, 0.0, -1.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0, -1.0],
        normal: [0.0, 0.0, -1.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0],
        normal: [0.0, 0.0, -1.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0],
        normal: [0.0, 0.0, -1.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, -1.0],
        normal: [-1.0, 0.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
        normal: [-1.0, 0.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0],
        normal: [-1.0, 0.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0],
        normal: [1.0, 0.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0],
        normal: [1.0, 0.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
        normal: [1.0, 0.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
        normal: [0.0, 1.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0],
        normal: [0.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0],
        normal: [0.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, -1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, -1.0, 1.0],
        normal: [0.0, -1.0, 0.0],
        uv: [0.0, 0.0],
    },
];
const INDICES: &[u16] = &[
    0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15, 16, 17, 18,
    16, 18, 19, 20, 21, 22, 20, 22, 23,
];

fn main() -> Result<()> {
    pollster::block_on(run())
}

async fn run() -> Result<()> {
    let args = Args::parse();
    if !["3d-scene", "mixed-2d-3d"].contains(&args.case_id.as_str()) {
        bail!("case {} is unsupported by native-wgpu", args.case_id);
    }
    let width = env_u32(
        "CINEKERNEL_WIDTH",
        match args.profile {
            Profile::Smoke => 640,
            Profile::Full => 1920,
        },
    );
    let height = env_u32(
        "CINEKERNEL_HEIGHT",
        match args.profile {
            Profile::Smoke => 360,
            Profile::Full => 1080,
        },
    );
    let duration = env_f64("CINEKERNEL_DURATION_SECONDS", 1.0);
    let fps = env_u32("CINEKERNEL_FPS", 30);
    let frame_count = (duration * f64::from(fps)).round() as u32;
    let fixtures = env::var_os("CINEKERNEL_FIXTURES")
        .map(PathBuf::from)
        .context("CINEKERNEL_FIXTURES is required")?;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .context("no wgpu adapter available")?;
    let info = adapter.get_info();
    let software_fallback = matches!(info.device_type, wgpu::DeviceType::Cpu);
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("cinekernel-phase0"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .await?;
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let color = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen-color"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let color_view = color.create_view(&wgpu::TextureViewDescriptor::default());
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth.create_view(&wgpu::TextureViewDescriptor::default());
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("lit-cube"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("vertices"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("indices"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });
    let texture_image = image::open(fixtures.join("texture.png"))?.to_rgba8();
    let texture_size = wgpu::Extent3d {
        width: texture_image.width(),
        height: texture_image.height(),
        depth_or_array_layers: 1,
    };
    let material = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("generated-material"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &material,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        texture_image.as_raw(),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * texture_image.width()),
            rows_per_image: Some(texture_image.height()),
        },
        texture_size,
    );
    let material_view = material.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("uniforms"),
        size: std::mem::size_of::<Uniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("scene-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("scene-bind-group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&material_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("pipeline-layout"),
        bind_group_layouts: &[&layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("lit-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x2],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview: None,
    });
    let padded_bytes_per_row = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: u64::from(padded_bytes_per_row) * u64::from(height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let frames = args
        .output
        .parent()
        .context("output has no parent")?
        .join("frames-native-wgpu");
    fs::create_dir_all(&frames)?;
    let started = Instant::now();
    let mut evaluation_order: Vec<u32> = (0..frame_count).collect();
    if matches!(args.frame_order, FrameOrder::Shuffled) {
        evaluation_order.sort_by_key(|frame| frame.wrapping_mul(2_654_435_761));
    }
    for frame in evaluation_order {
        let time = f64::from(frame) / f64::from(fps);
        let phase = (time / duration).clamp(0.0, 1.0) as f32;
        let camera_angle = phase * 0.8 - 0.4;
        let eye = Vec3::new(camera_angle.sin() * 4.5, 2.6, camera_angle.cos() * 4.5);
        let view = Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y);
        let projection = Mat4::perspective_rh(
            45_f32.to_radians(),
            width as f32 / height as f32,
            0.1,
            100.0,
        );
        let model = Mat4::from_rotation_y(phase * std::f32::consts::TAU)
            * Mat4::from_rotation_x(phase * 0.7);
        let uniforms = Uniforms {
            view_proj: (projection * view).to_cols_array_2d(),
            model: model.to_cols_array_2d(),
            light_dir: [-0.6, -1.0, -0.4, 0.0],
        };
        queue.write_buffer(&uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("offscreen-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.015,
                            g: 0.03,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &color,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &readback,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(Some(encoder.finish()));
        let slice = readback.slice(..);
        let (sender, receiver) = mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device.poll(wgpu::Maintain::Wait);
        receiver.recv()?.context("map readback")?;
        let mapped = slice.get_mapped_range();
        let mut pixels = vec![0_u8; (width * height * 4) as usize];
        for row in 0..height as usize {
            let source = &mapped[row * padded_bytes_per_row as usize
                ..row * padded_bytes_per_row as usize + (width * 4) as usize];
            pixels[row * (width * 4) as usize..(row + 1) * (width * 4) as usize]
                .copy_from_slice(source);
        }
        drop(mapped);
        readback.unmap();
        composite_overlay(
            &mut pixels,
            width,
            height,
            phase,
            args.case_id == "mixed-2d-3d",
        );
        let image: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width, height, pixels).context("invalid readback size")?;
        image.save(frames.join(format!("frame-{frame:06}.png")))?;
    }
    let frame_ms = started.elapsed().as_secs_f64() * 1000.0;
    let encode_started = Instant::now();
    let mut ffmpeg = Command::new("ffmpeg");
    ffmpeg
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-framerate",
            &fps.to_string(),
            "-i",
        ])
        .arg(frames.join("frame-%06d.png"));
    if args.case_id == "mixed-2d-3d" {
        ffmpeg
            .args(["-i"])
            .arg(fixtures.join("tone-windows.wav"))
            .args(["-shortest", "-c:a", "aac", "-b:a", "192k"]);
    }
    let status = ffmpeg
        .args([
            "-c:v", "libx264", "-preset", "medium", "-crf", "18", "-pix_fmt", "yuv420p",
        ])
        .arg(&args.output)
        .status()?;
    if !status.success() {
        bail!("ffmpeg encode failed with {status}");
    }
    let encode_ms = encode_started.elapsed().as_secs_f64() * 1000.0;
    fs::remove_dir_all(&frames)?;
    println!(
        "{}",
        serde_json::json!({"ok":true,"engine":"native-wgpu","case":args.case_id,"frames":frame_count,"frame_order":match args.frame_order {FrameOrder::Sequential=>"sequential",FrameOrder::Shuffled=>"shuffled"},"adapter":info.name,"backend":format!("{:?}",info.backend),"device_type":format!("{:?}",info.device_type),"driver":info.driver,"driver_info":info.driver_info,"software_fallback":software_fallback,"gpu_completion":"device.poll(Maintain::Wait)","frame_production_ms":frame_ms,"encode_ms":encode_ms})
    );
    Ok(())
}

fn composite_overlay(pixels: &mut [u8], width: u32, height: u32, progress: f32, mixed: bool) {
    let header = (height / 9).max(18);
    let progress_width = (width as f32 * progress) as u32;
    for y in 0..header {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            let alpha = if mixed { 0.72 } else { 0.45 };
            pixels[i] = ((pixels[i] as f32) * (1.0 - alpha) + (10.0 * alpha)) as u8;
            pixels[i + 1] = ((pixels[i + 1] as f32) * (1.0 - alpha) + (20.0 * alpha)) as u8;
            pixels[i + 2] = ((pixels[i + 2] as f32) * (1.0 - alpha) + (40.0 * alpha)) as u8;
        }
    }
    let start = height.saturating_sub((height / 30).max(4));
    for y in start..height {
        for x in 0..progress_width {
            let i = ((y * width + x) * 4) as usize;
            pixels[i] = 101;
            pixels[i + 1] = 214;
            pixels[i + 2] = 255;
            pixels[i + 3] = 255;
        }
    }
}
fn env_u32(name: &str, fallback: u32) -> u32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(fallback)
}
fn env_f64(name: &str, fallback: f64) -> f64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(fallback)
}
