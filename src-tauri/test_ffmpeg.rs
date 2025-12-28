use std::process::Command;

fn main() {
    println!("Testing FFmpeg availability...");

    // Test multiple possible FFmpeg paths
    let ffmpeg_paths = vec![
        "./binaries/ffmpeg.exe",
        "./binaries/ffmpeg-x86_64-pc-windows-msvc.exe",
        "./binaries/ffmpeg",
    ];

    for ffmpeg_path in ffmpeg_paths {
        println!("Testing: {}", ffmpeg_path);

        if std::path::Path::new(ffmpeg_path).exists() {
            println!("FFmpeg found at: {}", ffmpeg_path);

            match Command::new(ffmpeg_path).arg("-version").output() {
                Ok(output) => {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout);
                        println!("FFmpeg version: {}", version.lines().next().unwrap_or("Unknown"));
                        println!("✅ FFmpeg is working!");
                        return;
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        println!("❌ FFmpeg execution failed: {}", stderr);
                    }
                }
                Err(e) => {
                    println!("❌ Failed to execute FFmpeg: {}", e);
                }
            }
        } else {
            println!("❌ FFmpeg not found at: {}", ffmpeg_path);
        }
    }

    println!("❌ No working FFmpeg found!");
}