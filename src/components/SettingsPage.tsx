/**
 * Settings Page Component
 *
 * Comprehensive settings UI for recording configuration, quality, audio, and advanced options
 */

import { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Switch } from '@/components/ui/switch';
import { Slider } from '@/components/ui/slider';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Settings, Video, Mic, Zap, Info, Save, RotateCcw } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { toast } from '@/components/ui/use-toast';

interface RecordingSettings {
  video_quality: 'low' | 'medium' | 'high' | 'ultra';
  resolution: '720p' | '1080p' | '1440p' | '2160p';
  fps: 30 | 60;
  hardware_encoding: boolean;
  encoder: 'h264' | 'h265' | 'av1';
  bitrate_kbps: number;
  audio_enabled: boolean;
  audio_device_id: string;
  audio_quality: 'low' | 'medium' | 'high';
  replay_buffer_duration_secs: number;
  auto_save_clips: boolean;
  save_full_game: boolean;
  clip_padding_secs: number;
  output_directory: string;
}

const defaultSettings: RecordingSettings = {
  video_quality: 'high',
  resolution: '1080p',
  fps: 60,
  hardware_encoding: true,
  encoder: 'h264',
  bitrate_kbps: 8000,
  audio_enabled: true,
  audio_device_id: 'default',
  audio_quality: 'high',
  replay_buffer_duration_secs: 120,
  auto_save_clips: true,
  save_full_game: false,
  clip_padding_secs: 5,
  output_directory: '',
};

export function SettingsPage() {
  const [settings, setSettings] = useState<RecordingSettings>(defaultSettings);
  const [isLoading, setIsLoading] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    setIsLoading(true);
    try {
      const savedSettings = await invoke<RecordingSettings>('get_recording_settings');
      setSettings(savedSettings);
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to load settings:', error);
      toast({
        title: 'Failed to Load Settings',
        description: 'Using default settings',
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleSaveSettings = async () => {
    setIsLoading(true);
    try {
      await invoke('save_recording_settings', { settings });
      setHasChanges(false);

      toast({
        title: 'Settings Saved',
        description: 'Recording settings updated successfully',
      });
    } catch (error) {
      toast({
        title: 'Failed to Save',
        description: String(error),
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleResetToDefaults = async () => {
    setIsLoading(true);
    try {
      await invoke('reset_settings_to_default');
      await loadSettings();

      toast({
        title: 'Settings Reset',
        description: 'All settings restored to defaults',
      });
    } catch (error) {
      toast({
        title: 'Failed to Reset',
        description: String(error),
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const updateSetting = <K extends keyof RecordingSettings>(
    key: K,
    value: RecordingSettings[K]
  ) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
    setHasChanges(true);
  };

  const applyQualityPreset = (quality: RecordingSettings['video_quality']) => {
    const preset = {
      low: { resolution: '720p' as const, fps: 30 as const, bitrate_kbps: 2000 },
      medium: { resolution: '1080p' as const, fps: 30 as const, bitrate_kbps: 5000 },
      high: { resolution: '1080p' as const, fps: 60 as const, bitrate_kbps: 8000 },
      ultra: { resolution: '1440p' as const, fps: 60 as const, bitrate_kbps: 15000 },
    }[quality];

    setSettings((prev) => ({
      ...prev,
      video_quality: quality,
      ...preset,
    }));
    setHasChanges(true);
  };

  const getEstimatedDiskUsage = (): string => {
    const bitrateKbps = settings.bitrate_kbps;
    const mbPerMinute = (bitrateKbps * 60) / 8 / 1024;
    const gbPerHour = (mbPerMinute * 60) / 1024;
    return `~${mbPerMinute.toFixed(0)} MB/min (~${gbPerHour.toFixed(1)} GB/hr)`;
  };

  return (
    <div className="space-y-4">
      {/* Header */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Settings className="h-5 w-5" />
                Recording Settings
              </CardTitle>
              <CardDescription>
                Configure video quality, audio, and recording behavior
              </CardDescription>
            </div>
            {hasChanges && (
              <Badge variant="secondary">Unsaved Changes</Badge>
            )}
          </div>
        </CardHeader>
      </Card>

      {/* Settings Tabs */}
      <Tabs defaultValue="video" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="video">
            <Video className="mr-2 h-4 w-4" />
            Video
          </TabsTrigger>
          <TabsTrigger value="audio">
            <Mic className="mr-2 h-4 w-4" />
            Audio
          </TabsTrigger>
          <TabsTrigger value="performance">
            <Zap className="mr-2 h-4 w-4" />
            Performance
          </TabsTrigger>
          <TabsTrigger value="advanced">
            <Info className="mr-2 h-4 w-4" />
            Advanced
          </TabsTrigger>
        </TabsList>

        {/* Video Settings Tab */}
        <TabsContent value="video" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Video Quality</CardTitle>
              <CardDescription>
                Choose a preset or customize individual settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Quality Preset */}
              <div className="space-y-2">
                <Label>Quality Preset</Label>
                <Select
                  value={settings.video_quality}
                  onValueChange={(value: any) => applyQualityPreset(value)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="low">Low (720p 30fps)</SelectItem>
                    <SelectItem value="medium">Medium (1080p 30fps)</SelectItem>
                    <SelectItem value="high">High (1080p 60fps)</SelectItem>
                    <SelectItem value="ultra">Ultra (1440p 60fps)</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* Resolution */}
              <div className="space-y-2">
                <Label>Resolution</Label>
                <Select
                  value={settings.resolution}
                  onValueChange={(value: any) => updateSetting('resolution', value)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="720p">720p (1280x720)</SelectItem>
                    <SelectItem value="1080p">1080p (1920x1080)</SelectItem>
                    <SelectItem value="1440p">1440p (2560x1440)</SelectItem>
                    <SelectItem value="2160p">4K (3840x2160)</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* FPS */}
              <div className="space-y-2">
                <Label>Frame Rate</Label>
                <Select
                  value={settings.fps.toString()}
                  onValueChange={(value) => updateSetting('fps', parseInt(value) as 30 | 60)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="30">30 FPS</SelectItem>
                    <SelectItem value="60">60 FPS</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* Bitrate */}
              <div className="space-y-2">
                <div className="flex justify-between">
                  <Label>Bitrate</Label>
                  <span className="text-sm text-muted-foreground">
                    {settings.bitrate_kbps} kbps
                  </span>
                </div>
                <Slider
                  value={[settings.bitrate_kbps]}
                  onValueChange={([value]) => updateSetting('bitrate_kbps', value)}
                  min={1000}
                  max={20000}
                  step={1000}
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>1 Mbps</span>
                  <span>10 Mbps</span>
                  <span>20 Mbps</span>
                </div>
              </div>

              {/* Disk Usage Estimate */}
              <Alert>
                <Info className="h-4 w-4" />
                <AlertTitle>Estimated Disk Usage</AlertTitle>
                <AlertDescription>
                  {getEstimatedDiskUsage()}
                </AlertDescription>
              </Alert>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Audio Settings Tab */}
        <TabsContent value="audio" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Audio Configuration</CardTitle>
              <CardDescription>
                Configure audio capture settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Audio Enabled */}
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Enable Audio</Label>
                  <div className="text-sm text-muted-foreground">
                    Capture game audio and microphone
                  </div>
                </div>
                <Switch
                  checked={settings.audio_enabled}
                  onCheckedChange={(checked) => updateSetting('audio_enabled', checked)}
                />
              </div>

              {settings.audio_enabled && (
                <>
                  {/* Audio Device */}
                  <div className="space-y-2">
                    <Label>Audio Device</Label>
                    <Select
                      value={settings.audio_device_id}
                      onValueChange={(value) => updateSetting('audio_device_id', value)}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="default">System Default</SelectItem>
                        <SelectItem value="headphones">Headphones</SelectItem>
                        <SelectItem value="speakers">Speakers</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  {/* Audio Quality */}
                  <div className="space-y-2">
                    <Label>Audio Quality</Label>
                    <Select
                      value={settings.audio_quality}
                      onValueChange={(value: any) => updateSetting('audio_quality', value)}
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="low">Low (96 kbps)</SelectItem>
                        <SelectItem value="medium">Medium (128 kbps)</SelectItem>
                        <SelectItem value="high">High (192 kbps)</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Performance Settings Tab */}
        <TabsContent value="performance" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Performance Optimization</CardTitle>
              <CardDescription>
                Hardware acceleration and encoder settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Hardware Encoding */}
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Hardware Encoding</Label>
                  <div className="text-sm text-muted-foreground">
                    Use GPU for video encoding (recommended)
                  </div>
                </div>
                <Switch
                  checked={settings.hardware_encoding}
                  onCheckedChange={(checked) => updateSetting('hardware_encoding', checked)}
                />
              </div>

              {/* Encoder */}
              <div className="space-y-2">
                <Label>Video Encoder</Label>
                <Select
                  value={settings.encoder}
                  onValueChange={(value: any) => updateSetting('encoder', value)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="h264">H.264 (Best Compatibility)</SelectItem>
                    <SelectItem value="h265">H.265 (Better Compression)</SelectItem>
                    <SelectItem value="av1">AV1 (Experimental)</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* Replay Buffer Duration */}
              <div className="space-y-2">
                <div className="flex justify-between">
                  <Label>Replay Buffer Duration</Label>
                  <span className="text-sm text-muted-foreground">
                    {settings.replay_buffer_duration_secs}s
                  </span>
                </div>
                <Slider
                  value={[settings.replay_buffer_duration_secs]}
                  onValueChange={([value]) => updateSetting('replay_buffer_duration_secs', value)}
                  min={30}
                  max={300}
                  step={30}
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>30s</span>
                  <span>2min</span>
                  <span>5min</span>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Advanced Settings Tab */}
        <TabsContent value="advanced" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Advanced Options</CardTitle>
              <CardDescription>
                Clip behavior and storage settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Auto Save Clips */}
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Auto-Save Clips</Label>
                  <div className="text-sm text-muted-foreground">
                    Automatically save detected highlights
                  </div>
                </div>
                <Switch
                  checked={settings.auto_save_clips}
                  onCheckedChange={(checked) => updateSetting('auto_save_clips', checked)}
                />
              </div>

              {/* Save Full Game */}
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label>Save Full Game</Label>
                  <div className="text-sm text-muted-foreground">
                    Record entire game in addition to clips
                  </div>
                </div>
                <Switch
                  checked={settings.save_full_game}
                  onCheckedChange={(checked) => updateSetting('save_full_game', checked)}
                />
              </div>

              {/* Clip Padding */}
              <div className="space-y-2">
                <div className="flex justify-between">
                  <Label>Clip Padding</Label>
                  <span className="text-sm text-muted-foreground">
                    {settings.clip_padding_secs}s before/after
                  </span>
                </div>
                <Slider
                  value={[settings.clip_padding_secs]}
                  onValueChange={([value]) => updateSetting('clip_padding_secs', value)}
                  min={0}
                  max={15}
                  step={1}
                />
                <div className="text-xs text-muted-foreground">
                  Extra seconds to include before and after each highlight
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Action Buttons */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex gap-2 justify-end">
            <Button
              variant="outline"
              onClick={handleResetToDefaults}
              disabled={isLoading}
            >
              <RotateCcw className="mr-2 h-4 w-4" />
              Reset to Defaults
            </Button>
            <Button
              onClick={handleSaveSettings}
              disabled={!hasChanges || isLoading}
            >
              <Save className="mr-2 h-4 w-4" />
              Save Settings
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
