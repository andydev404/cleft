import { Check } from "lucide-react";
import { useToastStore } from "@/store/toastStore";

// Exact styling/animation from the design's toast component — the design
// has no delete feature, but this is the closest existing pattern to reuse
// for that feedback (and for anything else that needs a quick confirm).
export function Toast() {
  const { message, key } = useToastStore();
  if (!message) return null;

  return (
    <div
      key={key}
      className="absolute bottom-[42px] left-1/2 z-[70] flex items-center gap-[9px] rounded-[11px] border px-[18px] py-[10px] text-[13px] font-semibold"
      style={{
        background: "var(--toast-bg)",
        color: "var(--toast-fg)",
        borderColor: "var(--border-2)",
        boxShadow: "var(--shadow)",
        animation: "cfToast 1.6s ease both",
      }}
    >
      <Check className="h-3.5 w-3.5 text-success" strokeWidth={3} />
      {message}
    </div>
  );
}
