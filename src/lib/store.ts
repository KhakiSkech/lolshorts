import { create } from "zustand";

// Note: User authentication state is managed in @/lib/auth.ts via useAuthStore
// This store is for UI-specific state only

export type RecordingStatus = "Idle" | "Recording" | "Processing";

interface AppState {
  // Recording state
  recordingStatus: RecordingStatus;
  setRecordingStatus: (status: RecordingStatus) => void;

  // UI state
  sidebarOpen: boolean;
  setSidebarOpen: (open: boolean) => void;
}

export const useAppStore = create<AppState>((set) => ({
  // Recording state
  recordingStatus: "Idle",
  setRecordingStatus: (status) => set({ recordingStatus: status }),

  // UI state
  sidebarOpen: true,
  setSidebarOpen: (open) => set({ sidebarOpen: open }),
}));
