import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useEditorStore } from '@/stores/editorStore';
import { useEditor } from '@/hooks/useEditor';
import { useToast } from '@/components/ui/use-toast';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { useConfirmDialog } from '@/components/ui/confirm-dialog';
import { CheckCircle2, XCircle, Loader2, FolderOpen, Film, Settings, Clock, LayoutTemplate } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { open as openPath } from '@tauri-apps/plugin-shell';
import { join, dirname } from '@tauri-apps/api/path';
import { getErrorMessage, formatDuration } from '@/lib/utils';

interface ExportModalProps {
  isOpen: boolean;
  onClose: () => void;
}

type ExportFormat = 'shorts' | 'montage';

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

  const { composeShorts, createMontage, isLoading } = useEditor();
  const { confirm, ConfirmDialog } = useConfirmDialog();
  const { toast } = useToast();
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [exportFormat, setExportFormat] = useState<ExportFormat>('shorts');

  // Reset modal state when opened
  useEffect(() => {
    if (isOpen) {
      setExportStatus('idle');
      setExportProgress(0);
      setExportError(null);
      setSelectedPath(null);
      setExportFormat('shorts');
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOpen]);

  const handleSelectPath = async () => {
    try {
      const defaultName = exportFormat === 'shorts' ? 'lolshorts_export.mp4' : 'lol_montage.mp4';
      const selected = await open({
        title: t('editor.export.saveTitle'),
        filters: [{
          name: t('editor.export.videoFiles'),
          extensions: ['mp4'],
        }],
        defaultPath: await join(await dirname(''), defaultName),
      });

      if (selected && typeof selected === 'string') {
        setSelectedPath(selected);
      }
    } catch (error) {
      console.error('Failed to select path:', error);
      setExportError(t('editor.export.selectLocationFailed'));
    }
  };

  const handleExport = async () => {
    if (!selectedPath || timelineClips.length === 0) {
      return;
    }

    try {
      let outputPath = '';
      if (exportFormat === 'shorts') {
        outputPath = await composeShorts(
          timelineClips,
          compositionSettings,
          selectedPath
        );
      } else {
        outputPath = await createMontage(
          timelineClips,
          selectedPath
        );
      }
      setExportOutputPath(outputPath);
      toast({
        title: t('toast.exportComplete'),
        description: t('toast.exportCompleteDesc'),
      });
    } catch (error) {
      console.error('Export failed:', error);
      setExportError(getErrorMessage(error));
      setExportStatus('error');
      toast({
        title: t('toast.error'),
        description: getErrorMessage(error),
        variant: 'destructive',
      });
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

  const handleClose = async () => {
    if (exportStatus === 'exporting') {
      const confirmed = await confirm({
        title: t('confirmations.cancelExportTitle'),
        description: t('confirmations.cancelExportDescription'),
        confirmText: t('common.cancel'),
        cancelText: t('common.continue'),
        variant: 'warning',
      });

      if (!confirmed) {
        return;
      }
    }
    onClose();
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
        return t('editor.export.exporting');
      case 'complete':
        return t('editor.export.complete');
      case 'error':
        return t('editor.export.failed');
      default:
        return t('editor.export.ready');
    }
  };

  return (
    <Dialog open={isOpen} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            {getStatusIcon()}
            {t('editor.export.exportVideo')}
          </DialogTitle>
          <DialogDescription>
            {getStatusText()}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Export Summary */}
          {exportStatus === 'idle' && (
            <>
              {/* Format Selection */}
              <div className="space-y-3">
                <Label className="text-sm font-medium">{t('editor.export.exportFormat')}</Label>
                <RadioGroup
                  defaultValue="shorts"
                  value={exportFormat}
                  onValueChange={(val) => setExportFormat(val as ExportFormat)}
                  className="grid grid-cols-2 gap-4"
                >
                  <div>
                    <RadioGroupItem value="shorts" id="shorts" className="peer sr-only" />
                    <Label
                      htmlFor="shorts"
                      className="flex flex-col items-center justify-between rounded-md border-2 border-muted bg-popover p-4 hover:bg-accent hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary cursor-pointer"
                    >
                      <LayoutTemplate className="mb-2 h-6 w-6" />
                      <span className="font-semibold">{t('editor.export.shorts')}</span>
                      <span className="text-xs text-muted-foreground mt-1">{t('editor.export.mobileOptimized')}</span>
                    </Label>
                  </div>
                  <div>
                    <RadioGroupItem value="montage" id="montage" className="peer sr-only" />
                    <Label
                      htmlFor="montage"
                      className="flex flex-col items-center justify-between rounded-md border-2 border-muted bg-popover p-4 hover:bg-accent hover:text-accent-foreground peer-data-[state=checked]:border-primary [&:has([data-state=checked])]:border-primary cursor-pointer"
                    >
                      <Film className="mb-2 h-6 w-6" />
                      <span className="font-semibold">{t('editor.export.montage')}</span>
                      <span className="text-xs text-muted-foreground mt-1">{t('editor.export.fullScreenHD')}</span>
                    </Label>
                  </div>
                </RadioGroup>
              </div>

              <Separator />

              <div className="space-y-3 text-sm">
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground flex items-center gap-2">
                    <Film className="w-4 h-4" />
                    {t('editor.export.totalClips')}
                  </span>
                  <Badge variant="secondary">{timelineClips.length}</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-muted-foreground flex items-center gap-2">
                    <Clock className="w-4 h-4" />
                    {t('editor.export.duration')}
                  </span>
                  <Badge variant="outline">{formatDuration(totalDuration)}</Badge>
                </div>
                {exportFormat === 'shorts' && (
                  <div className="flex items-center justify-between">
                    <span className="text-muted-foreground flex items-center gap-2">
                      <Settings className="w-4 h-4" />
                      {t('editor.export.transitions')}
                    </span>
                    <Badge variant="outline">
                      {compositionSettings.transitionType === 'none'
                        ? t('editor.export.none')
                        : `${compositionSettings.transitionType} (${compositionSettings.transitionDuration}s)`
                      }
                    </Badge>
                  </div>
                )}
              </div>

              <Separator />

              {/* File Path Selection */}
              <div className="space-y-2">
                <span className="text-sm font-medium" id="save-location-label">{t('editor.export.saveLocation')}</span>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleSelectPath}
                    className="flex-1 justify-start"
                    aria-labelledby="save-location-label"
                  >
                    <FolderOpen className="w-4 h-4 mr-2" />
                    {selectedPath ? t('editor.export.changeLocation') : t('editor.export.selectLocation')}
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
                  {exportFormat === 'shorts'
                    ? t('editor.export.processingClips')
                    : t('editor.export.mergingClips')}
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
                  {t('editor.export.successMessage')}
                </AlertDescription>
              </Alert>
              <div className="text-sm space-y-2">
                <p className="text-muted-foreground">{t('editor.export.savedTo')}</p>
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
                {t('editor.export.openFolder')}
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
                {t('editor.export.cancel')}
              </Button>
              <Button
                onClick={handleExport}
                disabled={!selectedPath || timelineClips.length === 0 || isLoading}
              >
                {isLoading ? (
                  <>
                    <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                    {t('editor.export.starting')}
                  </>
                ) : (
                  <>
                    <Film className="w-4 h-4 mr-2" />
                    {t('editor.export.exportButton')}
                  </>
                )}
              </Button>
            </>
          )}

          {exportStatus === 'exporting' && (
            <Button variant="outline" onClick={handleClose}>
              {t('editor.export.cancelExport')}
            </Button>
          )}

          {(exportStatus === 'complete' || exportStatus === 'error') && (
            <Button onClick={handleClose}>
              {t('editor.export.close')}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
      <ConfirmDialog />
    </Dialog>
  );
}
