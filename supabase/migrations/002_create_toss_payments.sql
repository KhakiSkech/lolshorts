-- Create Toss Payments transaction tracking table
CREATE TABLE IF NOT EXISTS toss_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    license_id UUID NOT NULL REFERENCES licenses(id) ON DELETE CASCADE,

    -- Toss Payment identifiers
    payment_key TEXT NOT NULL UNIQUE, -- Toss의 결제 키
    order_id TEXT NOT NULL, -- 주문 ID
    transaction_id TEXT, -- Toss 거래 ID

    -- Payment details
    amount INTEGER NOT NULL, -- 결제 금액 (원화)
    method TEXT NOT NULL CHECK (method IN ('카드', '가상계좌', '계좌이체', '휴대폰', '간편결제')),
    status TEXT NOT NULL CHECK (status IN ('READY', 'IN_PROGRESS', 'DONE', 'CANCELED', 'PARTIAL_CANCELED', 'ABORTED', 'EXPIRED')) DEFAULT 'READY',

    -- Subscription details (if applicable)
    is_subscription BOOLEAN NOT NULL DEFAULT FALSE,
    subscription_period TEXT CHECK (subscription_period IN ('MONTHLY', 'YEARLY')),
    next_billing_date TIMESTAMPTZ,

    -- Timestamps
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_at TIMESTAMPTZ,
    canceled_at TIMESTAMPTZ,

    -- Webhook data
    webhook_received_at TIMESTAMPTZ,
    raw_webhook_data JSONB,

    -- Metadata
    metadata JSONB DEFAULT '{}',

    CONSTRAINT unique_payment_key UNIQUE (payment_key),
    CONSTRAINT unique_order_id UNIQUE (order_id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_toss_payments_user_id ON toss_payments(user_id);
CREATE INDEX IF NOT EXISTS idx_toss_payments_license_id ON toss_payments(license_id);
CREATE INDEX IF NOT EXISTS idx_toss_payments_status ON toss_payments(status);
CREATE INDEX IF NOT EXISTS idx_toss_payments_payment_key ON toss_payments(payment_key);
CREATE INDEX IF NOT EXISTS idx_toss_payments_created_at ON toss_payments(requested_at DESC);

-- Row Level Security
ALTER TABLE toss_payments ENABLE ROW LEVEL SECURITY;

-- Users can view their own payments
CREATE POLICY "Users can view own payments"
    ON toss_payments FOR SELECT
    USING (auth.uid() = user_id);

-- Only backend can insert/update payments (webhook handler)
-- For testing, allow authenticated users to insert
CREATE POLICY "Authenticated users can insert payments"
    ON toss_payments FOR INSERT
    WITH CHECK (auth.uid() = user_id);

-- Function to update license on successful payment
CREATE OR REPLACE FUNCTION process_toss_payment_success()
RETURNS TRIGGER AS $$
BEGIN
    -- Only process if status changed to DONE
    IF NEW.status = 'DONE' AND (OLD.status IS NULL OR OLD.status != 'DONE') THEN
        -- Update license to PRO
        UPDATE licenses
        SET
            tier = 'PRO',
            status = 'ACTIVE',
            toss_customer_id = NEW.user_id::TEXT,
            toss_subscription_id = NEW.payment_key,
            expires_at = CASE
                WHEN NEW.subscription_period = 'MONTHLY' THEN NOW() + INTERVAL '1 month'
                WHEN NEW.subscription_period = 'YEARLY' THEN NOW() + INTERVAL '1 year'
                ELSE NULL
            END
        WHERE id = NEW.license_id;

        -- Update approved timestamp
        NEW.approved_at = NOW();
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Trigger to update license on payment success
CREATE TRIGGER on_toss_payment_success
    BEFORE UPDATE ON toss_payments
    FOR EACH ROW
    EXECUTE FUNCTION process_toss_payment_success();

-- Function to handle payment cancellation
CREATE OR REPLACE FUNCTION process_toss_payment_cancel()
RETURNS TRIGGER AS $$
BEGIN
    -- Only process if status changed to CANCELED or ABORTED
    IF NEW.status IN ('CANCELED', 'ABORTED', 'EXPIRED') AND
       (OLD.status IS NULL OR OLD.status NOT IN ('CANCELED', 'ABORTED', 'EXPIRED')) THEN

        -- If this was the active subscription, downgrade to FREE
        UPDATE licenses
        SET
            tier = 'FREE',
            status = 'ACTIVE',
            toss_subscription_id = NULL,
            expires_at = NULL
        WHERE id = NEW.license_id
          AND toss_subscription_id = NEW.payment_key;

        -- Update canceled timestamp
        NEW.canceled_at = NOW();
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Trigger to handle payment cancellation
CREATE TRIGGER on_toss_payment_cancel
    BEFORE UPDATE ON toss_payments
    FOR EACH ROW
    EXECUTE FUNCTION process_toss_payment_cancel();

-- Function to get user's payment history
CREATE OR REPLACE FUNCTION get_user_payment_history(target_user_id UUID, limit_count INT DEFAULT 10)
RETURNS TABLE (
    payment_id UUID,
    payment_key TEXT,
    amount INTEGER,
    method TEXT,
    status TEXT,
    requested_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        id as payment_id,
        toss_payments.payment_key,
        toss_payments.amount,
        toss_payments.method,
        toss_payments.status,
        toss_payments.requested_at,
        toss_payments.approved_at
    FROM toss_payments
    WHERE user_id = target_user_id
    ORDER BY requested_at DESC
    LIMIT limit_count;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;
