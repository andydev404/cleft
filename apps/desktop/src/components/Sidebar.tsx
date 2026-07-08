import { Link, useNavigate, useRouterState } from "@tanstack/react-router";
import { History, FolderOpen, Zap, Settings, Star } from "lucide-react";
import { useClipboardStore } from "@/store/clipboardStore";
import { useAutomationStore } from "@/store/automationStore";

function RailSection({ label, onCollapse }: { label: string; onCollapse?: () => void }) {
  return (
    <div className="flex items-center justify-between px-2.5 pb-[5px] pt-1.5">
      <span className="text-[10.5px] font-bold tracking-[.05em] text-text-tertiary">{label}</span>
      {onCollapse && (
        <button
          onClick={onCollapse}
          title="Collapse sidebar"
          className="rounded-[5px] p-0.5 text-[13px] leading-none text-text-tertiary hover:bg-accent hover:text-foreground"
        >
          ‹
        </button>
      )}
    </div>
  );
}

function railItemStyle(active: boolean): React.CSSProperties {
  return {
    color: active ? "var(--foreground)" : "var(--muted-foreground)",
    fontWeight: active ? 600 : 500,
    background: active ? "var(--row-sel)" : "transparent",
    boxShadow: active ? "inset 0 0 0 1px var(--border)" : "none",
    letterSpacing: "-.005em",
  };
}

function RailRow({
  icon,
  glyphColor,
  label,
  count,
}: {
  icon: React.ReactNode;
  glyphColor: string;
  label: string;
  count?: string | number;
}) {
  return (
    <>
      <span className="flex w-[15px] shrink-0 justify-center" style={{ color: glyphColor }}>
        {icon}
      </span>
      <span className="min-w-0 flex-1 truncate">{label}</span>
      {count !== undefined && <span className="shrink-0 text-[10.5px] tabular-nums text-text-tertiary">{count}</span>}
    </>
  );
}

export function Sidebar() {
  const pathname = useRouterState({ select: (s) => s.location.pathname });
  const navigate = useNavigate();
  const { history, railFilter, setRailFilter, railOpen, toggleRail } = useClipboardStore();
  const favoriteCount = history.filter((c) => c.is_favorite).length;
  const rules = useAutomationStore((s) => s.rules);
  const activeRuleCount = rules.filter((r) => r.enabled).length;

  const rowClass = "flex w-full items-center gap-2 rounded-[8px] px-2.5 py-[7px] text-[12.5px] transition-colors hover:bg-accent whitespace-nowrap";

  return (
    <div
      className="flex shrink-0 flex-col gap-px overflow-hidden border-r transition-[width,padding] duration-200 ease-[cubic-bezier(.2,.8,.2,1)]"
      style={railOpen ? { width: 172, padding: "10px 8px" } : { width: 0, padding: "10px 0" }}
    >
      <RailSection label="LIBRARY" onCollapse={toggleRail} />
      <button
        onClick={() => {
          setRailFilter("all");
          navigate({ to: "/" });
        }}
        className={rowClass}
        style={railItemStyle(pathname === "/" && railFilter === "all")}
      >
        <RailRow icon={<span className="text-[12px]">◷</span>} glyphColor="var(--primary)" label="All clips" count={history.length} />
      </button>
      <button
        onClick={() => {
          setRailFilter("favorites");
          navigate({ to: "/" });
        }}
        className={rowClass}
        style={railItemStyle(pathname === "/" && railFilter === "favorites")}
      >
        <RailRow icon={<Star className="h-[11px] w-[11px]" />} glyphColor="#e0a23c" label="Favorites" count={favoriteCount} />
      </button>

      <div className="my-1 h-px bg-border" />

      <RailSection label="NAVIGATE" />
      <Link to="/timeline" className={rowClass} style={railItemStyle(pathname === "/timeline")}>
        <RailRow icon={<History className="h-[13px] w-[13px]" />} glyphColor="var(--primary)" label="Timeline" />
      </Link>
      <Link to="/collections" className={rowClass} style={railItemStyle(pathname === "/collections")}>
        <RailRow icon={<FolderOpen className="h-[13px] w-[13px]" />} glyphColor="#b06cff" label="Collections" />
      </Link>
      <Link to="/automation" className={rowClass} style={railItemStyle(pathname === "/automation")}>
        <RailRow
          icon={<Zap className="h-[13px] w-[13px]" />}
          glyphColor="#46b29d"
          label="Automation"
          count={activeRuleCount > 0 ? `${activeRuleCount} on` : undefined}
        />
      </Link>
      <Link to="/settings" className={rowClass} style={railItemStyle(pathname === "/settings")}>
        <RailRow icon={<Settings className="h-[13px] w-[13px]" />} glyphColor="var(--text-tertiary)" label="Settings" />
      </Link>
    </div>
  );
}
