import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useConfirmDialog } from "@/components/ui/confirm-dialog";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { SpinnerCenter } from "@/components/ui/spinner";
import { Skeleton, SkeletonStats } from "@/components/ui/skeleton";
import { EmptyState } from "@/components/ui/empty-state";
import { useStorage } from "@/hooks/useStorage";
import { useFeatureAccess } from "@/components/auth/ProtectedFeature";
import { GameMetadata, Game } from "@/types/storage";
import { Trash2, Play, Calendar, Clock, Trophy, Sparkles, Lock, Gamepad2 } from "lucide-react";

export function Games() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { listGames, getGameMetadata, deleteGame, getStorageStats, isLoading, error } = useStorage();
  const { confirm, ConfirmDialog } = useConfirmDialog();
  const { isPro } = useFeatureAccess();
  const [games, setGames] = useState<Game[]>([]); // Changed from gameIds:string[]
  const [gamesData, setGamesData] = useState<Map<string, GameMetadata>>(new Map());
  const [stats, setStats] = useState({ total_games: 0, total_clips: 0, total_size_bytes: 0 });

  const loadGames = useCallback(async () => {
    try {
      const loadedGames = await listGames(); // Now returns Game[]
      setGames(loadedGames);

      // Load metadata for each game
      const dataMap = new Map<string, GameMetadata>();
      for (const game of loadedGames) { // Iterate through Game objects
        try {
          // getGameMetadata expects gameId: string
          const metadata = await getGameMetadata(game.game_id); 
          dataMap.set(game.game_id, metadata);
        } catch (err) {
          console.error(`Failed to load metadata for game ${game.game_id}:`, err);
        }
      }
      setGamesData(dataMap);
    } catch (err) {
      console.error("Failed to load games:", err);
    }
  }, [listGames, getGameMetadata]);


  const loadStats = useCallback(async () => {
    try {
      const storageStats = await getStorageStats();
      setStats(storageStats);
    } catch (err) {
      console.error("Failed to load stats:", err);
    }
  }, [getStorageStats]);

  useEffect(() => {
    loadGames();
    loadStats();
  }, [loadGames, loadStats]);

  const handleDeleteGame = async (gameId: string) => {
    const confirmed = await confirm({
      title: t('games.deleteConfirmTitle'),
      description: t('games.deleteConfirmDescription'),
      confirmText: t('common.delete'),
      cancelText: t('common.cancel'),
      variant: 'danger',
    });

    if (!confirmed) {
      return;
    }

    try {
      await deleteGame(gameId);
      await loadGames();
      await loadStats();
    } catch (err) {
      console.error("Failed to delete game:", err);
    }
  };

  const handleAutoEdit = (gameId: string) => {
    // Navigate to auto-edit page with pre-selected game
    navigate({ to: '/auto-edit', search: { gameId } });
  };

  const formatBytes = (bytes: number): string => {
    const gb = bytes / (1024 * 1024 * 1024);
    return gb.toFixed(2) + " GB";
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

  if (isLoading && games.length === 0) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between mb-6">
          <Skeleton className="h-9 w-48" />
          <Skeleton className="h-9 w-24" />
        </div>
        <SkeletonStats />
        <div className="space-y-4">
          {[1, 2, 3].map((i) => (
            <Card key={i}>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="space-y-2">
                    <Skeleton className="h-6 w-48" />
                    <Skeleton className="h-4 w-32" />
                  </div>
                  <div className="flex gap-2">
                    <Skeleton className="h-9 w-28" />
                    <Skeleton className="h-9 w-28" />
                    <Skeleton className="h-9 w-9" />
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                  {[1, 2, 3, 4].map((j) => (
                    <div key={j} className="space-y-1">
                      <Skeleton variant="text" className="w-16" />
                      <Skeleton className="h-5 w-24" />
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-3xl font-bold">{t('games.recordedGames')}</h2>
        <Button onClick={loadGames} variant="outline" size="sm">
          {t('games.refresh')}
        </Button>
      </div>

      {/* Storage Stats */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>{t('games.stats.totalGames')}</CardDescription>
            <CardTitle className="text-3xl">{stats.total_games}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>{t('games.stats.totalClips')}</CardDescription>
            <CardTitle className="text-3xl">{stats.total_clips}</CardTitle>
          </CardHeader>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardDescription>{t('games.stats.storageUsed')}</CardDescription>
            <CardTitle className="text-3xl">{formatBytes(stats.total_size_bytes)}</CardTitle>
          </CardHeader>
        </Card>
      </div>

      {error && (
        <div className="p-4 mb-6 bg-destructive/10 border border-destructive rounded-lg">
          <p className="text-sm text-destructive">{String(error)}</p>
        </div>
      )}

      {/* Games List */}
      {games.length === 0 ? (
        <Card>
          <CardContent>
            <EmptyState
              icon={Gamepad2}
              title={t('games.noGamesRecorded')}
              description={t('games.startRecordingPrompt')}
              action={{
                label: t('games.goToDashboard'),
                onClick: () => navigate({ to: '/' }),
              }}
              size="lg"
            />
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {games.map((game) => { // Map over Game objects directly
            // We can get metadata directly from game object now
            // The `games` state now holds Game objects, and `gamesData` map is still GameMetadata
            // There's a slight confusion between `Game` and `GameMetadata`.
            // `listGames` returns `Game[]`
            // `getAllGames` returns `GameMetadata[]`
            // `games` state should probably be `GameMetadata[]`
            // My previous `loadGames` logic was:
            // const loadedGames = await listGames(); // returns Game[]
            // setGames(loadedGames); // games is Game[]
            // Then loop loadedGames to get metadata: getGameMetadata(game.game_id)
            // But if `games` is `Game[]`, then it doesn't have `champion` etc.
            // My state setup:
            // const [games, setGames] = useState<Game[]>([]); // This is wrong if I want to display GameMetadata
            // const [gamesData, setGamesData] = useState<Map<string, GameMetadata>>(new Map());
            // This means I'm storing `Game[]` in `games`, then fetching `GameMetadata` for each and storing in `gamesData`.
            // The display logic uses `game.champion`, `game.game_mode` etc., which come from `GameMetadata`.
            // So I should populate `gamesData` map.

            const gameMetadata = gamesData.get(game.game_id);

            if (!gameMetadata) {
              return (
                <Card key={game.game_id}>
                  <CardHeader>
                    <div className="flex items-start justify-between">
                      <div className="space-y-2">
                        <Skeleton className="h-6 w-48" />
                        <Skeleton className="h-4 w-32" />
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <SpinnerCenter size="md" label={t('games.loadingGameData')} className="py-4" />
                  </CardContent>
                </Card>
              );
            }

            return (
              <Card key={game.game_id}>
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <CardTitle className="flex items-center gap-2 mb-2">
                        <Trophy className="w-5 h-5" />
                        {gameMetadata.champion} - {gameMetadata.game_mode}
                        <Badge variant={getResultVariant(gameMetadata.result)}>
                          {gameMetadata.result.toUpperCase()}
                        </Badge>
                      </CardTitle>
                      <CardDescription>
                        {gameMetadata.summoner_name} • Game ID: {gameMetadata.game_id}
                      </CardDescription>
                    </div>
                    <div className="flex gap-2">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => navigate({ to: '/editor', search: { gameId: gameMetadata.game_id } })}
                      >
                        <Play className="w-4 h-4 mr-2" />
                        {t('games.game.viewClips')}
                      </Button>
                      <TooltipProvider>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <span className="inline-block">
                              <Button
                                variant="default"
                                size="sm"
                                onClick={() => isPro && handleAutoEdit(gameMetadata.game_id)}
                                disabled={!isPro}
                                className={isPro
                                  ? "bg-gradient-to-r from-purple-500 to-pink-500 hover:from-purple-600 hover:to-pink-600"
                                  : "bg-muted text-muted-foreground cursor-not-allowed"}
                              >
                                {isPro ? (
                                  <Sparkles className="w-4 h-4 mr-2" />
                                ) : (
                                  <Lock className="w-4 h-4 mr-2" />
                                )}
                                {t('games.game.autoEdit')}
                                {!isPro && (
                                  <Badge variant="secondary" className="ml-2 text-xs">
                                    PRO
                                  </Badge>
                                )}
                              </Button>
                            </span>
                          </TooltipTrigger>
                          {!isPro && (
                            <TooltipContent>
                              <p>{t('tooltips.proFeature')}</p>
                            </TooltipContent>
                          )}
                        </Tooltip>
                      </TooltipProvider>
                      <Button
                        variant="destructive"
                        size="sm"
                        onClick={() => handleDeleteGame(gameMetadata.game_id)}
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
                        {t('games.game.date')}
                      </p>
                      <p className="font-medium">
                        {new Date(gameMetadata.game_start_time).toLocaleDateString()}
                      </p>
                    </div>
                    <div>
                      <p className="text-muted-foreground flex items-center gap-1">
                        <Clock className="w-4 h-4" />
                        {t('games.game.duration')}
                      </p>
                      <p className="font-medium">{formatDuration(gameMetadata.game_duration)}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">{t('games.game.kda')}</p>
                      <p className="font-medium">
                        {gameMetadata.kills} / {gameMetadata.deaths} / {gameMetadata.assists}
                      </p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">{t('games.game.recorded')}</p>
                      <p className="font-medium">
                        {new Date(gameMetadata.created_at).toLocaleString()}
                      </p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>
      )}

      <ConfirmDialog />
    </div>
  );
}
