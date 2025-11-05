# Recording Settings Design

자동 하이라이트 클립 녹화를 위한 포괄적인 설정 시스템 설계

---

## 설정 카테고리

### 1. Event Filtering (이벤트 필터링) - 어떤 이벤트를 녹화할 것인가

사용자가 자동으로 녹화하고 싶은 이벤트 유형을 선택합니다.

#### 이벤트 카테고리

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilterSettings {
    // 킬 관련
    pub record_kills: bool,              // 챔피언 킬
    pub record_multikills: bool,         // 더블킬, 트리플킬 등
    pub record_first_blood: bool,        // 퍼스트 블러드

    // 데스 관련
    pub record_deaths: bool,             // 내 데스 (복기용)
    pub record_shutdown: bool,           // 현상금 데스

    // 어시스트 관련
    pub record_assists: bool,            // 어시스트 참여

    // 오브젝트
    pub record_dragon: bool,             // 드래곤
    pub record_baron: bool,              // 바론
    pub record_elder: bool,              // 장로 드래곤
    pub record_herald: bool,             // 전령

    // 구조물
    pub record_turret: bool,             // 타워 파괴
    pub record_inhibitor: bool,          // 억제기 파괴
    pub record_nexus: bool,              // 넥서스 파괴

    // 특수 이벤트
    pub record_ace: bool,                // 에이스
    pub record_game_end: bool,           // 게임 종료 (승리/패배)
    pub record_steal: bool,              // 오브젝트 스틸

    // 우선순위 필터
    pub min_priority: u8,                // 최소 우선순위 (1-5)
}

impl Default for EventFilterSettings {
    fn default() -> Self {
        Self {
            // 기본적으로 하이라이트만 녹화
            record_kills: true,
            record_multikills: true,
            record_first_blood: true,

            record_deaths: false,        // 데스는 기본적으로 OFF
            record_shutdown: false,

            record_assists: false,       // 어시스트는 기본적으로 OFF

            record_dragon: true,
            record_baron: true,
            record_elder: true,
            record_herald: true,

            record_turret: false,        // 타워는 너무 많아서 OFF
            record_inhibitor: true,
            record_nexus: true,

            record_ace: true,
            record_game_end: true,
            record_steal: true,

            min_priority: 2,             // 우선순위 2 이상만
        }
    }
}
```

### 2. Game Mode Filtering (게임 모드 필터링)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameModeSettings {
    pub record_ranked_solo: bool,       // 개인/2인 랭크
    pub record_ranked_flex: bool,       // 자유 랭크
    pub record_normal: bool,            // 빠른 대전
    pub record_quick_play: bool,        // 신속 대전
    pub record_aram: bool,              // 칼바람 나락
    pub record_arena: bool,             // 아레나
    pub record_special: bool,           // 특별 모드 (URF 등)
    pub record_custom: bool,            // 사용자 설정
    pub record_practice: bool,          // 연습 모드
}

impl Default for GameModeSettings {
    fn default() -> Self {
        Self {
            record_ranked_solo: true,
            record_ranked_flex: true,
            record_normal: true,
            record_quick_play: true,
            record_aram: true,
            record_arena: true,
            record_special: false,       // 특별 모드는 기본 OFF
            record_custom: false,        // 커스텀은 기본 OFF
            record_practice: false,      // 연습은 기본 OFF
        }
    }
}
```

### 3. Video Settings (영상 설정)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    pub resolution: Resolution,
    pub frame_rate: FrameRate,
    pub bitrate_preset: BitratePreset,
    pub codec: VideoCodec,
    pub encoder: EncoderPreference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    R1920x1080,   // 1080p (추천)
    R2560x1440,   // 1440p
    R3840x2160,   // 4K
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameRate {
    Fps30,
    Fps60,        // 추천
    Fps120,
    Fps144,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BitratePreset {
    Low,          // 10 Mbps (720p60)
    Medium,       // 20 Mbps (1080p60) - 추천
    High,         // 40 Mbps (1440p60)
    VeryHigh,     // 80 Mbps (4K60)
    Custom(u32),  // 사용자 지정 (kbps)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    H264,         // 호환성 최고
    H265,         // 효율성 최고 (추천)
    Av1,          // 차세대 (실험적)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncoderPreference {
    Auto,         // 자동 선택 (추천)
    Nvenc,        // NVIDIA GPU
    Qsv,          // Intel GPU
    Amf,          // AMD GPU
    Software,     // CPU (느림, 호환성 높음)
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            resolution: Resolution::R1920x1080,
            frame_rate: FrameRate::Fps60,
            bitrate_preset: BitratePreset::Medium,
            codec: VideoCodec::H265,
            encoder: EncoderPreference::Auto,
        }
    }
}
```

### 4. Audio Settings (오디오 설정)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    // 마이크 녹음
    pub record_microphone: bool,
    pub microphone_device: Option<String>,
    pub microphone_volume: u8,           // 0-200%

    // 시스템 오디오 녹음
    pub record_system_audio: bool,
    pub system_audio_device: Option<String>,
    pub system_audio_volume: u8,         // 0-200%

    // 오디오 품질
    pub sample_rate: SampleRate,
    pub bitrate: AudioBitrate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleRate {
    Hz44100,
    Hz48000,      // 추천
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioBitrate {
    Kbps128,
    Kbps192,      // 추천
    Kbps256,
    Kbps320,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            record_microphone: true,
            microphone_device: None,      // 기본 장치
            microphone_volume: 120,       // 120%

            record_system_audio: true,
            system_audio_device: None,    // 기본 장치
            system_audio_volume: 100,     // 100%

            sample_rate: SampleRate::Hz48000,
            bitrate: AudioBitrate::Kbps192,
        }
    }
}
```

### 5. Clip Timing Settings (클립 타이밍 설정)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipTimingSettings {
    // 기본 클립 길이
    pub default_pre_duration: u32,       // 이벤트 이전 (초)
    pub default_post_duration: u32,      // 이벤트 이후 (초)

    // 이벤트별 커스텀 타이밍
    pub event_timings: HashMap<String, EventTiming>,

    // 이벤트 병합
    pub merge_consecutive_events: bool,
    pub merge_time_threshold: f64,       // 15초 기본
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTiming {
    pub pre_duration: u32,
    pub post_duration: u32,
}

impl Default for ClipTimingSettings {
    fn default() -> Self {
        let mut event_timings = HashMap::new();

        // 멀티킬은 길게
        event_timings.insert("multikill".to_string(), EventTiming {
            pre_duration: 15,
            post_duration: 5,
        });

        // 스틸은 더 길게 (빌드업 포함)
        event_timings.insert("steal".to_string(), EventTiming {
            pre_duration: 20,
            post_duration: 5,
        });

        // 일반 킬은 짧게
        event_timings.insert("kill".to_string(), EventTiming {
            pre_duration: 10,
            post_duration: 3,
        });

        Self {
            default_pre_duration: 10,
            default_post_duration: 3,
            event_timings,
            merge_consecutive_events: true,
            merge_time_threshold: 15.0,
        }
    }
}
```

### 6. Hotkey Settings (핫키 설정)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySettings {
    pub manual_save_clip: String,        // "F8" 기본
    pub toggle_recording: String,        // "F9" 기본
    pub delete_last_clip: String,        // "F10" 기본
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            manual_save_clip: "F8".to_string(),
            toggle_recording: "F9".to_string(),
            delete_last_clip: "F10".to_string(),
        }
    }
}
```

### 7. Unified Recording Settings

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSettings {
    pub event_filter: EventFilterSettings,
    pub game_mode: GameModeSettings,
    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub clip_timing: ClipTimingSettings,
    pub hotkeys: HotkeySettings,

    // 일반 설정
    pub auto_start_with_league: bool,
    pub minimize_to_tray: bool,
    pub show_notifications: bool,
}

impl Default for RecordingSettings {
    fn default() -> Self {
        Self {
            event_filter: EventFilterSettings::default(),
            game_mode: GameModeSettings::default(),
            video: VideoSettings::default(),
            audio: AudioSettings::default(),
            clip_timing: ClipTimingSettings::default(),
            hotkeys: HotkeySettings::default(),

            auto_start_with_league: true,
            minimize_to_tray: true,
            show_notifications: true,
        }
    }
}
```

---

## 설정 UI 구현 (React/TypeScript)

### Settings.tsx

```tsx
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

interface RecordingSettings {
  event_filter: EventFilterSettings;
  game_mode: GameModeSettings;
  video: VideoSettings;
  audio: AudioSettings;
  clip_timing: ClipTimingSettings;
  hotkeys: HotkeySettings;
}

interface EventFilterSettings {
  record_kills: boolean;
  record_multikills: boolean;
  record_first_blood: boolean;
  record_deaths: boolean;
  record_shutdown: boolean;
  record_assists: boolean;
  record_dragon: boolean;
  record_baron: boolean;
  record_elder: boolean;
  record_herald: boolean;
  record_turret: boolean;
  record_inhibitor: boolean;
  record_nexus: boolean;
  record_ace: boolean;
  record_game_end: boolean;
  record_steal: boolean;
  min_priority: number;
}

// ... other interfaces

export function Settings() {
  const [settings, setSettings] = useState<RecordingSettings | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const settings = await invoke<RecordingSettings>("get_recording_settings");
      setSettings(settings);
    } catch (error) {
      console.error("Failed to load settings:", error);
    } finally {
      setLoading(false);
    }
  };

  const saveSettings = async () => {
    try {
      await invoke("save_recording_settings", { settings });
      // Show success notification
    } catch (error) {
      console.error("Failed to save settings:", error);
    }
  };

  if (loading || !settings) {
    return <div>Loading...</div>;
  }

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Settings</h1>
        <Button onClick={saveSettings}>Save Settings</Button>
      </div>

      <Tabs defaultValue="events" className="w-full">
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="events">이벤트</TabsTrigger>
          <TabsTrigger value="modes">게임 모드</TabsTrigger>
          <TabsTrigger value="video">영상</TabsTrigger>
          <TabsTrigger value="audio">오디오</TabsTrigger>
          <TabsTrigger value="timing">타이밍</TabsTrigger>
        </TabsList>

        {/* 이벤트 필터링 탭 */}
        <TabsContent value="events" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>녹화할 이벤트 선택</CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* 킬 관련 */}
              <div className="space-y-3">
                <h3 className="text-lg font-semibold">킬 관련</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_kills">챔피언 킬</Label>
                    <Switch
                      id="record_kills"
                      checked={settings.event_filter.record_kills}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_kills: checked,
                          },
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_multikills">멀티킬 (더블, 트리플 등)</Label>
                    <Switch
                      id="record_multikills"
                      checked={settings.event_filter.record_multikills}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_multikills: checked,
                          },
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_first_blood">퍼스트 블러드</Label>
                    <Switch
                      id="record_first_blood"
                      checked={settings.event_filter.record_first_blood}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_first_blood: checked,
                          },
                        })
                      }
                    />
                  </div>
                </div>
              </div>

              {/* 데스 관련 */}
              <div className="space-y-3">
                <h3 className="text-lg font-semibold">데스 관련</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_deaths">내 데스 (복기용)</Label>
                    <Switch
                      id="record_deaths"
                      checked={settings.event_filter.record_deaths}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_deaths: checked,
                          },
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_shutdown">현상금 데스</Label>
                    <Switch
                      id="record_shutdown"
                      checked={settings.event_filter.record_shutdown}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_shutdown: checked,
                          },
                        })
                      }
                    />
                  </div>
                </div>
              </div>

              {/* 어시스트 */}
              <div className="space-y-3">
                <h3 className="text-lg font-semibold">어시스트</h3>
                <div className="flex items-center justify-between">
                  <Label htmlFor="record_assists">어시스트 참여</Label>
                  <Switch
                    id="record_assists"
                    checked={settings.event_filter.record_assists}
                    onCheckedChange={(checked) =>
                      setSettings({
                        ...settings,
                        event_filter: {
                          ...settings.event_filter,
                          record_assists: checked,
                        },
                      })
                    }
                  />
                </div>
              </div>

              {/* 오브젝트 */}
              <div className="space-y-3">
                <h3 className="text-lg font-semibold">오브젝트</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_dragon">드래곤</Label>
                    <Switch
                      id="record_dragon"
                      checked={settings.event_filter.record_dragon}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_dragon: checked,
                          },
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_baron">바론</Label>
                    <Switch
                      id="record_baron"
                      checked={settings.event_filter.record_baron}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_baron: checked,
                          },
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_elder">장로 드래곤</Label>
                    <Switch
                      id="record_elder"
                      checked={settings.event_filter.record_elder}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_elder: checked,
                          },
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_herald">전령</Label>
                    <Switch
                      id="record_herald"
                      checked={settings.event_filter.record_herald}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_herald: checked,
                          },
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_steal">오브젝트 스틸</Label>
                    <Switch
                      id="record_steal"
                      checked={settings.event_filter.record_steal}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_steal: checked,
                          },
                        })
                      }
                    />
                  </div>
                </div>
              </div>

              {/* 구조물 */}
              <div className="space-y-3">
                <h3 className="text-lg font-semibold">타워 / 억제기</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_turret">타워 파괴</Label>
                    <Switch
                      id="record_turret"
                      checked={settings.event_filter.record_turret}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_turret: checked,
                          },
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_inhibitor">억제기 파괴</Label>
                    <Switch
                      id="record_inhibitor"
                      checked={settings.event_filter.record_inhibitor}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_inhibitor: checked,
                          },
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_nexus">넥서스 파괴</Label>
                    <Switch
                      id="record_nexus"
                      checked={settings.event_filter.record_nexus}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_nexus: checked,
                          },
                        })
                      }
                    />
                  </div>
                </div>
              </div>

              {/* 특수 이벤트 */}
              <div className="space-y-3">
                <h3 className="text-lg font-semibold">특수 이벤트</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_ace">에이스</Label>
                    <Switch
                      id="record_ace"
                      checked={settings.event_filter.record_ace}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_ace: checked,
                          },
                        })
                      }
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <Label htmlFor="record_game_end">게임 종료 (승리/패배)</Label>
                    <Switch
                      id="record_game_end"
                      checked={settings.event_filter.record_game_end}
                      onCheckedChange={(checked) =>
                        setSettings({
                          ...settings,
                          event_filter: {
                            ...settings.event_filter,
                            record_game_end: checked,
                          },
                        })
                      }
                    />
                  </div>
                </div>
              </div>

              {/* 우선순위 필터 */}
              <div className="space-y-3">
                <h3 className="text-lg font-semibold">우선순위 필터</h3>
                <div className="space-y-2">
                  <Label>최소 우선순위: {settings.event_filter.min_priority}</Label>
                  <Slider
                    min={1}
                    max={5}
                    step={1}
                    value={[settings.event_filter.min_priority]}
                    onValueChange={(value) =>
                      setSettings({
                        ...settings,
                        event_filter: {
                          ...settings.event_filter,
                          min_priority: value[0],
                        },
                      })
                    }
                  />
                  <p className="text-sm text-muted-foreground">
                    {settings.event_filter.min_priority === 1 && "모든 이벤트"}
                    {settings.event_filter.min_priority === 2 && "일반 이벤트 이상"}
                    {settings.event_filter.min_priority === 3 && "중요 이벤트만"}
                    {settings.event_filter.min_priority === 4 && "매우 중요한 이벤트만"}
                    {settings.event_filter.min_priority === 5 && "펜타킬만"}
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* 게임 모드 탭 */}
        <TabsContent value="modes" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>녹화할 게임 모드 선택</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <div className="flex items-center justify-between">
                  <Label>개인/2인 랭크</Label>
                  <Switch
                    checked={settings.game_mode.record_ranked_solo}
                    onCheckedChange={(checked) =>
                      setSettings({
                        ...settings,
                        game_mode: {
                          ...settings.game_mode,
                          record_ranked_solo: checked,
                        },
                      })
                    }
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label>자유 랭크</Label>
                  <Switch
                    checked={settings.game_mode.record_ranked_flex}
                    onCheckedChange={(checked) =>
                      setSettings({
                        ...settings,
                        game_mode: {
                          ...settings.game_mode,
                          record_ranked_flex: checked,
                        },
                      })
                    }
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label>빠른 대전</Label>
                  <Switch
                    checked={settings.game_mode.record_normal}
                    onCheckedChange={(checked) =>
                      setSettings({
                        ...settings,
                        game_mode: {
                          ...settings.game_mode,
                          record_normal: checked,
                        },
                      })
                    }
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label>신속 대전</Label>
                  <Switch
                    checked={settings.game_mode.record_quick_play}
                    onCheckedChange={(checked) =>
                      setSettings({
                        ...settings,
                        game_mode: {
                          ...settings.game_mode,
                          record_quick_play: checked,
                        },
                      })
                    }
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label>칼바람 나락 (ARAM)</Label>
                  <Switch
                    checked={settings.game_mode.record_aram}
                    onCheckedChange={(checked) =>
                      setSettings({
                        ...settings,
                        game_mode: {
                          ...settings.game_mode,
                          record_aram: checked,
                        },
                      })
                    }
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label>아레나</Label>
                  <Switch
                    checked={settings.game_mode.record_arena}
                    onCheckedChange={(checked) =>
                      setSettings({
                        ...settings,
                        game_mode: {
                          ...settings.game_mode,
                          record_arena: checked,
                        },
                      })
                    }
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label>특별 모드 (URF 등)</Label>
                  <Switch
                    checked={settings.game_mode.record_special}
                    onCheckedChange={(checked) =>
                      setSettings({
                        ...settings,
                        game_mode: {
                          ...settings.game_mode,
                          record_special: checked,
                        },
                      })
                    }
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label>사용자 설정</Label>
                  <Switch
                    checked={settings.game_mode.record_custom}
                    onCheckedChange={(checked) =>
                      setSettings({
                        ...settings,
                        game_mode: {
                          ...settings.game_mode,
                          record_custom: checked,
                        },
                      })
                    }
                  />
                </div>

                <div className="flex items-center justify-between">
                  <Label>연습 모드</Label>
                  <Switch
                    checked={settings.game_mode.record_practice}
                    onCheckedChange={(checked) =>
                      setSettings({
                        ...settings,
                        game_mode: {
                          ...settings.game_mode,
                          record_practice: checked,
                        },
                      })
                    }
                  />
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* 비디오 설정, 오디오 설정, 타이밍 설정 탭은 생략 (유사한 패턴) */}
      </Tabs>
    </div>
  );
}
```

---

## Tauri Commands (Rust Backend)

```rust
// src-tauri/src/settings/mod.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSettings {
    pub event_filter: EventFilterSettings,
    pub game_mode: GameModeSettings,
    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub clip_timing: ClipTimingSettings,
    pub hotkeys: HotkeySettings,

    pub auto_start_with_league: bool,
    pub minimize_to_tray: bool,
    pub show_notifications: bool,
}

// ... (위의 Rust 구조체들)

impl RecordingSettings {
    /// Load settings from file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let settings_path = Self::get_settings_path()?;

        if settings_path.exists() {
            let json = fs::read_to_string(&settings_path)?;
            let settings = serde_json::from_str(&json)?;
            Ok(settings)
        } else {
            Ok(Self::default())
        }
    }

    /// Save settings to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = Self::get_settings_path()?;

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&settings_path, json)?;

        Ok(())
    }

    fn get_settings_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let app_data = dirs::config_dir()
            .ok_or("Failed to get config directory")?;

        let lolshorts_dir = app_data.join("LoLShorts");
        fs::create_dir_all(&lolshorts_dir)?;

        Ok(lolshorts_dir.join("settings.json"))
    }

    /// Check if an event should be recorded based on settings
    pub fn should_record_event(&self, event_type: &EventType) -> bool {
        let filter = &self.event_filter;

        match event_type {
            EventType::ChampionKill => filter.record_kills,
            EventType::Multikill(_) => filter.record_multikills,
            EventType::FirstBlood => filter.record_first_blood,
            EventType::DragonKill => filter.record_dragon,
            EventType::BaronKill => filter.record_baron,
            EventType::TurretKill => filter.record_turret,
            EventType::InhibitorKill => filter.record_inhibitor,
            EventType::Ace => filter.record_ace,
            _ => true, // 기타 이벤트는 기본 녹화
        }
    }

    /// Check if a game mode should be recorded
    pub fn should_record_game_mode(&self, queue_type: &str) -> bool {
        let mode = &self.game_mode;

        match queue_type {
            "RANKED_SOLO_5x5" => mode.record_ranked_solo,
            "RANKED_FLEX_SR" => mode.record_ranked_flex,
            "NORMAL" => mode.record_normal,
            "ARAM" => mode.record_aram,
            "ARENA" => mode.record_arena,
            "CUSTOM" => mode.record_custom,
            _ => true,
        }
    }
}

// Tauri commands
#[tauri::command]
pub async fn get_recording_settings() -> Result<RecordingSettings, String> {
    RecordingSettings::load()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_recording_settings(settings: RecordingSettings) -> Result<(), String> {
    settings.save()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_settings_to_default() -> Result<RecordingSettings, String> {
    let settings = RecordingSettings::default();
    settings.save()
        .map_err(|e| e.to_string())?;
    Ok(settings)
}
```

---

## 요약

✅ **이벤트 필터링**
- 킬, 어시스트, 데스, 오브젝트, 타워/억제기, 게임 종료 등 **세부 선택 가능**
- 각 이벤트 카테고리를 **토글 스위치**로 활성화/비활성화
- **우선순위 필터** (1-5)로 중요도에 따라 자동 필터링

✅ **게임 모드 필터링**
- 개인/2인 랭크, 자유 랭크, 빠른 대전, 신속 대전, ARAM, 아레나, 특별 모드, 사용자 설정, 연습 모드
- 각 모드별로 녹화 여부 선택 가능

✅ **설정 UI**
- **5개 탭**: 이벤트, 게임 모드, 영상, 오디오, 타이밍
- **실시간 저장**: Save Settings 버튼으로 설정 저장
- **사용자 친화적**: shadcn/ui 컴포넌트로 깔끔한 UI

✅ **설정 적용 로직**
- `should_record_event()`: 이벤트 녹화 여부 판단
- `should_record_game_mode()`: 게임 모드 녹화 여부 판단
- 자동 클립 저장 시스템에 통합 가능

이제 사용자는 **어떤 이벤트를 자동으로 녹화할지 세밀하게 제어**할 수 있습니다! 🎯
