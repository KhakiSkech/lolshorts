import { useAutoEditQuota } from '@/hooks/useAutoEditQuota';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Sparkles, Crown, AlertCircle, Loader2 } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

export function AutoEditQuotaBadge() {
  const { t } = useTranslation();
  const { quota, isLoading, getQuotaWarningLevel } = useAutoEditQuota();
  const [showUpgradeDialog, setShowUpgradeDialog] = useState(false);

  if (isLoading || !quota) {
    return (
      <Badge variant="secondary" className="flex items-center gap-1">
        <Loader2 className="w-3 h-3 animate-spin" />
        {t('autoEdit.loadingQuota')}
      </Badge>
    );
  }

  const warningLevel = getQuotaWarningLevel();

  // PRO user badge
  if (quota.is_pro) {
    return (
      <Badge variant="default" className="flex items-center gap-1 bg-gradient-to-r from-yellow-400 to-yellow-600">
        <Crown className="w-3 h-3" />
        PRO • {t('autoEdit.unlimitedEdits')}
      </Badge>
    );
  }

  // FREE user quota badge
  const badgeVariant = warningLevel === 'exhausted' ? 'destructive' : warningLevel === 'low' ? 'default' : 'secondary';
  const badgeText = `${quota.remaining}/${quota.limit} ${t('autoEdit.remaining')}`;

  return (
    <>
      <div className="flex items-center gap-2">
        <Badge variant={badgeVariant} className="flex items-center gap-1">
          <Sparkles className="w-3 h-3" />
          {badgeText}
        </Badge>
        {warningLevel !== 'none' && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setShowUpgradeDialog(true)}
            className="text-xs h-7"
          >
            <Crown className="w-3 h-3 mr-1" />
            {t('autoEdit.upgradeToPro')}
          </Button>
        )}
      </div>

      {/* Upgrade Dialog */}
      <Dialog open={showUpgradeDialog} onOpenChange={setShowUpgradeDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Crown className="w-5 h-5 text-yellow-600" />
              {t('autoEdit.upgradeToProTitle')}
            </DialogTitle>
            <DialogDescription>
              {t('autoEdit.upgradeToProDescription')}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            {/* Current Plan */}
            <div className="p-4 bg-muted rounded-lg">
              <div className="flex items-center justify-between mb-2">
                <span className="font-medium">{t('autoEdit.currentPlan')}: FREE</span>
                <Badge variant="secondary">{quota.remaining}/{quota.limit} {t('autoEdit.remaining')}</Badge>
              </div>
              <p className="text-sm text-muted-foreground">
                {t('autoEdit.freeQuotaDescription')}
              </p>
            </div>

            {/* PRO Plan */}
            <div className="p-4 bg-gradient-to-r from-yellow-50 to-yellow-100 dark:from-yellow-950 dark:to-yellow-900 rounded-lg border-2 border-yellow-600">
              <div className="flex items-center justify-between mb-2">
                <span className="font-medium flex items-center gap-2">
                  <Crown className="w-4 h-4 text-yellow-600" />
                  PRO {t('autoEdit.plan')}
                </span>
                <Badge className="bg-yellow-600">∞ {t('autoEdit.unlimited')}</Badge>
              </div>
              <ul className="space-y-1 text-sm">
                <li className="flex items-center gap-2">
                  ✓ {t('autoEdit.unlimitedAutoEdits')}
                </li>
                <li className="flex items-center gap-2">
                  ✓ {t('autoEdit.priorityProcessing')}
                </li>
                <li className="flex items-center gap-2">
                  ✓ {t('autoEdit.advancedFeatures')}
                </li>
              </ul>
            </div>

            {warningLevel === 'exhausted' && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  {t('autoEdit.quotaExhausted')}
                </AlertDescription>
              </Alert>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setShowUpgradeDialog(false)}>
              {t('common.cancel')}
            </Button>
            <Button
              className="bg-gradient-to-r from-yellow-400 to-yellow-600"
              onClick={() => {
                // TODO: Navigate to payment page
                alert('Navigate to payment page');
                setShowUpgradeDialog(false);
              }}
            >
              <Crown className="w-4 h-4 mr-2" />
              {t('autoEdit.upgradeToPro')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
