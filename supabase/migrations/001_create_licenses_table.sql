-- Create licenses table for user subscription management
CREATE TABLE IF NOT EXISTS licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    tier TEXT NOT NULL CHECK (tier IN ('FREE', 'PRO')),
    status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'EXPIRED', 'CANCELLED')) DEFAULT 'ACTIVE',

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,

    -- Toss Payments tracking
    toss_customer_id TEXT,
    toss_billing_key TEXT,
    toss_subscription_id TEXT,

    -- Metadata
    metadata JSONB DEFAULT '{}',

    CONSTRAINT unique_user_license UNIQUE (user_id)
);

-- Create index on user_id for fast lookups
CREATE INDEX IF NOT EXISTS idx_licenses_user_id ON licenses(user_id);

-- Create index on status for filtering active licenses
CREATE INDEX IF NOT EXISTS idx_licenses_status ON licenses(status);

-- Row Level Security (RLS) Policies
ALTER TABLE licenses ENABLE ROW LEVEL SECURITY;

-- Users can only read their own license
CREATE POLICY "Users can view own license"
    ON licenses FOR SELECT
    USING (auth.uid() = user_id);

-- Only admins can insert/update/delete licenses (will be managed by backend)
-- For now, allow authenticated users to insert their own license (for testing)
CREATE POLICY "Users can insert own license"
    ON licenses FOR INSERT
    WITH CHECK (auth.uid() = user_id);

-- Function to automatically create FREE tier license for new users
CREATE OR REPLACE FUNCTION create_default_license()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO licenses (user_id, tier, status)
    VALUES (NEW.id, 'FREE', 'ACTIVE')
    ON CONFLICT (user_id) DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Trigger to create default license on user signup
CREATE TRIGGER on_auth_user_created
    AFTER INSERT ON auth.users
    FOR EACH ROW
    EXECUTE FUNCTION create_default_license();

-- Function to check if license is valid
CREATE OR REPLACE FUNCTION is_license_valid(license_id UUID)
RETURNS BOOLEAN AS $$
DECLARE
    license_status TEXT;
    license_expires TIMESTAMPTZ;
BEGIN
    SELECT status, expires_at INTO license_status, license_expires
    FROM licenses
    WHERE id = license_id;

    IF license_status = 'EXPIRED' OR license_status = 'CANCELLED' THEN
        RETURN FALSE;
    END IF;

    IF license_expires IS NOT NULL AND license_expires < NOW() THEN
        -- Auto-expire the license
        UPDATE licenses
        SET status = 'EXPIRED'
        WHERE id = license_id;
        RETURN FALSE;
    END IF;

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;
