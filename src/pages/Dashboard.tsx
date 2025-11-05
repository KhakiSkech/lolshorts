import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useAuthStore } from "@/lib/auth";
import { AuthModal } from "@/components/auth";

interface GameInfo {
  game_id: string;
  champion: string;
  game_mode: string;
  game_time: number;
}

export function Dashboard() {
  const { isAuthenticated, checkAuth } = useAuthStore();
  const [showAuthModal, setShowAuthModal] = useState(false);
  const [recordingStatus, setRecordingStatus] = useState<"Idle" | "Recording" | "Processing">("Idle");
  const [lcuConnected, setLcuConnected] = useState<boolean>(false);
  const [currentGame, setCurrentGame] = useState<GameInfo | null>(null);
  const [isConnecting, setIsConnecting] = useState<boolean>(false);

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

  // Auto-recording: Start when game detected, stop when game ends
  useEffect(() => {
    if (!isAuthenticated) {
      return;
    }

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
  }, [currentGame, isAuthenticated]);

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
    if (!isAuthenticated) {
      setShowAuthModal(true);
      return;
    }

    try {
      await invoke("start_recording");
      setRecordingStatus("Recording");
    } catch (error) {
      console.error("Failed to start recording:", error);
      if ((error as string).includes("authenticated")) {
        setShowAuthModal(true);
      }
    }
  };

  const handleStopRecording = async () => {
    if (!isAuthenticated) {
      setShowAuthModal(true);
      return;
    }

    try {
      await invoke("stop_recording");
      setRecordingStatus("Idle");
    } catch (error) {
      console.error("Failed to stop recording:", error);
      if ((error as string).includes("authenticated")) {
        setShowAuthModal(true);
      }
    }
  };

  return (
    <div>
      <h2 className="text-3xl font-bold mb-6">Dashboard</h2>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
        {/* League of Legends Connection Status */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              League of Legends
              <Badge variant={lcuConnected ? "default" : "destructive"}>
                {lcuConnected ? "🟢 Connected" : "🔴 Disconnected"}
              </Badge>
            </CardTitle>
            <CardDescription>
              {lcuConnected ? "Client connected successfully" : "Connect to League client to start"}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {!lcuConnected ? (
                <>
                  {isConnecting && (
                    <div className="flex items-center gap-2">
                      <div className="animate-spin h-4 w-4 border-2 border-primary border-t-transparent rounded-full" />
                      <p className="text-sm text-muted-foreground">Connecting...</p>
                    </div>
                  )}
                  <div className="space-y-1">
                    <p className="text-xs text-muted-foreground">
                      • Auto-reconnecting every 3 seconds
                    </p>
                    <p className="text-xs text-muted-foreground">
                      • Make sure League of Legends client is running
                    </p>
                  </div>
                </>
              ) : (
                <>
                  <p className="text-sm text-green-600 font-medium">✓ Ready to detect games</p>
                  <p className="text-xs text-muted-foreground">
                    Auto-monitoring for game sessions
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
              Current Game
              <Badge variant={currentGame ? "default" : "secondary"}>
                {currentGame ? "🎮 In Game" : "⚫ No Game"}
              </Badge>
            </CardTitle>
            <CardDescription>
              {currentGame ? "Game session detected" : "Not in a game"}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {currentGame ? (
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">Champion:</span>
                  <span className="font-medium">{currentGame.champion}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">Game Mode:</span>
                  <span className="font-medium">{currentGame.game_mode}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">Game Time:</span>
                  <span className="font-medium">{Math.floor(currentGame.game_time / 60)}:{String(Math.floor(currentGame.game_time % 60)).padStart(2, '0')}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">Game ID:</span>
                  <span className="font-mono text-xs">{currentGame.game_id}</span>
                </div>
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">
                {lcuConnected ? "Waiting for game to start..." : "Connect to League first"}
              </p>
            )}
          </CardContent>
        </Card>

        {/* Recording Status */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              Recording
              <Badge variant={recordingStatus === "Recording" ? "destructive" : "secondary"}>
                {recordingStatus === "Recording" ? "🔴 Recording" : recordingStatus === "Processing" ? "⏳ Processing" : "⚫ Idle"}
              </Badge>
            </CardTitle>
            <CardDescription>
              {recordingStatus === "Recording"
                ? "Auto-recording gameplay"
                : "Will auto-start when game begins"}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {isAuthenticated && lcuConnected && !currentGame && (
                <p className="text-sm text-muted-foreground">
                  ✓ Ready - Recording will start automatically when you enter a game
                </p>
              )}
              {isAuthenticated && currentGame && recordingStatus === "Recording" && (
                <p className="text-sm text-green-600 font-medium">
                  🔴 Recording in progress - Auto-capturing highlights
                </p>
              )}
              {!isAuthenticated && (
                <p className="text-sm text-muted-foreground">
                  Login required to enable auto-recording
                </p>
              )}
              {isAuthenticated && !lcuConnected && (
                <p className="text-sm text-muted-foreground">
                  Connect to League first to enable auto-recording
                </p>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Quick Stats */}
        <Card>
          <CardHeader>
            <CardTitle>Quick Stats</CardTitle>
            <CardDescription>Your recording statistics</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Total Games:</span>
                <span className="font-medium">Coming soon</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Total Clips:</span>
                <span className="font-medium">Coming soon</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-muted-foreground">Storage Used:</span>
                <span className="font-medium">Coming soon</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Getting Started Guide */}
      <Card>
        <CardHeader>
          <CardTitle>Getting Started</CardTitle>
          <CardDescription>Simple setup - automatic recording</CardDescription>
        </CardHeader>
        <CardContent>
          <ol className="list-decimal list-inside space-y-2 text-sm">
            <li className={isAuthenticated ? "text-muted-foreground line-through" : "font-medium"}>
              Login or create an account
            </li>
            <li className={lcuConnected ? "text-muted-foreground line-through" : isAuthenticated ? "font-medium" : "text-muted-foreground"}>
              Start League of Legends client (auto-connection will begin)
            </li>
            <li className={currentGame ? "text-muted-foreground line-through" : lcuConnected ? "font-medium" : "text-muted-foreground"}>
              Enter a game (any game mode)
            </li>
            <li className={recordingStatus === "Recording" ? "text-muted-foreground line-through" : currentGame ? "font-medium" : "text-muted-foreground"}>
              Recording starts automatically when game begins
            </li>
            <li className="text-muted-foreground">
              Play normally - highlights will be automatically detected based on kills, objectives, and multikills
            </li>
            <li className="text-muted-foreground">
              After the game, recording stops automatically and clips are saved for editing
            </li>
          </ol>
        </CardContent>
      </Card>

      {/* Auth Modal */}
      {showAuthModal && <AuthModal onClose={() => setShowAuthModal(false)} />}
    </div>
  );
}
