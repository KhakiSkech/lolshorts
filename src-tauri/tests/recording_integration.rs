// Integration tests for FFmpeg-based recording system
// Tests actual FFmpeg process execution and file generation

use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use wait_timeout::ChildExt;

/// Test FFmpeg availability
#[test]
fn test_ffmpeg_available() {
    let output = std::process::Command::new("ffmpeg")
        .arg("-version")
        .output();

    assert!(
        output.is_ok(),
        "FFmpeg not found. Please install FFmpeg (see docs/FFMPEG_SETUP.md)"
    );

    let output = output.unwrap();
    assert!(
        output.status.success(),
        "FFmpeg command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let version = String::from_utf8_lossy(&output.stdout);
    assert!(
        version.contains("ffmpeg version"),
        "Unexpected FFmpeg output: {}",
        version
    );

    println!("✅ FFmpeg found: {}", version.lines().next().unwrap());
}

/// Test gdigrab (Windows GDI screen capture)
#[test]
#[cfg(target_os = "windows")]
fn test_gdigrab_available() {
    let output = std::process::Command::new("ffmpeg")
        .args(["-f", "gdigrab", "-list_devices", "true", "-i", "dummy"])
        .output();

    assert!(output.is_ok(), "FFmpeg gdigrab test failed");

    // gdigrab will exit with error when listing devices, but stderr contains device list
    let output = output.unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stderr.contains("gdigrab") || stderr.contains("desktop"),
        "gdigrab not available. Output: {}",
        stderr
    );

    println!("✅ gdigrab (screen capture) available");
}

/// Test H.265 encoder availability (hardware or software)
#[test]
fn test_h265_encoder_available() {
    let output = std::process::Command::new("ffmpeg")
        .args(["-encoders"])
        .output();

    assert!(output.is_ok(), "FFmpeg encoders check failed");

    let output = output.unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for any H.265 encoder
    let has_nvenc = stdout.contains("hevc_nvenc");
    let has_qsv = stdout.contains("hevc_qsv");
    let has_amf = stdout.contains("hevc_amf");
    let has_libx265 = stdout.contains("libx265");

    assert!(
        has_nvenc || has_qsv || has_amf || has_libx265,
        "No H.265 encoder found. At least libx265 (software) should be available."
    );

    println!("✅ H.265 encoders available:");
    if has_nvenc {
        println!("  - hevc_nvenc (NVIDIA hardware)");
    }
    if has_qsv {
        println!("  - hevc_qsv (Intel hardware)");
    }
    if has_amf {
        println!("  - hevc_amf (AMD hardware)");
    }
    if has_libx265 {
        println!("  - libx265 (software fallback)");
    }
}

/// Test short recording (5 seconds) with H.264 (faster for testing)
#[test]
#[cfg(target_os = "windows")]
fn test_short_recording() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("test_recording.mp4");

    println!("🎬 Starting 5-second test recording...");
    println!("📁 Output: {}", output_file.display());

    let mut child = std::process::Command::new("ffmpeg")
        .args([
            "-f",
            "gdigrab",
            "-framerate",
            "30",
            "-i",
            "desktop",
            "-t",
            "5", // 5 seconds
            "-c:v",
            "libx264", // H.264 for faster encoding
            "-preset",
            "ultrafast",
            "-y",
            output_file.to_str().unwrap(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start FFmpeg");

    // Wait for completion (max 10 seconds)
    let status = child
        .wait_timeout(Duration::from_secs(10))
        .expect("Failed to wait for FFmpeg");

    assert!(
        status.is_some(),
        "FFmpeg process timed out after 10 seconds"
    );

    let status = status.unwrap();
    if !status.success() {
        let stderr_output = child.wait_with_output().unwrap();
        panic!(
            "FFmpeg recording failed: {}",
            String::from_utf8_lossy(&stderr_output.stderr)
        );
    }

    // Verify output file
    assert!(
        output_file.exists(),
        "Output file not created: {}",
        output_file.display()
    );

    let metadata = std::fs::metadata(&output_file).expect("Failed to get file metadata");
    assert!(
        metadata.len() > 1000,
        "Output file too small: {} bytes",
        metadata.len()
    );

    println!("✅ Recording successful");
    println!("📊 File size: {} KB", metadata.len() / 1024);
}

/// Test H.265 hardware encoding (if available)
#[test]
#[cfg(target_os = "windows")]
fn test_h265_hardware_encoding() {
    // Try NVENC first
    let encoders = ["hevc_nvenc", "hevc_qsv", "hevc_amf", "libx265"];
    let mut working_encoder = None;

    for encoder in &encoders {
        let output = std::process::Command::new("ffmpeg")
            .args(["-encoders"])
            .output()
            .expect("FFmpeg encoders check failed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(encoder) {
            working_encoder = Some(*encoder);
            break;
        }
    }

    assert!(working_encoder.is_some(), "No H.265 encoder available");

    let encoder = working_encoder.unwrap();
    println!("🎬 Testing H.265 encoding with: {}", encoder);

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("test_h265.mp4");

    let mut child = std::process::Command::new("ffmpeg")
        .args([
            "-f",
            "gdigrab",
            "-framerate",
            "30",
            "-i",
            "desktop",
            "-t",
            "3", // 3 seconds
            "-c:v",
            encoder,
            "-preset",
            "fast",
            "-y",
            output_file.to_str().unwrap(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start FFmpeg");

    let status = child
        .wait_timeout(Duration::from_secs(10))
        .expect("Failed to wait for FFmpeg");

    assert!(status.is_some(), "FFmpeg process timed out");

    let status = status.unwrap();
    if !status.success() {
        let stderr_output = child.wait_with_output().unwrap();
        let error = String::from_utf8_lossy(&stderr_output.stderr);

        // If hardware encoder failed, it's okay (might not have GPU)
        if encoder != "libx265" {
            println!(
                "⚠️ Hardware encoder {} not available, this is okay",
                encoder
            );
            println!("   Error: {}", error);
            return;
        } else {
            panic!("Software encoder libx265 failed: {}", error);
        }
    }

    assert!(output_file.exists(), "Output file not created");

    println!("✅ H.265 encoding successful with {}", encoder);
}

/// Test segment file creation pattern
#[test]
fn test_segment_file_pattern() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Test segment naming pattern
    let segments = vec![
        temp_dir.path().join("segment_0001.mp4"),
        temp_dir.path().join("segment_0002.mp4"),
        temp_dir.path().join("segment_0003.mp4"),
    ];

    // Create dummy files
    for segment in &segments {
        std::fs::write(segment, b"test").expect("Failed to create test file");
    }

    // Verify pattern
    assert!(segments[0].exists());
    assert!(segments[1].exists());
    assert!(segments[2].exists());

    // Test sorting
    let mut paths: Vec<PathBuf> = std::fs::read_dir(temp_dir.path())
        .expect("Failed to read dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    paths.sort();

    assert_eq!(paths.len(), 3);
    assert!(paths[0].to_string_lossy().contains("0001"));
    assert!(paths[1].to_string_lossy().contains("0002"));
    assert!(paths[2].to_string_lossy().contains("0003"));

    println!("✅ Segment file pattern works correctly");
}

/// Test FFmpeg process termination
#[test]
#[cfg(target_os = "windows")]
fn test_ffmpeg_process_termination() {
    println!("🎬 Testing FFmpeg process termination...");

    // Start a long recording
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("test_termination.mp4");

    let mut child = std::process::Command::new("ffmpeg")
        .args([
            "-f",
            "gdigrab",
            "-framerate",
            "30",
            "-i",
            "desktop",
            "-t",
            "60", // 60 seconds (but we'll kill it early)
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-y",
            output_file.to_str().unwrap(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start FFmpeg");

    // Wait 2 seconds
    std::thread::sleep(Duration::from_secs(2));

    // Kill the process
    println!("🛑 Terminating FFmpeg process...");
    child.kill().expect("Failed to kill FFmpeg");

    let status = child.wait().expect("Failed to wait for process");

    // Process should be killed (not successful exit)
    assert!(!status.success(), "Process should have been killed");

    println!("✅ Process termination works correctly");
}
