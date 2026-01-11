import { create } from "zustand";
import { persist } from "zustand/middleware";
import { supabase } from "./supabase";
import { authApi } from "@/api/auth";
import { getErrorKey } from "./errorMapper";
import type { User as SupabaseUser } from "@supabase/supabase-js";

// Safe development mode check for Jest compatibility
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const getIsDev = (): boolean => {
  try {
    // Check for test environment first (Jest)
    if (typeof process !== 'undefined' && process.env?.NODE_ENV === 'test') {
      return true;
    }
    // Check for Vite's import.meta.env (runtime check)
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const meta = (globalThis as any).__vite_import_meta_env__ || {};
    return meta.DEV === true;
  } catch {
    return false;
  }
};
const isDev = getIsDev();

// Types matching Supabase
export interface UserProfile {
  id: string;
  email: string;
  display_name: string | null;
  avatar_url: string | null;
  tier: "FREE" | "PRO";
  subscription_status: "active" | "canceled" | "expired" | "trialing" | null;
  subscription_expires_at: string | null;
}

export interface User {
  id: string;
  email: string;
  tier: "FREE" | "PRO";
  profile: UserProfile | null;
  supabaseUser: SupabaseUser;
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
  tier: "FREE" | "PRO";
  expires_at?: string;
  features: string[];
}

// Token refresh interval reference (module-level for cleanup)
let tokenRefreshInterval: ReturnType<typeof setInterval> | null = null;

// Auth store with persistence
interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;

  // Actions
  login: (credentials: LoginCredentials) => Promise<void>;
  loginWithGoogle: () => Promise<void>;
  signup: (credentials: SignupCredentials) => Promise<void>;
  logout: () => Promise<void>;
  refreshToken: () => Promise<void>;
  checkAuth: () => Promise<void>;
  getLicenseInfo: () => Promise<LicenseInfo | null>;
  clearError: () => void;
  startTokenRefresh: () => void;
  stopTokenRefresh: () => void;
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
          const { data, error } = await supabase.auth.signInWithPassword({
            email: credentials.email,
            password: credentials.password,
          });

          if (error) throw error;
          if (!data.user) throw new Error("No user returned");

          // Sync with Backend
          await authApi.setSession(
            data.session?.access_token || "",
            data.session?.refresh_token || "",
            data.user.id,
            data.user.email || ""
          );

          // Fetch user profile
          const { data: profile } = await supabase
            .from("user_profiles")
            .select("*")
            .eq("id", data.user.id)
            .single();

          const user: User = {
            id: data.user.id,
            email: data.user.email!,
            tier: profile?.tier || "FREE",
            profile: profile || null,
            supabaseUser: data.user,
          };

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          });
        } catch (error: unknown) {
          const errorKey = getErrorKey(error);
          set({
            error: errorKey,
            isLoading: false,
          });
          throw new Error(errorKey);
        }
      },

      loginWithGoogle: async () => {
        set({ isLoading: true, error: null });
        try {
          const { error } = await supabase.auth.signInWithOAuth({
            provider: "google",
            options: {
              redirectTo: window.location.origin,
            },
          });

          if (error) throw error;

          // OAuth flow continues in background
          set({ isLoading: false });
        } catch (error: unknown) {
          const errorKey = getErrorKey(error);
          set({
            error: errorKey,
            isLoading: false,
          });
          throw new Error(errorKey);
        }
      },

      signup: async (credentials) => {
        if (credentials.password !== credentials.confirm_password) {
          const errorKey = "errors.passwordsDoNotMatch";
          set({ error: errorKey });
          throw new Error(errorKey);
        }

        set({ isLoading: true, error: null });
        try {
          const { data, error } = await supabase.auth.signUp({
            email: credentials.email,
            password: credentials.password,
          });

          if (error) throw error;
          if (!data.user) throw new Error("No user returned");

          // Sync with Backend
          await authApi.setSession(
            data.session?.access_token || "",
            data.session?.refresh_token || "",
            data.user.id,
            data.user.email || ""
          );

          // Create user profile
          const { error: profileError } = await supabase
            .from("user_profiles")
            .insert({
              id: data.user.id,
              email: data.user.email!,
              tier: "FREE",
            });

          if (profileError) throw profileError;

          // Fetch created profile
          const { data: profile } = await supabase
            .from("user_profiles")
            .select("*")
            .eq("id", data.user.id)
            .single();

          const user: User = {
            id: data.user.id,
            email: data.user.email!,
            tier: "FREE",
            profile: profile || null,
            supabaseUser: data.user,
          };

          set({
            user,
            isAuthenticated: true,
            isLoading: false,
            error: null,
          });
        } catch (error: unknown) {
          const errorKey = getErrorKey(error);
          set({
            error: errorKey,
            isLoading: false,
          });
          throw new Error(errorKey);
        }
      },

      logout: async () => {
        set({ isLoading: true, error: null });
        try {
          // Stop token refresh before logging out
          get().stopTokenRefresh();

          const { error } = await supabase.auth.signOut();


          if (error) throw error;

          set({
            user: null,
            isAuthenticated: false,
            isLoading: false,
            error: null,
          });
        } catch (error: unknown) {
          const errorKey = getErrorKey(error);
          set({
            error: errorKey,
            isLoading: false,
          });
          throw new Error(errorKey);
        }
      },

      refreshToken: async () => {
        try {
          const { data, error } = await supabase.auth.refreshSession();
          if (error) throw error;

          if (data.user && data.session) {
            // Sync with Backend
            await authApi.setSession(
              data.session.access_token,
              data.session.refresh_token,
              data.user.id,
              data.user.email || ""
            );

            const { data: profile } = await supabase
              .from("user_profiles")
              .select("*")
              .eq("id", data.user.id)
              .single();

            const user: User = {
              id: data.user.id,
              email: data.user.email!,
              tier: profile?.tier || "FREE",
              profile: profile || null,
              supabaseUser: data.user,
            };

            set({ user });
          }
        } catch (error) {
          // Log error with context for debugging
          if (isDev) {
            console.error("[AuthStore] Token refresh failed:", error);
          }
          set({
            user: null,
            isAuthenticated: false,
            error: "errors.sessionExpired",
          });
        }
      },

      checkAuth: async () => {
        set({ isLoading: true, error: null });
        try {
          const { data: { session }, error } = await supabase.auth.getSession();

          if (error) throw error;

          if (session?.user) {
            // Sync with Backend
            await authApi.setSession(
              session.access_token,
              session.refresh_token,
              session.user.id,
              session.user.email || ""
            );

            const { data: profile } = await supabase
              .from("user_profiles")
              .select("*")
              .eq("id", session.user.id)
              .single();

            const user: User = {
              id: session.user.id,
              email: session.user.email!,
              tier: profile?.tier || "FREE",
              profile: profile || null,
              supabaseUser: session.user,
            };

            set({
              user,
              isAuthenticated: true,
              isLoading: false,
            });
          } else {
            set({
              user: null,
              isAuthenticated: false,
              isLoading: false,
            });
          }
        } catch (error: unknown) {
          set({
            error: getErrorKey(error),
            isLoading: false,
            user: null,
            isAuthenticated: false,
          });
        }
      },

      getLicenseInfo: async () => {
        try {
          const { user } = get();
          if (!user?.profile) return null;

          const license: LicenseInfo = {
            tier: user.profile.tier,
            expires_at: user.profile.subscription_expires_at || undefined,
            features: user.profile.tier === "PRO"
              ? ["unlimited_clips", "advanced_editor", "priority_support", "no_watermarks"]
              : ["basic_clips", "basic_editor"],
          };

          return license;
        } catch (error) {
          // Log error with context for debugging
          if (isDev) {
            console.error("[AuthStore] Failed to get license info:", error);
          }
          return null;
        }
      },

      clearError: () => set({ error: null }),

      startTokenRefresh: () => {
        // Clear any existing interval first
        if (tokenRefreshInterval) {
          clearInterval(tokenRefreshInterval);
        }

        // Get session info to calculate dynamic refresh interval
        const calculateRefreshInterval = (): number => {
          const user = get().user;
          if (!user || !user.supabaseUser) {
            return 30 * 60 * 1000; // Default 30 minutes
          }

          // Extract session expiration from Supabase user
          interface SessionExpiration {
            expires_at?: number;
            expires_in?: number;
          }
          const session = user.supabaseUser as unknown as SessionExpiration;
          const expiresAt = session?.expires_at;
          const expiresInSeconds = session?.expires_in;

          if (expiresAt) {
            const now = Math.floor(Date.now() / 1000);
            const timeUntilExpiry = expiresAt - now;
            // Refresh 5 minutes before expiry
            const refreshInterval = Math.max(timeUntilExpiry - 300, 60) * 1000;
            // Log in development only
            if (isDev) {
              console.log(`[AuthStore] Session expires in ${timeUntilExpiry}s, refreshing in ${refreshInterval / 1000}s`);
            }
            return refreshInterval;
          } else if (expiresInSeconds) {
            // Fallback to expires_in if expires_at is not available
            const refreshInterval = Math.max(expiresInSeconds - 300, 60) * 1000;
            // Log in development only
            if (isDev) {
              console.log(`[AuthStore] Session expires in ${expiresInSeconds}s, refreshing in ${refreshInterval / 1000}s`);
            }
            return refreshInterval;
          }

          // Default to 30 minutes if no session info available
          return 30 * 60 * 1000;
        };

        const refreshInterval = calculateRefreshInterval();

        // Auto-refresh token at calculated interval
        tokenRefreshInterval = setInterval(() => {
          const { user, refreshToken } = get();
          if (user) {
            refreshToken().catch((err) => {
              // Log error with context for debugging
              if (isDev) {
                console.error("[AuthStore] Token refresh failed:", err);
              }
            });
          }
        }, refreshInterval);
      },

      stopTokenRefresh: () => {
        if (tokenRefreshInterval) {
          clearInterval(tokenRefreshInterval);
          tokenRefreshInterval = null;
        }
      },
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

// Start token refresh on module load (will be stopped on logout)
// This is called once when the module is first imported
if (typeof window !== 'undefined') {
  useAuthStore.getState().startTokenRefresh();
}
