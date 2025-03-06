-- Create totp_secrets table
CREATE TABLE IF NOT EXISTS totp_secrets (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    secret VARCHAR(255) NOT NULL,
    algorithm VARCHAR(10) NOT NULL DEFAULT 'SHA1',
    digits INTEGER NOT NULL DEFAULT 6,
    period INTEGER NOT NULL DEFAULT 30,
    recovery_codes JSONB NOT NULL DEFAULT '[]'::JSONB,
    enabled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ NULL,
    UNIQUE(user_id, tenant_id)
);

-- Add indexes
CREATE INDEX idx_totp_secrets_user_id ON totp_secrets(user_id);
CREATE INDEX idx_totp_secrets_tenant_id ON totp_secrets(tenant_id);

-- Add MFA related columns to users table
ALTER TABLE users ADD COLUMN has_mfa_enabled BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE users ADD COLUMN mfa_methods JSONB NOT NULL DEFAULT '[]'::JSONB;

-- Add MFA completed flag to sessions
ALTER TABLE sessions ADD COLUMN mfa_completed BOOLEAN NOT NULL DEFAULT false;