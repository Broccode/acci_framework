-- Create verification_codes table
CREATE TABLE IF NOT EXISTS verification_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code VARCHAR(32) NOT NULL,
    verification_type VARCHAR(20) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    attempts INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    metadata JSONB
);

-- Create indexes for faster lookups
CREATE INDEX idx_verification_codes_tenant_user ON verification_codes(tenant_id, user_id);
CREATE INDEX idx_verification_codes_status ON verification_codes(status);
CREATE INDEX idx_verification_codes_expires_at ON verification_codes(expires_at);