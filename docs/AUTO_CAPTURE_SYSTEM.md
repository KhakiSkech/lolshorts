# 🎮 Auto-Capture System Documentation

## Overview

LoLShorts의 자동 캡처 시스템은 **리플레이 버퍼** 방식으로 작동합니다. 게임 중 항상 최근 60초를 메모리에 보관하고 있다가, 특정 이벤트가 발생하면 자동으로 해당 순간의 앞뒤 영상을 저장합니다.

## 🔄 작동 원리

```
게임 시작
    ↓
[상시 녹화 시작] → 메모리에 60초 순환 버퍼
    ↓
[Live Client API 모니터링] (500ms 간격)
    ↓
[이벤트 감지!] (킬, 멀티킬, 오브젝트 등)
    ↓
[자동 클립 생성]
    ├─ 이벤트 전 10-20초
    ├─ 이벤트 순간
    └─ 이벤트 후 3-10초
    ↓
[MP4 파일로 저장]
```

## 📊 이벤트 트리거 및 우선순위

| 이벤트 | 우선순위 | 이전 시간 | 이후 시간 | 총 길이 |
|--------|----------|-----------|-----------|---------|
| **펜타킬** | ⭐⭐⭐⭐⭐ (5) | 15초 | 5초 | 20초 |
| **쿼드라킬** | ⭐⭐⭐⭐ (4) | 15초 | 5초 | 20초 |
| **트리플킬** | ⭐⭐⭐ (3) | 15초 | 5초 | 20초 |
| **더블킬** | ⭐⭐ (2) | 15초 | 5초 | 20초 |
| **에이스** | ⭐⭐⭐⭐ (4) | 10초 | 10초 | 20초 |
| **바론 처치** | ⭐⭐⭐ (3) | 10초 | 5초 | 15초 |
| **스틸** | ⭐⭐⭐⭐ (4) | 20초 | 3초 | 23초 |
| **드래곤 처치** | ⭐⭐ (2) | 10초 | 3초 | 13초 |
| **솔로킬** | ⭐ (1) | 10초 | 3초 | 13초 |

## 💾 메모리 사용량

### 리플레이 버퍼 계산
- **화면 해상도**: 1920x1080
- **프레임레이트**: 30 FPS
- **픽셀 형식**: RGBA (4 bytes/pixel)
- **프레임 크기**: 1920 × 1080 × 4 = 8.3 MB
- **60초 버퍼**: 8.3 MB × 30 FPS × 60s = **약 15 GB**

### 최적화 방법
1. **압축**: 실시간 프레임 압축 (zstd)
2. **해상도 조절**: 1280x720으로 다운스케일
3. **버퍼 시간 조절**: 30초로 단축

## 🚀 사용 방법

### 1. 자동 캡처 시작

```rust
// Rust 백엔드
let mut manager = RecordingManager::new();
manager.start_auto_capture().await?;
```

### 2. 프론트엔드에서 상태 확인

```typescript
// React 프론트엔드
const [autoCapture, setAutoCapture] = useState(false);
const [savedClips, setSavedClips] = useState<SavedClip[]>([]);

// 자동 캡처 시작
const startAutoCapture = async () => {
  await invoke('start_auto_capture');
  setAutoCapture(true);
};

// 저장된 클립 조회
const getSavedClips = async () => {
  const clips = await invoke<SavedClip[]>('get_saved_clips');
  setSavedClips(clips);
};
```

### 3. 저장된 클립 구조

```typescript
interface SavedClip {
  path: string;        // "recordings/1234567890_pentakill_p5.mp4"
  trigger: string;     // "Pentakill"
  priority: number;    // 5
  timestamp: number;   // Unix timestamp
  duration: number;    // 20 (seconds)
}
```

## 🎯 핵심 기능

### 1. 리플레이 버퍼 (✅ 구현 완료)
- 최근 60초를 항상 메모리에 보관
- 순환 버퍼로 오래된 프레임 자동 삭제
- 이벤트 발생 시 즉시 접근 가능

### 2. Live Client API 연동 (✅ 구현 완료)
- 500ms 간격으로 게임 이벤트 모니터링
- 플레이어 킬, 어시스트, 오브젝트 감지
- 멀티킬 판정 (10초 내 연속 킬)

### 3. 자동 클립 생성 (✅ 구현 완료)
- 이벤트별 최적화된 클립 길이
- 우선순위 기반 파일명 생성
- 비동기 저장으로 게임 영향 최소화

### 4. 비디오 인코딩 (❌ 미구현)
- H.264/H.265 코덱 지원
- 9:16 세로 형식 변환
- 워터마크/오버레이 추가

## 📁 파일 저장 구조

```
recordings/
├── 1234567890_pentakill_p5.mp4      # 펜타킬 (우선순위 5)
├── 1234567891_baronkill_p3.mp4      # 바론 처치 (우선순위 3)
├── 1234567892_multikill(3)_p3.mp4   # 트리플킬 (우선순위 3)
└── 1234567893_championkill_p1.mp4   # 일반 킬 (우선순위 1)
```

## 🔧 설정 옵션

```rust
pub struct RecordingConfig {
    pub fps: u32,                    // 프레임레이트 (30/60)
    pub capture_audio: bool,          // 게임 오디오 캡처
    pub capture_microphone: bool,     // 마이크 캡처
    pub output_dir: PathBuf,         // 저장 경로
    pub buffer_duration_secs: u32,   // 버퍼 시간 (30-120초)
}
```

## ⚠️ 시스템 요구사항

### 최소 사양
- **RAM**: 8GB (30초 버퍼)
- **CPU**: 4코어 이상
- **저장공간**: 10GB 이상

### 권장 사양
- **RAM**: 16GB (60초 버퍼)
- **CPU**: 6코어 이상
- **GPU**: 하드웨어 인코딩 지원
- **저장공간**: 50GB 이상 (SSD 권장)

## 🐛 알려진 제한사항

1. **메모리 사용량**: 60초 버퍼 시 15GB RAM 사용
2. **비디오 인코딩**: 아직 미구현 (프레임만 저장)
3. **오디오 동기화**: 구현 필요
4. **Linux/macOS**: 일부 기능 제한

## 📈 성능 최적화 팁

1. **버퍼 크기 조절**
   - 메모리 부족 시 30초로 단축
   - 고사양 PC는 120초까지 가능

2. **해상도 조절**
   - 1280x720으로 다운스케일 시 메모리 55% 절약
   - 최종 출력은 1080x1920 업스케일

3. **선택적 캡처**
   - 우선순위 3 이상만 저장
   - 랭크 게임만 자동 캡처

## 🎬 향후 개선 계획

1. **실시간 인코딩**: FFmpeg 하드웨어 가속 최적화
2. **AI 하이라이트**: 최고의 플레이 자동 선택
3. **클라우드 백업**: 자동 업로드
4. **편집 기능**: 트랜지션, 효과, BGM 추가
5. **소셜 공유**: YouTube Shorts 직접 업로드