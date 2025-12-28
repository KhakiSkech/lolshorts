import { createClient } from '@supabase/supabase-js';

// Development fallback key - DO NOT use in production
// This is the default Supabase local development anonymous key
const DEV_ANON_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyAgCiAgICAicm9sZSI6ICJhbm9uIiwKICAgICJpc3MiOiAic3VwYWJhc2UtZGVtbyIsCiAgICAiaWF0IjogMTY0MTc2OTIwMCwKICAgICJleHAiOiAxNzk5NTM1NjAwCn0.dc_X5iR_VP_qT0zsiyj_I_OZ2T9FtRU2BBNWN8Bu4GE';

// Use environment variables with fallbacks for development
const supabaseUrl = import.meta.env.VITE_SUPABASE_URL || 'http://localhost:54321';
const supabaseAnonKey = import.meta.env.VITE_SUPABASE_ANON_KEY || DEV_ANON_KEY;

// Warn if using development credentials in production
if (import.meta.env.PROD && !import.meta.env.VITE_SUPABASE_ANON_KEY) {
  console.warn('[Security] Using development Supabase credentials in production. Set VITE_SUPABASE_ANON_KEY environment variable.');
}

export const supabase = createClient(supabaseUrl, supabaseAnonKey, {
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
