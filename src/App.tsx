import { useEffect, lazy, Suspense } from "react";
import { Router, Route, RootRoute, RouterProvider, Outlet } from "@tanstack/react-router";
import { AppShell } from "@/components/layout/AppShell";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Card, CardContent } from "@/components/ui/card";
import { Loader2 } from "lucide-react";
// Lazy load pages for better performance
const Dashboard = lazy(() => import("@/pages/Dashboard").then(m => ({ default: m.Dashboard })));
const Games = lazy(() => import("@/pages/Games").then(m => ({ default: m.Games })));
const Editor = lazy(() => import("@/pages/Editor").then(m => ({ default: m.Editor })));
const AutoEdit = lazy(() => import("@/pages/AutoEdit").then(m => ({ default: m.AutoEdit })));
const Results = lazy(() => import("@/pages/Results").then(m => ({ default: m.Results })));
const Replays = lazy(() => import("@/pages/Replays").then(m => ({ default: m.Replays }))); // Added Replays
const YouTube = lazy(() => import("@/pages/YouTube").then(m => ({ default: m.YouTube })));
const Settings = lazy(() => import("@/pages/Settings").then(m => ({ default: m.Settings })));
const PaymentSuccess = lazy(() => import("@/pages/PaymentSuccess").then(m => ({ default: m.PaymentSuccess })));
const PaymentFail = lazy(() => import("@/pages/PaymentFail").then(m => ({ default: m.PaymentFail })));

// Loading component for lazy loaded pages
const LoadingSpinner = () => (
  <Card className="w-full h-96 flex items-center justify-center">
    <CardContent>
      <Loader2 className="h-8 w-8 animate-spin" />
      <p className="mt-2 text-sm text-muted-foreground">Loading...</p>
    </CardContent>
  </Card>
);
import { supabase } from "@/lib/supabase";
import { useAuthStore } from "@/lib/auth";
import "./i18n"; // Initialize i18n with auto language detection
import { useState } from "react";
import { ReplayTargetModal } from "@/components/overlay/ReplayTargetModal";
import { lcuApi } from "@/api/lcu";

// Define root route
const rootRoute = new RootRoute({
  component: () => (
    <AppShell>
      <Outlet />
    </AppShell>
  ),
});

// Define individual routes
const indexRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/",
  component: () => (
    <Suspense fallback={<LoadingSpinner />}>
      <Dashboard />
    </Suspense>
  ),
});

const gamesRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/games",
  component: () => (
    <Suspense fallback={<LoadingSpinner />}>
      <Games />
    </Suspense>
  ),
});

const replaysRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/replays",
  component: () => (
    <Suspense fallback={<LoadingSpinner />}>
      <Replays />
    </Suspense>
  ),
});

const editorRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/editor",
  component: () => (
    <Suspense fallback={<LoadingSpinner />}>
      <Editor />
    </Suspense>
  ),
});

const autoEditRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/auto-edit",
  component: () => (
    <Suspense fallback={<LoadingSpinner />}>
      <AutoEdit />
    </Suspense>
  ),
});

const resultsRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/results",
  component: () => (
    <Suspense fallback={<LoadingSpinner />}>
      <Results />
    </Suspense>
  ),
});

const youtubeRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/youtube",
  component: () => (
    <Suspense fallback={<LoadingSpinner />}>
      <YouTube />
    </Suspense>
  ),
});

const settingsRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/settings",
  component: () => (
    <Suspense fallback={<LoadingSpinner />}>
      <Settings />
    </Suspense>
  ),
});

const paymentSuccessRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/payment/success",
  component: () => (
    <Suspense fallback={<LoadingSpinner />}>
      <PaymentSuccess />
    </Suspense>
  ),
});

const paymentFailRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/payment/fail",
  component: () => (
    <Suspense fallback={<LoadingSpinner />}>
      <PaymentFail />
    </Suspense>
  ),
});

// Create route tree
const routeTree = rootRoute.addChildren([
  indexRoute,
  gamesRoute,
  replaysRoute,
  editorRoute,
  autoEditRoute,
  resultsRoute,
  youtubeRoute,
  settingsRoute,
  paymentSuccessRoute,
  paymentFailRoute,
]);

// Create router instance
const router = new Router({ routeTree });

// Type augmentation for TypeScript
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

import { useRecordingStore } from "@/stores/recordingStore"; // Add import

export default function App() {
  const { checkAuth } = useAuthStore();
  const { startStatusPolling, stopStatusPolling } = useRecordingStore(); // Use hook
  const [isReplayModalOpen, setIsReplayModalOpen] = useState(false);

  useEffect(() => {
    let isMounted = true;

    // Check for existing session on mount
    const initAuth = async () => {
      if (isMounted) {
        await checkAuth();
      }
    };
    initAuth();

    // Start polling recording status (sync frontend with backend)
    startStatusPolling();

    // Listen for auth state changes (OAuth callbacks, logout, etc.)
    const { data: { subscription } } = supabase.auth.onAuthStateChange(async (event, session) => {
      if (!isMounted) return;

      if (event === 'SIGNED_IN' && session?.user) {
        // User signed in via OAuth or email/password
        // Fetch or create user profile
        const { error } = await supabase
          .from('user_profiles')
          .select('*')
          .eq('id', session.user.id)
          .single();

        if (error && error.code === 'PGRST116') {
          // Profile doesn't exist, create it (for OAuth signups)
          const { error: insertError } = await supabase
            .from('user_profiles')
            .insert({
              id: session.user.id,
              email: session.user.email!,
              tier: 'FREE',
            });

          if (insertError) {
            console.error('Failed to create profile:', insertError);
          }
        }

        // Refresh auth state
        if (isMounted) {
          await checkAuth();
        }
      } else if (event === 'SIGNED_OUT') {
        // User signed out
        if (isMounted) {
          await checkAuth();
        }
      }
    });

    // Replay Detection Polling
    const pollingInterval = setInterval(async () => {
      if (!isMounted) return;
      try {
        const isConnected = await lcuApi.checkStatus();
        if (isConnected) {
          // Check if game started
          // This logic needs refinement to differentiate Replay vs Live
          // For MVP, we rely on manual trigger or assume if launch_replay was called recently
        }
      } catch {
        // Ignore polling errors
      }
    }, 5000);

    return () => {
      isMounted = false;
      subscription.unsubscribe();
      clearInterval(pollingInterval);
      stopStatusPolling();
    };
    // Note: These functions are stable from Zustand store, but we include them for eslint
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <ErrorBoundary
      onError={(error, errorInfo) => {
        console.error('App-level error caught:', error, errorInfo);
        if (import.meta.env.PROD) {
          // sendToErrorTracking(error, errorInfo);
        }
      }}
    >
      <RouterProvider router={router} />
      <ReplayTargetModal 
        isOpen={isReplayModalOpen} 
        onClose={() => setIsReplayModalOpen(false)} 
      />
    </ErrorBoundary>
  );
}
