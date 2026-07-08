import { create } from "zustand";

interface ConfirmOptions {
  title: string;
  description: string;
  confirmLabel?: string;
  variant?: "destructive" | "default";
  onConfirm: () => void;
}

interface ConfirmState {
  open: boolean;
  title: string;
  description: string;
  confirmLabel: string;
  variant: "destructive" | "default";
  onConfirm: (() => void) | null;
  show: (opts: ConfirmOptions) => void;
  close: () => void;
}

// One global confirm dialog (mounted once in RootLayout, see
// ConfirmDialog.tsx) rather than each delete button owning its own
// open/AlertDialog pair — same pattern as toastStore. Reused for the
// update prompt too (variant: "default") so that flow doesn't need its own
// dialog component.
export const useConfirmStore = create<ConfirmState>((set) => ({
  open: false,
  title: "",
  description: "",
  confirmLabel: "Delete",
  variant: "destructive",
  onConfirm: null,

  show: (opts) =>
    set({
      open: true,
      title: opts.title,
      description: opts.description,
      confirmLabel: opts.confirmLabel ?? "Delete",
      variant: opts.variant ?? "destructive",
      onConfirm: opts.onConfirm,
    }),

  close: () => set({ open: false }),
}));
