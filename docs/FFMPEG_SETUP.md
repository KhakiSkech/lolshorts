# FFmpeg Setup Guide

**목적**: 개발 및 테스트를 위한 FFmpeg 설치 가이드

---

## 현재 상태

❌ **FFmpeg가 설치되지 않음**

```bash
$ ffmpeg -version
command not found
```

**필요성**:
- LoLShorts는 FFmpeg CLI를 사용하여 화면 녹화
- 개발/테스트를 위해 FFmpeg 바이너리 필요
- 프로덕션 배포 시에는 앱과 함께 번들링 예정

---

## Windows 설치 방법

### 방법 1: Chocolatey (권장)

**장점**: 자동 PATH 설정, 업데이트 용이

```powershell
# PowerShell (관리자 권한)
choco install ffmpeg
```

**설치 확인**:
```bash
ffmpeg -version
```

### 방법 2: 수동 설치

**1. FFmpeg 다운로드**
- 공식 사이트: https://ffmpeg.org/download.html
- Windows builds: https://www.gyan.dev/ffmpeg/builds/
- 추천: `ffmpeg-release-essentials.zip` (최소 용량)

**2. 압축 해제**
```
C:\ffmpeg\
├── bin\
│   ├── ffmpeg.exe
│   ├── ffplay.exe
│   └── ffprobe.exe
├── doc\
└── presets\
```

**3. PATH 환경 변수 추가**

**PowerShell (관리자 권한)**:
```powershell
$env:Path += ";C:\ffmpeg\bin"
[Environment]::SetEnvironmentVariable("Path", $env:Path, [EnvironmentVariableTarget]::Machine)
```

**수동 설정**:
1. `Win + R` → `sysdm.cpl` → 확인
2. "고급" 탭 → "환경 변수"
3. "시스템 변수" → "Path" 선택 → "편집"
4. "새로 만들기" → `C:\ffmpeg\bin` 추가
5. 확인 후 PowerShell 재시작

**4. 설치 확인**
```bash
ffmpeg -version
```

---

## 필수 기능 확인

### 1. gdigrab (화면 캡처)

Windows GDI 화면 캡처 지원 확인:

```bash
ffmpeg -f gdigrab -list_devices true -i dummy
```

**예상 출력**: 사용 가능한 화면/디바이스 목록

### 2. H.265 인코더

하드웨어 인코더 지원 확인:

```bash
# NVIDIA (NVENC)
ffmpeg -encoders | findstr hevc_nvenc

# Intel (QSV)
ffmpeg -encoders | findstr hevc_qsv

# AMD (AMF)
ffmpeg -encoders | findstr hevc_amf

# Software fallback
ffmpeg -encoders | findstr libx265
```

**예상 출력**: 최소 하나 이상의 인코더 표시

---

## 테스트 녹화

### 기본 화면 녹화 테스트 (5초)

```bash
ffmpeg -f gdigrab -framerate 30 -i desktop -t 5 -c:v libx264 test_capture.mp4
```

**성공 시**:
- `test_capture.mp4` 파일 생성 (약 1-2MB)
- 에러 메시지 없음

### H.265 하드웨어 인코딩 테스트

**NVIDIA GPU**:
```bash
ffmpeg -f gdigrab -framerate 30 -i desktop -t 5 -c:v hevc_nvenc test_nvenc.mp4
```

**Intel GPU**:
```bash
ffmpeg -f gdigrab -framerate 30 -i desktop -t 5 -c:v hevc_qsv test_qsv.mp4
```

**AMD GPU**:
```bash
ffmpeg -f gdigrab -framerate 30 -i desktop -t 5 -c:v hevc_amf test_amf.mp4
```

**실패 시**: Software fallback (libx265) 사용됨 - 정상 동작

---

## LoLShorts 통합 테스트

### 1. Rust 통합 테스트 실행

```bash
cd src-tauri
cargo test --test recording_integration -- --nocapture
```

### 2. 수동 녹화 테스트

```bash
cargo run --bin test_recording
```

**예상 동작**:
1. 10초 세그먼트 녹화 시작
2. `recordings/segment_0001.mp4` 생성
3. 60초 후 자동 회전 (6개 세그먼트 유지)

---

## 문제 해결

### "ffmpeg: command not found"

**원인**: PATH 설정 안 됨

**해결**:
1. PowerShell 재시작
2. PATH 확인: `$env:Path`
3. FFmpeg 경로가 포함되어 있는지 확인

### "gdigrab: Cannot find a device"

**원인**: Windows GDI 화면 캡처 실패

**해결**:
1. Windows 10/11 업데이트 확인
2. 그래픽 드라이버 업데이트
3. 관리자 권한으로 실행

### "hevc_nvenc not found"

**원인**: NVIDIA GPU 또는 드라이버 문제

**해결**:
1. NVIDIA GPU 확인 (GeForce GTX 900 시리즈 이상)
2. NVIDIA 드라이버 업데이트 (최신 버전)
3. Software fallback (libx265) 사용 - 자동 처리됨

---

## 프로덕션 배포

### FFmpeg 번들링 계획

**개발 환경**:
- 수동 FFmpeg 설치 필요 (이 가이드 참조)

**프로덕션 배포**:
- ✅ FFmpeg 바이너리 앱과 함께 번들링
- ✅ 사용자는 별도 설치 불필요
- ✅ Tauri `externalBin` 설정으로 자동 패키징

**Tauri 설정** (`src-tauri/tauri.conf.json`):
```json
{
  "bundle": {
    "externalBin": ["ffmpeg"],
    "resources": []
  }
}
```

**FFmpeg 바이너리 위치**:
- 개발: 시스템 PATH의 `ffmpeg.exe`
- 프로덕션: `resources/ffmpeg.exe` (앱과 함께 패키징)

---

## 다음 단계

### 즉시
1. ✅ FFmpeg 설치 (Chocolatey 또는 수동)
2. ✅ 설치 확인 (`ffmpeg -version`)
3. ✅ 테스트 녹화 실행 (5초)

### 테스트 단계
1. Rust 통합 테스트 작성
2. 세그먼트 녹화 테스트
3. 버퍼 회전 테스트
4. 에러 처리 테스트

### 배포 준비
1. FFmpeg 바이너리 다운로드 (배포용)
2. Tauri 번들링 설정
3. 인스톨러 테스트

---

**마지막 업데이트**: 2025-01-04
**상태**: FFmpeg 설치 필요
**다음 작업**: FFmpeg 설치 후 통합 테스트
