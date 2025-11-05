import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Mic, Volume2 } from "lucide-react";

type SampleRate = "hz44100" | "hz48000";
type AudioBitrate = "kbps128" | "kbps192" | "kbps256" | "kbps320";

interface AudioSettings {
  record_microphone: boolean;
  microphone_device: string | null;
  microphone_volume: number; // 0-200
  record_system_audio: boolean;
  system_audio_device: string | null;
  system_audio_volume: number; // 0-200
  sample_rate: SampleRate;
  bitrate: AudioBitrate;
}

interface AudioSettingsProps {
  settings: AudioSettings;
  onChange: (settings: AudioSettings) => void;
}

export function AudioSettings({ settings, onChange }: AudioSettingsProps) {
  const updateSetting = <K extends keyof AudioSettings>(
    key: K,
    value: AudioSettings[K]
  ) => {
    onChange({ ...settings, [key]: value });
  };

  const getSampleRateLabel = (rate: SampleRate): string => {
    const labels: Record<SampleRate, string> = {
      hz44100: "44.1 kHz (CD Quality)",
      hz48000: "48 kHz (Professional)",
    };
    return labels[rate];
  };

  const getBitrateLabel = (bitrate: AudioBitrate): string => {
    const labels: Record<AudioBitrate, string> = {
      kbps128: "128 kbps (Standard)",
      kbps192: "192 kbps (High)",
      kbps256: "256 kbps (Very High)",
      kbps320: "320 kbps (Maximum)",
    };
    return labels[bitrate];
  };

  return (
    <div className="space-y-6">
      {/* Microphone Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <Mic className="w-4 h-4" />
            Microphone Recording
          </CardTitle>
          <CardDescription>
            Record your voice commentary during gameplay
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label htmlFor="record_microphone" className="flex-1 cursor-pointer">
              Enable Microphone Recording
            </Label>
            <Switch
              id="record_microphone"
              checked={settings.record_microphone}
              onCheckedChange={(checked: boolean) => updateSetting("record_microphone", checked)}
            />
          </div>

          {settings.record_microphone && (
            <>
              <div className="space-y-2">
                <Label>Microphone Device</Label>
                <Select
                  value={settings.microphone_device || "default"}
                  onValueChange={(value) =>
                    updateSetting("microphone_device", value === "default" ? null : value)
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select device" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="default">Default Device</SelectItem>
                    {/* TODO: List actual audio input devices via Tauri command */}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <Label>Microphone Volume</Label>
                  <Badge variant="secondary">{settings.microphone_volume}%</Badge>
                </div>
                <Slider
                  value={[settings.microphone_volume]}
                  onValueChange={([value]) => updateSetting("microphone_volume", value)}
                  min={0}
                  max={200}
                  step={5}
                  className="w-full"
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>Muted</span>
                  <span>100%</span>
                  <span>200% (Boost)</span>
                </div>
              </div>
            </>
          )}
        </CardContent>
      </Card>

      {/* System Audio Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <Volume2 className="w-4 h-4" />
            System Audio Recording
          </CardTitle>
          <CardDescription>
            Record game sounds, music, and other system audio
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label htmlFor="record_system_audio" className="flex-1 cursor-pointer">
              Enable System Audio Recording
            </Label>
            <Switch
              id="record_system_audio"
              checked={settings.record_system_audio}
              onCheckedChange={(checked: boolean) => updateSetting("record_system_audio", checked)}
            />
          </div>

          {settings.record_system_audio && (
            <>
              <div className="space-y-2">
                <Label>System Audio Device</Label>
                <Select
                  value={settings.system_audio_device || "default"}
                  onValueChange={(value) =>
                    updateSetting("system_audio_device", value === "default" ? null : value)
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select device" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="default">Default Device</SelectItem>
                    {/* TODO: List actual audio output devices via Tauri command */}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <Label>System Audio Volume</Label>
                  <Badge variant="secondary">{settings.system_audio_volume}%</Badge>
                </div>
                <Slider
                  value={[settings.system_audio_volume]}
                  onValueChange={([value]) => updateSetting("system_audio_volume", value)}
                  min={0}
                  max={200}
                  step={5}
                  className="w-full"
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>Muted</span>
                  <span>100%</span>
                  <span>200% (Boost)</span>
                </div>
              </div>
            </>
          )}
        </CardContent>
      </Card>

      {/* Audio Quality Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Audio Quality</CardTitle>
          <CardDescription>
            Configure sample rate and bitrate for recordings
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>Sample Rate</Label>
            <div className="flex items-center gap-4">
              <div className="flex-1">
                <Select
                  value={settings.sample_rate}
                  onValueChange={(value) => updateSetting("sample_rate", value as SampleRate)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="hz44100">{getSampleRateLabel("hz44100")}</SelectItem>
                    <SelectItem value="hz48000">{getSampleRateLabel("hz48000")}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              {settings.sample_rate === "hz48000" && (
                <Badge variant="secondary">Recommended</Badge>
              )}
            </div>
          </div>

          <div className="space-y-2">
            <Label>Audio Bitrate</Label>
            <div className="flex items-center gap-4">
              <div className="flex-1">
                <Select
                  value={settings.bitrate}
                  onValueChange={(value) => updateSetting("bitrate", value as AudioBitrate)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="kbps128">{getBitrateLabel("kbps128")}</SelectItem>
                    <SelectItem value="kbps192">{getBitrateLabel("kbps192")}</SelectItem>
                    <SelectItem value="kbps256">{getBitrateLabel("kbps256")}</SelectItem>
                    <SelectItem value="kbps320">{getBitrateLabel("kbps320")}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              {settings.bitrate === "kbps192" && (
                <Badge variant="secondary">Recommended</Badge>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Info Card */}
      {(settings.record_microphone || settings.record_system_audio) && (
        <Card className="bg-muted/50">
          <CardContent className="pt-6">
            <div className="space-y-2 text-sm">
              <p className="font-semibold">Current Audio Configuration</p>
              {settings.record_microphone && (
                <>
                  <p className="text-muted-foreground">
                    • Microphone: Enabled ({settings.microphone_volume}% volume)
                  </p>
                  <p className="text-muted-foreground">
                    • Device: {settings.microphone_device || "Default"}
                  </p>
                </>
              )}
              {settings.record_system_audio && (
                <>
                  <p className="text-muted-foreground">
                    • System Audio: Enabled ({settings.system_audio_volume}% volume)
                  </p>
                  <p className="text-muted-foreground">
                    • Device: {settings.system_audio_device || "Default"}
                  </p>
                </>
              )}
              <p className="text-muted-foreground">
                • Quality: {getSampleRateLabel(settings.sample_rate)} @ {getBitrateLabel(settings.bitrate)}
              </p>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
