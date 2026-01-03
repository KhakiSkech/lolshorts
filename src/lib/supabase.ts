import { createClient } from '@supabase/supabase-js';

// Environment helpers - works in both Vite and Jest
const getEnvVar = (key: string): string | undefined => {
  try {
    // Vite environment
    return (import.meta.env as Record<string, string>)?.[key];
  } catch {
    // Jest/Node environment
    return process.env[key];
  }
};

const isProd = (): boolean => {
  try {
    return import.meta.env?.PROD ?? false;
  } catch {
    return process.env.NODE_ENV === 'production';
  }
};

const isDev = (): boolean => {
  try {
    return import.meta.env?.DEV ?? false;
  } catch {
    return process.env.NODE_ENV !== 'production';
  }
};

// Supabase configuration - requires environment variables
const supabaseUrl = getEnvVar('VITE_SUPABASE_URL');
const supabaseAnonKey = getEnvVar('VITE_SUPABASE_ANON_KEY');

// Validate configuration in production
if (isProd()) {
  if (!supabaseUrl) {
    throw new Error('[Security] VITE_SUPABASE_URL is required in production');
  }
  if (!supabaseAnonKey) {
    throw new Error('[Security] VITE_SUPABASE_ANON_KEY is required in production');
  }
}

// Development fallback - only for local development
const devSupabaseUrl = supabaseUrl || 'http://localhost:54321';
const devSupabaseAnonKey = supabaseAnonKey || 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyAgCiAgICAicm9sZSI6ICJhbm9uIiwKICAgICJpc3MiOiAic3VwYWJhc2UtZGVtbyIsCiAgICAiaWF0IjogMTY0MTc2OTIwMCwKICAgICJleHAiOiAxNzk5NTM1NjAwCn0.dc_X5iR_VP_qT0zsiyj_I_OZ2T9FtRU2BBNWN8Bu4GE';

// Log warning in development if using fallbacks
if (isDev() && (!supabaseUrl || !supabaseAnonKey)) {
  console.info('[Dev] Using local Supabase development server. Set VITE_SUPABASE_URL and VITE_SUPABASE_ANON_KEY for remote server.');
}

export const supabase = createClient(devSupabaseUrl, devSupabaseAnonKey, {
  auth: {
    autoRefreshToken: true,
    persistSession: true,
    detectSessionInUrl: true,
    flowType: 'pkce',
  },
});

export type Database = {
  public: {
    Tables: {
      user_profiles: {
        Row: {
          id: string;
          email: string;
          display_name: string | null;
          avatar_url: string | null;
          tier: 'FREE' | 'PRO';
          subscription_status: 'active' | 'canceled' | 'expired' | 'trialing' | null;
          subscription_expires_at: string | null;
          created_at: string;
          updated_at: string;
        };
        Insert: {
          id: string;
          email: string;
          display_name?: string | null;
          avatar_url?: string | null;
          tier?: 'FREE' | 'PRO';
          subscription_status?: 'active' | 'canceled' | 'expired' | 'trialing' | null;
          subscription_expires_at?: string | null;
        };
        Update: {
          id?: string;
          email?: string;
          display_name?: string | null;
          avatar_url?: string | null;
          tier?: 'FREE' | 'PRO';
          subscription_status?: 'active' | 'canceled' | 'expired' | 'trialing' | null;
          subscription_expires_at?: string | null;
        };
      };
      games: {
        Row: {
          game_id: number;
          user_id: string;
          game_start_time: string;
          game_end_time: string | null;
          champion_name: string | null;
          game_mode: string | null;
          game_result: 'Victory' | 'Defeat' | 'Remake' | null;
          kills: number;
          deaths: number;
          assists: number;
          metadata: Record<string, unknown>;
          created_at: string;
          updated_at: string;
        };
        Insert: {
          game_id: number;
          user_id: string;
          game_start_time: string;
          game_end_time?: string | null;
          champion_name?: string | null;
          game_mode?: string | null;
          game_result?: 'Victory' | 'Defeat' | 'Remake' | null;
          kills?: number;
          deaths?: number;
          assists?: number;
          metadata?: Record<string, unknown>;
        };
        Update: {
          game_id?: number;
          user_id?: string;
          game_start_time?: string;
          game_end_time?: string | null;
          champion_name?: string | null;
          game_mode?: string | null;
          game_result?: 'Victory' | 'Defeat' | 'Remake' | null;
          kills?: number;
          deaths?: number;
          assists?: number;
          metadata?: Record<string, unknown>;
        };
      };
      clips: {
        Row: {
          id: number;
          game_id: number;
          user_id: string;
          file_path: string;
          event_type: string;
          event_time: number;
          priority: number;
          duration_secs: number;
          thumbnail_path: string | null;
          metadata: Record<string, unknown>;
          created_at: string;
        };
        Insert: {
          game_id: number;
          user_id: string;
          file_path: string;
          event_type: string;
          event_time: number;
          priority: number;
          duration_secs?: number;
          thumbnail_path?: string | null;
          metadata?: Record<string, unknown>;
        };
        Update: {
          game_id?: number;
          user_id?: string;
          file_path?: string;
          event_type?: string;
          event_time?: number;
          priority?: number;
          duration_secs?: number;
          thumbnail_path?: string | null;
          metadata?: Record<string, unknown>;
        };
      };
    };
  };
};
