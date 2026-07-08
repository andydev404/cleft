import { useEffect } from "react";
import { Outlet, useNavigate } from "@tanstack/react-router";
import { Button } from "@/components/ui/button";
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
  const { trusted, init } = useRunwayStore();
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
      {trusted === false && (
        <div className="flex items-center justify-between gap-3 border-b bg-muted px-3.5 py-2 text-[12.5px]">
          <span>
            Context capture is off. Cleft still saves your clipboard history, but can't remember where you copied
            things from.
          </span>
          <Button size="sm" onClick={() => useRunwayStore.getState().requestPermission()}>
            Enable
          </Button>
        </div>
      )}
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
