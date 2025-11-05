import { Router, Route, RootRoute, RouterProvider, Outlet } from "@tanstack/react-router";
import { AppShell } from "@/components/layout/AppShell";
import { Dashboard } from "@/pages/Dashboard";
import { Games } from "@/pages/Games";
import { Editor } from "@/pages/Editor";
import { Settings } from "@/pages/Settings";
import { PaymentSuccess } from "@/pages/PaymentSuccess";
import { PaymentFail } from "@/pages/PaymentFail";

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
  component: Dashboard,
});

const gamesRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/games",
  component: Games,
});

const editorRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/editor",
  component: Editor,
});

const settingsRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/settings",
  component: Settings,
});

const paymentSuccessRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/payment/success",
  component: PaymentSuccess,
});

const paymentFailRoute = new Route({
  getParentRoute: () => rootRoute,
  path: "/payment/fail",
  component: PaymentFail,
});

// Create route tree
const routeTree = rootRoute.addChildren([
  indexRoute,
  gamesRoute,
  editorRoute,
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

export default function App() {
  return <RouterProvider router={router} />;
}
