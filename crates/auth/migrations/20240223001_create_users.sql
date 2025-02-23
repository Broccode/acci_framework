-- Migration: 20240223001_create_users
-- Description: Creates the initial users table and audit log

-- Up Migration
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    last_login TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verification_token UUID,
    verification_token_expires_at TIMESTAMPTZ,
    reset_token UUID,
    reset_token_expires_at TIMESTAMPTZ,
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    last_failed_login_at TIMESTAMPTZ
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active = true;
CREATE INDEX idx_users_verification_token ON users(verification_token) WHERE verification_token IS NOT NULL;
CREATE INDEX idx_users_reset_token ON users(reset_token) WHERE reset_token IS NOT NULL;

CREATE TABLE user_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    action VARCHAR(50) NOT NULL,
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_action CHECK (action IN (
        'REGISTRATION',
        'LOGIN_SUCCESS',
        'LOGIN_FAILED',
        'PASSWORD_RESET_REQUEST',
        'PASSWORD_RESET_SUCCESS',
        'EMAIL_VERIFICATION_REQUEST',
        'EMAIL_VERIFICATION_SUCCESS',
        'ACCOUNT_DEACTIVATED',
        'ACCOUNT_ACTIVATED'
    ))
);

CREATE INDEX idx_audit_user_id ON user_audit_log(user_id);
CREATE INDEX idx_audit_action ON user_audit_log(action);
CREATE INDEX idx_audit_created_at ON user_audit_log(created_at);

-- Down Migration
DROP TABLE IF EXISTS user_audit_log;
DROP TABLE IF EXISTS users;
