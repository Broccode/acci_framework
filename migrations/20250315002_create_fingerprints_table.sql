-- Migration: 20250315002_create_fingerprints_table
-- Description: Create table for storing browser fingerprints for security verification

-- Up Migration
CREATE TABLE IF NOT EXISTS fingerprints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    user_id UUID NOT NULL,
    fingerprint JSONB NOT NULL,
    first_seen TIMESTAMPTZ NOT NULL,
    last_seen TIMESTAMPTZ NOT NULL,
    last_ip INET NOT NULL,
    session_id UUID,
    trusted BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS idx_fingerprints_tenant_user ON fingerprints(tenant_id, user_id);
CREATE INDEX IF NOT EXISTS idx_fingerprints_last_seen ON fingerprints(last_seen);
CREATE INDEX IF NOT EXISTS idx_fingerprints_trusted ON fingerprints(trusted) WHERE trusted = true;

-- Down Migration
/*
DROP TABLE IF EXISTS fingerprints;
*/