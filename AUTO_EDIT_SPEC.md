# Auto-Edit Feature Specification

**Version**: 1.0
**Target Release**: v1.2.0
**Status**: Planning
**Created**: 2025-11-06

---

## Overview

Complete automated video composition system for YouTube Shorts creation with intelligent clip selection, background editing, audio mixing, and one-click generation.

**Core Value Proposition**: Transform hours of League of Legends gameplay into publication-ready 60-second Shorts videos with minimal user intervention.

---

## User Workflow

### 1. Game Selection
**Interface**: Multi-select dropdown in Editor page

```
[Select Games ▼]
☑ Game #12345 - 2025-11-05 14:30 - Victory (Yasuo)
☐ Game #12344 - 2025-11-05 12:15 - Defeat (Zed)
☑ Game #12343 - 2025-11-04 20:45 - Victory (Ahri)
```

**Backend**: Load clips from multiple games simultaneously

### 2. Automatic Clip Selection
**Interface**: Clip library with pre-selected clips (editable)

```
Clip Library (12 clips loaded, 5 auto-selected)
─────────────────────────────────
✓ [⭐⭐⭐⭐⭐] Pentakill - 3:45      Priority: 5
✓ [⭐⭐⭐⭐] Quadrakill - 7:22       Priority: 4
☐ [⭐⭐⭐] Triple Kill - 2:10        Priority: 3
✓ [⭐⭐⭐⭐] Baron Steal - 12:30     Priority: 4
✓ [⭐⭐⭐] Dragon Steal - 5:15      Priority: 3
☐ [⭐⭐] Double Kill - 8:40         Priority: 2
✓ [⭐⭐⭐⭐] Quadrakill - 15:20      Priority: 4
```

**Algorithm**: Priority-based selection to fit target duration
- Sort clips by priority (5 → 1)
- Select until total duration ≤ target (default: 60s)
- Apply smart trimming if needed
- User can manually toggle any clip on/off

### 3. Background Canvas Editor
**Interface**: Canva-style layer system

```
┌────────────────────────────────────┐
│ Canvas Layer Editor                │
├────────────────────────────────────┤
│ Background Image:                  │
│ [Choose File] current_bg.png       │
│                                    │
│ Text Elements:                     │
│ ┌──────────────────────┐          │
│ │ "INSANE PENTAKILL!" │◀── Drag   │
│ └──────────────────────┘          │
│ Font: [Bebas Neue ▼] Size: [48▼]  │
│ Color: [#FFD700] Outline: [#000]  │
│                                    │
│ Logo/Watermark:                    │
│ [Upload Image] Position: [BR ▼]   │
│                                    │
│ [Save Template] [Load Template]   │
└────────────────────────────────────┘
```

**Template Storage**: JSON format for reusability
```json
{
  "name": "Pentakill Style",
  "background": {
    "type": "image",
    "path": "backgrounds/gold_gradient.png"
  },
  "elements": [
    {
      "type": "text",
      "content": "INSANE PENTAKILL!",
      "font": "Bebas Neue",
      "size": 48,
      "color": "#FFD700",
      "outline": "#000000",
      "position": { "x": 50, "y": 10 }
    }
  ]
}
```

### 4. Duration-Based Composition
**Interface**: Target duration selector

```
Video Duration: ● 60s  ○ 120s  ○ 180s

Estimated Timeline:
─────────────────────────────────────
[Intro: 3s] [Clip 1: 12s] [Transition: 1s]
[Clip 2: 15s] [Transition: 1s] [Clip 3: 10s]
[Transition: 1s] [Clip 4: 8s] [Outro: 3s]
─────────────────────────────────────
Total: 54s (6s buffer remaining)
```

**Composition Rules**:
- **Intro**: 3s fade-in with text overlay
- **Clips**: Trimmed to essential action (kill sequence)
- **Transitions**: 1s crossfade between clips
- **Outro**: 3s fade-out with call-to-action
- **Buffer**: Reserve 10% for transitions and padding

### 5. Background Music Integration
**Interface**: Audio mixer panel

```
┌──────────────────────────────────────┐
│ Audio Mixing                         │
├──────────────────────────────────────┤
│ Background Music:                    │
│ [Choose MP3] epic_music.mp3          │
│ Duration: 02:45 (will be trimmed)    │
│                                      │
│ Volume Controls:                     │
│ Game Audio:  ████████░░░░ 60%       │
│ Background:  ████████████░ 80%      │
│                                      │
│ [Preview Mix] [Reset to Default]     │
└──────────────────────────────────────┘
```

**Audio Processing**:
- Trim music to match video duration
- Apply fade-in (3s) and fade-out (3s)
- Mix game audio + music using FFmpeg filters
- Normalize audio levels to prevent clipping

### 6. Auto-Edit Execution
**Interface**: Start button with progress tracking

```
┌──────────────────────────────────────┐
│ [⚡ Start Auto Edit]                 │
├──────────────────────────────────────┤
│ Processing...                        │
│ ▓▓▓▓▓▓▓▓▓░░░░░░░░░ 60%              │
│                                      │
│ Current Step: Rendering transitions  │
│ Elapsed: 00:45 | Estimated: 01:15   │
└──────────────────────────────────────┘
```

---

## Technical Architecture

### Frontend Components (React/TypeScript)

#### New Components
```
src/components/auto-edit/
├── AutoEditPanel.tsx          # Main auto-edit interface
├── GameSelector.tsx           # Multi-game selection dropdown
├── ClipAutoSelector.tsx       # Priority-based clip selection UI
├── CanvasEditor.tsx           # Background/text editing canvas
│   ├── BackgroundLayer.tsx
│   ├── TextElement.tsx
│   └── ImageElement.tsx
├── DurationSelector.tsx       # 60/120/180s selection
├── AudioMixer.tsx             # Music upload + volume controls
├── TemplateManager.tsx        # Save/load template UI
└── AutoEditProgress.tsx       # Progress bar + status
```

#### Modified Components
```
src/pages/Editor.tsx
- Add <AutoEditPanel /> tab
- Switch between manual/auto-edit modes

src/stores/editorStore.ts
- Add autoEditConfig state
- Add selectedGames: string[]
- Add canvasTemplate: CanvasTemplate
- Add targetDuration: 60 | 120 | 180
- Add backgroundMusic: AudioFile | null
```

### Backend Implementation (Rust)

#### New Tauri Commands
```rust
// src-tauri/src/commands/auto_edit.rs

#[tauri::command]
async fn auto_select_clips(
    game_ids: Vec<String>,
    target_duration: u32, // 60, 120, or 180
    state: State<'_, AppState>
) -> Result<Vec<SelectedClip>, String> {
    // 1. Load all clips from specified games
    // 2. Sort by priority (5 → 1)
    // 3. Calculate cumulative duration
    // 4. Select clips until target duration reached
    // 5. Apply intelligent trimming if needed
}

#[tauri::command]
async fn save_canvas_template(
    template: CanvasTemplate,
    state: State<'_, AppState>
) -> Result<String, String> {
    // Save template as JSON to AppData/templates/
}

#[tauri::command]
async fn load_canvas_template(
    template_id: String,
    state: State<'_, AppState>
) -> Result<CanvasTemplate, String> {
    // Load template from AppData/templates/{id}.json
}

#[tauri::command]
async fn start_auto_edit(
    config: AutoEditConfig,
    state: State<'_, AppState>
) -> Result<String, String> {
    // Main auto-edit orchestration
    // Returns job_id for progress tracking
}

#[tauri::command]
async fn get_auto_edit_progress(
    job_id: String,
    state: State<'_, AppState>
) -> Result<AutoEditProgress, String> {
    // Return current progress (0-100%)
}
```

#### Video Composition Engine
```rust
// src-tauri/src/video/auto_composer.rs

pub struct AutoComposer {
    ffmpeg: Arc<FFmpegController>,
    config: AutoEditConfig,
}

impl AutoComposer {
    /// Main composition workflow
    pub async fn compose(&self) -> Result<PathBuf> {
        // 1. Generate intro sequence (3s)
        let intro = self.generate_intro().await?;

        // 2. Process selected clips
        let processed_clips = self.process_clips().await?;

        // 3. Generate transitions (1s each)
        let transitions = self.generate_transitions().await?;

        // 4. Generate outro sequence (3s)
        let outro = self.generate_outro().await?;

        // 5. Apply canvas overlay (background + text)
        let with_overlay = self.apply_canvas_overlay().await?;

        // 6. Mix audio (game audio + background music)
        let final_video = self.mix_audio(with_overlay).await?;

        Ok(final_video)
    }

    /// Apply canvas template as overlay
    async fn apply_canvas_overlay(&self) -> Result<PathBuf> {
        // Use FFmpeg overlay filter to apply:
        // - Background image
        // - Text elements (drawtext filter)
        // - Logo/watermark
    }

    /// Mix game audio with background music
    async fn mix_audio(&self, video_path: PathBuf) -> Result<PathBuf> {
        // FFmpeg audio mixing:
        // -filter_complex "[0:a]volume=0.6[a0];[1:a]volume=0.8[a1];[a0][a1]amix=inputs=2[aout]"
    }
}
```

#### FFmpeg Command Templates

**Clip Trimming + Concatenation**:
```bash
# Create file list for concatenation
echo "file 'intro.mp4'" > concat_list.txt
echo "file 'clip1_trimmed.mp4'" >> concat_list.txt
echo "file 'transition1.mp4'" >> concat_list.txt
echo "file 'clip2_trimmed.mp4'" >> concat_list.txt
echo "file 'outro.mp4'" >> concat_list.txt

# Concatenate all segments
ffmpeg -f concat -safe 0 -i concat_list.txt -c copy output.mp4
```

**Canvas Overlay Application**:
```bash
# Apply background + text overlays
ffmpeg -i video.mp4 \
  -i background.png \
  -filter_complex "\
    [1:v]scale=1080:1920[bg]; \
    [0:v][bg]overlay=(main_w-overlay_w)/2:(main_h-overlay_h)/2[v1]; \
    [v1]drawtext=text='INSANE PENTAKILL':fontfile=BebasNeue.ttf:fontsize=48:fontcolor=white:bordercolor=black:borderw=3:x=(w-text_w)/2:y=50[vout]" \
  -map "[vout]" -map 0:a output_with_overlay.mp4
```

**Audio Mixing**:
```bash
# Mix game audio (60%) + background music (80%)
ffmpeg -i video_with_game_audio.mp4 \
  -i background_music.mp3 \
  -filter_complex "\
    [0:a]volume=0.6[a0]; \
    [1:a]volume=0.8,afade=t=in:st=0:d=3,afade=t=out:st=57:d=3[a1]; \
    [a0][a1]amix=inputs=2:duration=first[aout]" \
  -map 0:v -map "[aout]" -c:v copy -shortest final_output.mp4
```

---

## Data Models

### AutoEditConfig
```typescript
interface AutoEditConfig {
  selectedGames: string[];           // Game IDs to include
  selectedClips: string[];           // Clip IDs (user can override auto-selection)
  canvasTemplate: CanvasTemplate;    // Background + text elements
  targetDuration: 60 | 120 | 180;    // Target video duration in seconds
  backgroundMusic: AudioFile | null; // Optional background music
  audioLevels: AudioLevels;          // Volume controls
}

interface CanvasTemplate {
  id: string;
  name: string;
  background: BackgroundLayer;
  elements: CanvasElement[];
}

interface BackgroundLayer {
  type: 'color' | 'gradient' | 'image';
  value: string; // hex color, gradient CSS, or image path
}

interface CanvasElement {
  id: string;
  type: 'text' | 'image';
  position: { x: number; y: number }; // Percentage-based (0-100)

  // Text-specific
  content?: string;
  font?: string;
  size?: number;
  color?: string;
  outline?: string;

  // Image-specific
  imagePath?: string;
  width?: number;
  height?: number;
}

interface AudioLevels {
  gameAudio: number;      // 0-100 (percentage)
  backgroundMusic: number; // 0-100 (percentage)
}

interface AutoEditProgress {
  jobId: string;
  status: 'queued' | 'processing' | 'completed' | 'failed';
  progress: number;        // 0-100
  currentStep: string;     // Human-readable step description
  elapsedSeconds: number;
  estimatedSeconds: number;
  outputPath?: string;     // Available when completed
  error?: string;          // Available when failed
}
```

---

## Implementation Phases

### Phase 1: Backend Foundation (Week 1-2)
**Priority**: High
**Dependencies**: None

**Tasks**:
- [ ] Create `AutoComposer` struct in `src-tauri/src/video/auto_composer.rs`
- [ ] Implement clip selection algorithm (`auto_select_clips`)
- [ ] Implement FFmpeg template rendering functions
- [ ] Add Tauri commands: `auto_select_clips`, `start_auto_edit`, `get_auto_edit_progress`
- [ ] Write unit tests for composition logic

**Acceptance Criteria**:
- Backend can select clips based on priority + duration
- Backend can concatenate clips with transitions
- Progress tracking works correctly

### Phase 2: Canvas Editor (Week 3-4)
**Priority**: High
**Dependencies**: Phase 1

**Tasks**:
- [ ] Create `CanvasEditor` React component with drag-and-drop
- [ ] Implement background layer selection (color/gradient/image)
- [ ] Implement text element creation and editing
- [ ] Implement image/logo upload and positioning
- [ ] Add template save/load functionality
- [ ] Backend: Implement `save_canvas_template`, `load_canvas_template`
- [ ] Backend: Implement FFmpeg overlay filter generation

**Acceptance Criteria**:
- Users can create and edit canvas templates
- Templates can be saved and reloaded
- Templates correctly render as video overlays

### Phase 3: Audio Mixing (Week 5)
**Priority**: Medium
**Dependencies**: Phase 1

**Tasks**:
- [ ] Create `AudioMixer` React component
- [ ] Implement MP3 file upload
- [ ] Add volume sliders for game audio + music
- [ ] Backend: Implement audio mixing with FFmpeg
- [ ] Add audio fade-in/fade-out (3s)
- [ ] Implement audio preview functionality

**Acceptance Criteria**:
- Users can upload MP3 files
- Volume levels correctly adjust mix
- Audio fades work smoothly
- Preview matches final output

### Phase 4: UI Integration (Week 6)
**Priority**: High
**Dependencies**: Phases 1, 2, 3

**Tasks**:
- [ ] Create `AutoEditPanel` main component
- [ ] Add multi-game selection UI
- [ ] Integrate clip auto-selection display
- [ ] Add duration selector (60/120/180s)
- [ ] Create progress bar with status messages
- [ ] Add "Start Auto Edit" button with validation
- [ ] Implement error handling + user feedback

**Acceptance Criteria**:
- Complete workflow is intuitive and user-friendly
- Progress tracking updates in real-time
- Errors display helpful messages
- Final video opens automatically when complete

### Phase 5: Testing & Optimization (Week 7-8)
**Priority**: Critical
**Dependencies**: All previous phases

**Tasks**:
- [ ] E2E testing of complete auto-edit workflow
- [ ] Performance optimization (FFmpeg settings)
- [ ] Memory leak testing (long video processing)
- [ ] Error recovery testing (FFmpeg failures)
- [ ] UI/UX refinement based on testing
- [ ] Documentation and user guide

**Acceptance Criteria**:
- 60s video generation completes in <2 minutes
- Memory usage stays under 2GB during processing
- All error cases handled gracefully
- User can successfully create publication-ready Shorts

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| **60s Video Generation** | <2 minutes | From "Start" click to completed file |
| **120s Video Generation** | <3 minutes | From "Start" click to completed file |
| **180s Video Generation** | <5 minutes | From "Start" click to completed file |
| **Memory Usage (Peak)** | <2GB | During FFmpeg processing |
| **Clip Selection Speed** | <1 second | Algorithm execution time |
| **Canvas Render Speed** | <10 seconds | Overlay application time |
| **Audio Mix Speed** | <5 seconds | Audio processing time |

---

## Testing Strategy

### Unit Tests (Rust)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_clip_selection_fits_duration() {
        let clips = create_test_clips(); // 10 clips, various priorities
        let selected = auto_select_clips(&clips, 60);

        let total_duration: f64 = selected.iter().map(|c| c.duration).sum();
        assert!(total_duration <= 60.0);
        assert!(selected[0].priority >= selected.last().unwrap().priority); // Sorted by priority
    }

    #[test]
    fn test_audio_mixing_levels() {
        let config = AudioLevels { gameAudio: 60, backgroundMusic: 80 };
        let ffmpeg_filter = generate_audio_filter(&config);

        assert!(ffmpeg_filter.contains("volume=0.6")); // 60% = 0.6
        assert!(ffmpeg_filter.contains("volume=0.8")); // 80% = 0.8
    }
}
```

### Integration Tests (E2E)
```typescript
// tests/e2e/auto-edit.spec.ts
import { test, expect } from '@playwright/test';

test('complete auto-edit workflow', async ({ page }) => {
  await page.goto('http://localhost:1420/editor');

  // Select games
  await page.click('[data-testid="game-selector"]');
  await page.check('[data-testid="game-12345"]');
  await page.check('[data-testid="game-12346"]');

  // Verify auto-selection
  const selectedClips = await page.locator('[data-testid="clip-selected"]').count();
  expect(selectedClips).toBeGreaterThan(0);

  // Add background
  await page.click('[data-testid="canvas-editor-tab"]');
  await page.setInputFiles('[data-testid="bg-upload"]', 'test-assets/background.png');

  // Add text
  await page.click('[data-testid="add-text-button"]');
  await page.fill('[data-testid="text-input"]', 'INSANE PENTAKILL!');

  // Set duration
  await page.click('[data-testid="duration-60s"]');

  // Upload music
  await page.setInputFiles('[data-testid="music-upload"]', 'test-assets/music.mp3');

  // Start auto-edit
  await page.click('[data-testid="start-auto-edit"]');

  // Wait for completion (max 3 minutes)
  await expect(page.locator('[data-testid="progress-complete"]')).toBeVisible({ timeout: 180000 });

  // Verify output file exists
  const outputPath = await page.textContent('[data-testid="output-path"]');
  expect(outputPath).toContain('auto_edit_');
});
```

---

## User Documentation

### Quick Start Guide
```markdown
# Auto-Edit Quick Start

1. **Select Games**: Choose one or more games from your match history
2. **Review Clips**: Top clips are auto-selected based on priority (Pentakills, Quadrakills, etc.)
3. **Customize Background**: Add images, text, or logos to your video
4. **Choose Duration**: Select 60s (Shorts), 120s, or 180s
5. **Add Music**: Upload your favorite MP3 (optional)
6. **Adjust Volumes**: Balance game audio vs. background music
7. **Click "Start Auto Edit"**: Sit back and let LoLShorts create your video!

Your video will be ready in ~2 minutes, optimized for YouTube Shorts (9:16 format).
```

---

## Security & Safety

### Input Validation
- **Game IDs**: Validate against user's own games only
- **Clip Paths**: Prevent path traversal attacks
- **Music Files**: Validate MP3 format, max size 50MB
- **Canvas Templates**: Sanitize JSON to prevent code injection
- **FFmpeg Commands**: Use parameterized commands, never string concatenation

### Resource Limits
- **Max Clips Per Auto-Edit**: 20 clips
- **Max Video Duration**: 180 seconds
- **Max Music File Size**: 50MB
- **Max Canvas Elements**: 10 text + 5 images
- **Concurrent Jobs**: 1 per user (prevent resource exhaustion)

---

## Known Limitations

1. **Single Job Queue**: Only one auto-edit can run at a time per user
2. **Music Format**: Only MP3 supported (no WAV, FLAC, OGG)
3. **Background Images**: PNG/JPG only (no GIF, WEBP)
4. **Text Fonts**: Limited to bundled fonts (no custom font upload in v1.0)
5. **Transition Types**: Only crossfade (no wipes, slides, etc. in v1.0)

---

## Future Enhancements (Post-v1.2.0)

### v1.3.0 Possibilities
- **Advanced Transitions**: Wipe, slide, zoom transitions
- **Audio Effects**: EQ, compression, reverb on game audio
- **Custom Fonts**: User font upload support
- **Animated Text**: Text entrance/exit animations
- **Multi-Track Audio**: Voice-over support (3rd audio track)

### v2.0.0 Possibilities
- **AI Voice-Over**: Auto-generated commentary using TTS
- **Beat Detection**: Sync transitions to music beats
- **Face Cam Integration**: Webcam footage overlay
- **Multi-Resolution**: Support 16:9, 4:3, 1:1 (not just 9:16)
- **Cloud Rendering**: Offload processing to cloud for faster generation

---

## Appendix A: FFmpeg Filter Reference

### Overlay Filter
```bash
-filter_complex "[1:v]scale=1080:1920[bg];[0:v][bg]overlay=x:y[out]"
```

### Text Overlay (drawtext)
```bash
-vf "drawtext=text='TEXT':fontfile=font.ttf:fontsize=48:fontcolor=white:x=(w-text_w)/2:y=50"
```

### Audio Mix (amix)
```bash
-filter_complex "[0:a]volume=0.6[a0];[1:a]volume=0.8[a1];[a0][a1]amix=inputs=2[out]"
```

### Fade In/Out
```bash
# Video fade in (3s)
-vf "fade=t=in:st=0:d=3"

# Audio fade out (3s at 57s mark)
-af "afade=t=out:st=57:d=3"
```

### Crossfade Transition (xfade)
```bash
-filter_complex "[0:v][1:v]xfade=transition=fade:duration=1:offset=10[out]"
```

---

## Appendix B: Sample Canvas Template JSON

```json
{
  "id": "pentakill_gold",
  "name": "Pentakill Gold Style",
  "background": {
    "type": "gradient",
    "value": "linear-gradient(135deg, #FFD700 0%, #FFA500 100%)"
  },
  "elements": [
    {
      "id": "title_text",
      "type": "text",
      "content": "INSANE PENTAKILL!",
      "font": "Bebas Neue",
      "size": 48,
      "color": "#FFFFFF",
      "outline": "#000000",
      "position": { "x": 50, "y": 5 }
    },
    {
      "id": "watermark",
      "type": "image",
      "imagePath": "logos/channel_logo.png",
      "width": 80,
      "height": 80,
      "position": { "x": 85, "y": 90 }
    },
    {
      "id": "subscribe_text",
      "type": "text",
      "content": "SUBSCRIBE FOR MORE!",
      "font": "Roboto",
      "size": 24,
      "color": "#FFD700",
      "outline": "#000000",
      "position": { "x": 50, "y": 92 }
    }
  ]
}
```

---

**End of Specification**
