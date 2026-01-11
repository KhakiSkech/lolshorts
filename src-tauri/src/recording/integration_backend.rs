//! FFmpeg gdigrab 기반 화면 캡처 모듈
//!
//! Windows: gdigrab (FFmpeg 내장 화면 캡처)
//! macOS: avfoundation
//! Linux: x11grab
//!
//! 세그먼트 기반 순환 버퍼로 60초 리플레이 윈도우 유지

use crate::storage::GameMetadata;
use crate::utils::ffmpeg::get_ffmpeg_path;
use anyhow::{Context, Result};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

/// 녹화 설정
#[derive(Debug, Clone)]
pub struct RecordingConfig {
    pub fps: u32,
    pub bitrate: u32,
    pub resolution: (u32, u32),
    pub encoder: VideoEncoder,
    pub output_dir: PathBuf,
    pub buffer_duration_secs: u64,
    pub segment_duration_secs: u64,
    pub audio_config: Option<crate::recording::audio::AudioConfig>,
    /// 캡처할 창 제목 (None이면 전체 화면)
    pub capture_target: Option<String>,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            fps: 60,
            bitrate: 15_000_000, // 15 Mbps
            resolution: (1920, 1080),
            encoder: VideoEncoder::H264, // H264가 더 호환성 좋음
            output_dir: PathBuf::from("./recordings"),
            buffer_duration_secs: 60, // 1분 버퍼
            segment_duration_secs: 10, // 10초 세그먼트
            audio_config: Some(crate::recording::audio::AudioConfig::default()),
            // 다국어 창 제목 지원: 한국어 > 영어(TM) > 영어 순서로 시도
            // 실제 gdigrab은 첫 번째 매칭만 시도하므로 가장 유력한 후보 사용
            // TODO: 이후 windows-rs 등을 사용하여 실행 중인 창 제목 열거 후 자동 감지 필요
            capture_target: Some(get_league_capture_title()),
        }
    }
}

/// 다국어 지원을 위한 LoL 클라이언트 창 제목 후보 목록
fn get_league_capture_title() -> String {
    const TITLES: &[&str] = &[
        "리그 오브 레전드", // 한국어 (우선순위)
        "League of Legends (TM) Client", // 영어 (TM 포함, 최신 버전)
        "League of Legends",        // 영어 (일반)
        "英雄联盟",                    // 중국어 (간체)
        "聯盟爭霸",                  // 중국어 (번체)
        "리그오브레전드",             // 한국어 (공백 없음)
        "LeagueClientUx",             // LCU 내부 창 이름 (Fallback)
    ];

    // 현재 구현에서는 첫 번째 후보(한국어)를 기본값으로 사용
    // 향후 창 열거 기능 구현 시 실제 존재하는 제목으로 동적 교체 가능
    TITLES[0].to_string()
}

/// 크로스 플랫폼 지원 비디오 인코더 옵션
#[derive(Debug, Clone, Copy)]
pub enum VideoEncoder {
    H264,
    H265,
}

impl VideoEncoder {
    /// 현재 플랫폼에 최적화된 FFmpeg 인코더 이름 반환 (하드웨어 가속)
    pub fn to_ffmpeg_name(&self) -> &'static str {
        #[cfg(target_os = "windows")]
        {
            match self {
                VideoEncoder::H264 => "h264_nvenc",
                VideoEncoder::H265 => "hevc_nvenc",
            }
        }

        #[cfg(target_os = "macos")]
        {
            match self {
                VideoEncoder::H264 => "h264_videotoolbox",
                VideoEncoder::H265 => "hevc_videotoolbox",
            }
        }

        #[cfg(target_os = "linux")]
        {
            match self {
                VideoEncoder::H264 => "libx264",
                VideoEncoder::H265 => "libx265",
            }
        }
    }

    /// 호환성을 위한 소프트웨어 폴백 인코더 반환
    pub fn to_software_fallback(&self) -> &'static str {
        match self {
            VideoEncoder::H264 => "libx264",
            VideoEncoder::H265 => "libx265",
        }
    }

    pub fn to_extension(&self) -> &'static str {
        match self {
            VideoEncoder::H264 => "mp4",
            VideoEncoder::H265 => "mp4",
        }
    }
}

/// 녹화 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingStatus {
    Idle,
    Buffering,
    Recording,
    Processing,
    Error,
}

impl Default for RecordingStatus {
    fn default() -> Self {
        RecordingStatus::Idle
    }
}

/// windows-capture에서 캡처된 프레임 데이터 (호환성 유지)
#[derive(Clone)]
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub timestamp: std::time::SystemTime,
}

impl CapturedFrame {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
            timestamp: std::time::SystemTime::now(),
        }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn size_mb(&self) -> f64 {
        self.size() as f64 / (1024.0 * 1024.0)
    }
}

/// FFmpeg 세그먼트 기반 녹화기
/// 세그먼트 muxer를 사용하여 순환 버퍼 구현
pub struct SegmentRecorder {
    config: RecordingConfig,
    ffmpeg_process: Option<tokio::process::Child>,
    segment_dir: PathBuf,
    segment_pattern: String,
    start_time: Option<Instant>,
}

impl SegmentRecorder {
    pub fn new(config: RecordingConfig) -> Result<Self> {
        let segment_dir = config.output_dir.join("segments");
        std::fs::create_dir_all(&segment_dir)?;

        let segment_pattern = segment_dir
            .join("segment_%03d.mp4")
            .to_string_lossy()
            .to_string();

        Ok(Self {
            config,
            ffmpeg_process: None,
            segment_dir,
            segment_pattern,
            start_time: None,
        })
    }

    /// 세그먼트 기반 녹화 시작
    pub async fn start(&mut self) -> Result<()> {
        if self.ffmpeg_process.is_some() {
            anyhow::bail!("이미 녹화가 진행 중입니다");
        }

        // 기존 세그먼트 파일 정리
        self.cleanup_old_segments()?;

        let ffmpeg_path = get_ffmpeg_path()
            .context("FFmpeg를 찾을 수 없습니다")?;

        let mut cmd = tokio::process::Command::new(&ffmpeg_path);

        // 공통 설정
        cmd.arg("-y"); // 파일 덮어쓰기

        // 플랫폼별 입력 설정
        #[cfg(target_os = "windows")]
        {
            // gdigrab으로 화면 캡처
            cmd.arg("-f").arg("gdigrab");
            cmd.arg("-framerate").arg(self.config.fps.to_string());

            // 캡처 대상 설정
            if let Some(ref target) = self.config.capture_target {
                // 특정 창 캡처 시도
                cmd.arg("-i").arg(format!("title={}", target));
            } else {
                // 전체 화면 캡처
                cmd.arg("-i").arg("desktop");
            }

            // 오디오 입력 (Windows WASAPI)
            if let Some(ref audio_cfg) = self.config.audio_config {
                if audio_cfg.record_system_audio {
                    // 시스템 오디오 루프백 캡처
                    cmd.arg("-f").arg("dshow");
                    cmd.arg("-i").arg("audio=virtual-audio-capturer");
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            cmd.arg("-f").arg("avfoundation");
            cmd.arg("-framerate").arg(self.config.fps.to_string());
            cmd.arg("-i").arg("1:0"); // 화면:오디오
        }

        #[cfg(target_os = "linux")]
        {
            cmd.arg("-f").arg("x11grab");
            cmd.arg("-framerate").arg(self.config.fps.to_string());
            cmd.arg("-video_size").arg(format!("{}x{}",
                self.config.resolution.0,
                self.config.resolution.1
            ));
            cmd.arg("-i").arg(":0.0");
        }

        // 인코딩 설정
        let encoder = self.config.encoder.to_ffmpeg_name();
        cmd.arg("-c:v").arg(encoder);
        cmd.arg("-b:v").arg(format!("{}k", self.config.bitrate / 1000));
        cmd.arg("-preset").arg("ultrafast");
        cmd.arg("-tune").arg("zerolatency");
        cmd.arg("-g").arg((self.config.fps * 2).to_string()); // GOP = 2초
        cmd.arg("-keyint_min").arg(self.config.fps.to_string());
        cmd.arg("-sc_threshold").arg("0");
        cmd.arg("-pix_fmt").arg("yuv420p");

        // 오디오 인코딩
        if self.config.audio_config.is_some() {
            cmd.arg("-c:a").arg("aac");
            cmd.arg("-b:a").arg("128k");
        }

        // 세그먼트 출력 설정
        let max_segments = (self.config.buffer_duration_secs / self.config.segment_duration_secs) as i32;

        cmd.arg("-f").arg("segment");
        cmd.arg("-segment_time").arg(self.config.segment_duration_secs.to_string());
        cmd.arg("-segment_wrap").arg(max_segments.to_string()); // 순환 버퍼
        cmd.arg("-segment_format").arg("mp4");
        cmd.arg("-reset_timestamps").arg("1");
        cmd.arg("-strftime").arg("0");
        cmd.arg(&self.segment_pattern);

        // stderr 캡처 (디버깅용)
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        {
            // use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        info!("FFmpeg 녹화 시작: {:?}", cmd);

        let child = cmd.spawn().context("FFmpeg 프로세스 시작 실패")?;
        self.ffmpeg_process = Some(child);
        self.start_time = Some(Instant::now());

        info!("세그먼트 기반 녹화 시작됨: {}", self.segment_dir.display());
        Ok(())
    }

    /// 녹화 중지
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.ffmpeg_process.take() {
            // FFmpeg에 종료 신호 전송
            #[cfg(target_os = "windows")]
            {
                // Windows에서는 프로세스 종료
                let _ = process.kill().await;
            }

            #[cfg(not(target_os = "windows"))]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                if let Some(pid) = process.id() {
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGINT);
                }
            }

            // 프로세스 종료 대기 (5초 타임아웃)
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                process.wait(),
            )
            .await
            {
                Ok(Ok(_)) => {
                    info!("FFmpeg 녹화 정상 중지됨");
                }
                Ok(Err(e)) => {
                    warn!("FFmpeg 프로세스 종료 오류: {}", e);
                }
                Err(_) => {
                    warn!("FFmpeg 프로세스 종료 타임아웃 (5초) - 강제 종료 시도");
                    let _ = process.kill().await;
                }
            }
        }

        self.start_time = None;
        Ok(())
    }

    /// 특정 시간 범위의 클립 저장
    pub async fn save_clip(
        &self,
        output_path: &PathBuf,
        start_offset_secs: f64,
        duration_secs: f64,
    ) -> Result<PathBuf> {
        info!("클립 저장 시작: {} (offset: {}s, duration: {}s)",
            output_path.display(), start_offset_secs, duration_secs);

        // 현재 세그먼트 파일 목록 가져오기
        let segments = self.get_sorted_segments()?;

        if segments.is_empty() {
            anyhow::bail!("저장된 세그먼트가 없습니다");
        }

        // concat 파일 생성
        let concat_file = self.segment_dir.join("concat_list.txt");
        let mut concat_content = String::new();
        for segment in &segments {
            concat_content.push_str(&format!("file '{}'\n", segment.display()));
        }
        tokio::fs::write(&concat_file, &concat_content).await?;

        // FFmpeg로 클립 추출
        let ffmpeg_path = get_ffmpeg_path()?;
        let mut cmd = tokio::process::Command::new(&ffmpeg_path);

        cmd.arg("-y");
        cmd.arg("-f").arg("concat");
        cmd.arg("-safe").arg("0");
        cmd.arg("-i").arg(&concat_file);
        cmd.arg("-ss").arg(start_offset_secs.to_string());
        cmd.arg("-t").arg(duration_secs.to_string());
        cmd.arg("-c").arg("copy"); // 재인코딩 없이 복사
        cmd.arg("-movflags").arg("+faststart");
        cmd.arg(output_path);

        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::piped());

        #[cfg(target_os = "windows")]
        {
            // use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        let output = cmd.output().await?;

        // concat 파일 정리
        let _ = tokio::fs::remove_file(&concat_file).await;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("클립 저장 실패: {}", stderr);
            anyhow::bail!("클립 저장 실패: {}", stderr);
        }

        info!("클립 저장 완료: {}", output_path.display());
        Ok(output_path.clone())
    }

    /// 세그먼트 파일 정렬된 목록 반환
    fn get_sorted_segments(&self) -> Result<Vec<PathBuf>> {
        let mut segments: Vec<PathBuf> = std::fs::read_dir(&self.segment_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .map(|ext| ext == "mp4")
                    .unwrap_or(false)
            })
            .collect();

        // 수정 시간 기준 정렬
        segments.sort_by(|a, b| {
            let a_time = std::fs::metadata(a)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_time = std::fs::metadata(b)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            a_time.cmp(&b_time)
        });

        Ok(segments)
    }

    /// 기존 세그먼트 파일 정리
    fn cleanup_old_segments(&self) -> Result<()> {
        if self.segment_dir.exists() {
            for entry in std::fs::read_dir(&self.segment_dir)? {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map(|e| e == "mp4").unwrap_or(false) {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
        Ok(())
    }

    /// 녹화 진행 시간 반환
    pub fn get_elapsed_secs(&self) -> f64 {
        self.start_time
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0)
    }

    /// 녹화 중인지 확인
    #[allow(dead_code)]
    pub fn is_recording(&self) -> bool {
        self.ffmpeg_process.is_some()
    }
}

impl Drop for SegmentRecorder {
    fn drop(&mut self) {
        if let Some(mut process) = self.ffmpeg_process.take() {
            // Attempt to kill the process
            #[cfg(target_os = "windows")]
            {
                let _ = process.start_kill();
            }
            #[cfg(not(target_os = "windows"))]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                if let Some(pid) = process.id() {
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGINT);
                }
            }
            // We can't wait here because drop is sync, but killing should be enough
            info!("SegmentRecorder dropped, FFmpeg process killed");
        }
    }
}

/// 메인 녹화 매니저 (WindowsCaptureRecorder 호환)
pub struct WindowsCaptureRecorder {
    config: RecordingConfig,
    status: Arc<TokioRwLock<RecordingStatus>>,
    segment_recorder: Arc<TokioRwLock<SegmentRecorder>>,
    current_game: Arc<TokioRwLock<Option<GameMetadata>>>,
    start_time: Arc<TokioRwLock<Option<Instant>>>,
    total_frames: Arc<TokioRwLock<u64>>,
}

impl WindowsCaptureRecorder {
    pub async fn new(config: RecordingConfig) -> Result<Self> {
        std::fs::create_dir_all(&config.output_dir)?;

        let segment_recorder = SegmentRecorder::new(config.clone())?;

        Ok(Self {
            config,
            status: Arc::new(TokioRwLock::new(RecordingStatus::Idle)),
            segment_recorder: Arc::new(TokioRwLock::new(segment_recorder)),
            current_game: Arc::new(TokioRwLock::new(None)),
            start_time: Arc::new(TokioRwLock::new(None)),
            total_frames: Arc::new(TokioRwLock::new(0)),
        })
    }

    /// 녹화 시작
    pub async fn start_recording(&mut self) -> Result<()> {
        let mut status = self.status.write().await;

        if *status != RecordingStatus::Idle {
            anyhow::bail!("이미 녹화가 진행 중입니다");
        }

        *status = RecordingStatus::Buffering;
        drop(status);

        // 세그먼트 레코더 시작
        {
            let mut recorder = self.segment_recorder.write().await;
            recorder.start().await?;
        }

        *self.start_time.write().await = Some(Instant::now());
        *self.total_frames.write().await = 0;
        *self.status.write().await = RecordingStatus::Recording;

        info!("녹화가 성공적으로 시작되었습니다");
        Ok(())
    }

    /// 녹화 중지
    /// Note: Processing 상태는 start_recording을 차단하여 race condition 방지
    pub async fn stop_recording(&mut self) -> Result<PathBuf> {
        // 1. 상태 확인 및 Processing으로 전환 (atomic operation)
        {
            let mut status = self.status.write().await;
            match *status {
                RecordingStatus::Idle => {
                    anyhow::bail!("진행 중인 녹화가 없습니다");
                }
                RecordingStatus::Processing => {
                    anyhow::bail!("이미 녹화 중지가 진행 중입니다");
                }
                RecordingStatus::Recording | RecordingStatus::Buffering => {
                    *status = RecordingStatus::Processing;
                }
                RecordingStatus::Error => {
                    // 에러 상태에서도 정리 허용
                    *status = RecordingStatus::Processing;
                }
            }
        }
        // Lock released - Processing 상태가 다른 작업 차단

        // 2. 세그먼트 레코더 중지 (에러 시 상태 복구)
        let stop_result = {
            let mut recorder = self.segment_recorder.write().await;
            recorder.stop().await
        };

        // 3. 에러 발생 시 상태를 Error로, 성공 시 정리
        match stop_result {
            Ok(()) => {
                *self.start_time.write().await = None;
                *self.status.write().await = RecordingStatus::Idle;
            }
            Err(e) => {
                *self.status.write().await = RecordingStatus::Error;
                return Err(e);
            }
        }

        // 세그먼트 디렉토리 반환
        let output_path = self.config.output_dir.join("segments");
        info!("녹화 중지됨: {}", output_path.display());
        Ok(output_path)
    }

    /// 마지막 N초 클립 저장
    pub async fn save_last_seconds(&self, seconds: u32) -> Result<PathBuf> {
        let status = self.status.read().await;
        if *status != RecordingStatus::Recording && *status != RecordingStatus::Buffering {
            anyhow::bail!("녹화가 진행 중이 아닙니다");
        }
        drop(status);

        let recorder = self.segment_recorder.read().await;
        let elapsed = recorder.get_elapsed_secs();

        // 시작 오프셋 계산 (버퍼 끝에서 N초 전)
        let start_offset = if elapsed > seconds as f64 {
            elapsed - seconds as f64
        } else {
            0.0
        };

        let duration = seconds as f64;

        // 출력 파일명 생성
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("clip_{}_{}.mp4", timestamp, seconds);
        let output_path = self.config.output_dir.join("clips").join(&filename);

        // clips 디렉토리 생성
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        recorder.save_clip(&output_path, start_offset, duration).await
    }

    /// 특정 이벤트 시점 기준 클립 저장 (상대 시간 적용)
    pub async fn save_event_clip(
        &self,
        _event_time_secs: f64, // Not used in circular buffer mode
        pre_secs: f64,
        post_secs: f64,
        clip_id: &str,
    ) -> Result<PathBuf> {
        let status = self.status.read().await;
        if *status != RecordingStatus::Recording && *status != RecordingStatus::Buffering {
            anyhow::bail!("녹화가 진행 중이 아닙니다");
        }
        drop(status);

        let recorder = self.segment_recorder.read().await;
        let elapsed = recorder.get_elapsed_secs();
        let total_duration = pre_secs + post_secs;

        // Circular Buffer Logic:
        // We capture the LAST (Pre + Post) seconds from NOW.
        // Assumes the caller has waited for 'post_secs' to elapse before calling this.
        
        let start_offset = if elapsed > total_duration {
            elapsed - total_duration
        } else {
            0.0
        };

        // 출력 파일명 생성
        let filename = format!("{}.mp4", clip_id);
        let output_path = self.config.output_dir.join("clips").join(&filename);

        // clips 디렉토리 생성
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        recorder.save_clip(&output_path, start_offset, total_duration).await
    }

    /// 프레임 처리 (호환성 유지 - 실제로는 사용하지 않음)
    pub async fn process_frame(&self, _frame: CapturedFrame) -> Result<()> {
        // gdigrab 모드에서는 FFmpeg가 직접 캡처하므로 이 메서드는 사용하지 않음
        debug!("process_frame은 gdigrab 모드에서 사용되지 않습니다");
        Ok(())
    }

    pub async fn get_status(&self) -> RecordingStatus {
        *self.status.read().await
    }

    pub async fn get_stats(&self) -> RecordingStats {
        let total_frames = *self.total_frames.read().await;
        let start_time = *self.start_time.read().await;
        let uptime = start_time.map(|t| t.elapsed().as_secs_f64());

        RecordingStats {
            total_frames,
            uptime_seconds: uptime.unwrap_or(0.0),
            current_fps: match uptime {
                Some(u) if u > 0.0 => total_frames as f64 / u,
                _ => self.config.fps as f64,
            },
        }
    }

    pub async fn set_current_game(&self, game: Option<GameMetadata>) {
        *self.current_game.write().await = game;
    }

    pub async fn get_current_game(&self) -> Option<GameMetadata> {
        self.current_game.read().await.clone()
    }

    /// 캡처 대상 설정 (게임 창 제목)
    pub async fn set_capture_target(&mut self, target: Option<String>) {
        self.config.capture_target = target;
    }

    /// 현재 설정 반환
    pub fn get_config(&self) -> &RecordingConfig {
        &self.config
    }
}

/// 녹화 통계
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecordingStats {
    pub total_frames: u64,
    pub uptime_seconds: f64,
    pub current_fps: f64,
}

/// 시스템 상태 정보
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct SystemStatus {
    pub recording_status: RecordingStatus,
    pub stats: RecordingStats,
    pub current_game: Option<GameMetadata>,
}

/// 메인 녹화 매니저 타입 정의
pub type RecordingManager = WindowsCaptureRecorder;

/// Video settings for recording initialization
#[derive(Debug, Clone)]
pub struct VideoSettingsConfig {
    pub resolution: (u32, u32),
    pub fps: u32,
    pub bitrate: u32,
    pub use_h265: bool,
}

impl Default for VideoSettingsConfig {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            fps: 60,
            bitrate: 15_000_000,
            use_h265: false,
        }
    }
}

/// 녹화 백엔드 초기화 (전체 옵션 지원)
///
/// # Arguments
/// * `output_dir` - 출력 디렉토리
/// * `audio_config` - 오디오 설정 (선택)
/// * `video_config` - 비디오 설정 (선택, 기본값: 1920x1080 60fps)
pub async fn initialize_recording_backend_full(
    output_dir: PathBuf,
    audio_config: Option<crate::recording::audio::AudioConfig>,
    video_config: Option<VideoSettingsConfig>,
) -> Result<RecordingManager> {
    let video = video_config.unwrap_or_default();

    let encoder = if video.use_h265 {
        VideoEncoder::H265
    } else {
        VideoEncoder::H264
    };

    let config = RecordingConfig {
        fps: video.fps,
        bitrate: video.bitrate,
        resolution: video.resolution,
        encoder,
        output_dir,
        audio_config,
        ..Default::default()
    };

    info!(
        "Initializing recording: {}x{} @ {}fps, {} Mbps, {:?}",
        config.resolution.0,
        config.resolution.1,
        config.fps,
        config.bitrate / 1_000_000,
        config.encoder
    );

    WindowsCaptureRecorder::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_recorder_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecordingConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let recorder = WindowsCaptureRecorder::new(config).await;
        assert!(recorder.is_ok());

        let recorder = recorder.unwrap();
        assert_eq!(recorder.get_status().await, RecordingStatus::Idle);
    }

    #[tokio::test]
    async fn test_video_encoder() {
        #[cfg(target_os = "windows")]
        {
            assert_eq!(VideoEncoder::H264.to_ffmpeg_name(), "h264_nvenc");
            assert_eq!(VideoEncoder::H265.to_ffmpeg_name(), "hevc_nvenc");
        }
        assert_eq!(VideoEncoder::H264.to_extension(), "mp4");
    }

    #[tokio::test]
    async fn test_config_default() {
        let config = RecordingConfig::default();
        assert_eq!(config.fps, 60);
        assert_eq!(config.bitrate, 15_000_000);
        assert_eq!(config.resolution, (1920, 1080));
        assert_eq!(config.buffer_duration_secs, 60);
        assert_eq!(config.segment_duration_secs, 10);
    }

    #[test]
    fn test_segment_recorder_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = RecordingConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let recorder = SegmentRecorder::new(config);
        assert!(recorder.is_ok());
    }
}
