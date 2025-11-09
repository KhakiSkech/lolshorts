import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useAutoEditResults } from '@/hooks/useAutoEditResults';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Video,
  Clock,
  Trash2,
  Download,
  Play,
  Upload,
  CheckCircle2,
  XCircle,
  Loader2,
  AlertCircle,
  Film,
  Calendar,
} from 'lucide-react';
import { AutoEditResultMetadata } from '@/types/autoEdit';

export function ResultsViewer() {
  const { t } = useTranslation();
  const [results, setResults] = useState<AutoEditResultMetadata[]>([]);
  const { getAllResults, deleteResult, isLoading, error } = useAutoEditResults();

  // Load results on mount
  useEffect(() => {
    loadResults();
  }, []);

  const loadResults = async () => {
    try {
      const fetchedResults = await getAllResults();
      setResults(fetchedResults);
    } catch (err) {
      console.error('Failed to load results:', err);
    }
  };

  const handleDelete = async (resultId: string) => {
    if (!confirm(t('results.confirmDelete'))) {
      return;
    }

    try {
      await deleteResult(resultId, true);
      setResults(results.filter(r => r.result_id !== resultId));
    } catch (err) {
      console.error('Failed to delete result:', err);
    }
  };

  const handlePlay = (outputPath: string) => {
    const videoUrl = convertFileSrc(outputPath);
    window.open(videoUrl, '_blank');
  };

  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  const formatFileSize = (bytes: number): string => {
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  };

  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleDateString();
  };

  const getUploadStatusBadge = (result: AutoEditResultMetadata) => {
    if (!result.youtube_status) {
      return <Badge variant="secondary">{t('results.notUploaded')}</Badge>;
    }

    const { status } = result.youtube_status;

    switch (status) {
      case 'NotUploaded':
        return <Badge variant="secondary">{t('results.notUploaded')}</Badge>;
      case 'Queued':
        return <Badge variant="outline"><Loader2 className="w-3 h-3 mr-1 animate-spin" />{t('results.queued')}</Badge>;
      case 'Uploading':
        return <Badge variant="outline"><Upload className="w-3 h-3 mr-1" />{t('results.uploading')}</Badge>;
      case 'Processing':
        return <Badge variant="outline"><Loader2 className="w-3 h-3 mr-1 animate-spin" />{t('results.processing')}</Badge>;
      case 'Completed':
        return <Badge variant="default"><CheckCircle2 className="w-3 h-3 mr-1" />{t('results.completed')}</Badge>;
      case 'Failed':
        return <Badge variant="destructive"><XCircle className="w-3 h-3 mr-1" />{t('results.failed')}</Badge>;
      default:
        return <Badge variant="secondary">{t('results.unknown')}</Badge>;
    }
  };

  return (
    <div className="flex flex-col h-full p-6 space-y-6 overflow-auto">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">{t('results.title')}</h1>
          <p className="text-muted-foreground">{t('results.description')}</p>
        </div>
        <Button onClick={loadResults} disabled={isLoading}>
          <Loader2 className={`w-4 h-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
          {t('results.refresh')}
        </Button>
      </div>

      {/* Error Alert */}
      {error && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {/* Loading State */}
      {isLoading && results.length === 0 && (
        <div className="flex items-center justify-center h-64">
          <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
        </div>
      )}

      {/* Empty State */}
      {!isLoading && results.length === 0 && (
        <Card className="flex flex-col items-center justify-center p-12">
          <Video className="w-16 h-16 text-muted-foreground mb-4" />
          <CardTitle className="mb-2">{t('results.empty.title')}</CardTitle>
          <CardDescription>{t('results.empty.description')}</CardDescription>
        </Card>
      )}

      {/* Results Grid */}
      {results.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {results.map((result) => (
            <Card key={result.result_id} className="flex flex-col">
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <CardTitle className="text-lg flex items-center gap-2">
                      <Film className="w-5 h-5" />
                      {result.target_duration}s {t('results.short')}
                    </CardTitle>
                    <CardDescription className="flex items-center gap-1 mt-1">
                      <Calendar className="w-3 h-3" />
                      {formatDate(result.created_at)}
                    </CardDescription>
                  </div>
                  {getUploadStatusBadge(result)}
                </div>
              </CardHeader>

              <CardContent className="flex-1 flex flex-col justify-between">
                {/* Video Info */}
                <div className="space-y-2 mb-4">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">{t('results.duration')}:</span>
                    <span className="font-medium flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {formatDuration(result.duration)}
                    </span>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">{t('results.clips')}:</span>
                    <span className="font-medium">{result.clip_count}</span>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">{t('results.fileSize')}:</span>
                    <span className="font-medium">{formatFileSize(result.file_size_bytes)}</span>
                  </div>
                  {result.canvas_template_name && (
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-muted-foreground">{t('results.template')}:</span>
                      <span className="font-medium">{result.canvas_template_name}</span>
                    </div>
                  )}
                  {result.has_background_music && (
                    <Badge variant="outline" className="w-fit">
                      {t('results.withMusic')}
                    </Badge>
                  )}
                </div>

                {/* Action Buttons */}
                <div className="flex gap-2">
                  <Button
                    onClick={() => handlePlay(result.output_path)}
                    className="flex-1"
                    variant="default"
                  >
                    <Play className="w-4 h-4 mr-2" />
                    {t('results.play')}
                  </Button>
                  <Button
                    onClick={() => handleDelete(result.result_id)}
                    variant="destructive"
                    size="icon"
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
