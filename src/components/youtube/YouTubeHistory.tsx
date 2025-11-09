import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useYouTube } from '@/hooks/useYouTube';
import { Card, CardHeader, CardTitle, CardContent, CardDescription } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { History, ExternalLink, Eye, ThumbsUp, MessageCircle } from 'lucide-react';
import { UploadHistoryEntry } from '@/types/youtube';
import { open } from '@tauri-apps/plugin-shell';

export function YouTubeHistory() {
  const { t } = useTranslation();
  const { authStatus, isLoading, error, getUploadHistory } = useYouTube();
  const [history, setHistory] = useState<UploadHistoryEntry[]>([]);

  useEffect(() => {
    if (authStatus.authenticated) {
      loadHistory();
    }
  }, [authStatus.authenticated]);

  const loadHistory = async () => {
    try {
      const data = await getUploadHistory();
      setHistory(data);
    } catch (err) {
      console.error('Failed to load history:', err);
    }
  };

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  };

  const formatNumber = (num: number): string => {
    if (num >= 1_000_000) {
      return (num / 1_000_000).toFixed(1) + 'M';
    } else if (num >= 1_000) {
      return (num / 1_000).toFixed(1) + 'K';
    }
    return num.toString();
  };

  const handleOpenVideo = (url: string) => {
    open(url);
  };

  if (!authStatus.authenticated) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>{t('youtube.history.uploadHistory')}</CardTitle>
          <CardDescription>
            {t('youtube.history.connectAccountFirst')}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Alert>
            <AlertDescription>
              {t('youtube.history.needToConnect')}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <History className="h-6 w-6" />
            <div>
              <CardTitle>{t('youtube.history.uploadHistory')}</CardTitle>
              <CardDescription>
                {t('youtube.history.recentVideos')}
              </CardDescription>
            </div>
          </div>
          <Button variant="outline" size="sm" onClick={loadHistory} disabled={isLoading}>
            {t('youtube.history.refresh')}
          </Button>
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {error && (
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {isLoading ? (
          <div className="text-center py-8 text-muted-foreground">
            {t('youtube.history.loadingHistory')}
          </div>
        ) : history.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <History className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>{t('youtube.history.noUploads')}</p>
            <p className="text-sm">{t('youtube.history.uploadFirstVideo')}</p>
          </div>
        ) : (
          <div className="space-y-3">
            {history.map((entry) => (
              <div
                key={entry.video.video_id}
                className="flex items-start gap-4 p-4 border rounded-lg hover:bg-muted/50 transition-colors"
              >
                {/* Thumbnail */}
                <div className="flex-shrink-0">
                  <img
                    src={entry.video.thumbnail_url}
                    alt={entry.video.title}
                    className="w-32 h-18 object-cover rounded"
                  />
                </div>

                {/* Video Info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                      <h3 className="font-medium truncate">{entry.video.title}</h3>
                      <p className="text-sm text-muted-foreground line-clamp-2 mt-1">
                        {entry.video.description || t('youtube.history.noDescription')}
                      </p>
                    </div>
                    <Badge variant="secondary" className="flex-shrink-0">
                      {entry.video.privacy_status}
                    </Badge>
                  </div>

                  <div className="flex items-center gap-4 mt-3 text-xs text-muted-foreground">
                    <span className="flex items-center gap-1">
                      <Eye className="h-3 w-3" />
                      {formatNumber(entry.video.view_count)}
                    </span>
                    <span className="flex items-center gap-1">
                      <ThumbsUp className="h-3 w-3" />
                      {formatNumber(entry.video.like_count)}
                    </span>
                    <span className="flex items-center gap-1">
                      <MessageCircle className="h-3 w-3" />
                      {formatNumber(entry.video.comment_count)}
                    </span>
                    <span className="ml-auto">
                      {t('youtube.history.uploaded')}: {formatDate(entry.uploaded_at)}
                    </span>
                  </div>

                  <div className="flex items-center gap-2 mt-3">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleOpenVideo(entry.video.url)}
                    >
                      <ExternalLink className="h-3 w-3 mr-1" />
                      {t('youtube.history.viewOnYouTube')}
                    </Button>
                    <span className="text-xs text-muted-foreground truncate">
                      {entry.local_file_path}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
