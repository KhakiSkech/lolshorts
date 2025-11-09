import { YouTubeAuth } from '@/components/youtube/YouTubeAuth';
import { YouTubeUpload } from '@/components/youtube/YouTubeUpload';
import { YouTubeHistory } from '@/components/youtube/YouTubeHistory';
import { QuotaDisplay } from '@/components/youtube/QuotaDisplay';
import { ProtectedFeature } from '@/components/auth/ProtectedFeature';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Youtube } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export function YouTube() {
  const { t } = useTranslation();

  return (
    <ProtectedFeature requiresPro={true} featureName={t('youtube.title')}>
      <div className="h-full flex flex-col overflow-hidden p-6">
        <div className="flex items-center gap-3 mb-6">
          <Youtube className="h-8 w-8 text-red-500" />
          <div>
            <h1 className="text-3xl font-bold">{t('youtube.title')}</h1>
            <p className="text-muted-foreground">
              {t('youtube.subtitle')}
            </p>
          </div>
        </div>

        <Tabs defaultValue="upload" className="flex-1 flex flex-col">
          <TabsList className="grid w-full grid-cols-3 mb-6">
            <TabsTrigger value="upload">{t('youtube.tabs.upload')}</TabsTrigger>
            <TabsTrigger value="history">{t('youtube.tabs.history')}</TabsTrigger>
            <TabsTrigger value="account">{t('youtube.tabs.account')}</TabsTrigger>
          </TabsList>

          <TabsContent value="upload" className="flex-1 overflow-y-auto space-y-6">
            <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
              <div className="xl:col-span-2">
                <YouTubeUpload />
              </div>
              <div className="space-y-6">
                <QuotaDisplay />
              </div>
            </div>
          </TabsContent>

          <TabsContent value="history" className="flex-1 overflow-y-auto">
            <YouTubeHistory />
          </TabsContent>

          <TabsContent value="account" className="flex-1 overflow-y-auto space-y-6">
            <div className="max-w-2xl">
              <YouTubeAuth />
              <div className="mt-6">
                <QuotaDisplay />
              </div>
            </div>
          </TabsContent>
        </Tabs>
      </div>
    </ProtectedFeature>
  );
}
