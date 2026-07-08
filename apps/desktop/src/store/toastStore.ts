import { create } from "zustand";

interface ToastState {
  message: string | null;
  key: number;
  show: (message: string) => void;
}

let hideTimer: ReturnType<typeof setTimeout> | null = null;

// Matches the design's 1.6s cfToast animation duration exactly.
export const useToastStore = create<ToastState>((set) => ({
  message: null,
  key: 0,

  show: (message) => {
    if (hideTimer) clearTimeout(hideTimer);
    set((s) => ({ message, key: s.key + 1 }));
    hideTimer = setTimeout(() => set({ message: null }), 1600);
  },
}));
