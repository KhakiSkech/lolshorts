import { useCallback, useRef } from 'react';
import { BackgroundMusic, AudioLevels } from '@/types/autoEdit';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Slider } from '@/components/ui/slider';
import { Switch } from '@/components/ui/switch';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { Music, Upload, X, Volume2, AlertCircle } from 'lucide-react';

interface AudioMixerProps {
  backgroundMusic: BackgroundMusic | null;
  audioLevels: AudioLevels;
  onBackgroundMusicChange: (music: BackgroundMusic | null) => void;
  onAudioLevelsChange: (levels: Partial<AudioLevels>) => void;
}

export function AudioMixer({
  backgroundMusic,
  audioLevels,
  onBackgroundMusicChange,
  onAudioLevelsChange,
}: AudioMixerProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileSelect = useCallback(async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    // Validate file type
    if (!file.type.startsWith('audio/')) {
      alert('Please select an audio file (MP3, WAV, etc.)');
      return;
    }

    // For now, we'll just store the file path
    // In production, you'd upload to a temporary location or process it
    onBackgroundMusicChange({
      file_path: file.name, // TODO: Get actual file path from file picker
      loop_music: backgroundMusic?.loop_music ?? true,
    });
  }, [backgroundMusic, onBackgroundMusicChange]);

  const handleRemoveMusic = useCallback(() => {
    onBackgroundMusicChange(null);
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  }, [onBackgroundMusicChange]);

  const handleLoopToggle = useCallback((checked: boolean) => {
    if (backgroundMusic) {
      onBackgroundMusicChange({
        ...backgroundMusic,
        loop_music: checked,
      });
    }
  }, [backgroundMusic, onBackgroundMusicChange]);

  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Music className="w-5 h-5" />
          Audio Mixer
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Background Music Upload */}
        <div className="space-y-3">
          <Label>Background Music</Label>

          {!backgroundMusic ? (
            <div className="border-2 border-dashed rounded-lg p-6 text-center">
              <Music className="w-12 h-12 mx-auto mb-3 text-muted-foreground" />
              <p className="text-sm text-muted-foreground mb-3">
                Add background music to your Short
              </p>
              <Button
                onClick={() => fileInputRef.current?.click()}
                variant="outline"
              >
                <Upload className="w-4 h-4 mr-2" />
                Upload Music
              </Button>
              <input
                ref={fileInputRef}
                type="file"
                accept="audio/*"
                className="hidden"
                onChange={handleFileSelect}
              />
            </div>
          ) : (
            <Card>
              <CardContent className="p-4">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-2">
                    <Music className="w-4 h-4" />
                    <span className="text-sm font-medium truncate">
                      {backgroundMusic.file_path.split('/').pop() || 'Background Music'}
                    </span>
                  </div>
                  <Button
                    size="icon"
                    variant="ghost"
                    onClick={handleRemoveMusic}
                  >
                    <X className="w-4 h-4" />
                  </Button>
                </div>

                <div className="flex items-center justify-between">
                  <Label htmlFor="loop-music" className="text-sm">
                    Loop music
                  </Label>
                  <Switch
                    id="loop-music"
                    checked={backgroundMusic.loop_music}
                    onCheckedChange={handleLoopToggle}
                  />
                </div>

                {!backgroundMusic.loop_music && (
                  <Alert className="mt-3">
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription className="text-xs">
                      Music will play once. Video may be longer than the music.
                    </AlertDescription>
                  </Alert>
                )}
              </CardContent>
            </Card>
          )}
        </div>

        <Separator />

        {/* Volume Controls */}
        <div className="space-y-4">
          <div>
            <Label className="flex items-center justify-between mb-2">
              <span className="flex items-center gap-2">
                <Volume2 className="w-4 h-4" />
                Game Audio
              </span>
              <span className="text-sm font-mono text-muted-foreground">
                {audioLevels.game_audio}%
              </span>
            </Label>
            <Slider
              value={[audioLevels.game_audio]}
              onValueChange={([value]) =>
                onAudioLevelsChange({ game_audio: value })
              }
              min={0}
              max={100}
              step={1}
              className="w-full"
            />
            <div className="flex justify-between text-xs text-muted-foreground mt-1">
              <span>Silent</span>
              <span>Full</span>
            </div>
          </div>

          <div>
            <Label className="flex items-center justify-between mb-2">
              <span className="flex items-center gap-2">
                <Music className="w-4 h-4" />
                Background Music
              </span>
              <span className="text-sm font-mono text-muted-foreground">
                {audioLevels.background_music}%
              </span>
            </Label>
            <Slider
              value={[audioLevels.background_music]}
              onValueChange={([value]) =>
                onAudioLevelsChange({ background_music: value })
              }
              min={0}
              max={100}
              step={1}
              className="w-full"
              disabled={!backgroundMusic}
            />
            <div className="flex justify-between text-xs text-muted-foreground mt-1">
              <span>Silent</span>
              <span>Full</span>
            </div>
          </div>
        </div>

        {/* Audio Balance Info */}
        <Alert>
          <AlertCircle className="h-4 w-4" />
          <AlertDescription className="text-xs">
            <strong>Tip:</strong> Keep game audio at 70% and music at 30% for best balance.
            The final mix will include fade-in and fade-out effects.
          </AlertDescription>
        </Alert>

        {/* Recommended Presets */}
        <div className="space-y-2">
          <Label className="text-xs">Quick Presets</Label>
          <div className="grid grid-cols-2 gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() =>
                onAudioLevelsChange({ game_audio: 100, background_music: 0 })
              }
            >
              Game Only
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() =>
                onAudioLevelsChange({ game_audio: 70, background_music: 30 })
              }
            >
              Balanced
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() =>
                onAudioLevelsChange({ game_audio: 40, background_music: 60 })
              }
            >
              Music Focus
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() =>
                onAudioLevelsChange({ game_audio: 0, background_music: 100 })
              }
            >
              Music Only
            </Button>
          </div>
        </div>

        {/* Final mix visualization */}
        <div className="space-y-2">
          <Label className="text-xs">Mix Preview</Label>
          <div className="flex h-8 rounded overflow-hidden border">
            <div
              className="bg-blue-500 flex items-center justify-center text-xs font-medium text-white"
              style={{ width: `${audioLevels.game_audio}%` }}
            >
              {audioLevels.game_audio > 15 && 'Game'}
            </div>
            <div
              className="bg-purple-500 flex items-center justify-center text-xs font-medium text-white"
              style={{ width: `${audioLevels.background_music}%` }}
            >
              {audioLevels.background_music > 15 && 'Music'}
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
