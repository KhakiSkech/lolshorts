import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useAuthStore } from "@/lib/auth";
import { AuthModal } from "@/components/auth";
import { formatStorage } from "@/lib/utils";

interface GameInfo {
  game_id: string;
  champion: string;
  game_mode: string;
  game_time: number;
}

interface StorageStats {
  total_games: number;
  total_clips: number;
  total_size_bytes: number;
}

export function Dashboard() {
  const { t } = useTranslation();
  const { isAuthenticated, checkAuth } = useAuthStore();
  const [showAuthModal, setShowAuthModal] = useState(false);
  const [recordingStatus, setRecordingStatus] = useState<"Idle" | "Recording" | "Processing">("Idle");
  const [lcuConnected, setLcuConnected] = useState<boolean>(false);
  const [currentGame, setCurrentGame] = useState<GameInfo | null>(null);
  const [isConnecting, setIsConnecting] = useState<boolean>(false);
  const [stats, setStats] = useState<StorageStats | null>(null);

  useEffect(() => {
    // Check authentication status on mount
    checkAuth();

    // Check recording status
    invoke<string>("get_recording_status")
      .then((status) => setRecordingStatus(status as "Idle" | "Recording" | "Processing"))
      .catch((error) => console.error("Failed to get recording status:", error));

    // Auto-connect to LCU on mount
    handleConnectLcu();

    // Poll LCU status and game info every 3 seconds
    const interval = setInterval(() => {
      checkLcuStatus();
      if (lcuConnected) {
        updateCurrentGame();
      }
    }, 3000);

    return () => clearInterval(interval);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Fetch storage stats on mount
  useEffect(() => {
    invoke<StorageStats>("get_dashboard_stats")
      .then((data) => setStats(data))
      .catch((error) => console.error("Failed to fetch storage stats:", error));
  }, []);

  // Auto-recording: Start when game detected, stop when game ends
  useEffect(() => {
    // Game started - auto start recording
    if (currentGame && recordingStatus === "Idle") {
      console.log("Game detected, auto-starting recording...");
      handleStartRecording();
    }

    // Game ended - auto stop recording
    if (!currentGame && recordingStatus === "Recording") {
      console.log("Game ended, auto-stopping recording...");
      handleStopRecording();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentGame]);

  const checkLcuStatus = async () => {
    try {
      const connected = await invoke<boolean>("check_lcu_status");
      setLcuConnected(connected);
    } catch (error) {
      console.error("Failed to check LCU status:", error);
      setLcuConnected(false);
    }
  };

  const handleConnectLcu = async () => {
    setIsConnecting(true);
    try {
      const success = await invoke<boolean>("connect_lcu");
      setLcuConnected(success);
      // No alert - UI already shows connection status
      // Auto-retry every 3 seconds handles reconnection
    } catch (error) {
      console.error("Failed to connect to LCU:", error);
      setLcuConnected(false);
      // No alert - this is expected when League client is not running
    } finally {
      setIsConnecting(false);
    }
  };

  const updateCurrentGame = async () => {
    try {
      const game = await invoke<GameInfo | null>("get_current_game");
      setCurrentGame(game);
    } catch (error) {
      console.error("Failed to get current game:", error);
      setCurrentGame(null);
    }
  };

  const handleStartRecording = async () => {
    try {
      await invoke("start_recording");
      setRecordingStatus("Recording");
    } catch (error) {
      console.error("Failed to start recording:", error);
    }
  };

  const handleStopRecording = async () => {
    try {
      await invoke("stop_recording");
      setRecordingStatus("Idle");
    } catch (error) {
      console.error("Failed to stop recording:", error);
    }
  };

  return (
    <div>
      <h2 className="text-3xl font-bold mb-6">{t('dashboard.title')}</h2>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
        {/* League of Legends Connection Status */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              {t('dashboard.lcuStatus.title')}
              <Badge variant={lcuConnected ? "default" : "destructive"}>
                {lcuConnected ? `🟢 ${t('dashboard.lcuStatus.connected')}` : `🔴 ${t('dashboard.lcuStatus.disconnected')}`}
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
                    <div className="flex items-center gap-2">
                      <div className="animate-spin h-4 w-4 border-2 border-primary border-t-transparent rounded-full" />
                      <p className="text-sm text-muted-foreground">{t('dashboard.lcuStatus.connecting')}</p>
                    </div>
                  )}
                  <div className="space-y-1">
                    <p className="text-xs text-muted-foreground">
                      • {t('dashboard.lcuStatus.messages.autoReconnect', { seconds: 3 })}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      • {t('dashboard.lcuStatus.messages.startLeague')}
                    </p>
                  </div>
                </>
              ) : (
                <>
                  <p className="text-sm text-green-600 font-medium">{t('dashboard.lcuStatus.messages.readyToDetect')}</p>
                  <p className="text-xs text-muted-foreground">
                    {t('dashboard.lcuStatus.messages.autoMonitoring')}
                  </p>
                </>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Current Game Info */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              {t('dashboard.gameStatus.title')}
              <Badge variant={currentGame ? "default" : "secondary"}>
                {currentGame ? t('dashboard.gameStatus.inGame') : t('dashboard.gameStatus.noGame')}
              </Badge>
            </CardTitle>
            <CardDescription>
              {currentGame ? t('dashboard.gameStatus.sessionDetected') : t('dashboard.gameStatus.messages.notInGame')}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {currentGame ? (
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">{t('dashboard.gameStatus.fields.champion')}</span>
                  <span className="font-medium">{currentGame.champion}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">{t('dashboard.gameStatus.fields.gameMode')}</span>
                  <span className="font-medium">{currentGame.game_mode}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">{t('dashboard.gameStatus.fields.gameTime')}</span>
                  <span className="font-medium">{Math.floor(currentGame.game_time / 60)}:{String(Math.floor(currentGame.game_time % 60)).padStart(2, '0')}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">{t('dashboard.gameStatus.fields.gameId')}</span>
                  <span className="font-mono text-xs">{currentGame.game_id}</span>
                </div>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                {lcuConnected ? t('dashboard.gameStatus.messages.waiting') : t('dashboard.gameStatus.messages.connectFirst')}
              </p>
            )}
          </CardContent>
        </Card>

        {/* Recording Status */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              {t('dashboard.recordingStatus.title')}
              <Badge variant={recordingStatus === "Recording" ? "destructive" : "secondary"}>
                {recordingStatus === "Recording" ? t('dashboard.recordingStatus.recording') : recordingStatus === "Processing" ? t('dashboard.recordingStatus.processing') : t('dashboard.recordingStatus.idle')}
              </Badge>
            </CardTitle>
            <CardDescription>
              {recordingStatus === "Recording"
                ? t('dashboard.recordingStatus.messages.autoRecording')
                : t('dashboard.recordingStatus.messages.autoStart')}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {lcuConnected && !currentGame && (
                <p className="text-sm text-muted-foreground">
                  {t('dashboard.recordingStatus.messages.ready')}
                </p>
              )}
              {currentGame && recordingStatus === "Recording" && (
                <p className="text-sm text-green-600 font-medium">
                  {t('dashboard.recordingStatus.messages.recordingHighlights')}
                </p>
              )}
              {!lcuConnected && (
                <p className="text-sm text-muted-foreground">
                  {t('dashboard.recordingStatus.messages.connectFirst')}
                </p>
              )}
            </div>
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

      {/* Getting Started Guide */}
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
            <li className={currentGame ? "text-muted-foreground line-through" : lcuConnected ? "font-medium" : "text-muted-foreground"}>
              {t('dashboard.gettingStarted.steps.enterGame')}
            </li>
            <li className={recordingStatus === "Recording" ? "text-muted-foreground line-through" : currentGame ? "font-medium" : "text-muted-foreground"}>
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

      {/* Auth Modal */}
      {showAuthModal && <AuthModal onClose={() => setShowAuthModal(false)} />}
    </div>
  );
}
