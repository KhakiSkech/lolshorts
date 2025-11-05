import { useState, useEffect } from 'react';
import { useEditorStore } from '@/stores/editorStore';
import { useEditor } from '@/hooks/useEditor';
import { useStorage } from '@/hooks/useStorage';
import { EditorLayout } from '@/components/editor/EditorLayout';
import { ClipLibrary } from '@/components/editor/ClipLibrary';
import { VideoPreview } from '@/components/editor/VideoPreview';
import { CompositionSettings } from '@/components/editor/CompositionSettings';
import { Timeline } from '@/components/editor/Timeline';
import { ExportModal } from '@/components/editor/ExportModal';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Loader2, Video, AlertCircle } from 'lucide-react';

export function Editor() {
  const { selectedGameId, setSelectedGameId, availableClips, setAvailableClips } = useEditorStore();
  const { loadGameClips, isLoading, error } = useEditor();
  const { getAllGames, isLoading: isLoadingGames } = useStorage();

  const [games, setGames] = useState<Array<{ id: string; date: string; name: string }>>([]);
  const [isExportModalOpen, setIsExportModalOpen] = useState(false);

  // Load games on mount
  useEffect(() => {
    const loadGames = async () => {
      try {
        const allGames = await getAllGames();
        // Transform GameMetadata[] to simplified format
        const gameList = allGames.map((game) => ({
          id: game.game_id,
          date: new Date(game.game_start_time).toLocaleDateString(),
          name: `${game.champion} - ${game.game_mode}`,
        }));
        setGames(gameList);
      } catch (err) {
        console.error('Failed to load games:', err);
      }
    };

    loadGames();
  }, [getAllGames]);

  // Load clips when game is selected
  useEffect(() => {
    if (selectedGameId) {
      const loadClips = async () => {
        try {
          const clips = await loadGameClips(selectedGameId);
          setAvailableClips(clips);
        } catch (err) {
          console.error('Failed to load clips:', err);
        }
      };

      loadClips();
    }
  }, [selectedGameId, loadGameClips, setAvailableClips]);

  const handleGameSelect = (gameId: string) => {
    setSelectedGameId(gameId);
  };

  // Show game selection screen if no game selected
  if (!selectedGameId) {
    return (
      <div className="flex items-center justify-center h-full p-6">
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Video className="w-6 h-6" />
              Select a Game to Edit
            </CardTitle>
            <CardDescription>
              Choose a game from your recorded sessions to start editing
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {isLoadingGames ? (
              <div className="flex items-center justify-center p-8">
                <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
              </div>
            ) : games.length === 0 ? (
              <Alert>
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  No games available. Record a game session first from the Dashboard.
                </AlertDescription>
              </Alert>
            ) : (
              <>
                <div className="space-y-2">
                  <label className="text-sm font-medium">Select Game</label>
                  <Select onValueChange={handleGameSelect}>
                    <SelectTrigger>
                      <SelectValue placeholder="Choose a game" />
                    </SelectTrigger>
                    <SelectContent>
                      {games.map((game) => (
                        <SelectItem key={game.id} value={game.id}>
                          <div className="flex items-center justify-between w-full">
                            <span>{game.name}</span>
                            <Badge variant="outline" className="ml-2">{game.date}</Badge>
                          </div>
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div className="p-4 bg-muted rounded-lg text-sm text-muted-foreground">
                  <p>
                    Select a game to load its clips and start building your video timeline.
                  </p>
                </div>
              </>
            )}
          </CardContent>
        </Card>
      </div>
    );
  }

  // Show loading state while clips are loading
  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center space-y-4">
          <Loader2 className="w-12 h-12 animate-spin text-primary mx-auto" />
          <p className="text-muted-foreground">Loading clips...</p>
        </div>
      </div>
    );
  }

  // Show error state if clips failed to load
  if (error) {
    return (
      <div className="flex items-center justify-center h-full p-6">
        <Card className="w-full max-w-md">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-destructive">
              <AlertCircle className="w-6 h-6" />
              Error Loading Clips
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={() => setSelectedGameId(null)}
                className="flex-1"
              >
                Back to Game Selection
              </Button>
              <Button
                onClick={() => window.location.reload()}
                className="flex-1"
              >
                Retry
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  // Main editor interface
  return (
    <>
      {/* Header Bar */}
      <div className="border-b p-4 bg-background">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Video className="w-6 h-6" />
            <div>
              <h2 className="text-lg font-semibold">Video Editor</h2>
              <p className="text-sm text-muted-foreground">
                Game {selectedGameId} • {availableClips.length} clips available
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setSelectedGameId(null)}
            >
              Change Game
            </Button>
            <Button
              size="sm"
              onClick={() => setIsExportModalOpen(true)}
              disabled={availableClips.length === 0}
            >
              Export Video
            </Button>
          </div>
        </div>
      </div>

      {/* Editor Layout */}
      <div className="flex-1 overflow-hidden">
        <EditorLayout
          clipLibrary={<ClipLibrary />}
          videoPreview={<VideoPreview />}
          compositionSettings={
            <CompositionSettings
              onExport={() => setIsExportModalOpen(true)}
            />
          }
          timeline={<Timeline />}
        />
      </div>

      {/* Export Modal */}
      <ExportModal
        isOpen={isExportModalOpen}
        onClose={() => setIsExportModalOpen(false)}
      />
    </>
  );
}
