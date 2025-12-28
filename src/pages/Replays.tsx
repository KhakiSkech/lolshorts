import { useEffect, useState } from 'react';
import { lcuApi, MatchInfo } from '@/api/lcu';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { useToast } from '@/components/ui/use-toast';
import { Loader2, Play, Download, RefreshCw } from 'lucide-react';
import { ReplayTargetModal } from '@/components/overlay/ReplayTargetModal';
import { cmd } from '@/api/client';

export function Replays() {
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
          title: "LCU Disconnected",
          description: "Please start League of Legends client to view match history.",
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
        title: "Error",
        description: "Failed to load match history.",
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
        title: "Download Started",
        description: "Replay download has started via League Client.",
      });
      // Poll for status or just wait a bit? 
      // Ideally we should poll status, but for MVP we assume it works.
    } catch (error) {
      toast({
        title: "Download Failed",
        description: "Could not start replay download.",
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
        title: "Launching Replay",
        description: "Select a player to record their highlights!",
      });

      // Show the target selection modal after a short delay (to let the game load)
      setTimeout(() => {
        setIsReplayModalOpen(true);
      }, 3000); // 3 second delay for game to load
    } catch (error) {
      toast({
        title: "Launch Failed",
        description: "Could not launch replay. Make sure it is downloaded.",
        variant: "destructive",
      });
    }
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Replays</h1>
        <Button variant="outline" onClick={loadMatches} disabled={loading}>
          <RefreshCw className={`mr-2 h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {matches.length === 0 && !loading && (
        <div className="text-center text-muted-foreground py-10">
          No matches found or LCU not connected.
        </div>
      )}

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {matches.map((match) => (
          <Card key={match.game_id}>
            <CardHeader className="pb-2">
              <CardTitle className="flex justify-between items-center text-lg">
                <span>{match.win ? "Victory" : "Defeat"}</span>
                <span className="text-sm text-muted-foreground">
                  {new Date(match.game_creation).toLocaleDateString()}
                </span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2 mb-4">
                <div className="flex justify-between text-sm">
                  <span>Mode:</span>
                  <span className="font-medium">{match.game_mode}</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span>KDA:</span>
                  <span className="font-medium text-yellow-500">
                    {match.kills} / {match.deaths} / {match.assists}
                  </span>
                </div>
                <div className="flex justify-between text-sm">
                  <span>Duration:</span>
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
                  Download
                </Button>
                <Button 
                  className="flex-1" 
                  onClick={() => handleLaunch(match.game_id)}
                >
                  <Play className="mr-2 h-4 w-4" />
                  Watch
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
