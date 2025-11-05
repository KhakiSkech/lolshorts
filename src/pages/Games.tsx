import { useState, useEffect } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useStorage, GameMetadata } from "@/hooks/useStorage";
import { Film, Trash2, Play, Calendar, Clock, Trophy } from "lucide-react";

export function Games() {
  const { listGames, getGameMetadata, deleteGame, getStorageStats, isLoading, error } = useStorage();
  const [gameIds, setGameIds] = useState<string[]>([]);
  const [gamesData, setGamesData] = useState<Map<string, GameMetadata>>(new Map());
  const [stats, setStats] = useState({ total_games: 0, total_clips: 0, total_size_bytes: 0 });

  useEffect(() => {
    loadGames();
    loadStats();
  }, []);

  const loadGames = async () => {
    try {
      const ids = await listGames();
      setGameIds(ids);

      // Load metadata for each game
      const dataMap = new Map<string, GameMetadata>();
      for (const id of ids) {
        try {
          const metadata = await getGameMetadata(id);
          dataMap.set(id, metadata);
        } catch (err) {
          console.error(`Failed to load metadata for game ${id}:`, err);
        }
      }
      setGamesData(dataMap);
    } catch (err) {
      console.error("Failed to load games:", err);
    }
  };

  const loadStats = async () => {
    try {
      const storageStats = await getStorageStats();
      setStats(storageStats);
    } catch (err) {
      console.error("Failed to load stats:", err);
    }
  };

  const handleDeleteGame = async (gameId: string) => {
    if (!confirm("Are you sure you want to delete this game and all its clips?")) {
      return;
    }

    try {
      await deleteGame(gameId);
      await loadGames();
      await loadStats();
    } catch (err) {
      console.error("Failed to delete game:", err);
      alert("Failed to delete game: " + err);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 Bytes";
    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  const getResultVariant = (result: string) => {
    if (result.toLowerCase() === "win") return "default";
    if (result.toLowerCase() === "loss") return "destructive";
    return "secondary";
  };

  if (isLoading && gameIds.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-muted-foreground">Loading games...</p>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-3xl font-bold">Recorded Games</h2>
        <Button onClick={loadGames} variant="outline" size="sm">
          Refresh
        </Button>
      </div>

      {/* Storage Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Total Games</CardDescription>
            <CardTitle className="text-3xl">{stats.total_games}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Total Clips</CardDescription>
            <CardTitle className="text-3xl">{stats.total_clips}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>Storage Used</CardDescription>
            <CardTitle className="text-3xl">{formatBytes(stats.total_size_bytes)}</CardTitle>
          </CardHeader>
        </Card>
      </div>

      {error && (
        <div className="p-4 mb-6 bg-destructive/10 border border-destructive rounded-lg">
          <p className="text-sm text-destructive">{error}</p>
        </div>
      )}

      {/* Games List */}
      {gameIds.length === 0 ? (
        <Card>
          <CardContent className="py-12 text-center">
            <Film className="w-16 h-16 mx-auto mb-4 text-muted-foreground" />
            <h3 className="text-lg font-semibold mb-2">No Games Recorded Yet</h3>
            <p className="text-sm text-muted-foreground mb-4">
              Start recording games from the Dashboard to see them here.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {gameIds.map((gameId) => {
            const game = gamesData.get(gameId);

            if (!game) {
              return (
                <Card key={gameId}>
                  <CardContent className="py-6">
                    <p className="text-sm text-muted-foreground">Loading game data...</p>
                  </CardContent>
                </Card>
              );
            }

            return (
              <Card key={gameId}>
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <CardTitle className="flex items-center gap-2 mb-2">
                        <Trophy className="w-5 h-5" />
                        {game.champion} - {game.game_mode}
                        <Badge variant={getResultVariant(game.result)}>
                          {game.result.toUpperCase()}
                        </Badge>
                      </CardTitle>
                      <CardDescription>
                        {game.summoner_name} • Game ID: {game.game_id}
                      </CardDescription>
                    </div>
                    <div className="flex gap-2">
                      <Button variant="outline" size="sm">
                        <Play className="w-4 h-4 mr-2" />
                        View Clips
                      </Button>
                      <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => handleDeleteGame(gameId)}
                      >
                        <Trash2 className="w-4 h-4" />
                      </Button>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div>
                      <p className="text-muted-foreground flex items-center gap-1">
                        <Calendar className="w-4 h-4" />
                        Date
                      </p>
                      <p className="font-medium">
                        {new Date(game.game_start_time).toLocaleDateString()}
                      </p>
                    </div>
                    <div>
                      <p className="text-muted-foreground flex items-center gap-1">
                        <Clock className="w-4 h-4" />
                        Duration
                      </p>
                      <p className="font-medium">{formatDuration(game.game_duration)}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">KDA</p>
                      <p className="font-medium">
                        {game.kills} / {game.deaths} / {game.assists}
                      </p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">Recorded</p>
                      <p className="font-medium">
                        {new Date(game.created_at).toLocaleString()}
                      </p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}
    </div>
  );
}
