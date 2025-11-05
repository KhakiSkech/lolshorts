import { ReactNode } from "react";
import { useAuthStore } from "@/lib/auth";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

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
  const { isAuthenticated, user } = useAuthStore();

  // Not authenticated
  if (!isAuthenticated || !user) {
    if (fallback) {
      return <>{fallback}</>;
    }

    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            🔒 Authentication Required
          </CardTitle>
          <CardDescription>
            Please login to access {featureName}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground mb-4">
            Create an account or login to unlock all features
          </p>
          <Button onClick={() => {/* Open auth modal */}} className="w-full">
            Login / Sign Up
          </Button>
        </CardContent>
      </Card>
    );
  }

  // Authenticated but needs PRO
  if (requiresPro && user.tier !== "Pro") {
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
    isPro: user?.tier === "Pro",
    canAccess: (requiresPro: boolean = false) => {
      if (!isAuthenticated) return false;
      if (requiresPro && user?.tier !== "Pro") return false;
      return true;
    },
  };
}
