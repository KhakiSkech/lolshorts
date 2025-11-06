import { useState, useEffect, useCallback } from 'react';
import { useAutoEditStore } from '@/stores/autoEditStore';
import { useAutoEdit } from '@/hooks/useAutoEdit';
import { useStorage } from '@/hooks/useStorage';
import { CanvasEditor } from './CanvasEditor';
import { AudioMixer } from './AudioMixer';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Separator } from '@/components/ui/separator';
import { Label } from '@/components/ui/label';
import {
  Video,
  Clock,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Play,
  Download,
  RefreshCw,
  Palette,
  Music,
  Sparkles,
} from 'lucide-react';
import { DurationOption, GameSelection } from '@/types/autoEdit';

export function AutoEditPanel() {
  const [localLoading, setLocalLoading] = useState(false);

  const {
    currentStep,
    setCurrentStep,
    availableGames,
    selectedGameIds,
    targetDuration,
    currentTemplate,
    backgroundMusic,
    audioLevels,
    progress,
    result,
    error,
    setAvailableGames,
    toggleGameSelection,
    setTargetDuration,
    setCurrentTemplate,
    setBackgroundMusic,
    setAudioLevels,
    buildConfig,
    resetProgress,
  } = useAutoEditStore();

  const {
    startAutoEdit,
    startProgressPolling,
    stopProgressPolling,
    isLoading: hookLoading,
  } = useAutoEdit();

  const { getAllGames, isLoading: gamesLoading } = useStorage();

  // Load available games on mount
  useEffect(() => {
    const loadGames = async () => {
      try {
        const games = await getAllGames();

        const gameSelections: GameSelection[] = games.map(game => ({
          game_id: game.game_id,
          champion: game.champion,
          game_mode: game.game_mode,
          date: new Date(game.game_start_time).toLocaleDateString(),
          clip_count: 0, // TODO: Add clip count from backend
          selected: false,
        }));

        setAvailableGames(gameSelections);
      } catch (err) {
        console.error('Failed to load games:', err);
      }
    };

    loadGames();
  }, [getAllGames, setAvailableGames]);

  // Start generation
  const handleStartGeneration = useCallback(async () => {
    if (selectedGameIds.length === 0) {
      alert('Please select at least one game');
      return;
    }

    setLocalLoading(true);
    setCurrentStep('generating');

    try {
      const config = buildConfig();
      await startAutoEdit(config);

      // Start polling for progress
      startProgressPolling(1000);
    } catch (err) {
      console.error('Failed to start auto-edit:', err);
      setCurrentStep('configure');
    } finally {
      setLocalLoading(false);
    }
  }, [selectedGameIds, buildConfig, startAutoEdit, startProgressPolling, setCurrentStep]);

  // Handle generation complete
  useEffect(() => {
    if (progress?.status === 'Complete') {
      stopProgressPolling();
      setCurrentStep('complete');
    } else if (progress?.status === 'Failed') {
      stopProgressPolling();
      setCurrentStep('configure');
    }
  }, [progress, stopProgressPolling, setCurrentStep]);

  // Reset for new generation
  const handleStartNew = useCallback(() => {
    resetProgress();
    setCurrentStep('configure');
  }, [resetProgress, setCurrentStep]);

  const isLoading = hookLoading || localLoading || gamesLoading;

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b p-4 bg-card">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Sparkles className="w-6 h-6 text-primary" />
            <div>
              <h2 className="text-lg font-semibold">Auto-Edit Shorts Generator</h2>
              <p className="text-sm text-muted-foreground">
                AI-powered YouTube Shorts creation from your best moments
              </p>
            </div>
          </div>
          <Badge variant={currentStep === 'generating' ? 'default' : 'secondary'}>
            {currentStep === 'configure' && 'Configure'}
            {currentStep === 'preview' && 'Preview'}
            {currentStep === 'generating' && 'Generating...'}
            {currentStep === 'complete' && 'Complete'}
          </Badge>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 overflow-y-auto p-6">
        {currentStep === 'configure' && (
          <div className="max-w-6xl mx-auto space-y-6">
            {/* Game Selection */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Video className="w-5 h-5" />
                  Select Games
                </CardTitle>
                <CardDescription>
                  Choose one or more games to pull clips from
                </CardDescription>
              </CardHeader>
              <CardContent>
                {gamesLoading ? (
                  <div className="flex items-center justify-center p-8">
                    <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
                  </div>
                ) : availableGames.length === 0 ? (
                  <Alert>
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>
                      No games available. Record some games first from the Dashboard.
                    </AlertDescription>
                  </Alert>
                ) : (
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                    {availableGames.map(game => (
                      <Card
                        key={game.game_id}
                        className={`cursor-pointer transition-all ${
                          selectedGameIds.includes(game.game_id)
                            ? 'ring-2 ring-primary bg-primary/5'
                            : 'hover:bg-muted/50'
                        }`}
                        onClick={() => toggleGameSelection(game.game_id)}
                      >
                        <CardContent className="p-4">
                          <div className="flex items-start justify-between">
                            <div className="flex-1">
                              <div className="font-medium">{game.champion}</div>
                              <div className="text-sm text-muted-foreground">
                                {game.game_mode}
                              </div>
                              <div className="text-xs text-muted-foreground mt-1">
                                {game.date} • {game.clip_count} clips
                              </div>
                            </div>
                            {selectedGameIds.includes(game.game_id) && (
                              <CheckCircle2 className="w-5 h-5 text-primary flex-shrink-0" />
                            )}
                          </div>
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                )}

                {selectedGameIds.length > 0 && (
                  <div className="mt-4 p-3 bg-primary/10 rounded-lg">
                    <p className="text-sm font-medium">
                      {selectedGameIds.length} game(s) selected
                    </p>
                  </div>
                )}
              </CardContent>
            </Card>

            {/* Duration Selection */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Clock className="w-5 h-5" />
                  Target Duration
                </CardTitle>
                <CardDescription>
                  Choose your YouTube Shorts length
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-3 gap-3">
                  {([60, 120, 180] as DurationOption[]).map(duration => (
                    <Card
                      key={duration}
                      className={`cursor-pointer transition-all ${
                        targetDuration === duration
                          ? 'ring-2 ring-primary bg-primary/5'
                          : 'hover:bg-muted/50'
                      }`}
                      onClick={() => setTargetDuration(duration)}
                    >
                      <CardContent className="p-6 text-center">
                        <div className="text-3xl font-bold text-primary">
                          {duration}s
                        </div>
                        <div className="text-sm text-muted-foreground mt-1">
                          {duration === 60 && 'Quick Short'}
                          {duration === 120 && 'Standard'}
                          {duration === 180 && 'Extended'}
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Optional Enhancements */}
            <Tabs defaultValue="canvas" className="w-full">
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="canvas">
                  <Palette className="w-4 h-4 mr-2" />
                  Canvas Overlay
                </TabsTrigger>
                <TabsTrigger value="audio">
                  <Music className="w-4 h-4 mr-2" />
                  Background Music
                </TabsTrigger>
              </TabsList>

              <TabsContent value="canvas" className="mt-4">
                <Card>
                  <CardContent className="p-6">
                    <CanvasEditor
                      template={currentTemplate}
                      onTemplateChange={setCurrentTemplate}
                    />
                  </CardContent>
                </Card>
              </TabsContent>

              <TabsContent value="audio" className="mt-4">
                <AudioMixer
                  backgroundMusic={backgroundMusic}
                  audioLevels={audioLevels}
                  onBackgroundMusicChange={setBackgroundMusic}
                  onAudioLevelsChange={setAudioLevels}
                />
              </TabsContent>
            </Tabs>

            {/* Generate Button */}
            <div className="flex justify-end gap-3">
              <Button
                size="lg"
                onClick={handleStartGeneration}
                disabled={selectedGameIds.length === 0 || isLoading}
              >
                {isLoading ? (
                  <>
                    <Loader2 className="w-5 h-5 mr-2 animate-spin" />
                    Starting...
                  </>
                ) : (
                  <>
                    <Sparkles className="w-5 h-5 mr-2" />
                    Generate Short
                  </>
                )}
              </Button>
            </div>
          </div>
        )}

        {currentStep === 'generating' && progress && (
          <div className="max-w-2xl mx-auto space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Loader2 className="w-5 h-5 animate-spin" />
                  Generating Your Short
                </CardTitle>
                <CardDescription>
                  Please wait while we create your YouTube Short...
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* Progress Bar */}
                <div className="space-y-2">
                  <div className="flex items-center justify-between text-sm">
                    <span className="font-medium">{progress.current_stage}</span>
                    <span className="text-muted-foreground">
                      {Math.round(progress.progress_percentage)}%
                    </span>
                  </div>
                  <Progress value={progress.progress_percentage} className="h-2" />
                </div>

                {/* Status Details */}
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div className="flex items-center gap-2">
                    <Video className="w-4 h-4 text-muted-foreground" />
                    <span>
                      {progress.clips_selected} / {progress.total_clips} clips
                    </span>
                  </div>
                  {progress.estimated_completion_seconds && (
                    <div className="flex items-center gap-2">
                      <Clock className="w-4 h-4 text-muted-foreground" />
                      <span>
                        ~{Math.ceil(progress.estimated_completion_seconds)}s remaining
                      </span>
                    </div>
                  )}
                </div>

                {/* Stage Indicators */}
                <div className="space-y-2">
                  {[
                    { stage: 'SelectingClips', label: 'Selecting Best Clips' },
                    { stage: 'PreparingClips', label: 'Preparing Clips' },
                    { stage: 'Concatenating', label: 'Concatenating Videos' },
                    { stage: 'ApplyingCanvas', label: 'Applying Canvas Overlay' },
                    { stage: 'MixingAudio', label: 'Mixing Audio' },
                  ].map(({ stage, label }) => (
                    <div
                      key={stage}
                      className={`flex items-center gap-2 text-sm ${
                        progress.status === stage
                          ? 'text-primary font-medium'
                          : progress.progress_percentage > getStageProgress(stage)
                          ? 'text-green-600'
                          : 'text-muted-foreground'
                      }`}
                    >
                      {progress.status === stage ? (
                        <Loader2 className="w-4 h-4 animate-spin" />
                      ) : progress.progress_percentage > getStageProgress(stage) ? (
                        <CheckCircle2 className="w-4 h-4" />
                      ) : (
                        <div className="w-4 h-4 rounded-full border-2" />
                      )}
                      {label}
                    </div>
                  ))}
                </div>

                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    This may take a few minutes depending on the number of clips and selected options.
                    Do not close this window.
                  </AlertDescription>
                </Alert>
              </CardContent>
            </Card>
          </div>
        )}

        {currentStep === 'complete' && result && (
          <div className="max-w-2xl mx-auto space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-green-600">
                  <CheckCircle2 className="w-6 h-6" />
                  Short Generated Successfully!
                </CardTitle>
                <CardDescription>
                  Your YouTube Short is ready to use
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* Result Details */}
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-1">
                    <div className="text-sm text-muted-foreground">Duration</div>
                    <div className="text-2xl font-bold">
                      {Math.round(result.duration)}s
                    </div>
                  </div>
                  <div className="space-y-1">
                    <div className="text-sm text-muted-foreground">Clips Used</div>
                    <div className="text-2xl font-bold">{result.clips_used}</div>
                  </div>
                  <div className="space-y-1">
                    <div className="text-sm text-muted-foreground">File Size</div>
                    <div className="text-2xl font-bold">
                      {formatFileSize(result.file_size_bytes)}
                    </div>
                  </div>
                  <div className="space-y-1">
                    <div className="text-sm text-muted-foreground">Job ID</div>
                    <div className="text-xs font-mono truncate">
                      {result.job_id}
                    </div>
                  </div>
                </div>

                <Separator />

                {/* Output Path */}
                <div className="space-y-2">
                  <Label className="text-sm font-medium">Output File</Label>
                  <div className="p-3 bg-muted rounded-lg">
                    <code className="text-xs break-all">{result.output_path}</code>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex gap-3">
                  <Button
                    onClick={() => {
                      // TODO: Open file location
                      alert('Open file location: ' + result.output_path);
                    }}
                    className="flex-1"
                  >
                    <Download className="w-4 h-4 mr-2" />
                    Open File Location
                  </Button>
                  <Button
                    onClick={() => {
                      // TODO: Play video
                      alert('Play video: ' + result.output_path);
                    }}
                    variant="outline"
                    className="flex-1"
                  >
                    <Play className="w-4 h-4 mr-2" />
                    Play Video
                  </Button>
                </div>

                <Button
                  onClick={handleStartNew}
                  variant="outline"
                  className="w-full"
                >
                  <RefreshCw className="w-4 h-4 mr-2" />
                  Create Another Short
                </Button>
              </CardContent>
            </Card>
          </div>
        )}

        {error && (
          <div className="max-w-2xl mx-auto">
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          </div>
        )}
      </div>
    </div>
  );
}

// Helper function to get stage progress percentage
function getStageProgress(stage: string): number {
  const stages: Record<string, number> = {
    SelectingClips: 10,
    PreparingClips: 30,
    Concatenating: 50,
    ApplyingCanvas: 70,
    MixingAudio: 90,
  };
  return stages[stage] || 0;
}

// Helper function to format file size
function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}
