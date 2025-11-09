import { useTranslation } from "react-i18next";
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
  const { t } = useTranslation();
  const updateSetting = <K extends keyof AudioSettings>(
    key: K,
    value: AudioSettings[K]
  ) => {
    onChange({ ...settings, [key]: value });
  };

  const getSampleRateLabel = (rate: SampleRate): string => {
    const labels: Record<SampleRate, string> = {
      hz44100: t('settings.recordingConfig.audioSettings.audioQuality.sampleRateLabels.hz44100'),
      hz48000: t('settings.recordingConfig.audioSettings.audioQuality.sampleRateLabels.hz48000'),
    };
    return labels[rate];
  };

  const getBitrateLabel = (bitrate: AudioBitrate): string => {
    const labels: Record<AudioBitrate, string> = {
      kbps128: t('settings.recordingConfig.audioSettings.audioQuality.bitrateLabels.kbps128'),
      kbps192: t('settings.recordingConfig.audioSettings.audioQuality.bitrateLabels.kbps192'),
      kbps256: t('settings.recordingConfig.audioSettings.audioQuality.bitrateLabels.kbps256'),
      kbps320: t('settings.recordingConfig.audioSettings.audioQuality.bitrateLabels.kbps320'),
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
            {t('settings.recordingConfig.audioSettings.microphoneRecording.title')}
          </CardTitle>
          <CardDescription>
            {t('settings.recordingConfig.audioSettings.microphoneRecording.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label htmlFor="record_microphone" className="flex-1 cursor-pointer">
              {t('settings.recordingConfig.audioSettings.microphoneRecording.enableMicrophone')}
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
                <Label>{t('settings.recordingConfig.audioSettings.microphoneRecording.microphoneDevice')}</Label>
                <Select
                  value={settings.microphone_device || "default"}
                  onValueChange={(value) =>
                    updateSetting("microphone_device", value === "default" ? null : value)
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('settings.recordingConfig.audioSettings.microphoneRecording.selectDevice')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="default">{t('settings.recordingConfig.audioSettings.microphoneRecording.defaultDevice')}</SelectItem>
                    {/* TODO: List actual audio input devices via Tauri command */}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <Label>{t('settings.recordingConfig.audioSettings.microphoneRecording.microphoneVolume')}</Label>
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
                  <span>{t('settings.recordingConfig.audioSettings.microphoneRecording.muted')}</span>
                  <span>100%</span>
                  <span>{t('settings.recordingConfig.audioSettings.microphoneRecording.boost')}</span>
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
            {t('settings.recordingConfig.audioSettings.systemAudioRecording.title')}
          </CardTitle>
          <CardDescription>
            {t('settings.recordingConfig.audioSettings.systemAudioRecording.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <Label htmlFor="record_system_audio" className="flex-1 cursor-pointer">
              {t('settings.recordingConfig.audioSettings.systemAudioRecording.enableSystemAudio')}
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
                <Label>{t('settings.recordingConfig.audioSettings.systemAudioRecording.systemAudioDevice')}</Label>
                <Select
                  value={settings.system_audio_device || "default"}
                  onValueChange={(value) =>
                    updateSetting("system_audio_device", value === "default" ? null : value)
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t('settings.recordingConfig.audioSettings.systemAudioRecording.selectDevice')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="default">{t('settings.recordingConfig.audioSettings.systemAudioRecording.defaultDevice')}</SelectItem>
                    {/* TODO: List actual audio output devices via Tauri command */}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <Label>{t('settings.recordingConfig.audioSettings.systemAudioRecording.systemAudioVolume')}</Label>
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
                  <span>{t('settings.recordingConfig.audioSettings.systemAudioRecording.muted')}</span>
                  <span>100%</span>
                  <span>{t('settings.recordingConfig.audioSettings.systemAudioRecording.boost')}</span>
                </div>
              </div>
            </>
          )}
        </CardContent>
      </Card>

      {/* Audio Quality Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t('settings.recordingConfig.audioSettings.audioQuality.title')}</CardTitle>
          <CardDescription>
            {t('settings.recordingConfig.audioSettings.audioQuality.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>{t('settings.recordingConfig.audioSettings.audioQuality.sampleRate')}</Label>
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
                <Badge variant="secondary">{t('settings.recordingConfig.audioSettings.audioQuality.recommended')}</Badge>
              )}
            </div>
          </div>

          <div className="space-y-2">
            <Label>{t('settings.recordingConfig.audioSettings.audioQuality.audioBitrate')}</Label>
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
                <Badge variant="secondary">{t('settings.recordingConfig.audioSettings.audioQuality.recommended')}</Badge>
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
              <p className="font-semibold">{t('settings.recordingConfig.audioSettings.currentConfiguration.title')}</p>
              {settings.record_microphone && (
                <>
                  <p className="text-muted-foreground">
                    • {t('settings.recordingConfig.audioSettings.currentConfiguration.microphone')}: {t('settings.recordingConfig.audioSettings.currentConfiguration.enabled')} ({settings.microphone_volume}% {t('settings.recordingConfig.audioSettings.currentConfiguration.volume')})
                  </p>
                  <p className="text-muted-foreground">
                    • {t('settings.recordingConfig.audioSettings.currentConfiguration.device')}: {settings.microphone_device || t('settings.recordingConfig.audioSettings.currentConfiguration.default')}
                  </p>
                </>
              )}
              {settings.record_system_audio && (
                <>
                  <p className="text-muted-foreground">
                    • {t('settings.recordingConfig.audioSettings.currentConfiguration.systemAudio')}: {t('settings.recordingConfig.audioSettings.currentConfiguration.enabled')} ({settings.system_audio_volume}% {t('settings.recordingConfig.audioSettings.currentConfiguration.volume')})
                  </p>
                  <p className="text-muted-foreground">
                    • {t('settings.recordingConfig.audioSettings.currentConfiguration.device')}: {settings.system_audio_device || t('settings.recordingConfig.audioSettings.currentConfiguration.default')}
                  </p>
                </>
              )}
              <p className="text-muted-foreground">
                • {t('settings.recordingConfig.audioSettings.currentConfiguration.quality')}: {getSampleRateLabel(settings.sample_rate)} @ {getBitrateLabel(settings.bitrate)}
              </p>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
