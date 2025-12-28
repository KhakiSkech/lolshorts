-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create auth schema if needed
CREATE SCHEMA IF NOT EXISTS auth;

-- Create auth.users table (simplified version for local dev)
CREATE TABLE IF NOT EXISTS auth.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT UNIQUE NOT NULL,
    encrypted_password TEXT,
    email_confirmed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create custom user profiles table
CREATE TABLE IF NOT EXISTS public.user_profiles (
    id UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,
    email TEXT NOT NULL UNIQUE,
    display_name TEXT,
    avatar_url TEXT,
    tier TEXT NOT NULL DEFAULT 'FREE' CHECK (tier IN ('FREE', 'PRO')),
    subscription_status TEXT CHECK (subscription_status IN ('active', 'canceled', 'expired', 'trialing')),
    subscription_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create games table
CREATE TABLE IF NOT EXISTS public.games (
    game_id BIGINT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES public.user_profiles(id) ON DELETE CASCADE,
    game_start_time TIMESTAMPTZ NOT NULL,
    game_end_time TIMESTAMPTZ,
    champion_name TEXT,
    game_mode TEXT,
    game_result TEXT CHECK (game_result IN ('Victory', 'Defeat', 'Remake')),
    kills INTEGER DEFAULT 0,
    deaths INTEGER DEFAULT 0,
    assists INTEGER DEFAULT 0,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create clips table
CREATE TABLE IF NOT EXISTS public.clips (
    id BIGSERIAL PRIMARY KEY,
    game_id BIGINT NOT NULL REFERENCES public.games(game_id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES public.user_profiles(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_time DOUBLE PRECISION NOT NULL,
    priority INTEGER NOT NULL CHECK (priority >= 1 AND priority <= 5),
    duration_secs DOUBLE PRECISION NOT NULL DEFAULT 30.0,
    thumbnail_path TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create auto_edit_results table
CREATE TABLE IF NOT EXISTS public.auto_edit_results (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES public.user_profiles(id) ON DELETE CASCADE,
    job_id UUID NOT NULL DEFAULT uuid_generate_v4(),
    output_file_path TEXT NOT NULL,
    duration_secs DOUBLE PRECISION NOT NULL,
    clips_used INTEGER NOT NULL DEFAULT 0,
    file_size_bytes BIGINT,
    template_name TEXT,
    with_music BOOLEAN DEFAULT FALSE,
    status TEXT NOT NULL DEFAULT 'completed' CHECK (status IN ('completed', 'failed')),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create youtube_uploads table
CREATE TABLE IF NOT EXISTS public.youtube_uploads (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES public.user_profiles(id) ON DELETE CASCADE,
    video_id TEXT UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'uploading', 'processing', 'completed', 'failed')),
    upload_progress INTEGER DEFAULT 0,
    file_path TEXT NOT NULL,
    thumbnail_path TEXT,
    privacy TEXT NOT NULL DEFAULT 'private' CHECK (privacy IN ('public', 'unlisted', 'private')),
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    uploaded_at TIMESTAMPTZ
);

-- Create quota_usage table (for FREE tier limits)
CREATE TABLE IF NOT EXISTS public.quota_usage (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES public.user_profiles(id) ON DELETE CASCADE,
    resource_type TEXT NOT NULL CHECK (resource_type IN ('auto_edit', 'upload', 'storage_gb')),
    usage_count INTEGER NOT NULL DEFAULT 0,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, resource_type, period_start)
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_games_user_id ON public.games(user_id);
CREATE INDEX IF NOT EXISTS idx_games_game_start_time ON public.games(game_start_time DESC);
CREATE INDEX IF NOT EXISTS idx_clips_game_id ON public.clips(game_id);
CREATE INDEX IF NOT EXISTS idx_clips_user_id ON public.clips(user_id);
CREATE INDEX IF NOT EXISTS idx_clips_priority ON public.clips(priority DESC);
CREATE INDEX IF NOT EXISTS idx_auto_edit_results_user_id ON public.auto_edit_results(user_id);
CREATE INDEX IF NOT EXISTS idx_youtube_uploads_user_id ON public.youtube_uploads(user_id);
CREATE INDEX IF NOT EXISTS idx_quota_usage_user_id ON public.quota_usage(user_id, resource_type);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Add triggers for updated_at
CREATE TRIGGER update_user_profiles_updated_at BEFORE UPDATE ON public.user_profiles
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_games_updated_at BEFORE UPDATE ON public.games
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Enable Row Level Security (RLS)
ALTER TABLE public.user_profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.games ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.clips ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auto_edit_results ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.youtube_uploads ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.quota_usage ENABLE ROW LEVEL SECURITY;

-- RLS Policies for user_profiles
CREATE POLICY "Users can view own profile" ON public.user_profiles
    FOR SELECT USING (auth.uid() = id);

CREATE POLICY "Users can update own profile" ON public.user_profiles
    FOR UPDATE USING (auth.uid() = id);

-- RLS Policies for games
CREATE POLICY "Users can view own games" ON public.games
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own games" ON public.games
    FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update own games" ON public.games
    FOR UPDATE USING (auth.uid() = user_id);

CREATE POLICY "Users can delete own games" ON public.games
    FOR DELETE USING (auth.uid() = user_id);

-- RLS Policies for clips
CREATE POLICY "Users can view own clips" ON public.clips
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own clips" ON public.clips
    FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can delete own clips" ON public.clips
    FOR DELETE USING (auth.uid() = user_id);

-- RLS Policies for auto_edit_results
CREATE POLICY "Users can view own results" ON public.auto_edit_results
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own results" ON public.auto_edit_results
    FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can delete own results" ON public.auto_edit_results
    FOR DELETE USING (auth.uid() = user_id);

-- RLS Policies for youtube_uploads
CREATE POLICY "Users can view own uploads" ON public.youtube_uploads
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own uploads" ON public.youtube_uploads
    FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update own uploads" ON public.youtube_uploads
    FOR UPDATE USING (auth.uid() = user_id);

CREATE POLICY "Users can delete own uploads" ON public.youtube_uploads
    FOR DELETE USING (auth.uid() = user_id);

-- RLS Policies for quota_usage
CREATE POLICY "Users can view own quota" ON public.quota_usage
    FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own quota" ON public.quota_usage
    FOR INSERT WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update own quota" ON public.quota_usage
    FOR UPDATE USING (auth.uid() = user_id);

-- Create database roles if they don't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'anon') THEN
        CREATE ROLE anon NOLOGIN;
    END IF;

    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'authenticated') THEN
        CREATE ROLE authenticated NOLOGIN;
    END IF;

    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'service_role') THEN
        CREATE ROLE service_role NOLOGIN;
    END IF;

    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'authenticator') THEN
        CREATE ROLE authenticator LOGIN PASSWORD 'postgres';
    END IF;

    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'supabase_admin') THEN
        CREATE ROLE supabase_admin LOGIN PASSWORD 'postgres';
    END IF;

    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'supabase_auth_admin') THEN
        CREATE ROLE supabase_auth_admin LOGIN PASSWORD 'postgres';
    END IF;

    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'supabase_storage_admin') THEN
        CREATE ROLE supabase_storage_admin LOGIN PASSWORD 'postgres';
    END IF;
END
$$;

-- Grant permissions
GRANT USAGE ON SCHEMA public TO anon, authenticated, service_role;
GRANT ALL ON ALL TABLES IN SCHEMA public TO anon, authenticated, service_role;
GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO anon, authenticated, service_role;
GRANT ALL ON ALL ROUTINES IN SCHEMA public TO anon, authenticated, service_role;

-- Grant authenticator the ability to switch to other roles
GRANT anon TO authenticator;
GRANT authenticated TO authenticator;
GRANT service_role TO authenticator;

-- Comment on tables
COMMENT ON TABLE public.user_profiles IS 'User profile information with subscription details';
COMMENT ON TABLE public.games IS 'League of Legends game sessions';
COMMENT ON TABLE public.clips IS 'Individual gameplay clips extracted from recordings';
COMMENT ON TABLE public.auto_edit_results IS 'Auto-generated highlight videos';
COMMENT ON TABLE public.youtube_uploads IS 'YouTube upload queue and status';
COMMENT ON TABLE public.quota_usage IS 'Tracks resource usage for FREE tier limits';
