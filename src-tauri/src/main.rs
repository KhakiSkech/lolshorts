// 릴리스 빌드에서 Windows의 추가 콘솔 창 방지, 제거하지 마세요!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber;

// Import everything from the library crate
use lolshorts::*;

#[tokio::main]
async fn main() {
    // .env 파일에서 환경 변수 로드 (개발용)
    dotenvy::dotenv().ok();

    // 애플리케이션 데이터 디렉토리 가져오기 (로깅 초기화 전에 수행)
    let app_data_dir = match dirs::data_dir() {
        Some(dir) => dir.join("lolshorts"),
        None => {
            eprintln!("Error: Cannot determine application data directory. Please ensure your system has a valid user data folder.");
            std::process::exit(1);
        }
    };

    // 로그 디렉토리 생성
    let log_dir = app_data_dir.join("logs");
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Failed to create log directory: {}", e);
    }

    // 파일 로거 설정 (일별 회전)
    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 로깅 초기화 (파일 + 표준 출력)
    // 개발 모드에서는 표준 출력도 활성화, 프로덕션에서는 파일 위주 (원한다면 둘 다 가능)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(non_blocking) // 파일에 기록
        .with_ansi(false) // 파일에는 ANSI 색상 코드 제거
        .init();

    tracing::info!("LoLShorts 애플리케이션 시작 중...");
    tracing::info!("로그 디렉토리: {:?}", log_dir);

    // 저장소(Storage) 초기화
    let storage = match storage::Storage::new(&app_data_dir) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            tracing::error!("Failed to initialize storage at {:?}: {}", app_data_dir, e);
            eprintln!(
                "Error: Storage initialization failed: {}. Check disk space and permissions.",
                e
            );
            std::process::exit(1);
        }
    };

    // 인증 관리자(Auth Manager) 초기화
    let auth = Arc::new(auth::AuthManager::new());

    // 기능 게이트(Feature Gate) 초기화
    let feature_gate = Arc::new(feature_gate::FeatureGate::new(auth.clone()));

    // 녹화 디렉토리 초기화
    let recordings_dir = app_data_dir.join("recordings");
    if let Err(e) = std::fs::create_dir_all(&recordings_dir) {
        tracing::error!(
            "Failed to create recordings directory at {:?}: {}",
            recordings_dir,
            e
        );
        eprintln!(
            "Error: Cannot create recordings folder: {}. Check disk space and permissions.",
            e
        );
        std::process::exit(1);
    }

    // 플랫폼 최적화 및 마이그레이션을 포함한 녹화 설정 로드
    let recording_settings =
        match settings::models::RecordingSettings::load_with_platform_optimization().await {
            Ok(settings) => {
                tracing::info!("플랫폼 최적화된 설정이 성공적으로 로드되었습니다");
                settings
            }
            Err(e) => {
                tracing::warn!("최적화된 설정 로드 실패, 마이그레이션 시도: {}", e);

                // 마이그레이션 시도 (폴백)
                match settings::models::RecordingSettings::migrate_settings().await {
                    Ok(migrated_settings) => {
                        tracing::info!("설정이 성공적으로 마이그레이션되었습니다");
                        migrated_settings
                    }
                    Err(migration_error) => {
                        tracing::error!("설정 마이그레이션 실패: {}, 기본값 사용", migration_error);
                        settings::models::RecordingSettings::default()
                    }
                }
            }
        };
    let recording_settings = Arc::new(RwLock::new(recording_settings));

    tracing::info!("녹화 설정이 로드되고 플랫폼에 최적화되었습니다");

    // 비디오 및 오디오 구성을 포함한 녹화 관리자(플랫폼별 백엔드) 초기화
    let settings_read = recording_settings.read().await;
    let audio_config = Some(settings_read.audio.to_audio_config());
    let video_config = Some(recording::VideoSettingsConfig {
        resolution: settings_read.video.get_resolution(),
        fps: settings_read.video.get_fps(),
        bitrate: settings_read.video.get_bitrate(),
        use_h265: settings_read.video.is_h265(),
    });
    drop(settings_read);

    let recording_manager: Arc<RwLock<recording::RecordingManager>> =
        match recording::initialize_recording_backend_full(
            recordings_dir.clone(),
            audio_config,
            video_config,
        )
        .await
        {
            Ok(manager) => Arc::new(RwLock::new(manager)),
            Err(e) => {
                tracing::error!("Failed to initialize recording backend with audio: {}", e);
                eprintln!("Error: Recording system initialization failed: {}. Check if FFmpeg is available and audio devices are accessible.", e);
                std::process::exit(1);
            }
        };

    tracing::info!(
        "{}용 녹화 백엔드 초기화 완료 (오디오 지원 포함)",
        recording::Platform::current().name()
    );

    tracing::info!("크로스 플랫폼 녹화 백엔드 초기화됨");

    // 자동 클립 관리자(Auto Clip Manager) 초기화
    let auto_clip_manager = Arc::new(recording::auto_clip_manager::AutoClipManager::new(
        Arc::clone(&recording_manager),
        Arc::clone(&storage),
        Arc::clone(&recording_settings),
    ));

    tracing::info!("자동 클립 관리자 초기화됨");

    // 핫키 관리자(Hotkey Manager) 초기화
    let hotkey_manager = Arc::new(hotkey::HotkeyManager::new());

    tracing::info!("핫키 관리자 초기화됨");

    // 메트릭 수집기(Metrics Collector) 초기화
    let metrics_collector = Arc::new(utils::metrics::MetricsCollector::new(
        utils::metrics::HealthThresholds::default(),
        recordings_dir.clone(),
    ));

    tracing::info!("메트릭 수집기 초기화됨");

    // 정리 관리자(Cleanup Manager) 초기화
    let cleanup_config = utils::cleanup::CleanupConfig::default();
    let cleanup_manager = Arc::new(utils::cleanup::CleanupManager::new(
        app_data_dir.clone(),
        cleanup_config,
    ));

    // 시작 시 정리 실행
    if let Err(e) = cleanup_manager.cleanup_on_startup().await {
        tracing::error!("시작 시 정리 실패: {}", e);
    }

    tracing::info!("정리 관리자 초기화됨");

    // 자동 편집(Auto-edit) 기능을 위한 Auto Composer 초기화
    let video_processor = match video::VideoProcessor::new() {
        Ok(processor) => {
            tracing::info!("✅ VideoProcessor가 성공적으로 초기화되었습니다");
            Arc::new(processor)
        }
        Err(e) => {
            tracing::warn!(
                "⚠️ 최적 설정으로 VideoProcessor 초기화 실패: {}. 폴백을 사용합니다.",
                e
            );
            Arc::new(video::VideoProcessor::new_with_fallback())
        }
    };

    let auto_composer = Arc::new(video::AutoComposer::new(
        Arc::clone(&video_processor),
        Arc::clone(&storage),
    ));

    tracing::info!("Auto Composer 초기화됨");

    // YouTube 관리자 초기화
    // 환경변수 검증: 플레이스홀더 값이면 설정되지 않은 것으로 처리
    let youtube_client_id = std::env::var("YOUTUBE_CLIENT_ID").ok().filter(|v| {
        !v.is_empty() && !v.contains("your-client-id") && v.ends_with(".apps.googleusercontent.com")
    });
    let youtube_client_secret = std::env::var("YOUTUBE_CLIENT_SECRET")
        .ok()
        .filter(|v| !v.is_empty() && !v.contains("your-client-secret"));
    let youtube_redirect_uri = std::env::var("YOUTUBE_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:8080/oauth2/callback".to_string());

    let youtube_manager = match (youtube_client_id, youtube_client_secret) {
        (Some(client_id), Some(client_secret)) => {
            match youtube::YouTubeManager::new(
                client_id,
                client_secret,
                youtube_redirect_uri,
                Arc::clone(&storage),
            ) {
                Ok(manager) => Arc::new(manager),
                Err(e) => {
                    tracing::error!("Failed to initialize YouTube manager: {}", e);
                    eprintln!("Error: YouTube integration initialization failed: {}. YouTube uploads will be unavailable.", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            tracing::warn!(
                "⚠️ YouTube API 자격증명이 설정되지 않았습니다. \
                YOUTUBE_CLIENT_ID와 YOUTUBE_CLIENT_SECRET 환경변수를 설정하세요. \
                YouTube 업로드 기능이 비활성화됩니다."
            );
            // 자격증명 없이도 앱은 동작해야 함 - 빈 자격증명으로 초기화
            // 실제 업로드 시도 시 적절한 에러 메시지 표시
            match youtube::YouTubeManager::new(
                String::new(),
                String::new(),
                youtube_redirect_uri,
                Arc::clone(&storage),
            ) {
                Ok(manager) => Arc::new(manager),
                Err(e) => {
                    tracing::error!("Failed to initialize YouTube manager: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    // 저장된 YouTube 자격 증명이 있으면 로드
    if let Err(e) = youtube_manager.load_credentials().await {
        tracing::warn!("YouTube 자격 증명 로드 실패: {}", e);
    }

    tracing::info!("YouTube 관리자 초기화됨");

    // 자동 녹화를 위한 게임 상태 모니터(Game State Monitor) 초기화
    let game_monitor = Arc::new(recording::game_monitor::GameStateMonitor::new(Arc::clone(
        &auto_clip_manager,
    )));

    // LCU 클라이언트 초기화
    let lcu_client = Arc::new(tokio::sync::Mutex::new(lcu::LcuClient::new()));

    // 게임 모니터 콜백을 위한 참조 복제
    let recording_manager_for_monitor = Arc::clone(&recording_manager);
    let clip_manager_for_monitor = Arc::clone(&auto_clip_manager);
    let game_monitor_for_callbacks = Arc::clone(&game_monitor);

    let app_state = AppState {
        storage,
        auth,
        feature_gate,
        recording_manager: Arc::clone(&recording_manager),
        clip_manager: Arc::clone(&auto_clip_manager),
        game_monitor: Arc::clone(&game_monitor),
        recording_settings,
        hotkey_manager: Arc::clone(&hotkey_manager),
        metrics_collector: Arc::clone(&metrics_collector),
        cleanup_manager: Arc::clone(&cleanup_manager),
        auto_composer,
        video_processor,
        youtube_manager,
        lcu_client,
    };

    // 콜백과 함께 핫키 시스템 시작
    let recording_manager_hotkey = Arc::clone(&recording_manager);
    let auto_clip_manager_hotkey = Arc::clone(&auto_clip_manager);

    tokio::spawn(async move {
        hotkey_manager
            .start(move |event| {
                let rm = Arc::clone(&recording_manager_hotkey);
                let acm = Arc::clone(&auto_clip_manager_hotkey);

                tokio::spawn(async move {
                    use hotkey::HotkeyEvent;

                    match event {
                        HotkeyEvent::ToggleAutoCapture => {
                            // 자동 캡처가 실행 중인지 확인
                            let is_monitoring = acm.is_monitoring().await;

                            if is_monitoring {
                                // 자동 캡처 중지
                                tracing::info!("핫키 F8: 자동 캡처 중지");
                                if let Err(e) = acm.stop_event_monitoring().await {
                                    tracing::error!("자동 캡처 중지 실패: {}", e);
                                }
                                if let Err(e) = rm.write().await.stop_recording().await {
                                    tracing::error!("리플레이 버퍼 중지 실패: {}", e);
                                }
                            } else {
                                // 자동 캡처 시작
                                tracing::info!("핫키 F8: 자동 캡처 시작");
                                if let Err(e) = rm.write().await.start_recording().await {
                                    tracing::error!("리플레이 버퍼 시작 실패: {}", e);
                                }
                                if let Err(e) = acm.start_event_monitoring().await {
                                    tracing::error!("이벤트 모니터링 시작 실패: {}", e);
                                }
                            }
                        }
                        HotkeyEvent::SaveReplay60 => {
                            // 최근 60초 저장 - 새로운 인터페이스 구현
                            tracing::info!("핫키 F9: 60초 리플레이 저장");

                            // 새로운 WindowsCaptureRecorder 인터페이스의 경우,
                            // 클립을 추출하기 위해 녹화를 중지해야 합니다.
                            match rm.write().await.stop_recording().await {
                                Ok(path) => {
                                    tracing::info!("60초 리플레이 저장됨: {:?}", path);
                                    // 이전에 실행 중이었다면 녹화 재시작
                                    if let Err(e) = rm.write().await.start_recording().await {
                                        tracing::error!("녹화 재시작 실패: {}", e);
                                    }
                                }
                                Err(e) => tracing::error!("60초 리플레이 저장 실패: {}", e),
                            }
                        }
                        HotkeyEvent::SaveReplay30 => {
                            // 최근 30초 저장 - 새로운 인터페이스 구현
                            tracing::info!("핫키 F10: 30초 리플레이 저장");

                            // 새로운 WindowsCaptureRecorder 인터페이스의 경우,
                            // 클립을 추출하기 위해 녹화를 중지해야 합니다.
                            match rm.write().await.stop_recording().await {
                                Ok(path) => {
                                    tracing::info!("30초 리플레이 저장됨: {:?}", path);
                                    // 이전에 실행 중이었다면 녹화 재시작
                                    if let Err(e) = rm.write().await.start_recording().await {
                                        tracing::error!("녹화 재시작 실패: {}", e);
                                    }
                                }
                                Err(e) => tracing::error!("30초 리플레이 저장 실패: {}", e),
                            }
                        }
                    }
                });
            })
            .await
            .unwrap_or_else(|e| tracing::error!("핫키 시스템 시작 실패: {}", e));

        // 자동 녹화를 위한 게임 모니터링 시작
        let recording_manager_start = Arc::clone(&recording_manager_for_monitor);
        let clip_manager_start = Arc::clone(&clip_manager_for_monitor);

        // 게임 시작 콜백 정의
        let game_start_callback = move || {
            let recording_mgr = Arc::clone(&recording_manager_start);
            let clip_mgr = Arc::clone(&clip_manager_start);

            async move {
                tracing::info!("🎮 게임 감지됨! 자동 녹화 시작...");

                // 녹화 시작
                if let Err(e) = recording_mgr.write().await.start_recording().await {
                    tracing::error!("리플레이 버퍼 시작 실패: {}", e);
                    return Err(e.to_string());
                }

                // 하이라이트 이벤트 모니터링 시작
                if let Err(e) = clip_mgr.start_event_monitoring().await {
                    tracing::error!("이벤트 모니터링 시작 실패: {}", e);
                    return Err(e.to_string());
                }

                tracing::info!("✅ 자동 녹화가 성공적으로 시작되었습니다");
                Ok(())
            }
        };

        // 게임 종료 콜백 정의
        let game_end_callback = move || {
            let recording_mgr = Arc::clone(&recording_manager_for_monitor);

            async move {
                tracing::info!("⏹️ 게임 종료. 자동 녹화 중지...");

                // 녹화 중지
                if let Err(e) = recording_mgr.write().await.stop_recording().await {
                    tracing::error!("리플레이 버퍼 중지 실패: {}", e);
                    return Err(e.to_string());
                }

                tracing::info!("✅ 자동 녹화 중지됨");
                Ok(())
            }
        };

        // 게임 모니터링 시작
        if let Err(e) = game_monitor_for_callbacks
            .start_monitoring(game_start_callback, game_end_callback)
            .await
        {
            tracing::error!("게임 모니터링 시작 실패: {}", e);
        } else {
            tracing::info!(
                "🔍 게임 상태 모니터링 시작됨 - League of Legends가 감지되면 자동으로 녹화합니다"
            );
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // 인증(Auth) 명령
            auth::commands::login,
            auth::commands::signup,
            auth::commands::logout,
            auth::commands::get_user_status,
            auth::commands::get_license_info,
            auth::commands::get_user_license,
            auth::commands::refresh_token,
            auth::commands::set_session,
            // 결제(Payment) 명령
            auth::commands::confirm_payment,
            auth::commands::get_subscription_details,
            auth::commands::cancel_subscription,
            auth::commands::open_payment_page,
            // 녹화(Recording) 명령
            recording::commands::get_unified_game_status,
            recording::commands::set_recording_target,
            recording::commands::notify_replay_launched,
            recording::commands::start_recording,
            recording::commands::stop_recording,
            recording::commands::get_recording_status,
            recording::commands::get_detailed_recording_status,
            recording::commands::start_auto_capture,
            recording::commands::stop_auto_capture,
            recording::commands::save_replay,
            recording::commands::get_saved_clips,
            recording::commands::clear_saved_clips,
            recording::commands::list_audio_devices,
            recording::commands::refresh_audio_devices,
            recording::commands::get_audio_devices_with_cache_info,
            recording::commands::get_recording_quality_info,
            recording::commands::detect_available_encoders,
            recording::commands::get_disk_usage_info,
            recording::commands::cleanup_temp_files,
            recording::commands::get_memory_pool_stats,
            recording::commands::get_performance_stats,
            recording::commands::get_recording_backend_info,
            // 비디오(Video) 명령
            video::commands::get_clips,
            video::commands::extract_clip,
            video::commands::compose_shorts,
            video::commands::create_longform_video,
            video::commands::generate_thumbnail,
            video::commands::generate_clip_thumbnail,
            video::commands::get_video_duration,
            video::commands::delete_clip,
            // 자동 편집(Auto-edit) 명령
            video::commands::start_auto_edit,
            video::commands::get_auto_edit_progress,
            // 캔버스 템플릿 명령
            video::commands::save_canvas_template,
            video::commands::load_canvas_template,
            video::commands::list_canvas_templates,
            video::commands::delete_canvas_template,
            // LCU 명령
            lcu::commands::connect_lcu,
            lcu::commands::check_lcu_status,
            lcu::commands::get_current_game,
            lcu::commands::is_in_game,
            lcu::commands::get_lcu_metrics,
            lcu::commands::refresh_lcu_caches,
            lcu::commands::list_match_history,
            lcu::commands::download_replay,
            lcu::commands::get_replay_status,
            lcu::commands::launch_replay,
            lcu::commands::get_game_participants,
            // 저장소(Storage) 명령
            storage::commands::list_games,
            storage::commands::get_game_metadata,
            storage::commands::save_game_metadata,
            storage::commands::get_game_events,
            storage::commands::save_game_events,
            storage::commands::save_clip_metadata,
            storage::commands::delete_game,
            storage::commands::get_storage_stats,
            storage::commands::get_dashboard_stats,
            storage::commands::list_clips,
            storage::commands::get_auto_edit_quota,
            storage::commands::get_auto_edit_results,
            storage::commands::get_auto_edit_result,
            storage::commands::delete_auto_edit_result,
            storage::commands::update_auto_edit_youtube_status,
            // 설정(Settings) 명령
            settings::commands::get_recording_settings,
            settings::commands::save_recording_settings,
            settings::commands::reset_settings_to_default,
            // 플랫폼 구성 명령
            settings::commands::detect_platform_config,
            settings::commands::get_recommended_settings,
            settings::commands::validate_settings_for_platform,
            settings::commands::optimize_settings_for_platform,
            settings::commands::check_settings_migration_needed,
            settings::commands::migrate_settings,
            settings::commands::load_settings_optimized,
            settings::commands::export_settings_backup,
            settings::commands::import_settings_backup,
            settings::commands::get_settings_diagnostics,
            // 유틸리티(Utils) 명령
            utils::commands::get_recording_metrics,
            utils::commands::get_system_metrics,
            utils::commands::get_health_status,
            utils::commands::get_app_version,
            utils::commands::force_cleanup,
            utils::commands::get_disk_space_info,
            utils::commands::show_in_folder,
            utils::commands::open_file_with_default_app,
            utils::commands::check_file_exists,
            utils::commands::get_ffmpeg_info,
            utils::commands::get_hardware_encoders,
            utils::commands::get_video_encoders,
            // 통계 명령
            video::commands::get_clip_statistics,
            video::commands::reset_clip_statistics,
            // YouTube 명령
            youtube::commands::youtube_start_auth,
            youtube::commands::youtube_start_auth_with_server,
            youtube::commands::youtube_complete_auth,
            youtube::commands::youtube_get_auth_status,
            youtube::commands::youtube_upload_video,
            youtube::commands::youtube_get_upload_progress,
            youtube::commands::youtube_get_video_details,
            youtube::commands::youtube_get_upload_history,
            youtube::commands::youtube_add_to_history,
            youtube::commands::youtube_get_quota_info,
            youtube::commands::youtube_logout,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            tracing::error!("Failed to run Tauri application: {}", e);
            eprintln!("Fatal Error: LoLShorts failed to start: {}", e);
            std::process::exit(1);
        });
}
