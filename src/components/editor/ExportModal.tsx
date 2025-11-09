import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useEditorStore } from '@/stores/editorStore';
import { useEditor } from '@/hooks/useEditor';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { CheckCircle2, XCircle, Loader2, FolderOpen, Film, Settings, Clock } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { open as openPath } from '@tauri-apps/plugin-shell';
import { join, dirname } from '@tauri-apps/api/path';

interface ExportModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ExportModal({ isOpen, onClose }: ExportModalProps) {
  const { t } = useTranslation();
  const {
    timelineClips,
    totalDuration,
    compositionSettings,
    exportProgress,
    exportStatus,
    exportError,
    exportOutputPath,
    setExportStatus,
    setExportProgress,
    setExportError,
    setExportOutputPath,
  } = useEditorStore();

  const { composeShorts, isLoading } = useEditor();
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  // Reset modal state when opened
  useEffect(() => {
    if (isOpen) {
      setExportStatus('idle');
      setExportProgress(0);
      setExportError(null);
      setSelectedPath(null);
    }
  }, [isOpen, setExportStatus, setExportProgress, setExportError]);

  const handleSelectPath = async () => {
    try {
      const selected = await open({
        title: 'Save Exported Video',
        filters: [{
          name: 'Video Files',
          extensions: ['mp4'],
        }],
        defaultPath: await join(await dirname(''), 'lolshorts_export.mp4'),
      });

      if (selected && typeof selected === 'string') {
        setSelectedPath(selected);
      }
    } catch (error) {
      console.error('Failed to select path:', error);
      setExportError('Failed to select save location');
    }
  };

  const handleExport = async () => {
    if (!selectedPath || timelineClips.length === 0) {
      return;
    }

    try {
      const outputPath = await composeShorts(
        timelineClips,
        compositionSettings,
        selectedPath
      );
      setExportOutputPath(outputPath);
    } catch (error) {
      console.error('Export failed:', error);
      setExportError(error as string);
      setExportStatus('error');
    }
  };

  const handleOpenFolder = async () => {
    if (exportOutputPath) {
      try {
        const dir = await dirname(exportOutputPath);
        await openPath(dir);
      } catch (error) {
        console.error('Failed to open folder:', error);
      }
    }
  };

  const handleClose = () => {
    if (exportStatus === 'exporting') {
      if (!confirm(t('confirmations.cancelExport'))) {
        return;
      }
    }
    onClose();
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const getStatusIcon = () => {
    switch (exportStatus) {
      case 'exporting':
        return <Loader2 className="w-5 h-5 animate-spin text-primary" />;
      case 'complete':
        return <CheckCircle2 className="w-5 h-5 text-green-500" />;
      case 'error':
        return <XCircle className="w-5 h-5 text-destructive" />;
      default:
        return <Film className="w-5 h-5 text-muted-foreground" />;
    }
  };

  const getStatusText = () => {
    switch (exportStatus) {
      case 'exporting':
        return 'Exporting video...';
      case 'complete':
        return 'Export complete!';
      case 'error':
        return 'Export failed';
      default:
        return 'Ready to export';
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            {getStatusIcon()}
            Export Video
          </DialogTitle>
          <DialogDescription>
            {getStatusText()}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Export Summary */}
          {exportStatus === 'idle' && (
            <>
              <div className="space-y-3 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground flex items-center gap-2">
                    <Film className="w-4 h-4" />
                    Total Clips
                  </span>
                  <Badge variant="secondary">{timelineClips.length}</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground flex items-center gap-2">
                    <Clock className="w-4 h-4" />
                    Duration
                  </span>
                  <Badge variant="outline">{formatDuration(totalDuration)}</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground flex items-center gap-2">
                    <Settings className="w-4 h-4" />
                    Aspect Ratio
                  </span>
                  <Badge variant="outline">{compositionSettings.aspectRatio}</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground flex items-center gap-2">
                    <Settings className="w-4 h-4" />
                    Transitions
                  </span>
                  <Badge variant="outline">
                    {compositionSettings.transitionType === 'none'
                      ? 'None'
                      : `${compositionSettings.transitionType} (${compositionSettings.transitionDuration}s)`
                    }
                  </Badge>
                </div>
              </div>

              <Separator />

              {/* File Path Selection */}
              <div className="space-y-2">
                <label className="text-sm font-medium">Save Location</label>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleSelectPath}
                    className="flex-1 justify-start"
                  >
                    <FolderOpen className="w-4 h-4 mr-2" />
                    {selectedPath ? 'Change Location' : 'Select Location'}
                  </Button>
                </div>
                {selectedPath && (
                  <p className="text-xs text-muted-foreground truncate" title={selectedPath}>
                    {selectedPath}
                  </p>
                )}
              </div>
            </>
          )}

          {/* Export Progress */}
          {exportStatus === 'exporting' && (
            <div className="space-y-3">
              <Progress value={exportProgress} className="w-full" />
              <p className="text-sm text-center text-muted-foreground">
                {Math.round(exportProgress)}% complete
              </p>
              <Alert>
                <Loader2 className="h-4 w-4 animate-spin" />
                <AlertDescription>
                  Processing video clips and applying transitions...
                  This may take a few minutes.
                </AlertDescription>
              </Alert>
            </div>
          )}

          {/* Export Complete */}
          {exportStatus === 'complete' && exportOutputPath && (
            <div className="space-y-3">
              <Alert className="border-green-500/50 bg-green-500/10">
                <CheckCircle2 className="h-4 w-4 text-green-500" />
                <AlertDescription className="text-green-500">
                  Video exported successfully!
                </AlertDescription>
              </Alert>
              <div className="text-sm space-y-2">
                <p className="text-muted-foreground">Saved to:</p>
                <p className="font-mono text-xs bg-muted p-2 rounded break-all">
                  {exportOutputPath}
                </p>
              </div>
              <Button
                variant="outline"
                size="sm"
                onClick={handleOpenFolder}
                className="w-full"
              >
                <FolderOpen className="w-4 h-4 mr-2" />
                Open Containing Folder
              </Button>
            </div>
          )}

          {/* Export Error */}
          {exportStatus === 'error' && exportError && (
            <Alert variant="destructive">
              <XCircle className="h-4 w-4" />
              <AlertDescription>
                {exportError}
              </AlertDescription>
            </Alert>
          )}
        </div>

        <DialogFooter>
          {exportStatus === 'idle' && (
            <>
              <Button variant="outline" onClick={handleClose}>
                Cancel
              </Button>
              <Button
                onClick={handleExport}
                disabled={!selectedPath || timelineClips.length === 0 || isLoading}
              >
                {isLoading ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    Starting...
                  </>
                ) : (
                  <>
                    <Film className="w-4 h-4 mr-2" />
                    Export
                  </>
                )}
              </Button>
            </>
          )}

          {exportStatus === 'exporting' && (
            <Button variant="outline" onClick={handleClose}>
              Cancel Export
            </Button>
          )}

          {(exportStatus === 'complete' || exportStatus === 'error') && (
            <Button onClick={handleClose}>
              Close
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
