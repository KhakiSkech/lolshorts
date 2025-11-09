import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useYouTube } from '@/hooks/useYouTube';
import { Button } from '@/components/ui/button';
import { Card, CardHeader, CardTitle, CardContent, CardDescription } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Upload, Film, CheckCircle } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { PrivacyStatus, VideoMetadata } from '@/types/youtube';

export function YouTubeUpload() {
  const { t } = useTranslation();
  const {
    authStatus,
    isLoading,
    error,
    uploadProgress,
    uploadVideo,
    startProgressPolling,
    stopProgressPolling,
  } = useYouTube();

  const [videoPath, setVideoPath] = useState('');
  const [thumbnailPath, setThumbnailPath] = useState('');
  const [metadata, setMetadata] = useState<VideoMetadata>({
    title: '',
    description: '',
    tags: [],
    privacy_status: 'Unlisted' as PrivacyStatus,
    made_for_kids: false,
    category_id: '20', // Gaming category
  });
  const [tagsInput, setTagsInput] = useState('');
  const [uploadSuccess, setUploadSuccess] = useState(false);

  const handleSelectVideo = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Video',
            extensions: ['mp4', 'mov', 'avi', 'mkv', 'webm'],
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        setVideoPath(selected);

        // Auto-fill title from filename if empty
        if (!metadata.title) {
          const filename = selected.split(/[\\/]/).pop()?.replace(/\.[^/.]+$/, '') || '';
          setMetadata((prev) => ({ ...prev, title: filename }));
        }
      }
    } catch (err) {
      console.error('File selection error:', err);
    }
  };

  const handleSelectThumbnail = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Image',
            extensions: ['jpg', 'jpeg', 'png'],
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        setThumbnailPath(selected);
      }
    } catch (err) {
      console.error('File selection error:', err);
    }
  };

  const handleUpload = async () => {
    if (!videoPath || !metadata.title) {
      return;
    }

    try {
      setUploadSuccess(false);

      // Parse tags
      const tags = tagsInput
        .split(',')
        .map((tag) => tag.trim())
        .filter((tag) => tag.length > 0);

      const uploadMetadata: VideoMetadata = {
        ...metadata,
        tags,
      };

      // Start upload
      startProgressPolling();
      await uploadVideo(videoPath, uploadMetadata, thumbnailPath || undefined);

      setUploadSuccess(true);
      stopProgressPolling();

      // Reset form
      setVideoPath('');
      setThumbnailPath('');
      setMetadata({
        title: '',
        description: '',
        tags: [],
        privacy_status: 'Unlisted',
        made_for_kids: false,
        category_id: '20',
      });
      setTagsInput('');
    } catch (err) {
      console.error('Upload error:', err);
      stopProgressPolling();
    }
  };

  const getUploadPercentage = () => {
    if (!uploadProgress) return 0;
    return Math.round(
      (uploadProgress.uploaded_bytes / uploadProgress.total_bytes) * 100
    );
  };

  if (!authStatus.authenticated) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>{t('youtube.upload.uploadVideo')}</CardTitle>
          <CardDescription>
            {t('youtube.upload.connectAccountFirst')}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Alert>
            <AlertDescription>
              {t('youtube.upload.connectRequired')}
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-3">
          <Film className="h-6 w-6" />
          <div>
            <CardTitle>{t('youtube.upload.uploadToYouTube')}</CardTitle>
            <CardDescription>
              {t('youtube.upload.uploadDescription')}
            </CardDescription>
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-6">
        {error && (
          <Alert variant="destructive">
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {uploadSuccess && (
          <Alert className="bg-green-50 text-green-900 border-green-200">
            <CheckCircle className="h-4 w-4" />
            <AlertDescription>
              {t('youtube.upload.uploadSuccessful')}
            </AlertDescription>
          </Alert>
        )}

        {uploadProgress && uploadProgress.status !== 'Completed' && (
          <div className="space-y-2">
            <div className="flex items-center justify-between text-sm">
              <span className="font-medium">{t('youtube.upload.uploadProgress')}</span>
              <Badge variant="secondary">{uploadProgress.status}</Badge>
            </div>
            <Progress value={getUploadPercentage()} />
            <p className="text-xs text-muted-foreground text-center">
              {getUploadPercentage()}% ({uploadProgress.uploaded_bytes.toLocaleString()} /{' '}
              {uploadProgress.total_bytes.toLocaleString()} bytes)
            </p>
          </div>
        )}

        <div className="space-y-4">
          {/* Video File Selection */}
          <div className="space-y-2">
            <Label>{t('youtube.upload.videoFile')}</Label>
            <div className="flex gap-2">
              <Input
                value={videoPath}
                placeholder={t('youtube.upload.selectVideoFile')}
                readOnly
                className="flex-1"
              />
              <Button variant="outline" onClick={handleSelectVideo}>
                {t('youtube.upload.browse')}
              </Button>
            </div>
          </div>

          {/* Thumbnail Selection (Optional) */}
          <div className="space-y-2">
            <Label>{t('youtube.upload.customThumbnail')}</Label>
            <div className="flex gap-2">
              <Input
                value={thumbnailPath}
                placeholder={t('youtube.upload.selectThumbnailImage')}
                readOnly
                className="flex-1"
              />
              <Button variant="outline" onClick={handleSelectThumbnail}>
                {t('youtube.upload.browse')}
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              {t('youtube.upload.thumbnailRecommendation')}
            </p>
          </div>

          {/* Title */}
          <div className="space-y-2">
            <Label>{t('youtube.upload.title')}</Label>
            <Input
              value={metadata.title}
              onChange={(e) =>
                setMetadata((prev) => ({ ...prev, title: e.target.value }))
              }
              placeholder={t('youtube.upload.titlePlaceholder')}
              maxLength={100}
            />
            <p className="text-xs text-muted-foreground">
              {t('youtube.upload.charactersCount', { count: metadata.title.length, max: 100 })}
            </p>
          </div>

          {/* Description */}
          <div className="space-y-2">
            <Label>{t('youtube.upload.description')}</Label>
            <textarea
              value={metadata.description}
              onChange={(e) =>
                setMetadata((prev) => ({ ...prev, description: e.target.value }))
              }
              placeholder={t('youtube.upload.descriptionPlaceholder')}
              className="w-full min-h-[100px] px-3 py-2 text-sm border rounded-md"
              maxLength={5000}
            />
            <p className="text-xs text-muted-foreground">
              {t('youtube.upload.charactersCount', { count: metadata.description.length, max: 5000 })}
            </p>
          </div>

          {/* Tags */}
          <div className="space-y-2">
            <Label>{t('youtube.upload.tags')}</Label>
            <Input
              value={tagsInput}
              onChange={(e) => setTagsInput(e.target.value)}
              placeholder={t('youtube.upload.tagsPlaceholder')}
            />
          </div>

          {/* Privacy Status */}
          <div className="space-y-2">
            <Label>{t('youtube.upload.privacy')}</Label>
            <Select
              value={metadata.privacy_status}
              onValueChange={(value: PrivacyStatus) =>
                setMetadata((prev) => ({ ...prev, privacy_status: value }))
              }
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="Public">{t('youtube.upload.privacyOptions.public')}</SelectItem>
                <SelectItem value="Unlisted">{t('youtube.upload.privacyOptions.unlisted')}</SelectItem>
                <SelectItem value="Private">{t('youtube.upload.privacyOptions.private')}</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Upload Button */}
          <Button
            onClick={handleUpload}
            disabled={!videoPath || !metadata.title || isLoading}
            className="w-full"
          >
            <Upload className="h-4 w-4 mr-2" />
            {isLoading ? t('youtube.upload.uploading') : t('youtube.upload.uploadButton')}
          </Button>

          <p className="text-xs text-muted-foreground">
            {t('youtube.upload.requiredFields')}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
