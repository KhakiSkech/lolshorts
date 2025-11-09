import { useEffect, useState } from 'react';
import { useYouTube } from '@/hooks/useYouTube';
import { Card, CardHeader, CardTitle, CardContent, CardDescription } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { BarChart3, AlertTriangle, CheckCircle } from 'lucide-react';
import { QuotaInfo } from '@/types/youtube';

export function QuotaDisplay() {
  const { authStatus, getQuotaInfo } = useYouTube();
  const [quota, setQuota] = useState<QuotaInfo | null>(null);

  useEffect(() => {
    if (authStatus.authenticated) {
      loadQuota();
    }
  }, [authStatus.authenticated]);

  const loadQuota = async () => {
    try {
      const data = await getQuotaInfo();
      setQuota(data);
    } catch (err) {
      console.error('Failed to load quota:', err);
    }
  };

  const getQuotaPercentage = () => {
    if (!quota) return 0;
    return Math.round((quota.used / quota.daily_limit) * 100);
  };

  const getQuotaStatus = (): 'low' | 'medium' | 'high' => {
    const percentage = getQuotaPercentage();
    if (percentage >= 80) return 'high';
    if (percentage >= 50) return 'medium';
    return 'low';
  };

  const formatResetDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString() + ' at midnight PT';
  };

  const getUploadsRemaining = () => {
    if (!quota) return 0;
    // Each upload costs approximately 1,600 units
    return Math.floor(quota.remaining / 1600);
  };

  if (!authStatus.authenticated || !quota) {
    return null;
  }

  const quotaStatus = getQuotaStatus();

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <BarChart3 className="h-6 w-6" />
            <div>
              <CardTitle>API Quota</CardTitle>
              <CardDescription>Daily YouTube API usage</CardDescription>
            </div>
          </div>
          {quotaStatus === 'high' ? (
            <Badge variant="destructive" className="gap-1">
              <AlertTriangle className="h-3 w-3" />
              High Usage
            </Badge>
          ) : quotaStatus === 'medium' ? (
            <Badge variant="secondary" className="gap-1">
              <AlertTriangle className="h-3 w-3" />
              Medium Usage
            </Badge>
          ) : (
            <Badge variant="default" className="gap-1">
              <CheckCircle className="h-3 w-3" />
              Low Usage
            </Badge>
          )}
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        <div className="space-y-2">
          <div className="flex justify-between text-sm">
            <span className="font-medium">Quota Usage</span>
            <span className="text-muted-foreground">
              {quota.used.toLocaleString()} / {quota.daily_limit.toLocaleString()} units
            </span>
          </div>
          <Progress
            value={getQuotaPercentage()}
            className={
              quotaStatus === 'high'
                ? '[&>div]:bg-destructive'
                : quotaStatus === 'medium'
                  ? '[&>div]:bg-yellow-500'
                  : ''
            }
          />
          <p className="text-xs text-muted-foreground">
            {getQuotaPercentage()}% used
          </p>
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-1">
            <p className="text-sm font-medium">Remaining</p>
            <p className="text-2xl font-bold">
              {quota.remaining.toLocaleString()}
            </p>
            <p className="text-xs text-muted-foreground">units</p>
          </div>

          <div className="space-y-1">
            <p className="text-sm font-medium">Uploads Left</p>
            <p className="text-2xl font-bold">{getUploadsRemaining()}</p>
            <p className="text-xs text-muted-foreground">~1,600 units/upload</p>
          </div>
        </div>

        {quotaStatus === 'high' && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              You're approaching your daily quota limit. Consider waiting until
              reset to avoid hitting the limit.
            </AlertDescription>
          </Alert>
        )}

        <div className="pt-4 border-t">
          <p className="text-xs text-muted-foreground">
            Quota resets: {formatResetDate(quota.reset_at)}
          </p>
        </div>
      </CardContent>
    </Card>
  );
}
