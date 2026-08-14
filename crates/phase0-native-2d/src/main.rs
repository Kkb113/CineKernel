use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use tiny_skia::{
    Color, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, Point, Rect, SpreadMode,
    Transform,
};

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

fn main() -> Result<()> {
    let args = Args::parse();
    if !["typography-layout", "vector-effects", "chart-diagram"].contains(&args.case_id.as_str()) {
        bail!("case {} is unsupported by native-2d", args.case_id);
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
    let frames = args
        .output
        .parent()
        .context("output has no parent")?
        .join("frames-native-2d");
    fs::create_dir_all(&frames)?;
    let svg = fs::read(fixtures.join("vector-fixture.svg"))?;
    let tree = resvg::usvg::Tree::from_data(&svg, &resvg::usvg::Options::default())?;
    let started = Instant::now();
    let mut evaluation_order: Vec<u32> = (0..frame_count).collect();
    if matches!(args.frame_order, FrameOrder::Shuffled) {
        evaluation_order.sort_by_key(|frame| frame.wrapping_mul(2_654_435_761));
    }
    for frame in evaluation_order {
        let exact_time = f64::from(frame) / f64::from(fps);
        render_frame(
            &args.case_id,
            width,
            height,
            exact_time,
            duration,
            &tree,
            &frames.join(format!("frame-{frame:06}.png")),
        )?;
    }
    let frame_ms = started.elapsed().as_secs_f64() * 1000.0;
    let encode_started = Instant::now();
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-framerate",
            &fps.to_string(),
            "-i",
        ])
        .arg(frames.join("frame-%06d.png"))
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
        serde_json::json!({"ok":true,"engine":"native-2d","case":args.case_id,"frames":frame_count,"frame_order":match args.frame_order {FrameOrder::Sequential=>"sequential",FrameOrder::Shuffled=>"shuffled"},"frame_production_ms":frame_ms,"encode_ms":encode_ms,"stack":["tiny-skia 0.11.4","resvg 0.44.0"]})
    );
    Ok(())
}

fn render_frame(
    case_id: &str,
    width: u32,
    height: u32,
    time: f64,
    duration: f64,
    tree: &resvg::usvg::Tree,
    output: &Path,
) -> Result<()> {
    let mut pixmap = Pixmap::new(width, height).context("allocate pixmap")?;
    let progress = (time / duration).clamp(0.0, 1.0) as f32;
    let background = Paint {
        shader: LinearGradient::new(
            Point::from_xy(0.0, 0.0),
            Point::from_xy(width as f32, height as f32),
            vec![
                GradientStop::new(0.0, Color::from_rgba8(5, 12, 28, 255)),
                GradientStop::new(1.0, Color::from_rgba8(35, 19, 68, 255)),
            ],
            SpreadMode::Pad,
            Transform::identity(),
        )
        .context("create background gradient")?,
        ..Paint::default()
    };
    pixmap.fill_rect(
        Rect::from_xywh(0.0, 0.0, width as f32, height as f32).context("frame rect")?,
        &background,
        Transform::identity(),
        None,
    );
    match case_id {
        "typography-layout" => typography(&mut pixmap, progress),
        "vector-effects" => vector(&mut pixmap, progress, tree),
        "chart-diagram" => chart(&mut pixmap, progress),
        _ => unreachable!(),
    }
    pixmap.save_png(output)?;
    Ok(())
}

fn typography(pixmap: &mut Pixmap, progress: f32) {
    let width = pixmap.width() as f32;
    let height = pixmap.height() as f32;
    let eased = 1.0 - (1.0 - progress).powi(3);
    let mut paint = Paint::default();
    paint.set_color_rgba8(225, 247, 255, (255.0 * eased) as u8);
    let title_width = width * 0.62 * eased;
    if let Some(rect) = Rect::from_xywh(
        width * 0.12,
        height * 0.30,
        title_width.max(0.1),
        height * 0.10,
    ) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
    paint.set_color_rgba8(101, 214, 255, (230.0 * eased) as u8);
    for line in 0..3 {
        let factor = 0.72 - line as f32 * 0.11;
        if let Some(rect) = Rect::from_xywh(
            width * 0.12,
            height * (0.50 + line as f32 * 0.07),
            width * factor * eased,
            height * 0.025,
        ) {
            pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        }
    }
}

fn vector(pixmap: &mut Pixmap, progress: f32, tree: &resvg::usvg::Tree) {
    let scale_x = pixmap.width() as f32 / tree.size().width();
    let scale_y = pixmap.height() as f32 / tree.size().height();
    resvg::render(
        tree,
        Transform::from_scale(scale_x, scale_y),
        &mut pixmap.as_mut(),
    );
    let mut paint = Paint::default();
    paint.set_color_rgba8(255, 255, 255, (80.0 * progress) as u8);
    if let Some(rect) = Rect::from_xywh(
        pixmap.width() as f32 * 0.2,
        pixmap.height() as f32 * 0.15,
        pixmap.width() as f32 * 0.6 * progress,
        pixmap.height() as f32 * 0.04,
    ) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
}

fn chart(pixmap: &mut Pixmap, progress: f32) {
    let values = [0.42_f32, 0.76, 0.61, 0.88];
    let width = pixmap.width() as f32;
    let height = pixmap.height() as f32;
    for (index, value) in values.into_iter().enumerate() {
        let local = ((progress * 1.5) - index as f32 * 0.12).clamp(0.0, 1.0);
        let bar_height = height * 0.55 * value * (1.0 - (1.0 - local).powi(3));
        let x = width * (0.16 + index as f32 * 0.19);
        let mut paint = Paint::default();
        paint.set_color_rgba8(101 + index as u8 * 18, 214 - index as u8 * 20, 255, 255);
        if let Some(rect) = Rect::from_xywh(
            x,
            height * 0.78 - bar_height,
            width * 0.11,
            bar_height.max(0.1),
        ) {
            pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        }
    }
    let mut path = PathBuilder::new();
    path.move_to(width * 0.16, height * 0.82);
    path.line_to(width * (0.16 + 0.57 * progress), height * 0.82);
    let mut stroke = Paint::default();
    stroke.set_color_rgba8(255, 255, 255, 180);
    pixmap.stroke_path(
        &path.finish().expect("chart path"),
        &stroke,
        &tiny_skia::Stroke {
            width: 3.0,
            ..Default::default()
        },
        Transform::identity(),
        None,
    );
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
