import { createRootRoute, createRoute, createRouter } from "@tanstack/react-router";
import { RootLayout } from "@/components/RootLayout";
import { ClipboardView } from "@/views/ClipboardView";
import { TimelineView } from "@/views/TimelineView";
import { CollectionsView } from "@/views/CollectionsView";
import { AutomationView } from "@/views/AutomationView";
import { SettingsView } from "@/views/SettingsView";

const rootRoute = createRootRoute({ component: RootLayout });

const clipboardRoute = createRoute({ getParentRoute: () => rootRoute, path: "/", component: ClipboardView });
const timelineRoute = createRoute({ getParentRoute: () => rootRoute, path: "/timeline", component: TimelineView });
const collectionsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/collections",
  component: CollectionsView,
});
const automationRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/automation",
  component: AutomationView,
});
const settingsRoute = createRoute({ getParentRoute: () => rootRoute, path: "/settings", component: SettingsView });

const routeTree = rootRoute.addChildren([
  clipboardRoute,
  timelineRoute,
  collectionsRoute,
  automationRoute,
  settingsRoute,
]);

export const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
