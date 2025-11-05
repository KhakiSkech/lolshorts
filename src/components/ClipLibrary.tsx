/**
 * Clip Library Component
 *
 * Displays, filters, and manages recorded gameplay clips
 */

import { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Play, Trash2, Edit, Download, Search, Filter } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { toast } from '@/components/ui/use-toast';

interface Clip {
  id: number;
  game_id: number;
  file_path: string;
  event_type: string;
  event_time: number;
  priority: number;
  duration_secs: number;
  created_at: string;
}

interface Game {
  game_id: number;
  game_start_time: string;
  game_end_time: string | null;
  champion_name: string | null;
  game_mode: string | null;
}

export function ClipLibrary() {
  const [clips, setClips] = useState<Clip[]>([]);
  const [games, setGames] = useState<Game[]>([]);
  const [selectedGame, setSelectedGame] = useState<number | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [filterPriority, setFilterPriority] = useState<string>('all');
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    loadGames();
  }, []);

  useEffect(() => {
    if (selectedGame !== null) {
      loadClipsForGame(selectedGame);
    }
  }, [selectedGame]);

  const loadGames = async () => {
    setIsLoading(true);
    try {
      const gameList = await invoke<Game[]>('list_games');
      setGames(gameList);

      if (gameList.length > 0) {
        setSelectedGame(gameList[0].game_id);
      }
    } catch (error) {
      console.error('Failed to load games:', error);
      toast({
        title: 'Failed to Load Games',
        description: String(error),
        variant: 'destructive',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const loadClipsForGame = async (gameId: number) => {
    setIsLoading(true);
    try {
      // Call actual backend to get clips
      const clipList = await invoke<Clip[]>('list_clips', {
        gameId: gameId.toString()
      });
      setClips(clipList);
    } catch (error) {
      console.error('Failed to load clips:', error);
      toast({
        title: 'Failed to Load Clips',
        description: String(error),
        variant: 'destructive',
      });
      // Set empty array on error
      setClips([]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDeleteClip = async (clipId: number) => {
    try {
      const clip = clips.find(c => c.id === clipId);
      if (!clip) {
        throw new Error('Clip not found');
      }

      // Call actual backend to delete clip
      await invoke('delete_clip', {
        gameId: clip.game_id.toString(),
        filePath: clip.file_path
      });

      // Remove from local state
      setClips(clips.filter(c => c.id !== clipId));

      toast({
        title: 'Clip Deleted',
        description: 'Clip removed successfully',
      });
    } catch (error) {
      toast({
        title: 'Failed to Delete',
        description: String(error),
        variant: 'destructive',
      });
    }
  };

  const handlePlayClip = (clip: Clip) => {
    // TODO: Implement video player
    toast({
      title: 'Playing Clip',
      description: `${clip.event_type} at ${formatTime(clip.event_time)}`,
    });
  };

  const getPriorityColor = (priority: number): string => {
    if (priority >= 5) return 'bg-purple-500';
    if (priority >= 4) return 'bg-red-500';
    if (priority >= 3) return 'bg-orange-500';
    return 'bg-blue-500';
  };

  const getPriorityLabel = (priority: number): string => {
    if (priority >= 5) return 'Legendary';
    if (priority >= 4) return 'Epic';
    if (priority >= 3) return 'Rare';
    return 'Common';
  };

  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatDate = (isoString: string): string => {
    const date = new Date(isoString);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  };

  const filteredClips = clips.filter(clip => {
    // Priority filter
    if (filterPriority !== 'all' && clip.priority.toString() !== filterPriority) {
      return false;
    }

    // Search filter
    if (searchQuery && !clip.event_type.toLowerCase().includes(searchQuery.toLowerCase())) {
      return false;
    }

    return true;
  });

  return (
    <div className="space-y-4">
      {/* Header with Game Selector */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Clip Library</CardTitle>
              <CardDescription>
                Browse and manage your recorded gameplay highlights
              </CardDescription>
            </div>
            <Select
              value={selectedGame?.toString() || ''}
              onValueChange={(value) => setSelectedGame(parseInt(value))}
            >
              <SelectTrigger className="w-[250px]">
                <SelectValue placeholder="Select a game" />
              </SelectTrigger>
              <SelectContent>
                {games.map((game) => (
                  <SelectItem key={game.game_id} value={game.game_id.toString()}>
                    {game.champion_name || 'Unknown Champion'} - {new Date(game.game_start_time).toLocaleDateString()}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
      </Card>

      {/* Filters */}
      <Card>
        <CardContent className="pt-6">
          <div className="flex gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search clips..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-9"
              />
            </div>
            <div className="flex items-center gap-2">
              <Filter className="h-4 w-4 text-muted-foreground" />
              <Select value={filterPriority} onValueChange={setFilterPriority}>
                <SelectTrigger className="w-[150px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Priorities</SelectItem>
                  <SelectItem value="5">Legendary (5)</SelectItem>
                  <SelectItem value="4">Epic (4)</SelectItem>
                  <SelectItem value="3">Rare (3)</SelectItem>
                  <SelectItem value="2">Common (2)</SelectItem>
                  <SelectItem value="1">Basic (1)</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Clips Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {isLoading ? (
          <Card className="col-span-full">
            <CardContent className="p-12 text-center">
              <div className="text-muted-foreground">Loading clips...</div>
            </CardContent>
          </Card>
        ) : filteredClips.length === 0 ? (
          <Card className="col-span-full">
            <CardContent className="p-12 text-center">
              <div className="text-muted-foreground">
                {clips.length === 0
                  ? 'No clips recorded yet. Start auto-capture to record highlights!'
                  : 'No clips match your filters'}
              </div>
            </CardContent>
          </Card>
        ) : (
          filteredClips.map((clip) => (
            <Card key={clip.id} className="overflow-hidden">
              <div className="aspect-video bg-muted relative">
                {/* TODO: Add video thumbnail */}
                <div className="absolute inset-0 flex items-center justify-center">
                  <Play className="h-12 w-12 text-muted-foreground" />
                </div>

                {/* Priority Badge */}
                <div className="absolute top-2 right-2">
                  <Badge className={getPriorityColor(clip.priority)}>
                    {getPriorityLabel(clip.priority)}
                  </Badge>
                </div>

                {/* Duration */}
                <div className="absolute bottom-2 right-2 bg-black/70 px-2 py-1 rounded text-xs text-white">
                  {clip.duration_secs}s
                </div>
              </div>

              <CardContent className="p-4">
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <h3 className="font-semibold">{clip.event_type}</h3>
                    <span className="text-xs text-muted-foreground">
                      @{formatTime(clip.event_time)}
                    </span>
                  </div>

                  <div className="text-xs text-muted-foreground">
                    {formatDate(clip.created_at)}
                  </div>

                  <div className="flex gap-2 pt-2">
                    <Button
                      variant="default"
                      size="sm"
                      className="flex-1"
                      onClick={() => handlePlayClip(clip)}
                    >
                      <Play className="mr-1 h-3 w-3" />
                      Play
                    </Button>
                    <Button variant="outline" size="sm">
                      <Edit className="h-3 w-3" />
                    </Button>
                    <Button variant="outline" size="sm">
                      <Download className="h-3 w-3" />
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleDeleteClip(clip.id)}
                    >
                      <Trash2 className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))
        )}
      </div>
    </div>
  );
}
