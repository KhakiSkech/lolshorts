import { DndContext, closestCenter, DragEndEvent, PointerSensor, useSensor, useSensors } from '@dnd-kit/core';
import { SortableContext, horizontalListSortingStrategy } from '@dnd-kit/sortable';
import { useEditorStore } from '@/stores/editorStore';
import { TimelineClip } from './TimelineClip';
import { TimelineControls } from './TimelineControls';
import { Film } from 'lucide-react';

export function Timeline() {
  const { timelineClips, reorderTimeline, zoom } = useEditorStore();

  // Configure sensors for drag and drop
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,  // 8px of movement required to start drag
      },
    })
  );

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (over && active.id !== over.id) {
      const oldIndex = timelineClips.findIndex(c => c.clip_id === active.id);
      const newIndex = timelineClips.findIndex(c => c.clip_id === over.id);

      if (oldIndex !== -1 && newIndex !== -1) {
        reorderTimeline(oldIndex, newIndex);
      }
    }
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header with Controls */}
      <div className="border-b p-3">
        <TimelineControls />
      </div>

      {/* Timeline Track */}
      <div className="flex-1 overflow-x-auto overflow-y-hidden">
        {timelineClips.length === 0 ? (
          <div className="h-full flex flex-col items-center justify-center">
            <Film className="w-12 h-12 text-muted-foreground mb-2" />
            <p className="text-sm text-muted-foreground">
              Add clips from the library to build your timeline
            </p>
            <p className="text-xs text-muted-foreground mt-1">
              Drag clips to reorder them
            </p>
          </div>
        ) : (
          <DndContext
            sensors={sensors}
            collisionDetection={closestCenter}
            onDragEnd={handleDragEnd}
          >
            <SortableContext
              items={timelineClips.map(c => c.clip_id)}
              strategy={horizontalListSortingStrategy}
            >
              <div className="flex gap-2 p-4 h-full items-center">
                {timelineClips.map((clip) => (
                  <TimelineClip key={clip.clip_id} clip={clip} zoom={zoom} />
                ))}
              </div>
            </SortableContext>
          </DndContext>
        )}
      </div>
    </div>
  );
}
