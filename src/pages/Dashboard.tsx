import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { Spinner } from "@/components/ui/spinner";
import { useAuthStore } from "@/lib/auth";
import { useRecordingStore } from "@/stores/recordingStore";
import { lcuApi, UnifiedGameStatus } from "@/api/lcu";
import { utilsApi } from "@/api/utils";
import { AuthModal } from "@/components/auth";
import { formatStorage, pageStyles } from "@/lib/utils";

interface StorageStats {
  total_games: number;
  total_clips: number;
  total_size_bytes: number;
}

export function Dashboard() {
  const { t } = useTranslation();
  const { checkAuth } = useAuthStore();
  // Use centralized recording store
  const {
    status: { state: recordingState, isRecording },
    startRecording,
    stopRecording
  } = useRecordingStore();

  const [showAuthModal, setShowAuthModal] = useState(false);
  // Unified game status from backend - single source of truth
  const [gameStatus, setGameStatus] = useState<UnifiedGameStatus | null>(null);
  const [isConnecting, setIsConnecting] = useState<boolean>(false);
  const [stats, setStats] = useState<StorageStats | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());

  // Derived state for convenience
  const lcuConnected = gameStatus?.lcu_connected ?? false;
  const inGame = gameStatus?.in_game ?? false;

  // Update unified game status from backend
  const updateGameStatus = useCallback(async () => {
    try {
      const status = await lcuApi.getUnifiedGameStatus();
      setGameStatus(status);
      setLastUpdate(new Date());
    } catch {
      // Reset game state on error (backend might be restarting)
      setGameStatus(prev => prev ? {
        ...prev,
        in_game: false,
        summoner_name: null,
        champion_name: null,
        game_time: null,
        is_recording: false,
      } : null);
    }
  }, []);

  // Initial LCU connection attempt
  const handleConnectLcu = useCallback(async () => {
    setIsConnecting(true);
    try {
      await lcuApi.connect();
      // After connect, immediately fetch status
      await updateGameStatus();
    } catch {
      // LCU connection failure - will retry on next poll
    } finally {
      setIsConnecting(false);
    }
  }, [updateGameStatus]);

  useEffect(() => {
    let isMounted = true;

    const initializeDashboard = async () => {
      try {
        setIsLoading(true);
        setError(null);

        // Check authentication status on mount
        await checkAuth();
        if (!isMounted) return;

        // Auto-connect to LCU on mount
        await handleConnectLcu();
        if (!isMounted) return;

        // Fetch storage stats
        const statsResult = await utilsApi.getDashboardStats();
        if (!isMounted) return;
        setStats(statsResult);

      } catch (err) {
        if (!isMounted) return;
        setError(err instanceof Error ? err.message : t('dashboard.errors.initialization'));
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    };

    initializeDashboard();

    // Poll unified game status every 2 seconds (faster for better UX)
    const interval = setInterval(() => {
      if (isMounted) {
        updateGameStatus();
      }
    }, 2000);

    return () => {
      isMounted = false;
      clearInterval(interval);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Auto-recording: Start when game detected, stop when game ends
  useEffect(() => {
    // Game started - auto start recording
    if (inGame && !isRecording) {
      handleStartRecording();
    }

    // Game ended - auto stop recording
    if (!inGame && isRecording) {
      handleStopRecording();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [inGame, isRecording]);

  const handleStartRecording = async () => {
    try {
      await startRecording(); // Use store action
    } catch {
      // Error is handled in recording store
    }
  };

  const handleStopRecording = async () => {
    try {
      await stopRecording(); // Use store action
    } catch {
      // Error is handled in recording store
    }
  };

  return (
    <div className={pageStyles.container}>
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
        <h2 className={pageStyles.title}>{t('dashboard.title')}</h2>
        <div className="flex items-center gap-4 text-sm text-muted-foreground">
          {isLoading && <Spinner size="sm" label={t('common.loading')} />}
          <span>{t('dashboard.lastUpdate', { time: lastUpdate.toLocaleTimeString() })}</span>
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="mb-6 p-4 bg-destructive/10 border border-destructive/20 rounded-lg">
          <div className="flex items-center gap-2 text-destructive">
            <span className="text-sm font-medium">{t('dashboard.error.title')}</span>
            <span className="text-sm">{error}</span>
            <button
              onClick={() => window.location.reload()}
              className="ml-auto text-xs underline hover:text-destructive/80"
            >
              {t('dashboard.error.retry')}
            </button>
          </div>
        </div>
      )}

      {/* Initial Loading State - Only show on first load, not during polling */}
      {isLoading && !gameStatus && (
        <div className="space-y-6 mb-8">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Skeleton for LCU Status Card */}
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <Skeleton className="h-6 w-40" />
                  <Skeleton className="h-6 w-20" />
                </div>
                <Skeleton className="h-4 w-60 mt-2" />
              </CardHeader>
              <CardContent>
                <Skeleton className="h-20 w-full" />
              </CardContent>
            </Card>
            {/* Skeleton for Game Status Card */}
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <Skeleton className="h-6 w-32" />
                  <Skeleton className="h-6 w-20" />
                </div>
                <Skeleton className="h-4 w-48 mt-2" />
              </CardHeader>
              <CardContent>
                <Skeleton className="h-20 w-full" />
              </CardContent>
            </Card>
            {/* Skeleton for Stats Card */}
            <Card>
              <CardHeader>
                <Skeleton className="h-6 w-28" />
                <Skeleton className="h-4 w-40 mt-2" />
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <Skeleton variant="text" className="w-full" />
                  <Skeleton variant="text" className="w-3/4" />
                  <Skeleton variant="text" className="w-1/2" />
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      )}

      {/* Main Content - Only show after initial load */}
      {(!isLoading || gameStatus) && (
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
        {/* League of Legends Connection Status */}
        <Card className={lcuConnected ? "border-green-200 bg-green-50/50 dark:border-green-800/20 dark:bg-green-950/20" : ""}>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className={`w-2 h-2 rounded-full ${lcuConnected ? "bg-green-500 animate-pulse" : "bg-red-500"}`} />
                {t('dashboard.lcuStatus.title')}
              </div>
              <Badge
                variant={lcuConnected ? "default" : "destructive"}
                className={lcuConnected ? "bg-green-600 hover:bg-green-700" : ""}
              >
                {lcuConnected ? (
                  <span className="flex items-center gap-1">
                    🟢 {t('dashboard.lcuStatus.connected')}
                  </span>
                ) : (
                  <span className="flex items-center gap-1">
                    🔴 {t('dashboard.lcuStatus.disconnected')}
                  </span>
                )}
              </Badge>
            </CardTitle>
            <CardDescription>
              {lcuConnected ? t('dashboard.lcuStatus.messages.connected') : t('dashboard.lcuStatus.messages.disconnected')}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {!lcuConnected ? (
                <>
                  {isConnecting && (
                    <div className="flex items-center gap-2 p-2 bg-info/10 rounded">
                      <Spinner size="sm" className="text-info" />
                      <p className="text-sm text-info font-medium">{t('dashboard.lcuStatus.connecting')}</p>
                    </div>
                  )}
                  <div className="p-3 bg-yellow-50 dark:bg-yellow-950/20 rounded border border-yellow-200 dark:border-yellow-800">
                    <p className="text-sm font-medium text-yellow-800 dark:text-yellow-200 mb-2">
                      {t('dashboard.lcuStatus.helpTitle')}
                    </p>
                    <div className="space-y-1">
                      <p className="text-xs text-yellow-700 dark:text-yellow-300">
                        • {t('dashboard.lcuStatus.messages.autoReconnect', { seconds: 3 })}
                      </p>
                      <p className="text-xs text-yellow-700 dark:text-yellow-300">
                        • {t('dashboard.lcuStatus.messages.startLeague')}
                      </p>
                      <p className="text-xs text-yellow-700 dark:text-yellow-300">
                        • {t('dashboard.lcuStatus.messages.adminRequired')}
                      </p>
                    </div>
                  </div>
                </>
              ) : (
                <div className="p-3 bg-green-50 dark:bg-green-950/20 rounded border border-green-200 dark:border-green-800">
                  <p className="text-sm text-green-600 dark:text-green-400 font-medium">
                    {t('dashboard.lcuStatus.messages.readyToDetect')}
                  </p>
                  <p className="text-xs text-green-600 dark:text-green-400 mt-1">
                    {t('dashboard.lcuStatus.messages.autoMonitoring')}
                  </p>
                </div>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Current Game Info */}
        <Card className={inGame ? "border-orange-200 bg-orange-50/50 dark:border-orange-800/20 dark:bg-orange-950/20" : ""}>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <div className={`w-2 h-2 rounded-full ${inGame ? "bg-orange-500 animate-pulse" : "bg-gray-400"}`} />
                {t('dashboard.gameStatus.title')}
              </div>
              <Badge
                variant={inGame ? "default" : "secondary"}
                className={inGame ? "bg-orange-600 hover:bg-orange-700" : ""}
              >
                {inGame ? (
                  <span className="flex items-center gap-1">
                    ⚔️ {t('dashboard.gameStatus.inGame')}
                  </span>
                ) : (
                  <span className="flex items-center gap-1">
                    🎮 {t('dashboard.gameStatus.noGame')}
                  </span>
                )}
              </Badge>
            </CardTitle>
            <CardDescription>
              {inGame ? t('dashboard.gameStatus.sessionDetected') : t('dashboard.gameStatus.messages.notInGame')}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {inGame && gameStatus ? (
              <div className="space-y-3">
                <div className="p-3 bg-orange-50 dark:bg-orange-950/20 rounded border border-orange-200 dark:border-orange-800">
                  <div className="grid grid-cols-2 gap-3">
                    <div className="flex justify-between text-sm">
                      <span className="text-muted-foreground">{t('dashboard.gameStatus.fields.champion')}</span>
                      <span className="font-medium">{gameStatus.champion_name ?? 'Unknown'}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-muted-foreground">{t('dashboard.gameStatus.fields.gameMode')}</span>
                      <span className="font-medium">
                        {typeof gameStatus.game_mode === 'string' ? gameStatus.game_mode : 'Replay'}
                      </span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-muted-foreground">{t('dashboard.gameStatus.fields.gameTime')}</span>
                      <span className="font-medium font-mono">
                        {gameStatus.game_time != null
                          ? `${Math.floor(gameStatus.game_time / 60)}:${String(Math.floor(gameStatus.game_time % 60)).padStart(2, '0')}`
                          : '--:--'}
                      </span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-muted-foreground">{t('dashboard.gameStatus.fields.summoner')}</span>
                      <span className="font-medium text-xs">{gameStatus.summoner_name ?? 'Unknown'}</span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-2 text-xs text-orange-600 dark:text-orange-400">
                  <div className="w-1.5 h-1.5 bg-orange-500 rounded-full animate-pulse" />
                  <span>{gameStatus.is_recording ? t('dashboard.gameStatus.messages.activelyRecording') : t('dashboard.gameStatus.messages.monitoring')}</span>
                </div>
              </div>
            ) : (
              <div className="p-3 bg-gray-50 dark:bg-gray-950/20 rounded border border-gray-200 dark:border-gray-800">
                <span className="text-sm text-muted-foreground">
                  {lcuConnected ? (
                    <span className="flex items-center gap-2">
                      <span className="w-2 h-2 bg-gray-400 rounded-full animate-pulse inline-block" />
                      {t('dashboard.gameStatus.messages.waiting')}
                    </span>
                  ) : (
                    <span className="flex items-center gap-2">
                      <span className="w-2 h-2 bg-gray-400 rounded-full inline-block" />
                      {t('dashboard.gameStatus.messages.connectFirst')}
                    </span>
                  )}
                </span>
              </div>
            )}
          </CardContent>
        </Card>


        {/* Quick Stats */}
        <Card>
          <CardHeader>
            <CardTitle>{t('dashboard.stats.title')}</CardTitle>
            <CardDescription>{t('dashboard.stats.subtitle')}</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">{t('dashboard.stats.totalGames')}</span>
                <span className="font-medium">
                  {stats ? stats.total_games : t('dashboard.stats.comingSoon')}
                </span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">{t('dashboard.stats.totalClips')}</span>
                <span className="font-medium">
                  {stats ? stats.total_clips : t('dashboard.stats.comingSoon')}
                </span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">{t('dashboard.stats.storageUsed')}</span>
                <span className="font-medium">
                  {stats ? formatStorage(stats.total_size_bytes) : t('dashboard.stats.comingSoon')}
                </span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
      )}

      {/* Getting Started Guide - Only show after initial load */}
      {(!isLoading || gameStatus) && (
        <Card>
          <CardHeader>
            <CardTitle>{t('dashboard.gettingStarted.title')}</CardTitle>
            <CardDescription>{t('dashboard.gettingStarted.subtitle')}</CardDescription>
          </CardHeader>
          <CardContent>
            <ol className="list-decimal list-inside space-y-2 text-sm">
              <li className={lcuConnected ? "text-muted-foreground line-through" : "font-medium"}>
                {t('dashboard.gettingStarted.steps.startLeague')}
              </li>
              <li className={inGame ? "text-muted-foreground line-through" : lcuConnected ? "font-medium" : "text-muted-foreground"}>
                {t('dashboard.gettingStarted.steps.enterGame')}
              </li>
              <li className={recordingState === "recording" ? "text-muted-foreground line-through" : inGame ? "font-medium" : "text-muted-foreground"}>
                {t('dashboard.gettingStarted.steps.autoRecord')}
              </li>
              <li className="text-muted-foreground">
                {t('dashboard.gettingStarted.steps.playNormal')}
              </li>
              <li className="text-muted-foreground">
                {t('dashboard.gettingStarted.steps.afterGame')}
              </li>
            </ol>
          </CardContent>
        </Card>
      )}

      {/* Auth Modal */}
      {showAuthModal && <AuthModal open={showAuthModal} onClose={() => setShowAuthModal(false)} />}
    </div>
  );
}
