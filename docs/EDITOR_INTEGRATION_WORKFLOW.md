# Video Editor Integration Workflow

This document explains how the video editor will consume and use the rich clip metadata.

---

## File Structure Overview

```
C:/LoLShorts/clips/
└── 1234567890/                        # Game ID
    ├── metadata.json                  # Game-level metadata
    ├── events.json                    # All raw events from game
    ├── clips.json                     # Lightweight clip index (V1 compatibility)
    └── clips/                         # Clip storage directory
        ├── clip_20250110_143052_pentakill.mp4    # Video file
        ├── clip_20250110_143052_pentakill.json   # Rich metadata (V2)
        ├── clip_20250110_143052_pentakill.jpg    # Thumbnail
        ├── clip_20250110_144521_baron.mp4
        ├── clip_20250110_144521_baron.json
        └── clip_20250110_144521_baron.jpg
```

---

## Editor Workflow: From Recording to Editing

### Phase 1: Recording (Auto-Capture)

```
Game Detected
    ↓
Recording Starts (60s circular buffer)
    ↓
Event Detected (e.g., Pentakill at 21:00)
    ↓
Event Window Created:
  - Pre-duration: 15s before event
  - Event: Pentakill moment
  - Post-duration: 5s after event
    ↓
Additional Events Merged (if within 15s threshold)
  - Dragon kill at 20:45 → Merged
  - Team fight cleanup at 21:05 → Merged
    ↓
Clip Saved:
  ├── Video: clip_xxx.mp4 (30 seconds)
  └── Metadata: clip_xxx.json (Rich JSON with all details)
```

### Phase 2: Editor Opens Clip

**Editor loads clip metadata:**

```rust
// Editor Rust/TypeScript code
let clip_metadata = storage.load_clip_metadata_v2(clip_path)?;
```

**What the editor receives:**

```json
{
  "clip_id": "clip_20250110_143052_pentakill",
  "game_id": "1234567890",
  "file_path": "C:/LoLShorts/clips/1234567890/clips/clip_20250110_143052_pentakill.mp4",

  "clip_duration": 30.0,

  "primary_event": {
    "event_type": {"multikill": 5},
    "timestamp": 1260.0,     // Game time
    "clip_timestamp": 15.0,  // Within this clip
    "priority": 5
  },

  "merged_events": [
    {
      "event_type": "dragon_kill",
      "timestamp": 1250.0,
      "clip_timestamp": 5.0,
      "priority": 3
    }
  ],

  "timeline": {
    "markers": [
      {"timestamp": 5.0, "label": "Dragon Secured"},
      {"timestamp": 15.0, "label": "PENTAKILL!", "marker_type": "event_peak"}
    ]
  },

  "video_info": {
    "resolution": "r1920x1080",
    "frame_rate": "fps60",
    "codec": "h265"
  },

  "audio_info": {
    "tracks": [
      {"track_type": "system_audio", "volume_percent": 100},
      {"track_type": "microphone", "volume_percent": 120}
    ]
  }
}
```

### Phase 3: Timeline Display

**Editor renders timeline with markers:**

```
0s          5s          10s         15s         20s         25s         30s
|-----------|-----------|-----------|-----------|-----------|-----------|
            🐉 Dragon                🔥 PENTAKILL!
         [Marker 1]              [Marker 2 - Peak]
```

**Timeline features:**

1. **Markers**: Visual indicators on timeline
   - Click to jump to exact moment
   - Color-coded by event type (dragon=🔵, pentakill=🟡)

2. **Chapters**: Logical sections
   - 0-10s: "Setup" (team positioning)
   - 10-20s: "Pentakill Sequence"
   - 20-30s: "Aftermath"

3. **Audio Tracks**: Separate waveforms
   - Track 0: System audio (game sounds)
   - Track 1: Microphone (user voice)

### Phase 4: Metadata-Driven Features

**1. Auto-Captioning**

```typescript
// Editor generates captions from events
function generateCaptions(clip: ClipMetadataV2): Caption[] {
  const captions = [];

  // Add caption for each event
  for (const event of clip.merged_events) {
    if (event.event_type === "dragon_kill") {
      captions.push({
        time: event.clip_timestamp,
        text: "DRAGON SECURED!",
        style: "dragon_caption"
      });
    }
  }

  // Primary event gets special treatment
  if (clip.primary_event.event_type.multikill === 5) {
    captions.push({
      time: clip.primary_event.clip_timestamp,
      text: "PENTAKILL!",
      style: "pentakill_caption",
      fontSize: 72,
      color: "#FFD700"
    });
  }

  return captions;
}
```

**2. Smart Transitions**

```typescript
// Apply transitions based on event priority
function applyTransitions(clip: ClipMetadataV2): Transition[] {
  const transitions = [];

  for (const event of clip.get_all_events()) {
    if (event.priority >= 4) {
      // High-priority events get dramatic transitions
      transitions.push({
        time: event.clip_timestamp - 0.5,
        type: "flash",
        duration: 0.3
      });
    } else if (event.priority >= 2) {
      // Medium-priority events get subtle transitions
      transitions.push({
        time: event.clip_timestamp - 0.2,
        type: "fade",
        duration: 0.2
      });
    }
  }

  return transitions;
}
```

**3. Audio Mixing**

```typescript
// Adjust audio levels based on event timing
function mixAudio(clip: ClipMetadataV2): AudioMix {
  const mix = {
    system_audio: [],
    microphone: []
  };

  // Duck system audio during microphone speech
  for (const event of clip.get_all_events()) {
    const start = event.clip_timestamp - 1.0;
    const end = event.clip_timestamp + 2.0;

    mix.system_audio.push({
      time: start,
      volume: 50  // Reduce game volume during commentary
    });

    mix.microphone.push({
      time: start,
      volume: 120  // Boost microphone
    });
  }

  return mix;
}
```

**4. Search & Filter UI**

```tsx
// React component for clip library
function ClipLibrary() {
  const [clips, setClips] = useState<ClipMetadataV2[]>([]);
  const [filter, setFilter] = useState({
    minPriority: 3,
    tags: ["pentakill"],
    favoritesOnly: false
  });

  const loadClips = async () => {
    let results = await invoke<ClipMetadataV2[]>('get_all_clips', {
      gameId: currentGameId
    });

    // Filter by priority
    results = results.filter(c => c.priority >= filter.minPriority);

    // Filter by tags
    if (filter.tags.length > 0) {
      results = results.filter(c =>
        filter.tags.some(tag => c.tags.includes(tag))
      );
    }

    // Filter favorites
    if (filter.favoritesOnly) {
      results = results.filter(c =>
        c.annotations?.favorite === true
      );
    }

    setClips(results);
  };

  return (
    <div className="clip-library">
      <FilterPanel filter={filter} onChange={setFilter} />

      <ClipGrid>
        {clips.map(clip => (
          <ClipCard
            key={clip.clip_id}
            clip={clip}
            onOpen={() => openInEditor(clip)}
          />
        ))}
      </ClipGrid>
    </div>
  );
}
```

### Phase 5: Exporting

**Editor re-encodes with quality preservation:**

```typescript
// Use original encoding settings for best quality
async function exportClip(clip: ClipMetadataV2, edits: Edit[]) {
  const exportSettings = {
    // Preserve original quality
    resolution: clip.video_info.resolution,
    frameRate: clip.video_info.frame_rate,
    codec: clip.video_info.codec,
    bitrate: clip.video_info.bitrate_kbps,

    // Apply edits
    captions: generateCaptions(clip),
    transitions: applyTransitions(clip),
    audioMix: mixAudio(clip),

    // User edits
    trim: edits.trim,
    effects: edits.effects,
    customCaptions: edits.captions
  };

  await invoke('export_video', {
    inputPath: clip.file_path,
    outputPath: getExportPath(),
    settings: exportSettings
  });
}
```

---

## API Methods for Editor

### Rust Backend (Tauri Commands)

```rust
#[tauri::command]
async fn get_all_clips(game_id: String) -> Result<Vec<ClipMetadataV2>, String> {
    let storage = get_storage()?;
    storage.load_all_clips_v2(&game_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_clip_metadata(clip_path: String) -> Result<ClipMetadataV2, String> {
    let storage = get_storage()?;
    storage.load_clip_metadata_v2(&clip_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn update_clip_annotations(
    clip_path: String,
    annotations: UserAnnotations,
) -> Result<(), String> {
    let storage = get_storage()?;
    let mut clip = storage.load_clip_metadata_v2(&clip_path)
        .map_err(|e| e.to_string())?;

    clip.annotations = Some(annotations);

    let game_id = ClipMetadataV2::extract_game_id(&clip_path);
    storage.save_clip_metadata_v2(&game_id, &clip)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn add_clip_marker(
    clip_path: String,
    marker: TimelineMarker,
) -> Result<(), String> {
    let storage = get_storage()?;
    let mut clip = storage.load_clip_metadata_v2(&clip_path)
        .map_err(|e| e.to_string())?;

    clip.add_marker(marker);

    let game_id = ClipMetadataV2::extract_game_id(&clip_path);
    storage.save_clip_metadata_v2(&game_id, &clip)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_clips_by_tags(
    game_id: String,
    tags: Vec<String>,
) -> Result<Vec<ClipMetadataV2>, String> {
    let storage = get_storage()?;
    storage.search_clips_by_tags(&game_id, &tags)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_favorite_clips(game_id: String) -> Result<Vec<ClipMetadataV2>, String> {
    let storage = get_storage()?;
    storage.get_favorite_clips(&game_id)
        .map_err(|e| e.to_string())
}
```

### TypeScript Frontend

```typescript
// src/lib/editor/types.ts
export interface ClipMetadataV2 {
  clip_id: string;
  game_id: string;
  file_path: string;
  thumbnail_path?: string;

  created_at: string;
  game_time_start: number;
  game_time_end: number;
  clip_duration: number;

  primary_event: EventInfo;
  merged_events: EventInfo[];
  event_window: EventWindow;

  priority: number;
  tags: string[];

  video_info: VideoInfo;
  audio_info: AudioInfo;
  timeline: ClipTimeline;
  game_context: GameContext;
  annotations?: UserAnnotations;
}

// src/lib/editor/api.ts
export async function loadClipForEditor(clipPath: string): Promise<ClipMetadataV2> {
  return invoke<ClipMetadataV2>('get_clip_metadata', { clipPath });
}

export async function updateClipRating(clipPath: string, rating: number): Promise<void> {
  const clip = await loadClipForEditor(clipPath);

  if (!clip.annotations) {
    clip.annotations = {
      title: null,
      description: null,
      rating,
      favorite: false,
      notes: [],
      custom_tags: []
    };
  } else {
    clip.annotations.rating = rating;
  }

  await invoke('update_clip_annotations', {
    clipPath,
    annotations: clip.annotations
  });
}

export async function addTimelineMarker(
  clipPath: string,
  timestamp: number,
  label: string
): Promise<void> {
  const marker: TimelineMarker = {
    timestamp,
    marker_type: 'custom',
    label,
    color: '#00FF00'
  };

  await invoke('add_clip_marker', { clipPath, marker });
}
```

---

## Advantages of This Architecture

### ✅ Rich Metadata Storage
- **Individual JSON per clip** enables detailed information
- **Timeline markers** for frame-accurate editing
- **Event merging info** shows what was combined
- **Video/audio specs** preserve quality during re-encoding

### ✅ Editor Integration
- **Auto-captions** from event types
- **Smart transitions** based on priority
- **Audio mixing** with separate tracks
- **Search & filter** by tags, priority, favorites

### ✅ Scalability
- **Backward compatible** with V1 clips.json
- **Extensible** - easy to add new fields
- **Performance** - individual JSONs prevent loading all data at once
- **Search optimization** - clips.json index for fast listing

### ✅ User Experience
- **Rich editing features** powered by metadata
- **Intelligent automation** reduces manual work
- **Quality preservation** via original encoding settings
- **Cross-session continuity** with annotations and ratings

---

## Summary

**Yes, clip metadata IS comprehensively stored as JSON** to enable advanced editing:

✅ **Each clip has its own JSON file** alongside the video
✅ **Rich event information** with timeline markers and merged events
✅ **Video/audio technical specs** for quality-preserving re-encoding
✅ **Game context** (champion, items, KDA, team state)
✅ **User annotations** (notes, ratings, favorites, custom tags)
✅ **Editor-friendly API** for loading, searching, and updating clips
✅ **Metadata-driven features** (auto-captions, transitions, audio mixing)

This architecture ensures that when you build the video editor later, it will have all the information needed to create intelligent, automated, and high-quality edited videos.
