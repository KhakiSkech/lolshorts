import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { lcuApi, MatchInfo } from '@/api/lcu';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useToast } from '@/components/ui/use-toast';
import { Loader2, Play, Download, RefreshCw } from 'lucide-react';
import { ReplayTargetModal } from '@/components/overlay/ReplayTargetModal';
import { cmd } from '@/api/client';
import { pageStyles } from '@/lib/utils';

export function Replays() {
  const { t } = useTranslation();
  const [matches, setMatches] = useState<MatchInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [downloadingId, setDownloadingId] = useState<number | null>(null);
  const [isReplayModalOpen, setIsReplayModalOpen] = useState(false);
  const { toast } = useToast();

  const loadMatches = async () => {
    setLoading(true);
    try {
      const isConnected = await lcuApi.checkStatus();
      if (!isConnected) {
        toast({
          title: t('replays.toast.lcuDisconnected'),
          description: t('replays.toast.lcuDisconnectedDesc'),
          variant: "destructive",
        });
        setMatches([]);
        return;
      }

      const history = await lcuApi.listMatchHistory(0, 20);
      setMatches(history);
    } catch (error) {
      console.error("Failed to load match history:", error);
      toast({
        title: t('replays.toast.loadError'),
        description: t('replays.toast.loadErrorDesc'),
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadMatches();
  }, []);

  const handleDownload = async (gameId: number) => {
    setDownloadingId(gameId);
    try {
      await lcuApi.downloadReplay(gameId);
      toast({
        title: t('replays.toast.downloadStarted'),
        description: t('replays.toast.downloadStartedDesc'),
      });
    } catch (error) {
      toast({
        title: t('replays.toast.downloadFailed'),
        description: t('replays.toast.downloadFailedDesc'),
        variant: "destructive",
      });
    } finally {
      setDownloadingId(null);
    }
  };

  const handleLaunch = async (gameId: number) => {
    try {
      // Launch the replay via LCU
      await lcuApi.launchReplay(gameId);

      // Notify backend to switch to replay mode
      await cmd<void>('notify_replay_launched', {});

      toast({
        title: t('replays.toast.launchingReplay'),
        description: t('replays.toast.launchingReplayDesc'),
      });

      // Show the target selection modal after a short delay (to let the game load)
      setTimeout(() => {
        setIsReplayModalOpen(true);
      }, 3000); // 3 second delay for game to load
    } catch (error) {
      toast({
        title: t('replays.toast.launchFailed'),
        description: t('replays.toast.launchFailedDesc'),
        variant: "destructive",
      });
    }
  };

  return (
    <div className={pageStyles.container}>
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
        <h1 className={pageStyles.title}>{t('replays.title')}</h1>
        <Button variant="outline" onClick={loadMatches} disabled={loading}>
          <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          {t('replays.refresh')}
        </Button>
      </div>

      {matches.length === 0 && !loading && (
        <div className="text-center text-muted-foreground py-10">
          {t('replays.noMatches')}
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {matches.map((match) => (
          <Card key={match.game_id}>
            <CardHeader className="pb-2">
              <CardTitle className="flex justify-between items-center text-lg">
                <span>{match.win ? t('replays.victory') : t('replays.defeat')}</span>
                <span className="text-sm text-muted-foreground">
                  {new Date(match.game_creation).toLocaleDateString()}
                </span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2 mb-4">
                <div className="flex justify-between text-sm">
                  <span>{t('replays.mode')}:</span>
                  <span className="font-medium">{match.game_mode}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span>{t('replays.kda')}:</span>
                  <span className="font-medium text-yellow-500">
                    {match.kills} / {match.deaths} / {match.assists}
                  </span>
                </div>
                <div className="flex justify-between text-sm">
                  <span>{t('replays.duration')}:</span>
                  <span>{Math.floor(match.game_duration / 60)}m {match.game_duration % 60}s</span>
                </div>
              </div>

              <div className="flex gap-2">
                <Button
                  className="flex-1"
                  variant="secondary"
                  onClick={() => handleDownload(match.game_id)}
                  disabled={downloadingId === match.game_id}
                >
                  {downloadingId === match.game_id ? (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  ) : (
                    <Download className="mr-2 h-4 w-4" />
                  )}
                  {t('replays.download')}
                </Button>
                <Button
                  className="flex-1"
                  onClick={() => handleLaunch(match.game_id)}
                >
                  <Play className="mr-2 h-4 w-4" />
                  {t('replays.watch')}
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Replay Target Selection Modal */}
      <ReplayTargetModal
        isOpen={isReplayModalOpen}
        onClose={() => setIsReplayModalOpen(false)}
      />
    </div>
  );
}
