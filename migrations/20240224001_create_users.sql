-- Migration: 20240223001_create_users
-- Description: Adds additional user fields and audit log

-- Up Migration
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS verification_token UUID,
    ADD COLUMN IF NOT EXISTS verification_token_expires_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS reset_token UUID,
    ADD COLUMN IF NOT EXISTS reset_token_expires_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS last_failed_login_at TIMESTAMPTZ;

-- Create missing indices
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_active ON users(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_users_verification_token ON users(verification_token) WHERE verification_token IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_users_reset_token ON users(reset_token) WHERE reset_token IS NOT NULL;

-- Add missing indices for user_audit_log (if not already created in initial migration)
CREATE INDEX IF NOT EXISTS idx_audit_user_id ON user_audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_action ON user_audit_log(action);
CREATE INDEX IF NOT EXISTS idx_audit_created_at ON user_audit_log(created_at);

-- Down Migration
ALTER TABLE users
    DROP COLUMN IF EXISTS verification_token,
    DROP COLUMN IF EXISTS verification_token_expires_at,
    DROP COLUMN IF EXISTS reset_token,
    DROP COLUMN IF EXISTS reset_token_expires_at,
    DROP COLUMN IF EXISTS failed_login_attempts,
    DROP COLUMN IF EXISTS last_failed_login_at;

DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_active;
DROP INDEX IF EXISTS idx_users_verification_token;
DROP INDEX IF EXISTS idx_users_reset_token;
DROP INDEX IF EXISTS idx_audit_user_id;
DROP INDEX IF EXISTS idx_audit_action;
DROP INDEX IF EXISTS idx_audit_created_at;
