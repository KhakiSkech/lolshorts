#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::{execute_ffmpeg_command, ClipInfo, Result, VideoError, VideoProcessor};
use super::thumbnail::auto_generate_thumbnail;
use crate::storage::Storage;
use crate::utils::ffmpeg::get_ffmpeg_path;

/// 자동 편집(Auto-Edit) 구성 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEditConfig {
    /// 목표 길이 (초 단위: 60, 120, 180 등)
    pub target_duration: u32,

    /// 클립을 가져올 게임 ID 목록
    pub game_ids: Vec<String>,

    /// 수동으로 선택된 클립 ID 목록 (자동 선택 무시)
    pub selected_clip_ids: Option<Vec<i64>>,

    /// 캔버스 템플릿 설정
    pub canvas_template: Option<CanvasTemplate>,

    /// 배경 음악 설정
    pub background_music: Option<BackgroundMusic>,

    /// 오디오 믹싱 레벨 설정
    pub audio_levels: AudioLevels,

    /// 중복 클립 사용 허용 여부 (기본값: false)
    #[serde(default)]
    pub allow_duplicates: bool,
}

/// 오버레이용 캔버스 템플릿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasTemplate {
    pub id: String,
    pub name: String,
    pub background: BackgroundLayer,
    pub elements: Vec<CanvasElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BackgroundLayer {
    Color { value: String },
    Gradient { value: String },
    Image { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CanvasElement {
    Text {
        id: String,
        content: String,
        font: String,
        size: u32,
        color: String,
        outline: Option<String>,
        position: Position,
    },
    Image {
        id: String,
        path: String,
        width: u32,
        height: u32,
        position: Position,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// X 위치 (백분율 0-100)
    pub x: f32,
    /// Y 위치 (백분율 0-100)
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundMusic {
    /// MP3 파일 경로
    pub file_path: String,
    /// 영상보다 짧을 경우 반복 재생 여부
    pub loop_music: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLevels {
    /// 게임 오디오 볼륨 (0-100)
    pub game_audio: u32,
    /// 배경 음악 볼륨 (0-100)
    pub background_music: u32,
}

impl Default for AudioLevels {
    fn default() -> Self {
        Self {
            game_audio: 60,
            background_music: 80,
        }
    }
}

/// 자동 편집 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEditResult {
    /// 최종 합성된 영상 경로
    pub output_path: String,

    /// 사용된 클립 목록
    pub selected_clips: Vec<ClipInfo>,

    /// 최종 영상 길이
    pub total_duration: f64,

    /// 사용된 클립 개수
    pub clip_count: usize,
}

/// 자동 편집 진행 상황 추적
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEditProgress {
    /// 고유 작업 ID
    pub job_id: String,

    /// 현재 상태
    pub status: AutoEditStatus,

    /// 진행률 (0-100)
    pub progress: f64,

    /// 현재 단계 설명
    pub current_step: String,

    /// 경과 시간 (초)
    pub elapsed_seconds: f64,

    /// 예상 소요 시간 (초)
    pub estimated_seconds: f64,

    /// 출력 경로 (완료 시 제공)
    pub output_path: Option<String>,

    /// 오류 메시지 (실패 시 제공)
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AutoEditStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

/// YouTube Shorts 생성을 위한 자동 편집기 (Auto-Composer)
pub struct AutoComposer {
    video_processor: Arc<VideoProcessor>,
    storage: Arc<Storage>,
    progress: Arc<RwLock<Option<AutoEditProgress>>>,
}

impl AutoComposer {
    /// 새로운 AutoComposer 인스턴스 생성
    pub fn new(video_processor: Arc<VideoProcessor>, storage: Arc<Storage>) -> Self {
        Self {
            video_processor,
            storage,
            progress: Arc::new(RwLock::new(None)),
        }
    }

    /// 메인 합성 워크플로우
    pub async fn compose(&self, config: AutoEditConfig, job_id: String, is_pro: bool) -> Result<AutoEditResult> {
        info!("자동 편집 작업 시작: {} (Pro: {})", job_id, is_pro);

        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            0.0,
            "자동 편집 초기화 중...".to_string(),
        )
        .await;

        let start_time = std::time::Instant::now();

        // ... (Skip intermediate steps for brevity, keeping logic same until overlay) ...

        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            10.0,
            "DB에서 클립 불러오는 중...".to_string(),
        )
        .await;

        let all_clips = self.load_clips_from_games(&config.game_ids).await?;

        if all_clips.is_empty() {
            return Err(VideoError::NoClipsFound);
        }

        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            20.0,
            format!("{}개의 클립 중 최적의 클립 선택 중...", all_clips.len()),
        )
        .await;

        let selected_clips = self.select_clips(&all_clips, &config).await?;

        if selected_clips.is_empty() {
            return Err(VideoError::NoClipsFound);
        }

        info!(
            "합성용 클립 {}개 선택됨 (목표: {}초)",
            selected_clips.len(),
            config.target_duration
        );

        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            40.0,
            "클립 트리밍 및 전처리 중...".to_string(),
        )
        .await;

        let prepared_clips = self
            .prepare_clips(&selected_clips, config.target_duration)
            .await?;

        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            60.0,
            "클립 연결 중...".to_string(),
        )
        .await;

        let concatenated_path = self.concatenate_clips(&prepared_clips).await?;

        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            75.0,
            "캔버스 및 워터마크 적용 중...".to_string(),
        )
        .await;

        // Apply Canvas Overlay AND Watermark (if needed)
        let with_overlay = if let Some(canvas) = &config.canvas_template {
            self.apply_canvas_overlay(&concatenated_path, canvas, is_pro)
                .await? 
        } else if !is_pro {
            // No canvas, but we need watermark for Free users
            self.apply_watermark_only(&concatenated_path)
                .await?
        } else {
            concatenated_path
        };

        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            90.0,
            "오디오 믹싱 중...".to_string(),
        )
        .await;

        let final_path = if let Some(music) = &config.background_music {
            self.mix_audio(&with_overlay, music, &config.audio_levels)
                .await? 
        } else {
            with_overlay
        };

        let total_duration = self.video_processor.get_duration(&final_path).await?;

        let elapsed = start_time.elapsed().as_secs_f64();
        self.update_progress_complete(&job_id, final_path.to_string_lossy().to_string(), elapsed)
            .await;

        let result = AutoEditResult {
            output_path: final_path.to_string_lossy().to_string(),
            selected_clips: selected_clips.clone(),
            total_duration,
            clip_count: prepared_clips.len(),
        };

        let file_size = std::fs::metadata(&final_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Generate thumbnail... (omitted for brevity)
        let thumbnail_path = match auto_generate_thumbnail(&final_path, final_path.parent().unwrap_or_else(|| std::path::Path::new("."))).await {
            Ok(path) => Some(path.to_string_lossy().to_string()),
            Err(e) => {
                warn!("썸네일 생성 실패: {}", e);
                None
            }
        };

        let result_metadata = crate::storage::AutoEditResultMetadata {
            result_id: job_id.clone(),
            job_id: job_id.clone(),
            output_path: final_path.to_string_lossy().to_string(),
            thumbnail_path,
            created_at: chrono::Utc::now(),
            duration: total_duration,
            clip_count: prepared_clips.len(),
            game_ids: config.game_ids.clone(),
            target_duration: config.target_duration,
            canvas_template_name: config.canvas_template.as_ref().map(|t| t.name.clone()),
            has_background_music: config.background_music.is_some(),
            youtube_status: Some(crate::storage::YouTubeUploadStatus {
                video_id: None,
                status: crate::storage::UploadStatus::NotUploaded,
                upload_started_at: None,
                upload_completed_at: None,
                progress: 0.0,
                error: None,
            }),
            file_size_bytes: file_size,
        };

        if let Err(e) = self.storage.save_auto_edit_result(&result_metadata) {
            warn!("자동 편집 결과 메타데이터 저장 실패: {}", e);
        }

        // Increment usage count for selected clips to avoid duplicates in future
        for clip in &selected_clips {
            // We need to load fresh metadata to update usage count
            if let Ok(mut clips) = self.storage.load_clip_metadata(&clip.game_id) {
                let file_path = &clip.file_path;
                
                if let Some(target_clip) = clips.iter_mut().find(|c| &c.file_path == file_path) {
                    target_clip.usage_count += 1;
                    
                    // Save back to storage
                    if let Err(e) = self.storage.save_clip_metadata(&clip.game_id, target_clip) {
                        warn!("클립 사용 횟수 업데이트 실패 ({}): {}", file_path, e);
                    } else {
                        info!("클립 사용 횟수 증가: {} (Total: {})", file_path, target_clip.usage_count);
                    }
                }
            }
        }

        info!(
            "자동 편집 완료 ({:.2}초): {:?}",
            elapsed, result.output_path
        );

        Ok(result)
    }

    pub async fn select_clips(
        &self,
        all_clips: &[ClipInfo],
        config: &AutoEditConfig,
    ) -> Result<Vec<ClipInfo>> {
        if let Some(selected_ids) = &config.selected_clip_ids {
            let selected: Vec<ClipInfo> = all_clips
                .iter()
                .filter(|c| selected_ids.contains(&c.id))
                .cloned()
                .collect();

            if selected.is_empty() {
                return Err(VideoError::NoClipsFound);
            }

            return Ok(selected);
        }

        let mut sorted_clips = all_clips.to_vec();
        
        // Sort Logic:
        // 1. Usage Count (Ascending) - Prefer unused clips if allow_duplicates is false
        // 2. Priority (Descending) - Prefer higher priority (Penta > Quadra)
        // 3. Event Time (Ascending) - Natural game flow order (secondary)
        sorted_clips.sort_by(|a, b| {
            if !config.allow_duplicates {
                // If duplicates NOT allowed, strictly prefer unused clips
                let usage_cmp = a.usage_count.cmp(&b.usage_count);
                if usage_cmp != std::cmp::Ordering::Equal {
                    return usage_cmp;
                }
            }
            // If usage count is same (or duplicates allowed), sort by priority
            b.priority.cmp(&a.priority)
        });

        let target_duration = config.target_duration as f64;
        let buffer_duration = target_duration * 0.9;

        let mut selected = Vec::new();
        let mut total_duration = 0.0;

        for clip in &sorted_clips {
            let clip_duration = clip.duration.unwrap_or(10.0);

            if total_duration + clip_duration <= buffer_duration {
                total_duration += clip_duration;
                selected.push(clip.clone());
            }

            if total_duration >= buffer_duration {
                break;
            }
        }

        if selected.is_empty() {
            if let Some(best_clip) = sorted_clips.first() {
                selected.push(best_clip.clone());
            } else {
                return Err(VideoError::NoClipsFound);
            }
        }
        
        Ok(selected)
    }

    async fn prepare_clips(
        &self,
        clips: &[ClipInfo],
        target_duration: u32,
    ) -> Result<Vec<PathBuf>> {
        let output_dir = std::env::temp_dir().join("lolshorts_auto_edit");
        tokio::fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| VideoError::ProcessingError {
                message: format!("임시 디렉토리 생성 실패: {}", e),
            })?;

        let total_duration: f64 = clips.iter().map(|c| c.duration.unwrap_or(10.0)).sum();

        let target = target_duration as f64;
        let buffer_target = target * 0.9;

        info!(
            "클립 {}개 준비 중: 총 {:.1}초, 목표 {:.1}초",
            clips.len(),
            total_duration,
            target
        );

        if total_duration <= buffer_target {
            info!("총 길이가 목표 범위 내이므로 원본 클립 사용");
            let paths: Vec<PathBuf> = clips.iter().map(|c| PathBuf::from(&c.file_path)).collect();

            for path in &paths {
                if !path.exists() {
                    return Err(VideoError::FileNotFound {
                        path: path.display().to_string(),
                    });
                }
            }

            return Ok(paths);
        }

        info!(
            "총 길이 {:.1}초가 목표 {:.1}초를 초과하여 지능형 트리밍 적용",
            total_duration, buffer_target
        );

        let trim_factor = buffer_target / total_duration;
        let mut prepared_paths = Vec::new();

        for (idx, clip) in clips.iter().enumerate() {
            let input_path = PathBuf::from(&clip.file_path);

            if !input_path.exists() {
                return Err(VideoError::FileNotFound {
                    path: input_path.display().to_string(),
                });
            }

            let clip_duration = clip.duration.unwrap_or(10.0);
            let trimmed_duration = (clip_duration * trim_factor).max(3.0);

            if (clip_duration - trimmed_duration).abs() < 0.5 {
                info!(
                    "클립 {} ({:.1}초): 원본 사용 (트리밍 차이 <0.5초)",
                    idx, clip_duration
                );
                prepared_paths.push(input_path);
                continue;
            }

            let start_time = (clip_duration - trimmed_duration) / 2.0;
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let output_path = output_dir.join(format!("trimmed_{}_{}.mp4", idx, timestamp));

            info!(
                "클립 {} 트리밍: {:.1}초 -> {:.1}초 (시작점={:.1}초)",
                idx, clip_duration, trimmed_duration, start_time
            );

            self.video_processor
                .extract_clip(&input_path, &output_path, start_time, trimmed_duration)
                .await
                .map_err(|e| VideoError::ProcessingError {
                    message: format!("클립 {} 트리밍 실패: {}", idx, e),
                })?;

            prepared_paths.push(output_path);
        }

        info!(
            "{}개 클립 준비 완료 ({}개 트리밍됨)",
            clips.len(),
            clips.len() - prepared_paths.len()
        );

        Ok(prepared_paths)
    }

    async fn concatenate_clips(&self, clip_paths: &[PathBuf]) -> Result<PathBuf> {
        let output_dir = std::env::temp_dir().join("lolshorts_auto_edit");
        tokio::fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| VideoError::ProcessingError {
                message: format!("임시 디렉토리 생성 실패: {}", e),
            })?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let output_path = output_dir.join(format!("concatenated_{}.mp4", timestamp));

        self.video_processor
            .compose_shorts(clip_paths, &output_path, 1080, 1920)
            .await
    }

    async fn apply_canvas_overlay(
        &self,
        video_path: &Path,
        canvas: &CanvasTemplate,
        is_pro: bool,
    ) -> Result<PathBuf> {
        let output_dir = std::env::temp_dir().join("lolshorts_auto_edit");
        tokio::fs::create_dir_all(&output_dir).await.map_err(|e| {
            VideoError::CanvasApplicationError {
                reason: format!("임시 디렉토리 생성 실패: {}", e),
            }
        })?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let output_path = output_dir.join(format!("with_canvas_{}.mp4", timestamp));

        info!("캔버스 템플릿 적용: {}", canvas.name);

        const WIDTH: u32 = 1080;
        const HEIGHT: u32 = 1920;

        let mut filter_parts: Vec<String> = Vec::new();

        match &canvas.background {
            BackgroundLayer::Color { value } => {
                filter_parts.push(format!("color=c={}:s={}x{}:d=1[bg]", value, WIDTH, HEIGHT));
                filter_parts.push("[0:v][bg]overlay=shortest=1".to_string());
            }
            BackgroundLayer::Gradient { value } => {
                let colors: Vec<&str> = value.split(':').collect();
                if colors.len() == 2 {
                    filter_parts.push(format!(
                        "color=c={}:s={}x{}:d=1,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)',fade=type=in:duration=0:color={}[bg]",
                        colors[0], WIDTH, HEIGHT, colors[1]
                    ));
                    filter_parts.push("[0:v][bg]overlay=shortest=1".to_string());
                } else {
                    // Fallback to black if gradient invalid
                    filter_parts.push(format!("color=c=black:s={}x{}:d=1[bg]", WIDTH, HEIGHT));
                    filter_parts.push("[0:v][bg]overlay=shortest=1".to_string());
                }
            }
            BackgroundLayer::Image { path } => {
                let bg_path = PathBuf::from(path);
                if bg_path.exists() {
                    let safe_path = path.replace('\\', "\\\\").replace(':', "\\:");
                    filter_parts.push(format!(
                        "movie={}[bg_img];[bg_img]scale={}:{}:force_original_aspect_ratio=increase,crop={}:{},boxblur=20[bg]",
                        safe_path, WIDTH, HEIGHT, WIDTH, HEIGHT
                    ));
                    filter_parts.push("[0:v][bg]overlay=shortest=1".to_string());
                } else {
                    // Fallback if image missing
                    warn!("배경 이미지를 찾을 수 없음: {}", path);
                }
            }
        }

        for (_idx, element) in canvas.elements.iter().enumerate() {
            if let CanvasElement::Text {
                content,
                font,
                size,
                color,
                outline,
                position,
                .. 
            } = element {
                let x = (position.x * WIDTH as f32 / 100.0) as u32;
                let y = (position.y * HEIGHT as f32 / 100.0) as u32;

                let safe_content = content.replace('‘', "'\\''");
                
                let mut drawtext = format!(
                    "drawtext=text='{}':fontfile={}:fontsize={}:fontcolor={}:x={}:y={}",
                    safe_content,
                    font,
                    size,
                    color,
                    x,
                    y
                );

                if let Some(outline_color) = outline {
                    drawtext.push_str(&format!(":borderw=2:bordercolor={}", outline_color));
                }

                filter_parts.push(drawtext);
            }
        }

        for (idx, element) in canvas.elements.iter().enumerate() {
            if let CanvasElement::Image {
                path,
                width,
                height,
                position,
                .. 
            } = element {
                let img_path = PathBuf::from(path);
                if !img_path.exists() {
                    warn!("오버레이 이미지를 찾을 수 없음: {}", path);
                    continue;
                }

                let x = (position.x * WIDTH as f32 / 100.0) as u32;
                let y = (position.y * HEIGHT as f32 / 100.0) as u32;

                let safe_path = path.replace('\\', "\\\\").replace(':', "\\:");

                filter_parts.push(format!(
                    "movie={}[img{}];[img{}]scale={}:{}[scaled_img{}]",
                    safe_path, idx, idx, width, height, idx
                ));
                filter_parts.push(format!("overlay={}:{}[out{}]", x, y, idx));
            }
        }

        // === WATERMARK LOGIC ===
        if !is_pro {
            info!("Free Tier 감지: 워터마크 추가");
            // Add a semi-transparent text watermark at bottom right
            let watermark_text = "LoLShorts Free Tier";
            // If we have existing filters, chain it to the last one
            if !filter_parts.is_empty() {
                 let last_idx = filter_parts.len() - 1;
                 filter_parts[last_idx].push_str(&format!(
                    ",drawtext=text='{}':fontsize=36:fontcolor=white@0.5:x=w-tw-20:y=h-th-20:shadowx=2:shadowy=2",
                    watermark_text
                 ));
            } else {
                // If no filters, just add drawtext
                filter_parts.push(format!(
                    "drawtext=text='{}':fontsize=36:fontcolor=white@0.5:x=w-tw-20:y=h-th-20:shadowx=2:shadowy=2",
                    watermark_text
                ));
            }
        }

        if filter_parts.is_empty() {
            info!("적용할 필터가 없음");
            return Ok(video_path.to_path_buf());
        }

        let filter_complex = filter_parts.join(";"); 
        
        let ffmpeg_path = get_ffmpeg_path().map_err(|e| VideoError::ProcessingError {
                message: format!("FFmpeg를 찾을 수 없음: {}", e),
        })?;
        
        let mut command = tokio::process::Command::new(ffmpeg_path);
        command.args([
            "-i",
            video_path.to_str().ok_or_else(|| VideoError::FileAccessError { path: video_path.display().to_string() })?,
            "-filter_complex",
            &filter_complex,
            "-c:v", "libx264", "-preset", "medium", "-crf", "23",
            "-c:a", "copy",
            "-y",
            output_path.to_str().ok_or_else(|| VideoError::FileAccessError { path: output_path.display().to_string() })?,
        ]);

        execute_ffmpeg_command(&mut command).await.map_err(|e| {
            VideoError::CanvasApplicationError { reason: e.to_string() }
        })?;

        info!("캔버스 오버레이 적용 완료");
        Ok(output_path)
    }

    async fn apply_watermark_only(&self, video_path: &Path) -> Result<PathBuf> {
        let output_dir = std::env::temp_dir().join("lolshorts_auto_edit");
        tokio::fs::create_dir_all(&output_dir).await.map_err(|e| {
            VideoError::ProcessingError { message: format!("임시 디렉토리 생성 실패: {}", e) }
        })?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let output_path = output_dir.join(format!("watermarked_{}.mp4", timestamp));

        info!("워터마크 적용 중 (Free Tier)...");

        let watermark_text = "LoLShorts Free Tier";
        let filter = format!(
            "drawtext=text='{}':fontsize=36:fontcolor=white@0.5:x=w-tw-20:y=h-th-20:shadowx=2:shadowy=2",
            watermark_text
        );

        let ffmpeg_path = get_ffmpeg_path().map_err(|e| VideoError::ProcessingError {
                message: format!("FFmpeg를 찾을 수 없음: {}", e),
        })?;

        let mut command = tokio::process::Command::new(ffmpeg_path);
        command.args([
            "-i", video_path.to_str().ok_or_else(|| VideoError::FileAccessError { path: video_path.display().to_string() })?,
            "-vf", &filter,
            "-c:v", "libx264", "-preset", "fast", "-crf", "23",
            "-c:a", "copy",
            "-y",
            output_path.to_str().ok_or_else(|| VideoError::FileAccessError { path: output_path.display().to_string() })?,
        ]);

        execute_ffmpeg_command(&mut command).await?;

        Ok(output_path)
    }

    async fn mix_audio(
        &self,
        video_path: &Path,
        music: &BackgroundMusic,
        levels: &AudioLevels,
    ) -> Result<PathBuf> {
        let output_dir = std::env::temp_dir().join("lolshorts_auto_edit");
        tokio::fs::create_dir_all(&output_dir)
            .await
            .map_err(|e| VideoError::AudioMixingError {
                reason: format!("임시 디렉토리 생성 실패: {}", e),
            })?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let output_path = output_dir.join(format!("with_audio_{}.mp4", timestamp));

        let music_path = PathBuf::from(&music.file_path);
        if !music_path.exists() {
            return Err(VideoError::BackgroundMusicNotFound {
                path: music.file_path.clone(),
            });
        }

        info!(
            "오디오 믹싱: 게임={}%, 음악={}%",
            levels.game_audio, levels.background_music
        );

        let game_volume = levels.game_audio as f64 / 100.0;
        let music_volume = levels.background_music as f64 / 100.0;

        let video_duration = self
            .video_processor
            .get_duration(video_path)
            .await
            .map_err(|e| VideoError::AudioMixingError {
                reason: format!("영상 길이 확인 실패: {}", e),
            })?;

        info!("영상 길이: {:.1}초", video_duration);

        let mut audio_filter = String::new();

        audio_filter.push_str(&format!("[0:a]volume={}[game_audio];", game_volume));

        let fade_duration = 3.0;
        let fade_out_start = (video_duration - fade_duration).max(0.0);

        if music.loop_music {
            audio_filter.push_str(&format!(
                "[1:a]aloop=loop=-1:size=2e+09,atrim=0:{},volume={},afade=t=in:st=0:d={},afade=t=out:st={}:d={}[bg_music]",
                video_duration, music_volume, fade_duration, fade_out_start, fade_duration
            ));
        } else {
            audio_filter.push_str(&format!(
                "[1:a]volume={},afade=t=in:st=0:d={},afade=t=out:st={}:d={}[bg_music]",
                music_volume, fade_duration, fade_out_start, fade_duration
            ));
        }

        audio_filter.push_str("[game_audio][bg_music]amix=inputs=2:duration=first[audio_out]");

        info!("오디오 필터 체인: {}", audio_filter);

        let ffmpeg_path = get_ffmpeg_path() 
            .map_err(|e| VideoError::ProcessingError {
                message: format!("FFmpeg를 찾을 수 없음: {}", e),
            })?;
        let mut command = tokio::process::Command::new(ffmpeg_path);
        command.args([
            "-i",
            video_path
                .to_str()
                .ok_or_else(|| VideoError::FileAccessError {
                    path: video_path.display().to_string(),
                })?,
            "-i",
            music_path
                .to_str()
                .ok_or_else(|| VideoError::FileAccessError {
                    path: music_path.display().to_string(),
                })?,
            "-filter_complex",
            &audio_filter,
            "-map",
            "0:v",
            "-map",
            "[audio_out]",
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-shortest",
            "-y",
            output_path
                .to_str()
                .ok_or_else(|| VideoError::FileAccessError {
                    path: output_path.display().to_string(),
                })?,
        ]);

        execute_ffmpeg_command(&mut command)
            .await
            .map_err(|e| VideoError::AudioMixingError {
                reason: e.to_string(),
            })?;

        info!("오디오 믹싱 완료");
        Ok(output_path)
    }

    async fn load_clips_from_games(&self, game_ids: &[String]) -> Result<Vec<ClipInfo>> {
        let mut all_clips = Vec::new();
        let mut clip_id_counter = 0i64;

        for game_id in game_ids {
            let storage_clips = self.storage.load_clip_metadata(game_id).map_err(|e| {
                VideoError::ProcessingError {
                    message: format!("게임 {}의 클립 로드 실패: {}", game_id, e),
                }
            })?;

            info!("게임 {}에서 {}개의 클립 로드됨", game_id, storage_clips.len());

            for clip in storage_clips {
                let event_type = match &clip.event_type {
                    crate::storage::models::EventType::ChampionKill => "ChampionKill".to_string(),
                    crate::storage::models::EventType::Multikill(2) => "DoubleKill".to_string(),
                    crate::storage::models::EventType::Multikill(3) => "TripleKill".to_string(),
                    crate::storage::models::EventType::Multikill(4) => "QuadraKill".to_string(),
                    crate::storage::models::EventType::Multikill(5) => "PentaKill".to_string(),
                    crate::storage::models::EventType::Multikill(n) => {
                        format!("Multikill({})", n)
                    }
                    crate::storage::models::EventType::TurretKill => "TurretKill".to_string(),
                    crate::storage::models::EventType::InhibitorKill => "InhibitorKill".to_string(),
                    crate::storage::models::EventType::DragonKill => "DragonKill".to_string(),
                    crate::storage::models::EventType::BaronKill => "BaronKill".to_string(),
                    crate::storage::models::EventType::Ace => "Ace".to_string(),
                    crate::storage::models::EventType::FirstBlood => "FirstBlood".to_string(),
                    crate::storage::models::EventType::Custom(s) => s.clone(),
                };

                all_clips.push(ClipInfo {
                    id: clip_id_counter,
                    game_id: game_id.clone(),
                    event_type,
                    event_time: clip.event_time,
                    priority: clip.priority as i32,
                    file_path: clip.file_path,
                    thumbnail_path: clip.thumbnail_path,
                    duration: Some(clip.duration),
                    usage_count: clip.usage_count,
                });

                clip_id_counter += 1;
            }
        }

        info!(
            "총 {}개 게임에서 {}개 클립 로드됨",
            game_ids.len(),
            all_clips.len()
        );

        Ok(all_clips)
    }

    async fn update_progress(
        &self,
        job_id: &str,
        status: AutoEditStatus,
        progress: f64,
        current_step: String,
    ) {
        let mut progress_guard = self.progress.write().await;
        *progress_guard = Some(AutoEditProgress {
            job_id: job_id.to_string(),
            status,
            progress,
            current_step,
            elapsed_seconds: 0.0,
            estimated_seconds: 120.0,
            output_path: None,
            error: None,
        });
    }

    async fn update_progress_complete(&self, job_id: &str, output_path: String, elapsed: f64) {
        let mut progress_guard = self.progress.write().await;
        *progress_guard = Some(AutoEditProgress {
            job_id: job_id.to_string(),
            status: AutoEditStatus::Completed,
            progress: 100.0,
            current_step: "자동 편집 완료!".to_string(),
            elapsed_seconds: elapsed,
            estimated_seconds: elapsed,
            output_path: Some(output_path),
            error: None,
        });
    }

    async fn update_progress_failed(&self, job_id: &str, error: String, elapsed: f64) {
        let mut progress_guard = self.progress.write().await;
        *progress_guard = Some(AutoEditProgress {
            job_id: job_id.to_string(),
            status: AutoEditStatus::Failed,
            progress: 0.0,
            current_step: "자동 편집 실패".to_string(),
            elapsed_seconds: elapsed,
            estimated_seconds: elapsed,
            output_path: None,
            error: Some(error),
        });
    }

    pub async fn get_progress(&self) -> Option<AutoEditProgress> {
        self.progress.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_storage() -> Arc<Storage> {
        let temp_dir = std::env::temp_dir().join(format!("lolshorts_test_{}", std::process::id()));
        Arc::new(Storage::new(&temp_dir).expect("테스트 저장소 생성 실패"))
    }

    fn create_test_clip(id: i64, priority: i32, duration: f64, event_type: &str) -> ClipInfo {
        ClipInfo {
            id,
            game_id: "test_game".to_string(),
            event_type: event_type.to_string(),
            event_time: 100.0,
            priority,
            file_path: format!("/tmp/clip_{}.mp4", id),
            thumbnail_path: None,
            duration: Some(duration),
            usage_count: 0,
        }
    }

    #[tokio::test]
    async fn test_clip_selection_by_priority() {
        let processor = Arc::new(VideoProcessor::new_with_fallback());
        let storage = create_test_storage();
        let composer = AutoComposer::new(processor, storage);

        let clips = vec![
            create_test_clip(1, 1, 10.0, "Kill"),
            create_test_clip(2, 3, 15.0, "Triple Kill"),
            create_test_clip(3, 5, 12.0, "Pentakill"),
            create_test_clip(4, 2, 8.0, "Double Kill"),
            create_test_clip(5, 4, 10.0, "Quadrakill"),
        ];

        let config = AutoEditConfig {
            target_duration: 60,
            game_ids: vec!["game1".to_string()],
            selected_clip_ids: None,
            canvas_template: None,
            background_music: None,
            audio_levels: AudioLevels::default(),
            allow_duplicates: false,
        };

        let selected = composer.select_clips(&clips, &config).await.unwrap();

        assert!(!selected.is_empty());
        assert_eq!(selected[0].priority, 5);
        assert!(selected.iter().all(|c| c.priority >= 2));

        let total_duration: f64 = selected.iter().map(|c| c.duration.unwrap()).sum();
        assert!(total_duration <= 54.0);
    }

    #[tokio::test]
    async fn test_clip_selection_fits_duration() {
        let processor = Arc::new(VideoProcessor::new_with_fallback());
        let storage = create_test_storage();
        let composer = AutoComposer::new(processor, storage);

        let clips = vec![
            create_test_clip(1, 5, 20.0, "Pentakill"),
            create_test_clip(2, 4, 25.0, "Quadrakill"),
            create_test_clip(3, 3, 30.0, "Triple Kill"),
        ];

        let config = AutoEditConfig {
            target_duration: 60,
            game_ids: vec!["game1".to_string()],
            selected_clip_ids: None,
            canvas_template: None,
            background_music: None,
            audio_levels: AudioLevels::default(),
            allow_duplicates: false,
        };

        let selected = composer.select_clips(&clips, &config).await.unwrap();

        let total_duration: f64 = selected.iter().map(|c| c.duration.unwrap()).sum();
        assert!(total_duration <= 54.0);
        assert_eq!(selected.len(), 2);
    }

    #[tokio::test]
    async fn test_manual_clip_selection() {
        let processor = Arc::new(VideoProcessor::new_with_fallback());
        let storage = create_test_storage();
        let composer = AutoComposer::new(processor, storage);

        let clips = vec![
            create_test_clip(1, 1, 10.0, "Kill"),
            create_test_clip(2, 3, 15.0, "Triple Kill"),
            create_test_clip(3, 5, 12.0, "Pentakill"),
        ];

        let config = AutoEditConfig {
            target_duration: 60,
            game_ids: vec!["game1".to_string()],
            selected_clip_ids: Some(vec![1, 3]),
            canvas_template: None,
            background_music: None,
            audio_levels: AudioLevels::default(),
            allow_duplicates: false,
        };

        let selected = composer.select_clips(&clips, &config).await.unwrap();

        assert_eq!(selected.len(), 2);
        assert!(selected.iter().any(|c| c.id == 1));
        assert!(selected.iter().any(|c| c.id == 3));
    }

    #[test]
    fn test_audio_levels_default() {
        let levels = AudioLevels::default();
        assert_eq!(levels.game_audio, 60);
        assert_eq!(levels.background_music, 80);
    }

    #[test]
    fn test_canvas_element_serialization() {
        let text_element = CanvasElement::Text {
            id: "title".to_string(),
            content: "PENTAKILL!".to_string(),
            font: "Bebas Neue".to_string(),
            size: 48,
            color: "#FFD700".to_string(),
            outline: Some("#000000".to_string()),
            position: Position { x: 50.0, y: 10.0 },
        };

        let json = serde_json::to_string(&text_element).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("PENTAKILL"));
    }
}
