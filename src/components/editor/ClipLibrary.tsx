import { useEditorStore } from '@/stores/editorStore';
import { ClipCard } from './ClipCard';
import { Film } from 'lucide-react';

export function ClipLibrary() {
  const { availableClips } = useEditorStore();

  if (availableClips.length === 0) {
    return (
      <div className="h-full flex flex-col items-center justify-center p-6 text-center">
        <Film className="w-16 h-16 text-muted-foreground mb-4" />
        <h3 className="text-lg font-semibold mb-2">No Clips Available</h3>
        <p className="text-sm text-muted-foreground">
          Select a game from the Games page to load clips for editing.
        </p>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="p-4 border-b">
        <h3 className="font-semibold">Clip Library</h3>
        <p className="text-sm text-muted-foreground">
          {availableClips.length} clips available
        </p>
      </div>

      {/* Clip Grid */}
      <div className="flex-1 overflow-y-auto p-4">
        <div className="grid grid-cols-1 gap-4">
          {availableClips.map((clip) => (
            <ClipCard key={clip.clip_id} clip={clip} />
          ))}
        </div>
      </div>
    </div>
  );
}
