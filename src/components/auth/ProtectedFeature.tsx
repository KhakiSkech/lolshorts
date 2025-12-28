import { ReactNode, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAuthStore } from "@/lib/auth";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { AuthModal } from "./AuthModal";

interface ProtectedFeatureProps {
  children: ReactNode;
  requiresPro?: boolean;
  featureName?: string;
  fallback?: ReactNode;
  onUpgrade?: () => void;
}

export function ProtectedFeature({
  children,
  requiresPro = false,
  featureName = "this feature",
  fallback,
  onUpgrade,
}: ProtectedFeatureProps) {
  const { t } = useTranslation();
  const { isAuthenticated, user } = useAuthStore();
  const [authModalOpen, setAuthModalOpen] = useState(false);

  // Not authenticated
  if (!isAuthenticated || !user) {
    if (fallback) {
      return <>{fallback}</>;
    }

    return (
      <>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              🔒 {t('auth.authenticationRequired')}
            </CardTitle>
            <CardDescription>
              {t('auth.pleaseLoginToAccess', { feature: featureName })}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground mb-4">
              {t('auth.createAccountOrLogin')}
            </p>
            <Button
              onClick={() => setAuthModalOpen(true)}
              className="w-full"
              data-testid="open-auth-modal-button"
            >
              {t('auth.loginSignup')}
            </Button>
          </CardContent>
        </Card>

        <AuthModal
          open={authModalOpen}
          onClose={() => setAuthModalOpen(false)}
          defaultMode="login"
        />
      </>
    );
  }

  // Authenticated but needs PRO
  if (requiresPro && user.tier !== "PRO") {
    if (fallback) {
      return <>{fallback}</>;
    }

    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center justify-between">
            <span className="flex items-center gap-2">
              ⭐ PRO Feature
            </span>
            <Badge variant="default">PRO</Badge>
          </CardTitle>
          <CardDescription>
            Upgrade to PRO to access {featureName}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <p className="text-sm text-muted-foreground">
              This feature is available exclusively for PRO subscribers:
            </p>
            <ul className="list-disc list-inside space-y-2 text-sm">
              <li>Manual clip extraction</li>
              <li>YouTube Shorts composition (9:16)</li>
              <li>Custom thumbnail generation</li>
              <li>Advanced video editing</li>
              <li>No watermarks on exports</li>
            </ul>
            <Button onClick={onUpgrade} className="w-full" variant="default">
              Upgrade to PRO
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  // All checks passed
  return <>{children}</>;
}

// Hook for imperative checks
export function useFeatureAccess() {
  const { isAuthenticated, user } = useAuthStore();

  return {
    isAuthenticated,
    isPro: user?.tier === "PRO",
    canAccess: (requiresPro: boolean = false) => {
      if (!isAuthenticated) return false;
      if (requiresPro && user?.tier !== "PRO") return false;
      return true;
    },
  };
}
