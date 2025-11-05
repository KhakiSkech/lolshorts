import { useEditorStore } from '@/stores/editorStore';
import type { AspectRatio, TransitionType } from '@/stores/editorStore';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Slider } from '@/components/ui/slider';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Download, Settings2, Film } from 'lucide-react';

interface CompositionSettingsProps {
  onExport?: () => void;
}

export function CompositionSettings({ onExport }: CompositionSettingsProps) {
  const {
    compositionSettings,
    setAspectRatio,
    setTransitionType,
    setTransitionDuration,
    timelineClips,
    totalDuration,
  } = useEditorStore();

  const aspectRatioOptions: Array<{ value: AspectRatio; label: string; description: string }> = [
    { value: '9:16', label: '9:16', description: 'Vertical (TikTok, Reels)' },
    { value: '16:9', label: '16:9', description: 'Horizontal (YouTube)' },
    { value: '1:1', label: '1:1', description: 'Square (Instagram)' },
  ];

  const transitionOptions: Array<{ value: TransitionType; label: string }> = [
    { value: 'none', label: 'None' },
    { value: 'fade', label: 'Fade' },
    { value: 'slide', label: 'Slide' },
  ];

  const handleExport = () => {
    if (timelineClips.length === 0 || !onExport) {
      return;
    }
    onExport();
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const getAspectRatioIcon = (ratio: AspectRatio) => {
    switch (ratio) {
      case '9:16':
        return '📱';
      case '16:9':
        return '🖥️';
      case '1:1':
        return '⬜';
    }
  };

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="p-4 border-b">
        <div className="flex items-center gap-2">
          <Settings2 className="w-5 h-5" />
          <h3 className="font-semibold">Composition Settings</h3>
        </div>
      </div>

      {/* Settings Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6">
        {/* Aspect Ratio Section */}
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Aspect Ratio</CardTitle>
          </CardHeader>
          <CardContent>
            <RadioGroup
              value={compositionSettings.aspectRatio}
              onValueChange={(value) => setAspectRatio(value as AspectRatio)}
            >
              {aspectRatioOptions.map((option) => (
                <div key={option.value} className="flex items-center space-x-3 space-y-0">
                  <RadioGroupItem value={option.value} id={option.value} />
                  <Label htmlFor={option.value} className="font-normal cursor-pointer flex-1">
                    <div className="flex items-center justify-between">
                      <div>
                        <span className="font-medium">{option.label}</span>
                        <p className="text-xs text-muted-foreground">{option.description}</p>
                      </div>
                      <span className="text-xl">{getAspectRatioIcon(option.value)}</span>
                    </div>
                  </Label>
                </div>
              ))}
            </RadioGroup>
          </CardContent>
        </Card>

        {/* Transition Section */}
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Transitions</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Transition Type */}
            <div className="space-y-2">
              <Label htmlFor="transition-type">Type</Label>
              <Select
                value={compositionSettings.transitionType}
                onValueChange={(value) => setTransitionType(value as TransitionType)}
              >
                <SelectTrigger id="transition-type">
                  <SelectValue placeholder="Select transition" />
                </SelectTrigger>
                <SelectContent>
                  {transitionOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* Transition Duration */}
            {compositionSettings.transitionType !== 'none' && (
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Label htmlFor="transition-duration">Duration</Label>
                  <span className="text-sm text-muted-foreground">
                    {compositionSettings.transitionDuration.toFixed(1)}s
                  </span>
                </div>
                <Slider
                  id="transition-duration"
                  min={0.1}
                  max={2.0}
                  step={0.1}
                  value={[compositionSettings.transitionDuration]}
                  onValueChange={(value) => setTransitionDuration(value[0])}
                  className="w-full"
                />
              </div>
            )}
          </CardContent>
        </Card>

        {/* Summary Section */}
        <Card>
          <CardHeader>
            <CardTitle className="text-sm flex items-center gap-2">
              <Film className="w-4 h-4" />
              Composition Summary
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Total Clips</span>
              <Badge variant="secondary">{timelineClips.length}</Badge>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Total Duration</span>
              <Badge variant="outline">{formatDuration(totalDuration)}</Badge>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Aspect Ratio</span>
              <Badge variant="outline">{compositionSettings.aspectRatio}</Badge>
            </div>
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted-foreground">Transitions</span>
              <Badge variant="outline">
                {compositionSettings.transitionType === 'none'
                  ? 'None'
                  : `${compositionSettings.transitionType} (${compositionSettings.transitionDuration}s)`
                }
              </Badge>
            </div>
          </CardContent>
        </Card>
      </div>

      <Separator />

      {/* Export Button */}
      <div className="p-4">
        <Button
          size="lg"
          className="w-full"
          onClick={handleExport}
          disabled={timelineClips.length === 0}
        >
          <Download className="w-4 h-4 mr-2" />
          Export Video
        </Button>
        {timelineClips.length === 0 && (
          <p className="text-xs text-muted-foreground text-center mt-2">
            Add clips to timeline to export
          </p>
        )}
      </div>
    </div>
  );
}
