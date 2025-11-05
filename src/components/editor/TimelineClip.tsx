import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { TimelineClip as TimelineClipType } from '@/stores/editorStore';
import { Button } from '@/components/ui/button';
import { X, GripVertical } from 'lucide-react';
import { useEditorStore } from '@/stores/editorStore';
import { convertFileSrc } from '@tauri-apps/api/core';

interface TimelineClipProps {
  clip: TimelineClipType;
  zoom: number;
}

const PIXELS_PER_SECOND = 50;
const MIN_CLIP_WIDTH = 100;

export function TimelineClip({ clip, zoom }: TimelineClipProps) {
  const { removeFromTimeline, setSelectedClipId, selectedClipId } = useEditorStore();

  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: clip.clip_id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  // Calculate clip width based on duration and zoom
  const clipWidth = Math.max(MIN_CLIP_WIDTH, clip.duration * PIXELS_PER_SECOND * zoom);

  const handleRemove = () => {
    removeFromTimeline(clip.clip_id);
  };

  const handleClick = () => {
    setSelectedClipId(clip.clip_id);
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const isSelected = selectedClipId === clip.clip_id;

  // Convert thumbnail path
  const thumbnailSrc = clip.thumbnail_path ? convertFileSrc(clip.thumbnail_path) : undefined;

  return (
    <div
      ref={setNodeRef}
      style={{ ...style, width: `${clipWidth}px` }}
      className={`
        relative h-32 bg-card border-2 rounded-lg overflow-hidden cursor-pointer
        transition-all hover:border-primary
        ${isSelected ? 'border-primary ring-2 ring-primary' : 'border-border'}
        ${isDragging ? 'shadow-lg' : ''}
      `}
      onClick={handleClick}
    >
      {/* Drag Handle */}
      <div
        {...attributes}
        {...listeners}
        className="absolute top-1 left-1 z-10 cursor-grab active:cursor-grabbing bg-background/80 rounded p-1"
      >
        <GripVertical className="w-4 h-4 text-muted-foreground" />
      </div>

      {/* Remove Button */}
      <Button
        size="sm"
        variant="destructive"
        className="absolute top-1 right-1 z-10 h-6 w-6 p-0"
        onClick={(e) => {
          e.stopPropagation();
          handleRemove();
        }}
      >
        <X className="w-3 h-3" />
      </Button>

      {/* Thumbnail */}
      <div className="relative w-full h-full">
        {thumbnailSrc ? (
          <img
            src={thumbnailSrc}
            alt="Clip thumbnail"
            className="w-full h-full object-cover"
          />
        ) : (
          <div className="w-full h-full bg-muted flex items-center justify-center">
            <span className="text-xs text-muted-foreground">No thumbnail</span>
          </div>
        )}

        {/* Clip Info Overlay */}
        <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-2">
          <div className="flex items-center justify-between text-white text-xs">
            <span className="font-medium">#{clip.order + 1}</span>
            <span>{formatDuration(clip.duration)}</span>
          </div>
        </div>
      </div>
    </div>
  );
}
