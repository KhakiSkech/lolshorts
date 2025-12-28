# LoLShorts

<div align="center">

![LoLShorts Banner](docs/images/banner.png)

**리그 오브 레전드 플레이를 자동으로 녹화하고, 쇼츠와 매드무비를 1초 만에 완성하는 AI 비서**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows-blue.svg)](https://www.microsoft.com/windows)
[![Version](https://img.shields.io/badge/version-1.2.0-green.svg)](https://github.com/KhakiSkech/lolshorts/releases)

[기능](#-features) • [다운로드](#-download) • [사용법](#-usage) • [배포 및 업데이트](#-distribution) • [개발](#-development)

</div>

---

## 📖 개요

**LoLShorts**는 단순한 녹화 프로그램이 아닙니다. 당신의 플레이를 분석하고, 가장 빛나는 순간을 찾아내어, **유튜브 쇼츠(Shorts)**와 **롱폼 몽타주(Montage)** 영상으로 자동 제작해 주는 **크리에이터를 위한 필수 도구**입니다.

이제 LoL 클라이언트를 켜지 않아도 전적을 검색하고, 리플레이를 실행하여 **'페이커'의 시점**으로 명장면을 추출할 수 있습니다.

---

## ✨ 주요 기능 (v1.2.0)

### 🎯 1. 리플레이 허브 & 타겟팅 녹화 (New)
*   **원스톱 관리:** 앱 내에서 내 전적(최근 20게임)을 조회하고, 리플레이를 바로 다운로드/실행합니다.
*   **스마트 타겟팅:** 리플레이 실행 시 **"누구를 녹화할까요?"** 팝업이 뜹니다. 원하는 선수(예: Faker)를 선택하면, 카메라가 그 선수를 따라다니며 킬 장면만 골라내어 녹화합니다.

### 🎬 2. 듀얼 포맷 에디터 (New)
*   **Shorts 모드 (9:16)**: 모바일에 최적화된 세로 영상. AI가 챔피언을 중심으로 화면을 자동 크롭합니다.
*   **Montage 모드 (16:9)**: PC/TV 시청용 가로 영상. 여러 하이라이트 클립을 시간 순서대로 매끄럽게 연결하여 **매드무비**를 만듭니다.

### 🤖 3. 지능형 자동 편집 (Auto-Edit)
*   **오늘의 하이라이트**: 오늘 플레이한 5게임을 선택하면, 그중 최고의 장면(펜타킬 > 쿼드라킬...)들만 뽑아 1분짜리 요약 영상을 만듭니다.
*   **중복 방지 시스템**: 한 번 영상으로 만들어진 클립은 다음 편집 시 자동으로 제외되어, 항상 새로운 장면을 보여줍니다.

### ⚡ 4. 강력한 성능
*   **하드웨어 가속**: NVIDIA(NVENC), AMD(AMF), Intel(QSV) 가속을 지원하여 프레임 드랍 없이 녹화합니다.
*   **로컬 처리**: 모든 영상 분석과 편집은 사용자 PC에서 이루어지며, 데이터는 외부로 전송되지 않습니다.

---

## 📥 다운로드 및 설치

### 시스템 요구 사항
- **OS**: Windows 10 (64-bit) 이상
- **League of Legends**: 설치 및 최신 업데이트 필요

### 설치 방법
1. [Releases 페이지](https://github.com/KhakiSkech/lolshorts/releases/latest)에서 최신 인스톨러를 다운로드합니다.
   - **MSI Installer (추천)**: `LoLShorts_1.2.0_x64_en-US.msi`
2. 파일을 실행하여 설치합니다. (FFmpeg가 자동으로 포함되어 있어 별도 설정이 필요 없습니다.)
3. 바탕화면의 **LoLShorts** 아이콘을 실행합니다.

---

## 🚀 사용 가이드

### Case A: 내가 한 게임 녹화하기 (Live)
1. **LoLShorts 실행**: 앱을 켜두기만 하세요.
2. **게임 시작**: 리그 오브 레전드를 플레이합니다.
3. **자동 녹화**: 킬, 멀티킬, 바론 스틸 등 중요 이벤트가 발생하면 자동으로 녹화되어 저장됩니다.

### Case B: 리플레이로 매드무비 만들기 (Replay)
1. **리플레이 탭**: 앱 좌측 메뉴에서 `Replays`를 클릭합니다.
2. **전적 선택**: 원하는 게임의 `Download` 버튼을 누르고, 완료되면 `Watch`를 클릭합니다.
3. **타겟 선택**: 게임 로딩 후 팝업이 뜨면 **녹화하고 싶은 선수**를 선택합니다.
4. **카메라 고정**: 게임 내에서 해당 챔피언을 **더블 클릭**하거나 **F1~F5** 키를 눌러 시점을 고정하세요.
5. **완성**: 게임이 끝나면 `Editor` 탭에서 추출된 클립들을 모아 **"Export Montage"**를 누르면 매드무비가 완성됩니다.

---

## 🚀 배포 및 자동 업데이트 (Distribution)

LoLShorts는 **GitHub Actions**와 **Tauri Updater**를 통해 자동 배포 및 업데이트 시스템을 갖추고 있습니다.

### 업데이트 원리
- 앱 실행 시 서버의 최신 버전을 확인합니다.
- 새 버전이 있으면 사용자에게 알림을 띄우고, 승인 시 백그라운드에서 업데이트를 설치합니다.

### 개발자 배포 가이드
새 버전을 배포하려면 다음 단계를 따르세요:

1. **버전 올리기**: `package.json`과 `src-tauri/Cargo.toml`의 버전을 동일하게 수정합니다.
2. **태그 푸시**:
   ```bash
   git add .
   git commit -m "chore: release v1.2.1"
   git tag v1.2.1
   git push origin v1.2.1
   ```
3. **자동 빌드**: GitHub Actions가 자동으로 빌드, 서명, 릴리스 생성을 수행합니다.

---

## 🛠️ 기술 스택

- **Core**: Tauri 2.0, Rust (Tokio)
- **Frontend**: React 18, TypeScript, Tailwind CSS, Shadcn/UI
- **Video Engine**: FFmpeg (Sidecar Pattern), Windows Media Foundation
- **Integration**: LCU API (League Client Update), Live Client Data API

---

## 📄 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

<br>

<div align="center">
  Made with ❤️ by the LoLShorts Team
</div>
