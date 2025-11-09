import { useState, useEffect, useCallback } from 'react';
import { useSearch } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';
import { useAutoEditStore } from '@/stores/autoEditStore';
import { useAutoEdit } from '@/hooks/useAutoEdit';
import { useStorage } from '@/hooks/useStorage';
import { useAutoEditQuota } from '@/hooks/useAutoEditQuota';
import { CanvasEditor } from './CanvasEditor';
import { AudioMixer } from './AudioMixer';
import { AutoEditQuotaBadge } from './AutoEditQuotaBadge';
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
  ChevronDown,
  ChevronUp,
  RotateCcw,
  XCircle,
} from 'lucide-react';
import { DurationOption, GameSelection } from '@/types/autoEdit';

export function AutoEditPanel() {
  const { t } = useTranslation();
  const searchParams = useSearch({ from: '/auto-edit' }) as { gameId?: string };
  const [localLoading, setLocalLoading] = useState(false);
  const [showTechnicalDetails, setShowTechnicalDetails] = useState(false);

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

  const { hasQuota, fetchQuota } = useAutoEditQuota();

  // Load available games on mount
  useEffect(() => {
    const loadGames = async () => {
      try {
        const games = await getAllGames();

        const preSelectedGameId = searchParams.gameId;

        const gameSelections: GameSelection[] = games.map(game => ({
          game_id: game.game_id,
          champion: game.champion,
          game_mode: game.game_mode,
          date: new Date(game.game_start_time).toLocaleDateString(),
          clip_count: 0, // TODO: Add clip count from backend
          selected: preSelectedGameId === game.game_id, // Pre-select if gameId in URL
        }));

        setAvailableGames(gameSelections);

        // Auto-toggle selection for pre-selected game
        if (preSelectedGameId) {
          toggleGameSelection(preSelectedGameId);
        }
      } catch (err) {
        console.error('Failed to load games:', err);
      }
    };

    loadGames();
  }, [getAllGames, setAvailableGames, searchParams.gameId, toggleGameSelection]);

  // Reset technical details visibility when error changes
  useEffect(() => {
    if (error) {
      setShowTechnicalDetails(false);
    }
  }, [error]);

  // Start generation
  const handleStartGeneration = useCallback(async () => {
    if (selectedGameIds.length === 0) {
      alert(t('errors.selectAtLeastOneGame'));
      return;
    }

    // Check quota before starting
    if (!hasQuota()) {
      alert(t('autoEdit.quotaExhaustedAlert'));
      return;
    }

    setLocalLoading(true);
    setCurrentStep('generating');

    try {
      const config = buildConfig();
      await startAutoEdit(config);

      // Refresh quota after successful start
      fetchQuota();

      // Start polling for progress
      startProgressPolling(1000);
    } catch (err) {
      console.error('Failed to start auto-edit:', err);
      setCurrentStep('configure');
    } finally {
      setLocalLoading(false);
    }
  }, [selectedGameIds, buildConfig, startAutoEdit, startProgressPolling, setCurrentStep, hasQuota, fetchQuota, t]);

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
              <h2 className="text-lg font-semibold">{t('autoEdit.title')}</h2>
              <p className="text-sm text-muted-foreground">
                {t('autoEdit.subtitle')}
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <AutoEditQuotaBadge />
            <Badge variant={currentStep === 'generating' ? 'default' : 'secondary'}>
              {currentStep === 'configure' && t('autoEdit.steps.configure')}
              {currentStep === 'preview' && t('autoEdit.steps.preview')}
              {currentStep === 'generating' && t('autoEdit.steps.generating')}
              {currentStep === 'complete' && t('autoEdit.steps.complete')}
            </Badge>
          </div>
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
                  {t('autoEdit.selectGames')}
                </CardTitle>
                <CardDescription>
                  {t('autoEdit.selectGamesDescription')}
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
                      {t('autoEdit.noGamesAvailable')}
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
                                {game.date} • {t('autoEdit.clipsCount', { count: game.clip_count })}
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
                      {t('autoEdit.gamesSelected', { count: selectedGameIds.length })}
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
                  {t('autoEdit.targetDuration')}
                </CardTitle>
                <CardDescription>
                  {t('autoEdit.targetDurationDescription')}
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
                          {duration === 60 && t('autoEdit.quickShort')}
                          {duration === 120 && t('autoEdit.standard')}
                          {duration === 180 && t('autoEdit.extended')}
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
                  {t('autoEdit.canvasOverlay')}
                </TabsTrigger>
                <TabsTrigger value="audio">
                  <Music className="w-4 h-4 mr-2" />
                  {t('autoEdit.backgroundMusic')}
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
                    {t('autoEdit.starting')}
                  </>
                ) : (
                  <>
                    <Sparkles className="w-5 h-5 mr-2" />
                    {t('autoEdit.generateShort')}
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
                  {t('autoEdit.generatingYourShort')}
                </CardTitle>
                <CardDescription>
                  {t('autoEdit.pleaseWait')}
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
                      {t('autoEdit.clipsProgress', {
                        selected: progress.clips_selected,
                        total: progress.total_clips
                      })}
                    </span>
                  </div>
                  {progress.estimated_completion_seconds && (
                    <div className="flex items-center gap-2">
                      <Clock className="w-4 h-4 text-muted-foreground" />
                      <span>
                        {t('autoEdit.timeRemaining', {
                          seconds: Math.ceil(progress.estimated_completion_seconds)
                        })}
                      </span>
                    </div>
                  )}
                </div>

                {/* Stage Indicators */}
                <div className="space-y-2">
                  {[
                    { stage: 'SelectingClips', label: t('autoEdit.stages.selectingClips') },
                    { stage: 'PreparingClips', label: t('autoEdit.stages.preparingClips') },
                    { stage: 'Concatenating', label: t('autoEdit.stages.concatenating') },
                    { stage: 'ApplyingCanvas', label: t('autoEdit.stages.applyingCanvas') },
                    { stage: 'MixingAudio', label: t('autoEdit.stages.mixingAudio') },
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
                    {t('autoEdit.generationWarning')}
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
                  {t('autoEdit.shortGeneratedSuccessfully')}
                </CardTitle>
                <CardDescription>
                  {t('autoEdit.shortReady')}
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-6">
                {/* Result Details */}
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-1">
                    <div className="text-sm text-muted-foreground">{t('autoEdit.duration')}</div>
                    <div className="text-2xl font-bold">
                      {Math.round(result.duration)}s
                    </div>
                  </div>
                  <div className="space-y-1">
                    <div className="text-sm text-muted-foreground">{t('autoEdit.clipsUsed')}</div>
                    <div className="text-2xl font-bold">{result.clips_used}</div>
                  </div>
                  <div className="space-y-1">
                    <div className="text-sm text-muted-foreground">{t('autoEdit.fileSize')}</div>
                    <div className="text-2xl font-bold">
                      {formatFileSize(result.file_size_bytes)}
                    </div>
                  </div>
                  <div className="space-y-1">
                    <div className="text-sm text-muted-foreground">{t('autoEdit.jobId')}</div>
                    <div className="text-xs font-mono truncate">
                      {result.job_id}
                    </div>
                  </div>
                </div>

                <Separator />

                {/* Output Path */}
                <div className="space-y-2">
                  <Label className="text-sm font-medium">{t('autoEdit.outputFile')}</Label>
                  <div className="p-3 bg-muted rounded-lg">
                    <code className="text-xs break-all">{result.output_path}</code>
                  </div>
                </div>

                {/* Actions */}
                <div className="flex gap-3">
                  <Button
                    onClick={() => {
                      // TODO: Open file location
                      alert(t('errors.todoFeature', { action: 'Open file location: ' + result.output_path }));
                    }}
                    className="flex-1"
                  >
                    <Download className="w-4 h-4 mr-2" />
                    {t('autoEdit.openFileLocation')}
                  </Button>
                  <Button
                    onClick={() => {
                      // TODO: Play video
                      alert(t('errors.todoFeature', { action: 'Play video: ' + result.output_path }));
                    }}
                    variant="outline"
                    className="flex-1"
                  >
                    <Play className="w-4 h-4 mr-2" />
                    {t('autoEdit.playVideo')}
                  </Button>
                </div>

                <Button
                  onClick={handleStartNew}
                  variant="outline"
                  className="w-full"
                >
                  <RefreshCw className="w-4 h-4 mr-2" />
                  {t('autoEdit.createAnotherShort')}
                </Button>
              </CardContent>
            </Card>
          </div>
        )}

        {error && (
          <div className="max-w-2xl mx-auto" data-testid="error-section">
            <Card className="border-destructive">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-destructive">
                  <XCircle className="w-5 h-5" />
                  {t('autoEdit.videoGenerationFailed')}
                </CardTitle>
                <CardDescription className="text-base" data-testid="error-message">
                  {error.message}
                </CardDescription>
              </CardHeader>

              <CardContent className="space-y-4">
                {/* Recovery Suggestions */}
                <div data-testid="error-recovery-suggestions">
                  <Label className="text-sm font-medium mb-2 block">
                    {t('autoEdit.trySolutions')}
                  </Label>
                  <ul className="space-y-2">
                    {error.recovery_suggestions.map((suggestion, idx) => (
                      <li key={idx} className="flex items-start gap-2 text-sm">
                        <CheckCircle2 className="w-4 h-4 mt-0.5 flex-shrink-0 text-muted-foreground" />
                        <span>{suggestion}</span>
                      </li>
                    ))}
                  </ul>
                </div>

                {/* Technical Details (expandable) */}
                {error.technical_details && (
                  <div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setShowTechnicalDetails(!showTechnicalDetails)}
                      className="w-full justify-between"
                    >
                      <span className="text-sm font-medium">{t('autoEdit.technicalDetails')}</span>
                      {showTechnicalDetails ? (
                        <ChevronUp className="w-4 h-4" />
                      ) : (
                        <ChevronDown className="w-4 h-4" />
                      )}
                    </Button>

                    {showTechnicalDetails && (
                      <Alert className="mt-2">
                        <AlertDescription className="text-xs font-mono whitespace-pre-wrap">
                          {error.technical_details}
                        </AlertDescription>
                      </Alert>
                    )}
                  </div>
                )}

                <Separator />

                {/* Action Buttons */}
                <div className="flex gap-2">
                  <Button
                    onClick={handleStartGeneration}
                    variant="default"
                    className="flex-1"
                    data-testid="retry-button"
                  >
                    <RotateCcw className="w-4 h-4 mr-2" />
                    {t('autoEdit.retryGeneration')}
                  </Button>

                  <Button
                    onClick={handleStartNew}
                    variant="outline"
                    className="flex-1"
                    data-testid="reset-button"
                  >
                    <RefreshCw className="w-4 h-4 mr-2" />
                    {t('autoEdit.startOver')}
                  </Button>
                </div>
              </CardContent>
            </Card>
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
