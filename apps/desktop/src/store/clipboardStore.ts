import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useToastStore } from "@/store/toastStore";
import type { ClipMetadata } from "@/types";

interface ClipboardState {
  history: ClipMetadata[];
  query: string;
  searchResults: ClipMetadata[];
  selected: ClipMetadata | null;
  selectedContent: string | null;
  railFilter: "all" | "favorites";
  setRailFilter: (filter: "all" | "favorites") => void;
  collectionFilter: string | null;
  setCollectionFilter: (name: string | null) => void;
  assignCollection: (id: string, collection: string | null) => void;
  addTag: (id: string, tag: string) => void;
  removeTag: (id: string, tag: string) => void;
  railOpen: boolean;
  toggleRail: () => void;
  setQuery: (query: string) => void;
  setSearchResults: (results: ClipMetadata[]) => void;
  loadHistory: () => Promise<void>;
  addClip: (clip: ClipMetadata) => void;
  updateContext: (id: string, window_title: string | null, url: string | null) => void;
  selectClip: (clip: ClipMetadata) => void;
  copyClip: (clip: ClipMetadata) => Promise<void>;
  clearSelection: () => void;
  toggleFavorite: (id: string) => void;
  removeEvicted: (ids: string[]) => void;

  bulkSelected: Set<string>;
  lastClickedId: string | null;
  toggleBulkSelect: (id: string) => void;
  rangeBulkSelect: (id: string, visibleIds: string[]) => void;
  selectAll: (ids: string[]) => void;
  clearBulkSelection: () => void;
  deleteClip: (id: string) => Promise<void>;
  deleteBulkSelected: () => Promise<void>;
}

function removeFromLists(history: ClipMetadata[], searchResults: ClipMetadata[], ids: Set<string>) {
  return {
    history: history.filter((c) => !ids.has(c.id)),
    searchResults: searchResults.filter((c) => !ids.has(c.id)),
  };
}

// Applies a patch to one clip everywhere it can appear: history, search
// results, and the current selection.
function patchClip(s: ClipboardState, id: string, fn: (c: ClipMetadata) => ClipMetadata) {
  const patch = (c: ClipMetadata) => (c.id === id ? fn(c) : c);
  return {
    history: s.history.map(patch),
    searchResults: s.searchResults.map(patch),
    selected: s.selected?.id === id ? fn(s.selected) : s.selected,
  };
}

export const useClipboardStore = create<ClipboardState>((set, get) => ({
  history: [],
  query: "",
  searchResults: [],
  selected: null,
  selectedContent: null,
  railFilter: "all",
  collectionFilter: null,
  railOpen: true,
  bulkSelected: new Set(),
  lastClickedId: null,

  setRailFilter: (railFilter) => set({ railFilter, collectionFilter: null }),
  setCollectionFilter: (collectionFilter) => set({ collectionFilter }),

  // Optimistic writes: apply the patch immediately, revert with a toast if
  // the backend write fails (full disk, stale/deleted clip id).
  assignCollection: (id, collection) => {
    const previous = get().history.find((c) => c.id === id)?.collection ?? null;
    set((s) => patchClip(s, id, (c) => ({ ...c, collection })));
    invoke("assign_collection", { id, collection }).catch(() => {
      set((s) => patchClip(s, id, (c) => ({ ...c, collection: previous })));
      useToastStore.getState().show("Couldn't update collection");
    });
  },

  addTag: (id, tag) => {
    set((s) => patchClip(s, id, (c) => (c.tags.includes(tag) ? c : { ...c, tags: [...c.tags, tag] })));
    invoke("add_tag", { id, tag }).catch(() => {
      set((s) => patchClip(s, id, (c) => ({ ...c, tags: c.tags.filter((t) => t !== tag) })));
      useToastStore.getState().show("Couldn't add tag");
    });
  },

  removeTag: (id, tag) => {
    set((s) => patchClip(s, id, (c) => ({ ...c, tags: c.tags.filter((t) => t !== tag) })));
    invoke("remove_tag", { id, tag }).catch(() => {
      set((s) => patchClip(s, id, (c) => (c.tags.includes(tag) ? c : { ...c, tags: [...c.tags, tag] })));
      useToastStore.getState().show("Couldn't remove tag");
    });
  },

  toggleRail: () => set((s) => ({ railOpen: !s.railOpen })),
  setQuery: (query) => set({ query }),
  setSearchResults: (searchResults) => set({ searchResults }),

  loadHistory: async () => {
    const history = await invoke<ClipMetadata[]>("get_history");
    set({ history });
  },

  addClip: (clip) => set((s) => ({ history: [clip, ...s.history] })),

  updateContext: (id, window_title, url) => set((s) => patchClip(s, id, (c) => ({ ...c, window_title, url }))),

  selectClip: (clip) => {
    set({ selected: clip, selectedContent: null, lastClickedId: clip.id });
    invoke<string | null>("get_clip_content", { id: clip.id }).then((content) => {
      if (get().selected?.id === clip.id) set({ selectedContent: content });
    });
  },

  // Fetches fresh rather than trusting selectedContent — this is also
  // called for clips that were never selected (Enter copies the top result).
  copyClip: async (clip) => {
    const content = await invoke<string | null>("get_clip_content", { id: clip.id });
    if (!content) return;
    invoke("copy_to_clipboard", { text: content });
    useToastStore.getState().show("Copied — press ⌘V to paste");
    invoke("hide_palette");
  },

  clearSelection: () =>
    set({
      selected: null,
      selectedContent: null,
      query: "",
      searchResults: [],
      bulkSelected: new Set(),
      collectionFilter: null,
    }),

  toggleFavorite: (id) => {
    const target = get().history.find((c) => c.id === id) ?? get().selected;
    const next = !(target?.is_favorite ?? false);
    set((s) => patchClip(s, id, (c) => ({ ...c, is_favorite: next })));
    invoke("set_favorite", { id, favorite: next });
  },

  removeEvicted: (ids) => {
    const idSet = new Set(ids);
    set((s) => {
      const { history, searchResults } = removeFromLists(s.history, s.searchResults, idSet);
      const clearedSelection = s.selected ? idSet.has(s.selected.id) : false;
      return {
        history,
        searchResults,
        selected: clearedSelection ? null : s.selected,
        selectedContent: clearedSelection ? null : s.selectedContent,
      };
    });
  },

  toggleBulkSelect: (id) =>
    set((s) => {
      const bulkSelected = new Set(s.bulkSelected);
      if (bulkSelected.has(id)) bulkSelected.delete(id);
      else bulkSelected.add(id);
      return { bulkSelected, lastClickedId: id };
    }),

  rangeBulkSelect: (id, visibleIds) =>
    set((s) => {
      const anchor = s.lastClickedId ?? id;
      const from = visibleIds.indexOf(anchor);
      const to = visibleIds.indexOf(id);
      if (from === -1 || to === -1) return { bulkSelected: new Set(s.bulkSelected).add(id), lastClickedId: id };
      const [start, end] = from < to ? [from, to] : [to, from];
      const bulkSelected = new Set(s.bulkSelected);
      for (const i of visibleIds.slice(start, end + 1)) bulkSelected.add(i);
      return { bulkSelected, lastClickedId: id };
    }),

  selectAll: (ids) => set({ bulkSelected: new Set(ids) }),

  clearBulkSelection: () => set({ bulkSelected: new Set() }),

  deleteClip: async (id) => {
    await invoke("delete_clips", { ids: [id] });
    set((s) => {
      const { history, searchResults } = removeFromLists(s.history, s.searchResults, new Set([id]));
      const clearedSelection = s.selected?.id === id;
      const bulkSelected = new Set(s.bulkSelected);
      bulkSelected.delete(id);
      return {
        history,
        searchResults,
        bulkSelected,
        selected: clearedSelection ? null : s.selected,
        selectedContent: clearedSelection ? null : s.selectedContent,
      };
    });
    useToastStore.getState().show("Deleted");
  },

  deleteBulkSelected: async () => {
    const ids = get().bulkSelected;
    if (ids.size === 0) return;
    const idArray = Array.from(ids);
    await invoke("delete_clips", { ids: idArray });
    const count = ids.size;
    set((s) => {
      const { history, searchResults } = removeFromLists(s.history, s.searchResults, ids);
      const clearedSelection = s.selected ? ids.has(s.selected.id) : false;
      return {
        history,
        searchResults,
        bulkSelected: new Set(),
        selected: clearedSelection ? null : s.selected,
        selectedContent: clearedSelection ? null : s.selectedContent,
      };
    });
    useToastStore.getState().show(`Deleted ${count} clip${count === 1 ? "" : "s"}`);
  },
}));
