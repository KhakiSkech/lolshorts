/**
 * Recording Controls Component
 *
 * Manual control interface for recording operations
 */

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Slider } from '@/components/ui/slider';
import { Label } from '@/components/ui/label';
import { invoke } from '@tauri-apps/api/core';
import { Play, Square, Save, Settings } from 'lucide-react';
import { toast } from '@/components/ui/use-toast';

interface RecordingSettings {
  audio_enabled: boolean;
  audio_device_id: string;
  video_quality: 'low' | 'medium' | 'high' | 'ultra';
  hardware_encoding: boolean;
}

export function RecordingControls() {
  const [isRecording, setIsRecording] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [replayDuration, setReplayDuration] = useState(60);
  const [settings, setSettings] = useState<RecordingSettings>({
    audio_enabled: true,
    audio_device_id: 'default',
    video_quality: 'high',
    hardware_encoding: true
  });

  const handleStartAutoCapture = async () => {
    setIsLoading(true);
    try {
      await invoke('start_auto_capture');
      setIsRecording(true);
      toast({
        title: 'Auto-Capture Started',
        description: 'Now monitoring game events for automatic clip creation',
      });
    } catch (error) {
      toast({
        title: 'Failed to Start',
        description: String(error),
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleStopAutoCapture = async () => {
    setIsLoading(true);
    try {
      await invoke('stop_auto_capture');
      setIsRecording(false);
      toast({
        title: 'Auto-Capture Stopped',
        description: 'Replay buffer cleared',
      });
    } catch (error) {
      toast({
        title: 'Failed to Stop',
        description: String(error),
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleSaveReplay = async () => {
    setIsLoading(true);
    try {
      await invoke<string>('save_replay', {
        durationSecs: replayDuration
      });
      toast({
        title: 'Replay Saved',
        description: `Saved ${replayDuration}s replay`,
      });
    } catch (error) {
      toast({
        title: 'Failed to Save Replay',
        description: String(error),
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
      toast({
        title: 'Settings Saved',
        description: 'Recording settings updated successfully',
      });
    } catch (error) {
      toast({
        title: 'Failed to Save Settings',
        description: String(error),
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      {/* Auto-Capture Controls */}
      <Card>
        <CardHeader>
          <CardTitle>Auto-Capture Controls</CardTitle>
          <CardDescription>
            Start/stop automatic event monitoring and clip creation
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            {!isRecording ? (
              <Button
                onClick={handleStartAutoCapture}
                disabled={isLoading}
                className="flex-1"
              >
                <Play className="mr-2 h-4 w-4" />
                Start Auto-Capture
              </Button>
            ) : (
              <Button
                onClick={handleStopAutoCapture}
                disabled={isLoading}
                variant="destructive"
                className="flex-1"
              >
                <Square className="mr-2 h-4 w-4" />
                Stop Auto-Capture
              </Button>
            )}
          </div>

          <div className="text-sm text-muted-foreground">
            {isRecording
              ? '✓ Recording buffer active - Press F8 to toggle'
              : 'Press F8 or click Start to begin monitoring'}
          </div>
        </CardContent>
      </Card>

      {/* Manual Replay Save */}
      <Card>
        <CardHeader>
          <CardTitle>Manual Replay Save</CardTitle>
          <CardDescription>
            Save the last N seconds of gameplay
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <div className="flex justify-between">
              <Label>Replay Duration</Label>
              <span className="text-sm text-muted-foreground">{replayDuration}s</span>
            </div>
            <Slider
              value={[replayDuration]}
              onValueChange={([value]) => setReplayDuration(value)}
              min={10}
              max={60}
              step={10}
              disabled={!isRecording}
            />
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>10s</span>
              <span>30s</span>
              <span>60s</span>
            </div>
          </div>

          <Button
            onClick={handleSaveReplay}
            disabled={!isRecording || isLoading}
            className="w-full"
          >
            <Save className="mr-2 h-4 w-4" />
            Save Replay
          </Button>

          <div className="text-sm text-muted-foreground">
            Or use: F9 (60s) / F10 (30s) hotkeys
          </div>
        </CardContent>
      </Card>

      {/* Recording Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            Recording Settings
          </CardTitle>
          <CardDescription>
            Configure video quality and audio capture
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Video Quality */}
          <div className="space-y-2">
            <Label>Video Quality</Label>
            <Select
              value={settings.video_quality}
              onValueChange={(value: any) =>
                setSettings({ ...settings, video_quality: value })
              }
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

          {/* Hardware Encoding */}
          <div className="flex items-center justify-between">
            <Label>Hardware Encoding</Label>
            <Button
              variant={settings.hardware_encoding ? 'default' : 'outline'}
              size="sm"
              onClick={() =>
                setSettings({
                  ...settings,
                  hardware_encoding: !settings.hardware_encoding,
                })
              }
            >
              {settings.hardware_encoding ? 'Enabled' : 'Disabled'}
            </Button>
          </div>

          {/* Audio */}
          <div className="flex items-center justify-between">
            <Label>Audio Capture</Label>
            <Button
              variant={settings.audio_enabled ? 'default' : 'outline'}
              size="sm"
              onClick={() =>
                setSettings({
                  ...settings,
                  audio_enabled: !settings.audio_enabled,
                })
              }
            >
              {settings.audio_enabled ? 'Enabled' : 'Disabled'}
            </Button>
          </div>

          <Button onClick={handleSaveSettings} disabled={isLoading} className="w-full">
            Save Settings
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
