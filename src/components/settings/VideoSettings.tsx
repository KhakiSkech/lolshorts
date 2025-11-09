import { useTranslation } from "react-i18next";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Info } from "lucide-react";

type Resolution = "r1920x1080" | "r2560x1440" | "r3840x2160";
type FrameRate = "fps30" | "fps60" | "fps120" | "fps144";
type BitratePreset = "low" | "medium" | "high" | "very_high";
type VideoCodec = "h264" | "h265" | "av1";
type EncoderPreference = "auto" | "nvenc" | "qsv" | "amf" | "software";

interface VideoSettings {
  resolution: Resolution;
  frame_rate: FrameRate;
  bitrate_preset: BitratePreset;
  codec: VideoCodec;
  encoder: EncoderPreference;
}

interface VideoSettingsProps {
  settings: VideoSettings;
  onChange: (settings: VideoSettings) => void;
}

export function VideoSettings({ settings, onChange }: VideoSettingsProps) {
  const { t } = useTranslation();
  const updateSetting = <K extends keyof VideoSettings>(
    key: K,
    value: VideoSettings[K]
  ) => {
    onChange({ ...settings, [key]: value });
  };

  const getResolutionLabel = (res: Resolution): string => {
    const labels: Record<Resolution, string> = {
      r1920x1080: t('settings.recordingConfig.videoSettings.resolution.labels.r1920x1080'),
      r2560x1440: t('settings.recordingConfig.videoSettings.resolution.labels.r2560x1440'),
      r3840x2160: t('settings.recordingConfig.videoSettings.resolution.labels.r3840x2160'),
    };
    return labels[res];
  };

  const getFrameRateLabel = (fps: FrameRate): string => {
    const labels: Record<FrameRate, string> = {
      fps30: t('settings.recordingConfig.videoSettings.frameRate.labels.fps30'),
      fps60: t('settings.recordingConfig.videoSettings.frameRate.labels.fps60'),
      fps120: t('settings.recordingConfig.videoSettings.frameRate.labels.fps120'),
      fps144: t('settings.recordingConfig.videoSettings.frameRate.labels.fps144'),
    };
    return labels[fps];
  };

  const getBitrateLabel = (bitrate: BitratePreset): string => {
    const labels: Record<BitratePreset, string> = {
      low: t('settings.recordingConfig.videoSettings.bitratePreset.labels.low'),
      medium: t('settings.recordingConfig.videoSettings.bitratePreset.labels.medium'),
      high: t('settings.recordingConfig.videoSettings.bitratePreset.labels.high'),
      very_high: t('settings.recordingConfig.videoSettings.bitratePreset.labels.veryHigh'),
    };
    return labels[bitrate];
  };

  const getCodecLabel = (codec: VideoCodec): string => {
    const labels: Record<VideoCodec, string> = {
      h264: t('settings.recordingConfig.videoSettings.videoCodec.labels.h264'),
      h265: t('settings.recordingConfig.videoSettings.videoCodec.labels.h265'),
      av1: t('settings.recordingConfig.videoSettings.videoCodec.labels.av1'),
    };
    return labels[codec];
  };

  const getEncoderLabel = (encoder: EncoderPreference): string => {
    const labels: Record<EncoderPreference, string> = {
      auto: t('settings.recordingConfig.videoSettings.encoderPreference.labels.auto'),
      nvenc: t('settings.recordingConfig.videoSettings.encoderPreference.labels.nvenc'),
      qsv: t('settings.recordingConfig.videoSettings.encoderPreference.labels.qsv'),
      amf: t('settings.recordingConfig.videoSettings.encoderPreference.labels.amf'),
      software: t('settings.recordingConfig.videoSettings.encoderPreference.labels.software'),
    };
    return labels[encoder];
  };

  const getEstimatedSize = (): string => {
    const bitrateMap: Record<BitratePreset, number> = {
      low: 10,
      medium: 20,
      high: 40,
      very_high: 80,
    };

    const mbps = bitrateMap[settings.bitrate_preset];
    const mbPerMinute = (mbps * 60) / 8; // Convert Mbps to MB per minute

    return `~${mbPerMinute.toFixed(0)} MB/min`;
  };

  return (
    <div className="space-y-6">
      {/* Resolution */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t('settings.recordingConfig.videoSettings.resolution.title')}</CardTitle>
          <CardDescription>
            {t('settings.recordingConfig.videoSettings.resolution.description')}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-4">
            <div className="flex-1">
              <Select
                value={settings.resolution}
                onValueChange={(value) => updateSetting("resolution", value as Resolution)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="r1920x1080">
                    {getResolutionLabel("r1920x1080")}
                  </SelectItem>
                  <SelectItem value="r2560x1440">
                    {getResolutionLabel("r2560x1440")}
                  </SelectItem>
                  <SelectItem value="r3840x2160">
                    {getResolutionLabel("r3840x2160")}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            {settings.resolution === "r1920x1080" && (
              <Badge variant="secondary">{t('settings.recordingConfig.videoSettings.resolution.recommended')}</Badge>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Frame Rate */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t('settings.recordingConfig.videoSettings.frameRate.title')}</CardTitle>
          <CardDescription>
            {t('settings.recordingConfig.videoSettings.frameRate.description')}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-4">
            <div className="flex-1">
              <Select
                value={settings.frame_rate}
                onValueChange={(value) => updateSetting("frame_rate", value as FrameRate)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="fps30">{getFrameRateLabel("fps30")}</SelectItem>
                  <SelectItem value="fps60">{getFrameRateLabel("fps60")}</SelectItem>
                  <SelectItem value="fps120">{getFrameRateLabel("fps120")}</SelectItem>
                  <SelectItem value="fps144">{getFrameRateLabel("fps144")}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            {settings.frame_rate === "fps60" && (
              <Badge variant="secondary">{t('settings.recordingConfig.videoSettings.frameRate.recommended')}</Badge>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Bitrate */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t('settings.recordingConfig.videoSettings.bitratePreset.title')}</CardTitle>
          <CardDescription>
            {t('settings.recordingConfig.videoSettings.bitratePreset.description')}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <Select
            value={settings.bitrate_preset}
            onValueChange={(value) => updateSetting("bitrate_preset", value as BitratePreset)}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="low">{getBitrateLabel("low")}</SelectItem>
              <SelectItem value="medium">{getBitrateLabel("medium")}</SelectItem>
              <SelectItem value="high">{getBitrateLabel("high")}</SelectItem>
              <SelectItem value="very_high">{getBitrateLabel("very_high")}</SelectItem>
            </SelectContent>
          </Select>

          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Info className="w-4 h-4" />
            <span>{t('settings.recordingConfig.videoSettings.bitratePreset.estimatedSize')}: {getEstimatedSize()}</span>
          </div>
        </CardContent>
      </Card>

      {/* Codec */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t('settings.recordingConfig.videoSettings.videoCodec.title')}</CardTitle>
          <CardDescription>
            {t('settings.recordingConfig.videoSettings.videoCodec.description')}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-4">
            <div className="flex-1">
              <Select
                value={settings.codec}
                onValueChange={(value) => updateSetting("codec", value as VideoCodec)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="h264">{getCodecLabel("h264")}</SelectItem>
                  <SelectItem value="h265">{getCodecLabel("h265")}</SelectItem>
                  <SelectItem value="av1">{getCodecLabel("av1")}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            {settings.codec === "h265" && (
              <Badge variant="secondary">{t('settings.recordingConfig.videoSettings.videoCodec.recommended')}</Badge>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Encoder */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">{t('settings.recordingConfig.videoSettings.encoderPreference.title')}</CardTitle>
          <CardDescription>
            {t('settings.recordingConfig.videoSettings.encoderPreference.description')}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-4">
            <div className="flex-1">
              <Select
                value={settings.encoder}
                onValueChange={(value) => updateSetting("encoder", value as EncoderPreference)}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="auto">{getEncoderLabel("auto")}</SelectItem>
                  <SelectItem value="nvenc">{getEncoderLabel("nvenc")}</SelectItem>
                  <SelectItem value="qsv">{getEncoderLabel("qsv")}</SelectItem>
                  <SelectItem value="amf">{getEncoderLabel("amf")}</SelectItem>
                  <SelectItem value="software">{getEncoderLabel("software")}</SelectItem>
                </SelectContent>
              </Select>
            </div>
            {settings.encoder === "auto" && (
              <Badge variant="secondary">{t('settings.recordingConfig.videoSettings.encoderPreference.recommended')}</Badge>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Info Card */}
      <Card className="bg-muted/50">
        <CardContent className="pt-6">
          <div className="space-y-2 text-sm">
            <p className="font-semibold">{t('settings.recordingConfig.videoSettings.currentConfiguration.title')}</p>
            <p className="text-muted-foreground">
              • {t('settings.recordingConfig.videoSettings.currentConfiguration.resolution')}: {getResolutionLabel(settings.resolution)}
            </p>
            <p className="text-muted-foreground">
              • {t('settings.recordingConfig.videoSettings.currentConfiguration.frameRate')}: {getFrameRateLabel(settings.frame_rate)}
            </p>
            <p className="text-muted-foreground">
              • {t('settings.recordingConfig.videoSettings.currentConfiguration.codec')}: {getCodecLabel(settings.codec)}
            </p>
            <p className="text-muted-foreground">
              • {t('settings.recordingConfig.videoSettings.currentConfiguration.encoder')}: {getEncoderLabel(settings.encoder)}
            </p>
            <p className="text-muted-foreground">
              • {t('settings.recordingConfig.videoSettings.currentConfiguration.estimatedSize')}: {getEstimatedSize()}
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
