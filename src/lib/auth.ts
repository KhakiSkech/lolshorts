import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import { persist } from "zustand/middleware";

// Types matching Rust backend
export interface User {
  id: string;
  email: string;
  tier: "Free" | "Pro";
  access_token: string;
  refresh_token: string;
  expires_at: number;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface SignupCredentials {
  email: string;
  password: string;
  confirm_password: string;
}

export interface LicenseInfo {
  tier: "Free" | "Pro";
  expires_at?: number;
  features: string[];
}

// Auth store with persistence
interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;

  // Actions
  login: (credentials: LoginCredentials) => Promise<void>;
  signup: (credentials: SignupCredentials) => Promise<void>;
  logout: () => Promise<void>;
  refreshToken: () => Promise<void>;
  checkAuth: () => Promise<void>;
  getLicenseInfo: () => Promise<LicenseInfo | null>;
  clearError: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,

      login: async (credentials) => {
        set({ isLoading: true, error: null });
        try {
          const user = await invoke<User>("login", {
            email: credentials.email,
            password: credentials.password,
          });
          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          });
        } catch (error) {
          set({
            error: error as string,
            isLoading: false,
          });
          throw error;
        }
      },

      signup: async (credentials) => {
        if (credentials.password !== credentials.confirm_password) {
          const error = "Passwords do not match";
          set({ error });
          throw new Error(error);
        }

        set({ isLoading: true, error: null });
        try {
          const user = await invoke<User>("signup", {
            email: credentials.email,
            password: credentials.password,
          });
          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          });
        } catch (error) {
          set({
            error: error as string,
            isLoading: false,
          });
          throw error;
        }
      },

      logout: async () => {
        set({ isLoading: true, error: null });
        try {
          await invoke("logout");
          set({
            user: null,
            isAuthenticated: false,
            isLoading: false,
            error: null,
          });
        } catch (error) {
          set({
            error: error as string,
            isLoading: false,
          });
          throw error;
        }
      },

      refreshToken: async () => {
        try {
          const user = await invoke<User>("refresh_token");
          set({ user });
        } catch (error) {
          console.error("Token refresh failed:", error);
          // If refresh fails, logout
          set({
            user: null,
            isAuthenticated: false,
            error: "Session expired. Please login again.",
          });
        }
      },

      checkAuth: async () => {
        set({ isLoading: true, error: null });
        try {
          const user = await invoke<User | null>("get_user_status");
          if (user) {
            // Check if token is about to expire (within 5 minutes)
            const now = Math.floor(Date.now() / 1000);
            const expiresIn = user.expires_at - now;

            if (expiresIn < 300) {
              // Less than 5 minutes, refresh
              await get().refreshToken();
            } else {
              set({
                user,
                isAuthenticated: true,
                isLoading: false,
              });
            }
          } else {
            set({
              user: null,
              isAuthenticated: false,
              isLoading: false,
            });
          }
        } catch (error) {
          set({
            error: error as string,
            isLoading: false,
            user: null,
            isAuthenticated: false,
          });
        }
      },

      getLicenseInfo: async () => {
        try {
          const license = await invoke<LicenseInfo>("get_license_info");
          return license;
        } catch (error) {
          console.error("Failed to get license info:", error);
          return null;
        }
      },

      clearError: () => set({ error: null }),
    }),
    {
      name: "lolshorts-auth",
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
);

// Auto-refresh token every 30 minutes
setInterval(() => {
  const { user, refreshToken } = useAuthStore.getState();
  if (user) {
    refreshToken();
  }
}, 30 * 60 * 1000);
