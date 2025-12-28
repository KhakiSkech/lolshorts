import { useEffect, useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { lcuApi, PlayerInfo } from '@/api/lcu';
import { cmd } from '@/api/client';
import { useToast } from '@/components/ui/use-toast';

// Create a recordingApi wrapper if not exists, or direct call for now
const setRecordingTarget = (summonerName: string | null) =>
  cmd<void>('set_recording_target', { summonerName });

interface ReplayTargetModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ReplayTargetModal({ isOpen, onClose }: ReplayTargetModalProps) {
  const [players, setPlayers] = useState<PlayerInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const { toast } = useToast();

  useEffect(() => {
    if (isOpen) {
      loadPlayers();
    }
  }, [isOpen]);

  const loadPlayers = async () => {
    setLoading(true);
    try {
      const participants = await lcuApi.getGameParticipants();
      setPlayers(participants);
    } catch (error) {
      console.error("Failed to load participants:", error);
      // Fallback or retry?
    } finally {
      setLoading(false);
    }
  };

  const handleSelectTarget = async (summonerName: string) => {
    try {
      await setRecordingTarget(summonerName);
      toast({
        title: "Target Set",
        description: `Recording highlights for: ${summonerName}. Double-click their champion to lock camera!`,
      });
      onClose();
    } catch (error) {
      toast({
        title: "Error",
        description: "Failed to set recording target.",
        variant: "destructive",
      });
    }
  };

  // Observer mode is available but commented out in the UI
  // To enable, uncomment the observer mode button and use:
  // await setRecordingTarget(null);

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Replay Recording Setup</DialogTitle>
          <DialogDescription>
            Who should we record? Select a player to auto-capture their highlights.
          </DialogDescription>
        </DialogHeader>

        <div className="grid grid-cols-2 gap-4 py-4">
          {loading ? (
            <div className="col-span-2 text-center py-4">Loading players...</div>
          ) : (
            <>
              {players.map((player) => (
                <Button
                  key={player.summoner_name}
                  variant="outline"
                  className="h-auto py-3 flex flex-col items-start"
                  onClick={() => handleSelectTarget(player.summoner_name)}
                >
                  <span className="font-bold">{player.summoner_name}</span>
                  <span className="text-xs text-muted-foreground">ID: {player.champion_id}</span>
                </Button>
              ))}
            </>
          )}
        </div>

        {/* Optional Observer Mode Button */}
        {/* 
        <div className="flex justify-center border-t pt-4">
            <Button variant="ghost" onClick={handleSelectAll} className="w-full">
                <Users className="mr-2 h-4 w-4" />
                Record All Highlights (Observer Mode)
            </Button>
        </div>
        */}
      </DialogContent>
    </Dialog>
  );
}
