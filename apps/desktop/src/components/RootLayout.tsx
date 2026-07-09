import { useEffect } from "react";
import { Outlet, useNavigate } from "@tanstack/react-router";
import { Sidebar } from "@/components/Sidebar";
import { FooterBar } from "@/components/FooterBar";
import { useAppEvents } from "@/hooks/useAppEvents";
import { useSystemTheme } from "@/hooks/useSystemTheme";
import { useRunwayStore } from "@/store/runwayStore";
import { useSettingsStore } from "@/store/settingsStore";
import { Toast } from "@/components/Toast";
import { ConfirmDialog } from "@/components/ConfirmDialog";

// The window blur is native macOS vibrancy (applied in lib.rs), not CSS
// backdrop-filter — bg-background here is a light tint over it.
export function RootLayout() {
  const { init } = useRunwayStore();
  const darkSync = useSettingsStore((s) => s.darkSync);
  const navigate = useNavigate();

  useSystemTheme(darkSync);

  useEffect(() => {
    init();
    useSettingsStore.getState().init();
  }, [init]);

  useAppEvents(() => navigate({ to: "/" }));

  return (
    <main
      className="relative flex h-screen w-screen flex-col overflow-hidden rounded-[16px] border bg-background"
      style={{ borderColor: "var(--border-2)", boxShadow: "var(--shadow)" }}
    >
      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <div className="flex min-w-0 flex-1 flex-col">
          <Outlet />
        </div>
      </div>
      <FooterBar />
      <Toast />
      <ConfirmDialog />
    </main>
  );
}
