use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Debug, Clone)]
pub struct VerifyRequest<'a> {
    pub output: &'a Path,
    pub fixtures: &'a Path,
    pub case_id: &'a str,
    pub engine: &'a str,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration_seconds: f64,
    pub expected_audio_tracks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub passed: bool,
    pub issues: Vec<String>,
    pub video: Value,
    pub audio: Value,
    pub timestamps: Value,
    pub decoded: Value,
    pub semantics: Value,
}

pub fn verify(request: &VerifyRequest<'_>) -> VerificationReport {
    match verify_inner(request) {
        Ok(report) => report,
        Err(error) => VerificationReport {
            passed: false,
            issues: vec![format!("verifier failure: {error:#}")],
            video: Value::Null,
            audio: Value::Null,
            timestamps: Value::Null,
            decoded: Value::Null,
            semantics: Value::Null,
        },
    }
}

fn verify_inner(request: &VerifyRequest<'_>) -> Result<VerificationReport> {
    if !request.output.is_file() {
        bail!("output file does not exist: {}", request.output.display());
    }
    let metadata = fs::metadata(request.output)?;
    if metadata.len() == 0 {
        bail!("output file is empty");
    }
    let probe = command_json(
        "ffprobe",
        &[
            "-v",
            "error",
            "-count_frames",
            "-show_streams",
            "-show_format",
            "-show_frames",
            "-select_streams",
            "v:0",
            "-of",
            "json",
        ],
        request.output,
    )?;
    let streams = probe["streams"].as_array().cloned().unwrap_or_default();
    let video_streams: Vec<&Value> = streams
        .iter()
        .filter(|stream| stream["codec_type"] == "video")
        .collect();
    let audio_probe = command_json(
        "ffprobe",
        &[
            "-v",
            "error",
            "-count_frames",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ],
        request.output,
    )?;
    let all_streams = audio_probe["streams"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let audio_streams: Vec<&Value> = all_streams
        .iter()
        .filter(|stream| stream["codec_type"] == "audio")
        .collect();
    let mut issues = Vec::new();
    if video_streams.len() != 1 {
        issues.push(format!(
            "expected exactly one video track, found {}",
            video_streams.len()
        ));
    }
    if audio_streams.len() != request.expected_audio_tracks {
        issues.push(format!(
            "expected {} audio track(s), found {}",
            request.expected_audio_tracks,
            audio_streams.len()
        ));
    }
    let video = video_streams
        .first()
        .copied()
        .cloned()
        .unwrap_or(Value::Null);
    check_video_metadata(request, &video, &audio_probe, &mut issues);

    let frames = probe["frames"].as_array().cloned().unwrap_or_default();
    let expected_frames = (request.duration_seconds * f64::from(request.fps)).round() as usize;
    let timestamp_report = verify_timestamps(&frames, request.fps, expected_frames, &mut issues);

    let decoded = decode_analysis(request, expected_frames, &mut issues)?;
    let audio = verify_audio(request, audio_streams.first().copied(), &mut issues)?;
    let semantics = verify_semantics(request, &decoded, &audio, &mut issues);
    if semantics["passed"] != true {
        issues.push("case-specific semantic verification failed".to_owned());
    }

    Ok(VerificationReport {
        passed: issues.is_empty(),
        issues,
        video: json!({
            "container": audio_probe["format"]["format_name"],
            "file_bytes": metadata.len(),
            "track_count": video_streams.len(),
            "width": video["width"],
            "height": video["height"],
            "codec": video["codec_name"],
            "pixel_format": video["pix_fmt"],
            "frame_rate": video["avg_frame_rate"],
            "time_base": video["time_base"],
            "start_time": video["start_time"],
            "frame_count": video["nb_read_frames"],
            "duration_seconds": audio_probe["format"]["duration"],
        }),
        audio,
        timestamps: timestamp_report,
        decoded,
        semantics,
    })
}

fn check_video_metadata(
    request: &VerifyRequest<'_>,
    video: &Value,
    probe: &Value,
    issues: &mut Vec<String>,
) {
    if video["width"].as_u64() != Some(u64::from(request.width))
        || video["height"].as_u64() != Some(u64::from(request.height))
    {
        issues.push(format!(
            "dimension mismatch: expected {}x{}, found {}x{}",
            request.width, request.height, video["width"], video["height"]
        ));
    }
    if video["codec_name"] != "h264" {
        issues.push(format!(
            "expected H.264 video, found {}",
            video["codec_name"]
        ));
    }
    if video["pix_fmt"] != "yuv420p" {
        issues.push(format!("expected yuv420p, found {}", video["pix_fmt"]));
    }
    if rational(&video["avg_frame_rate"]) != Some(f64::from(request.fps)) {
        issues.push(format!(
            "frame-rate mismatch: expected {}, found {}",
            request.fps, video["avg_frame_rate"]
        ));
    }
    match rational(&video["time_base"]) {
        Some(time_base) if time_base > 0.0 && time_base <= 1.0 / f64::from(request.fps) => {}
        actual => issues.push(format!(
            "invalid video time base: expected a positive tick no coarser than 1/{}, found {actual:?}",
            request.fps
        )),
    }
    let start = number(&video["start_time"]).unwrap_or_default();
    if start.abs() > 0.02 {
        issues.push(format!("unexpected video start time {start:.6}s"));
    }
    let duration = number(&probe["format"]["duration"]);
    if duration.is_none_or(|value| (value - request.duration_seconds).abs() > 0.12) {
        issues.push(format!(
            "duration mismatch: expected {:.3}s, found {duration:?}",
            request.duration_seconds
        ));
    }
    let expected_frames = (request.duration_seconds * f64::from(request.fps)).round() as u64;
    let actual_frames = number(&video["nb_read_frames"]).map(|value| value as u64);
    if actual_frames != Some(expected_frames) {
        issues.push(format!(
            "frame-count mismatch: expected {expected_frames}, found {actual_frames:?}"
        ));
    }
}

fn verify_timestamps(
    frames: &[Value],
    fps: u32,
    expected: usize,
    issues: &mut Vec<String>,
) -> Value {
    let timestamps: Vec<f64> = frames
        .iter()
        .filter_map(|frame| {
            number(&frame["best_effort_timestamp_time"]).or_else(|| number(&frame["pts_time"]))
        })
        .collect();
    let expected_step = 1.0 / f64::from(fps);
    let mut non_monotonic = 0;
    let mut unexpected_gaps = 0;
    let mut duplicate_timestamps = 0;
    let mut maximum_gap = 0.0_f64;
    for pair in timestamps.windows(2) {
        let gap = pair[1] - pair[0];
        maximum_gap = maximum_gap.max(gap);
        if gap < -1e-7 {
            non_monotonic += 1;
        } else if gap.abs() < 1e-7 {
            duplicate_timestamps += 1;
        } else if (gap - expected_step).abs() > 0.0015 {
            unexpected_gaps += 1;
        }
    }
    if timestamps.len() != expected {
        issues.push(format!(
            "timestamp count mismatch: expected {expected}, found {}",
            timestamps.len()
        ));
    }
    if non_monotonic > 0 || duplicate_timestamps > 0 || unexpected_gaps > 0 {
        issues.push(format!(
            "timestamp integrity failure: non_monotonic={non_monotonic}, duplicates={duplicate_timestamps}, gaps={unexpected_gaps}"
        ));
    }
    json!({
        "count": timestamps.len(),
        "monotonic": non_monotonic == 0,
        "duplicate_count": duplicate_timestamps,
        "unexpected_gap_count": unexpected_gaps,
        "maximum_gap_seconds": maximum_gap,
        "expected_step_seconds": expected_step,
    })
}

fn decode_analysis(
    request: &VerifyRequest<'_>,
    expected_frames: usize,
    issues: &mut Vec<String>,
) -> Result<Value> {
    let output = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(request.output)
        .args([
            "-vf",
            "scale=64:36:flags=area,format=rgb24",
            "-fps_mode",
            "passthrough",
            "-f",
            "rawvideo",
            "-",
        ])
        .output()
        .context("decode analysis frames")?;
    ensure_success("ffmpeg decoded analysis", &output)?;
    let frame_bytes = 64 * 36 * 3;
    if output.stdout.len() % frame_bytes != 0 {
        issues.push("decoded RGB analysis stream has a partial frame".to_owned());
    }
    let frames: Vec<&[u8]> = output.stdout.chunks_exact(frame_bytes).collect();
    if frames.len() != expected_frames {
        issues.push(format!(
            "decoded frame count mismatch: expected {expected_frames}, found {}",
            frames.len()
        ));
    }
    let mut black_frames = 0;
    let mut longest_frozen_run = 1_usize;
    let mut current_frozen_run = 1_usize;
    let mut hashes = Vec::new();
    let selected = selected_indices(expected_frames);
    let mut checkpoint_metrics = Vec::new();
    for (index, frame) in frames.iter().enumerate() {
        let mean_luma = frame
            .chunks_exact(3)
            .map(|pixel| {
                0.2126 * f64::from(pixel[0])
                    + 0.7152 * f64::from(pixel[1])
                    + 0.0722 * f64::from(pixel[2])
            })
            .sum::<f64>()
            / (64.0 * 36.0);
        if mean_luma < 2.0 {
            black_frames += 1;
        }
        if index > 0 {
            let difference = mean_absolute_difference(frames[index - 1], frame);
            if difference < 0.01 {
                current_frozen_run += 1;
                longest_frozen_run = longest_frozen_run.max(current_frozen_run);
            } else {
                current_frozen_run = 1;
            }
        }
    }
    for frame_index in selected {
        if let Some(frame) = frames.get(frame_index) {
            hashes
                .push(json!({"frame": frame_index, "sha256": hex::encode(Sha256::digest(frame))}));
            checkpoint_metrics.push(frame_metrics(frame_index, frame));
        }
    }
    let selected_pair_mae = checkpoint_metrics
        .windows(2)
        .filter_map(|pair| {
            let left = pair[0]["frame"].as_u64()? as usize;
            let right = pair[1]["frame"].as_u64()? as usize;
            Some(mean_absolute_difference(
                frames.get(left)?,
                frames.get(right)?,
            ))
        })
        .collect::<Vec<_>>();
    let black_ratio = if frames.is_empty() {
        1.0
    } else {
        black_frames as f64 / frames.len() as f64
    };
    if black_ratio > 0.05 {
        issues.push(format!("black-frame ratio {black_ratio:.4} exceeds 0.05"));
    }
    let maximum_allowed_frozen = maximum_allowed_frozen_run(request.case_id, request.fps);
    if longest_frozen_run > maximum_allowed_frozen {
        issues.push(format!(
            "frozen-frame run {longest_frozen_run} exceeds {maximum_allowed_frozen}"
        ));
    }
    let media_oracle = if request.case_id == "media-frame-sampling" {
        verify_media_oracle(request, expected_frames, issues)?
    } else {
        Value::Null
    };
    Ok(json!({
        "decoded_frame_count": frames.len(),
        "selected_frame_hashes": hashes,
        "checkpoint_metrics": checkpoint_metrics,
        "selected_pair_mae": selected_pair_mae,
        "black_frame_count": black_frames,
        "black_frame_ratio": black_ratio,
        "longest_frozen_frame_run": longest_frozen_run,
        "analysis_resolution": "64x36",
        "media_oracle": media_oracle,
    }))
}

fn maximum_allowed_frozen_run(case_id: &str, fps: u32) -> usize {
    let seconds = match case_id {
        "media-frame-sampling" => return 2,
        "3d-scene" => 2,
        // Full-profile chart and mixed workloads intentionally hold their final
        // chart/CTA state for up to four seconds after the authored motion ends.
        "chart-diagram" | "mixed-2d-3d" => 5,
        _ => 4,
    };
    fps as usize * seconds
}

fn verify_media_oracle(
    request: &VerifyRequest<'_>,
    expected_frames: usize,
    issues: &mut Vec<String>,
) -> Result<Value> {
    let oracle_path = request.fixtures.join("color-coded-oracle.json");
    let oracle: Value = serde_json::from_slice(
        &fs::read(&oracle_path).with_context(|| format!("read {}", oracle_path.display()))?,
    )?;
    let oracle_frames = oracle["frames"]
        .as_array()
        .context("oracle frames missing")?;
    let source_output = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(request.fixtures.join("color-coded-source.mp4"))
        .args([
            "-vf",
            "crop=2:2:159:89,format=rgb24",
            "-fps_mode",
            "passthrough",
            "-f",
            "rawvideo",
            "-",
        ])
        .output()?;
    ensure_success("ffmpeg source-oracle decode", &source_output)?;
    let source_colors = source_output
        .stdout
        .chunks_exact(12)
        .map(|frame| [frame[0], frame[1], frame[2]])
        .collect::<Vec<_>>();
    if source_colors.len() != oracle_frames.len() {
        issues.push(format!(
            "source oracle decode count mismatch: JSON has {}, source has {}",
            oracle_frames.len(),
            source_colors.len()
        ));
    }
    let x = request.width / 2;
    let y = request.height / 2;
    let filter = format!(
        "crop=2:2:{}:{},format=rgb24",
        x.saturating_sub(1),
        y.saturating_sub(1)
    );
    let output = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(request.output)
        .args([
            "-vf",
            &filter,
            "-fps_mode",
            "passthrough",
            "-f",
            "rawvideo",
            "-",
        ])
        .output()?;
    ensure_success("ffmpeg media oracle decode", &output)?;
    let decoded: Vec<&[u8]> = output.stdout.chunks_exact(12).collect();
    let mut first_mismatch = None;
    let mut classified = Vec::with_capacity(decoded.len());
    let mut maximum_expected_distance_squared = 0.0_f64;
    const MAXIMUM_EXPECTED_DISTANCE_SQUARED: f64 = 2_500.0;
    for (index, frame) in decoded.iter().enumerate() {
        let observed = [frame[0], frame[1], frame[2]];
        let nearest = source_colors
            .iter()
            .enumerate()
            .map(|(source_frame, rgb)| {
                let distance = (0..3)
                    .map(|channel| {
                        let delta = f64::from(observed[channel]) - f64::from(rgb[channel]);
                        delta * delta
                    })
                    .sum::<f64>();
                (source_frame, distance)
            })
            .min_by(|left, right| left.1.total_cmp(&right.1))
            .context("oracle is empty")?;
        classified.push(nearest.0);
        let expected = index + 15;
        let expected_rgb = source_colors
            .get(expected)
            .with_context(|| format!("source oracle does not contain frame {expected}"))?;
        let expected_distance_squared = (0..3)
            .map(|channel| {
                let delta = f64::from(observed[channel]) - f64::from(expected_rgb[channel]);
                delta * delta
            })
            .sum::<f64>();
        maximum_expected_distance_squared =
            maximum_expected_distance_squared.max(expected_distance_squared);
        if expected_distance_squared > MAXIMUM_EXPECTED_DISTANCE_SQUARED && first_mismatch.is_none()
        {
            first_mismatch = Some(json!({
                "output_frame": index,
                "expected_source_frame": expected,
                "expected_source_rgb": expected_rgb,
                "expected_distance_squared": expected_distance_squared,
                "maximum_expected_distance_squared": MAXIMUM_EXPECTED_DISTANCE_SQUARED,
                "classified_source_frame": nearest.0,
                "observed_rgb": observed,
                "nearest_distance_squared": nearest.1,
                "neighbors": classified.iter().rev().take(3).copied().collect::<Vec<_>>(),
            }));
        }
    }
    if decoded.len() != expected_frames {
        issues.push(format!(
            "media oracle decoded {} frames, expected {expected_frames}",
            decoded.len()
        ));
    }
    if let Some(mismatch) = &first_mismatch {
        issues.push(format!("media frame oracle mismatch: {mismatch}"));
    }
    Ok(json!({
        "mapping": "output frame n -> source frame n + 15",
        "checked_frame_count": decoded.len(),
        "expected_frame_count": expected_frames,
        "all_frames_match": first_mismatch.is_none() && decoded.len() == expected_frames,
        "first_mismatch": first_mismatch,
        "maximum_expected_distance_squared": maximum_expected_distance_squared,
        "per_frame_distance_threshold_squared": MAXIMUM_EXPECTED_DISTANCE_SQUARED,
    }))
}

fn verify_audio(
    request: &VerifyRequest<'_>,
    stream: Option<&Value>,
    issues: &mut Vec<String>,
) -> Result<Value> {
    if request.expected_audio_tracks == 0 {
        return Ok(json!({"expected": false, "track_count": 0}));
    }
    let Some(stream) = stream else {
        issues.push("expected audio track is missing".to_owned());
        return Ok(json!({"expected": true, "track_count": 0}));
    };
    let sample_rate = number(&stream["sample_rate"]).unwrap_or_default() as u32;
    if sample_rate != 48_000 {
        issues.push(format!("expected 48000 Hz audio, found {sample_rate}"));
    }
    let channels = stream["channels"].as_u64().unwrap_or_default();
    if channels == 0 || channels > 2 {
        issues.push(format!("unexpected audio channel count {channels}"));
    }
    let output = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(request.output)
        .args([
            "-map", "0:a:0", "-ac", "1", "-ar", "48000", "-f", "f32le", "-",
        ])
        .output()?;
    ensure_success("ffmpeg audio decode", &output)?;
    let samples: Vec<f32> = output
        .stdout
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().expect("four bytes")))
        .collect();
    let expected_samples = (request.duration_seconds * 48_000.0).round() as usize;
    let tolerance = 2_048_usize;
    if samples.len().abs_diff(expected_samples) > tolerance {
        issues.push(format!("decoded audio sample count {} outside AAC-aware tolerance of {expected_samples}±{tolerance}", samples.len()));
    }
    let peak = samples
        .iter()
        .fold(0.0_f32, |value, sample| value.max(sample.abs()));
    if peak > 1.001 {
        issues.push(format!("decoded audio clipping peak {peak:.5}"));
    }
    let base_duration = if request.case_id == "mixed-2d-3d" {
        15.0
    } else {
        8.0
    };
    let scale = request.duration_seconds / base_duration;
    let signatures = if request.case_id == "mixed-2d-3d" {
        [(0.1, 1.8, 440.0), (6.1, 7.8, 660.0), (12.1, 13.8, 880.0)]
    } else {
        [(0.1, 1.8, 440.0), (3.1, 4.8, 660.0), (6.1, 7.8, 880.0)]
    };
    let mut windows = Vec::new();
    let mut present = 0;
    for (start, end, frequency) in signatures {
        let start = start * scale;
        let end = end * scale;
        let energy = rms(&samples, start, end);
        let magnitude = goertzel(&samples, start, end, frequency);
        let ok = energy > 0.005 && magnitude > 0.001;
        present += usize::from(ok);
        windows.push(json!({"start_seconds":start,"end_seconds":end,"expected_frequency_hz":frequency,"rms":energy,"goertzel":magnitude,"present":ok}));
    }
    let silence = if request.case_id == "mixed-2d-3d" {
        [
            (2.0 * scale + 0.08, 6.0 * scale),
            (8.0 * scale + 0.08, 12.0 * scale),
        ]
    } else {
        [
            (2.0 * scale + 0.08, 3.0 * scale),
            (5.0 * scale + 0.08, 6.0 * scale),
        ]
    };
    let silence_windows: Vec<Value> = silence
        .into_iter()
        .map(|(start, end)| json!({"start_seconds":start,"end_seconds":end,"rms":rms(&samples,start,end)}))
        .collect();
    let overlap_detected = silence_windows
        .iter()
        .any(|window| window["rms"].as_f64().unwrap_or(1.0) > 0.003);
    if present != 3 {
        issues.push(format!(
            "expected three audio frequency signatures, detected {present}"
        ));
    }
    for window in &silence_windows {
        if window["rms"].as_f64().unwrap_or(1.0) > 0.003 {
            issues.push(format!(
                "unexpected audio energy in silence window {window}"
            ));
        }
    }
    let boundaries: Vec<f64> = if request.case_id == "mixed-2d-3d" {
        vec![2.0, 6.0, 8.0, 12.0, 14.0]
    } else {
        vec![2.0, 3.0, 5.0, 6.0]
    }
    .into_iter()
    .map(|time| time * scale)
    .collect();
    let seam_jumps: Vec<Value> = boundaries
        .iter()
        .map(|time| json!({"time_seconds":time,"maximum_jump":maximum_jump(&samples,*time)}))
        .collect();
    if seam_jumps
        .iter()
        .any(|value| value["maximum_jump"].as_f64().unwrap_or(1.0) > 0.12)
    {
        issues.push("audio seam jump exceeds 0.12".to_owned());
    }
    Ok(json!({
        "expected": true,
        "track_count": 1,
        "codec": stream["codec_name"],
        "sample_rate": sample_rate,
        "channels": channels,
        "channel_layout": stream["channel_layout"],
        "decoded_sample_count": samples.len(),
        "expected_sample_count": expected_samples,
        "aac_delay_padding_tolerance_samples": tolerance,
        "decoded_duration_seconds": samples.len() as f64 / 48_000.0,
        "peak": peak,
        "frequency_windows": windows,
        "silence_windows": silence_windows,
        "seam_jumps": seam_jumps,
        "clip_signature_count": present,
        "overlap_detected": overlap_detected,
    }))
}

fn verify_semantics(
    request: &VerifyRequest<'_>,
    decoded: &Value,
    audio: &Value,
    issues: &mut Vec<String>,
) -> Value {
    let hashes = decoded["selected_frame_hashes"]
        .as_array()
        .map_or(0, Vec::len);
    if hashes < 3 {
        issues.push(format!("too few decoded semantic checkpoints: {hashes}"));
    }
    let changes = decoded["selected_pair_mae"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let maximum_change = changes.iter().filter_map(Value::as_f64).fold(0.0, f64::max);
    let metrics = decoded["checkpoint_metrics"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let maximum_stddev = metrics
        .iter()
        .filter_map(|metric| metric["luma_stddev"].as_f64())
        .fold(0.0, f64::max);
    let maximum_saturated_ratio = metrics
        .iter()
        .filter_map(|metric| metric["saturated_pixel_ratio"].as_f64())
        .fold(0.0, f64::max);
    let actual_visual = hashes >= 3 && maximum_change > 0.05 && maximum_stddev > 4.0;
    let mut checks = vec![json!({
        "name":"decoded-visual-evidence",
        "passed":actual_visual,
        "selected_hash_count":hashes,
        "maximum_selected_frame_mae":maximum_change,
        "maximum_luma_stddev":maximum_stddev,
        "maximum_saturated_pixel_ratio":maximum_saturated_ratio
    })];
    match request.case_id {
        "typography-layout" => checks.push(json!({"name":"decoded-typography-structure","passed":maximum_change>0.1 && maximum_stddev>8.0,"engine":request.engine})),
        "vector-effects" => checks.push(json!({"name":"decoded-vector-motion-and-color","passed":maximum_change>0.1 && maximum_saturated_ratio>0.01,"fixture":"vector-fixture.svg"})),
        "chart-diagram" => checks.push(json!({"name":"decoded-chart-growth-and-structure","passed":maximum_change>0.1 && maximum_stddev>8.0})),
        "media-frame-sampling" => checks.push(json!({"name":"complete-media-oracle","passed":decoded["media_oracle"]["all_frames_match"]})),
        "audio-captions" => checks.push(json!({"name":"three-clip-audio","passed":audio["clip_signature_count"]==3,"caption_intervals":3})),
        "3d-scene" => checks.push(json!({"name":"decoded-3d-motion-and-texture","passed":maximum_change>0.1 && maximum_saturated_ratio>0.005})),
        "mixed-2d-3d" => checks.push(json!({"name":"decoded-mixed-scene-diversity","passed":maximum_change>1.0 && maximum_saturated_ratio>0.005 && audio["clip_signature_count"]==3,"audio_signatures":audio["clip_signature_count"]})),
        _ => issues.push(format!("unknown case-specific verifier contract: {}", request.case_id)),
    }
    json!({"case_id":request.case_id,"checks":checks,"passed":checks.iter().all(|check|check["passed"]==true)})
}

fn frame_metrics(frame_index: usize, frame: &[u8]) -> Value {
    let luma = frame
        .chunks_exact(3)
        .map(|pixel| {
            0.2126 * f64::from(pixel[0])
                + 0.7152 * f64::from(pixel[1])
                + 0.0722 * f64::from(pixel[2])
        })
        .collect::<Vec<_>>();
    let mean = luma.iter().sum::<f64>() / luma.len().max(1) as f64;
    let stddev = (luma.iter().map(|value| (value - mean).powi(2)).sum::<f64>()
        / luma.len().max(1) as f64)
        .sqrt();
    let saturated = frame
        .chunks_exact(3)
        .filter(|pixel| {
            let maximum = *pixel.iter().max().unwrap_or(&0);
            let minimum = *pixel.iter().min().unwrap_or(&0);
            maximum.saturating_sub(minimum) > 64 && maximum > 96
        })
        .count();
    json!({
        "frame":frame_index,
        "mean_luma":mean,
        "luma_stddev":stddev,
        "saturated_pixel_ratio":saturated as f64 / (64.0 * 36.0)
    })
}

fn command_json(program: &str, args: &[&str], path: &Path) -> Result<Value> {
    let output = Command::new(program)
        .args(args)
        .arg(path)
        .output()
        .with_context(|| format!("start {program}"))?;
    ensure_success(program, &output)?;
    serde_json::from_slice(&output.stdout).with_context(|| format!("parse {program} JSON"))
}

fn ensure_success(label: &str, output: &Output) -> Result<()> {
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "{label} failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn selected_indices(count: usize) -> Vec<usize> {
    if count == 0 {
        return Vec::new();
    }
    let candidates = [
        0,
        count / 10,
        count / 3,
        count / 2,
        count * 2 / 3,
        count.saturating_sub(2),
        count - 1,
    ];
    let mut output = candidates
        .into_iter()
        .filter(|index| *index < count)
        .collect::<Vec<_>>();
    output.sort_unstable();
    output.dedup();
    output
}

fn mean_absolute_difference(left: &[u8], right: &[u8]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(a, b)| (f64::from(*a) - f64::from(*b)).abs())
        .sum::<f64>()
        / left.len().max(1) as f64
}

fn number(value: &Value) -> Option<f64> {
    value.as_f64().or_else(|| value.as_str()?.parse().ok())
}

fn rational(value: &Value) -> Option<f64> {
    let text = value.as_str()?;
    let (numerator, denominator) = text.split_once('/')?;
    let denominator = denominator.parse::<f64>().ok()?;
    if denominator == 0.0 {
        None
    } else {
        Some(numerator.parse::<f64>().ok()? / denominator)
    }
}

fn rms(samples: &[f32], start_seconds: f64, end_seconds: f64) -> f64 {
    let start = (start_seconds * 48_000.0).round().max(0.0) as usize;
    let end = (end_seconds * 48_000.0).round().max(0.0) as usize;
    let window = &samples[start.min(samples.len())..end.min(samples.len())];
    (window
        .iter()
        .map(|sample| f64::from(*sample).powi(2))
        .sum::<f64>()
        / window.len().max(1) as f64)
        .sqrt()
}

fn goertzel(samples: &[f32], start_seconds: f64, end_seconds: f64, frequency: f64) -> f64 {
    let start = (start_seconds * 48_000.0).round().max(0.0) as usize;
    let end = (end_seconds * 48_000.0).round().max(0.0) as usize;
    let window = &samples[start.min(samples.len())..end.min(samples.len())];
    if window.is_empty() {
        return 0.0;
    }
    let coefficient = 2.0 * (2.0 * std::f64::consts::PI * frequency / 48_000.0).cos();
    let mut previous = 0.0;
    let mut previous_two = 0.0;
    for sample in window {
        let current = f64::from(*sample) + coefficient * previous - previous_two;
        previous_two = previous;
        previous = current;
    }
    (previous_two.powi(2) + previous.powi(2) - coefficient * previous * previous_two).sqrt()
        / window.len() as f64
}

fn maximum_jump(samples: &[f32], center_seconds: f64) -> f64 {
    let start = ((center_seconds - 0.04).max(0.0) * 48_000.0).round() as usize;
    let end = ((center_seconds + 0.04) * 48_000.0).round() as usize;
    samples[start.min(samples.len())..end.min(samples.len())]
        .windows(2)
        .map(|pair| f64::from((pair[1] - pair[0]).abs()))
        .fold(0.0, f64::max)
}

pub fn hash_file(path: &Path) -> Result<String> {
    Ok(hex::encode(Sha256::digest(fs::read(path)?)))
}

pub fn artifact_manifest_path(result_path: &Path) -> PathBuf {
    result_path.with_file_name("verification-manifest.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request<'a>(output: &'a Path, fixtures: &'a Path, audio: usize) -> VerifyRequest<'a> {
        VerifyRequest {
            output,
            fixtures,
            case_id: "typography-layout",
            engine: "test",
            width: 640,
            height: 360,
            fps: 30,
            duration_seconds: 1.0,
            expected_audio_tracks: audio,
        }
    }

    #[test]
    fn selected_hash_indices_cover_boundaries_and_middle() {
        let indices = selected_indices(180);
        assert_eq!(indices.first(), Some(&0));
        assert_eq!(indices.last(), Some(&179));
        assert!(indices.contains(&90));
    }

    #[test]
    fn frozen_run_limits_allow_authored_full_profile_holds_without_becoming_unbounded() {
        assert_eq!(maximum_allowed_frozen_run("media-frame-sampling", 30), 2);
        assert_eq!(maximum_allowed_frozen_run("3d-scene", 30), 60);
        assert_eq!(maximum_allowed_frozen_run("chart-diagram", 30), 150);
        assert_eq!(maximum_allowed_frozen_run("mixed-2d-3d", 30), 150);
        assert!(maximum_allowed_frozen_run("mixed-2d-3d", 30) < 15 * 30);
    }

    #[test]
    fn timestamp_verifier_rejects_gaps_and_duplicates() {
        let frames = vec![
            json!({"best_effort_timestamp_time":"0.000000"}),
            json!({"best_effort_timestamp_time":"0.033333"}),
            json!({"best_effort_timestamp_time":"0.033333"}),
            json!({"best_effort_timestamp_time":"0.133333"}),
        ];
        let mut issues = Vec::new();
        let report = verify_timestamps(&frames, 30, 4, &mut issues);
        assert_eq!(report["duplicate_count"], 1);
        assert_eq!(report["unexpected_gap_count"], 1);
        assert!(!issues.is_empty());
    }

    #[test]
    fn frequency_detector_distinguishes_expected_tone() {
        let samples: Vec<f32> = (0..48_000)
            .map(|index| (2.0 * std::f32::consts::PI * 440.0 * index as f32 / 48_000.0).sin() * 0.2)
            .collect();
        assert!(goertzel(&samples, 0.0, 1.0, 440.0) > 0.05);
        assert!(goertzel(&samples, 0.0, 1.0, 880.0) < 0.01);
    }

    #[test]
    fn metadata_checks_reject_dimensions_codec_and_pixel_format() {
        let mut issues = Vec::new();
        check_video_metadata(
            &request(Path::new("output.mp4"), Path::new("fixtures"), 0),
            &json!({"width":320,"height":180,"codec_name":"vp9","pix_fmt":"yuv444p","avg_frame_rate":"30/1","time_base":"1/90000","start_time":"0","nb_read_frames":"30"}),
            &json!({"format":{"duration":"1.0"}}),
            &mut issues,
        );
        assert!(issues
            .iter()
            .any(|issue| issue.contains("dimension mismatch")));
        assert!(issues.iter().any(|issue| issue.contains("expected H.264")));
        assert!(issues
            .iter()
            .any(|issue| issue.contains("expected yuv420p")));
    }

    #[test]
    fn missing_audio_is_a_verifier_failure() {
        let mut issues = Vec::new();
        let report = verify_audio(
            &request(Path::new("output.mp4"), Path::new("fixtures"), 1),
            None,
            &mut issues,
        )
        .expect("missing audio report");
        assert_eq!(report["track_count"], 0);
        assert!(issues.iter().any(|issue| issue.contains("missing")));
    }

    #[test]
    fn installed_ffmpeg_accepts_per_output_fps_mode() {
        let output = Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=size=16x16:rate=30:duration=0.04",
                "-frames:v",
                "1",
                "-fps_mode",
                "passthrough",
                "-f",
                "null",
                "-",
            ])
            .output()
            .expect("run ffmpeg fps-mode compatibility probe");
        assert!(
            output.status.success(),
            "ffmpeg rejected -fps_mode passthrough: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn corrupt_output_is_rejected_without_panicking() {
        let output = std::env::temp_dir().join(format!(
            "cinekernel-corrupt-output-{}.mp4",
            std::process::id()
        ));
        fs::write(&output, b"not an mp4").expect("write corrupt fixture");
        let report = verify(&request(&output, Path::new("fixtures"), 0));
        assert!(!report.passed);
        assert!(report.issues[0].contains("verifier failure"));
        fs::remove_file(output).expect("cleanup corrupt fixture");
    }
}
