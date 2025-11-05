import { ClipMetadata } from '@/hooks/useStorage';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Play, Plus } from 'lucide-react';
import { useEditorStore } from '@/stores/editorStore';
import { convertFileSrc } from '@tauri-apps/api/core';

interface ClipCardProps {
  clip: ClipMetadata;
}

export function ClipCard({ clip }: ClipCardProps) {
  const { addToTimeline, setSelectedClipId } = useEditorStore();

  const handleAddToTimeline = () => {
    addToTimeline(clip);
  };

  const handlePreview = () => {
    setSelectedClipId(clip.clip_id);
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  // Get priority variant color
  const getPriorityVariant = (priority: number) => {
    if (priority >= 5) return 'default';  // Gold for pentakills
    if (priority >= 3) return 'destructive';  // Red for high priority
    return 'secondary';  // Gray for normal
  };

  // Convert file path for Tauri
  const thumbnailSrc = clip.thumbnail_path ? convertFileSrc(clip.thumbnail_path) : undefined;

  return (
    <Card className="overflow-hidden hover:border-primary transition-colors cursor-pointer group">
      {/* Thumbnail */}
      <div className="relative aspect-video bg-muted">
        {thumbnailSrc ? (
          <img
            src={thumbnailSrc}
            alt="Clip thumbnail"
            className="w-full h-full object-cover"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center">
            <Play className="w-12 h-12 text-muted-foreground" />
          </div>
        )}

        {/* Duration overlay */}
        <div className="absolute bottom-2 right-2 bg-black/80 text-white text-xs px-2 py-1 rounded">
          {formatDuration(clip.duration)}
        </div>

        {/* Priority badge */}
        <div className="absolute top-2 right-2">
          <Badge variant={getPriorityVariant(clip.event_id)}>
            Priority {clip.event_id}
          </Badge>
        </div>
      </div>

      {/* Clip Info */}
      <div className="p-3 space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium truncate">
            Clip #{clip.clip_id}
          </span>
        </div>

        {/* Actions */}
        <div className="flex gap-2">
          <Button
            size="sm"
            variant="outline"
            className="flex-1"
            onClick={handlePreview}
          >
            <Play className="w-3 h-3 mr-1" />
            Preview
          </Button>
          <Button
            size="sm"
            variant="default"
            className="flex-1"
            onClick={handleAddToTimeline}
          >
            <Plus className="w-3 h-3 mr-1" />
            Add
          </Button>
        </div>
      </div>
    </Card>
  );
}
