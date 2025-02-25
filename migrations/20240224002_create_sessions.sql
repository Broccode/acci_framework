-- Migration: 20250224181436_create_sessions
-- Description: Creates tables for session management and token tracking

-- Up Migration

-- Create enum for session invalidation reasons
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'session_invalidation_reason') THEN
        CREATE TYPE session_invalidation_reason AS ENUM (
            'USER_LOGOUT',
            'ADMIN_ACTION',
            'PASSWORD_CHANGED',
            'SECURITY_BREACH',
            'INACTIVITY_TIMEOUT',
            'TOKEN_EXPIRED',
            'DEVICE_CHANGED',
            'MANUAL_INVALIDATION'
        );
    END IF;
END$$;

CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,           -- Argon2 hash of the session token
    previous_token_hash TEXT,           -- For token rotation tracking
    token_rotation_at TIMESTAMPTZ,      -- When the token was last rotated
    expires_at TIMESTAMPTZ NOT NULL,    -- Session expiration timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_activity_update_at TIMESTAMPTZ,-- For optimized activity tracking
    ip_address INET,                    -- IP address of the session
    user_agent TEXT,                    -- User agent string
    device_id TEXT,                     -- Unique device identifier
    device_fingerprint JSONB,           -- Additional device fingerprinting data
    is_valid BOOLEAN NOT NULL DEFAULT true,  -- Whether the session is still valid
    invalidated_reason session_invalidation_reason, -- Typed reason if session was invalidated
    metadata JSONB,                     -- Additional session metadata
    CONSTRAINT check_expiry CHECK (expires_at > created_at),
    CONSTRAINT check_activity CHECK (last_activity_at >= created_at)
);

-- Create indices for efficient session lookups and cleanup
CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_sessions_prev_token ON sessions(previous_token_hash) WHERE previous_token_hash IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_sessions_expiry ON sessions(expires_at) WHERE is_valid = true;
CREATE INDEX IF NOT EXISTS idx_sessions_activity ON sessions(last_activity_at) WHERE is_valid = true;
CREATE INDEX IF NOT EXISTS idx_sessions_valid ON sessions(is_valid) WHERE is_valid = true;
CREATE INDEX IF NOT EXISTS idx_sessions_device ON sessions(device_id) WHERE device_id IS NOT NULL;

-- Create session audit log with more granular actions
CREATE TABLE IF NOT EXISTS session_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL,
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    retention_expires_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP + INTERVAL '90 days',
    CONSTRAINT valid_session_action CHECK (action IN (
        'SESSION_CREATED',
        'SESSION_RENEWED',
        'SESSION_RENEW_ATTEMPT',
        'SESSION_RENEW_FAILED',
        'SESSION_EXPIRED',
        'SESSION_INVALIDATED_BY_ADMIN',
        'SESSION_INVALIDATED_BY_USER',
        'SESSION_INVALIDATED_DUE_TO_INACTIVITY',
        'SESSION_INVALIDATED_PASSWORD_CHANGED',
        'SESSION_INVALIDATED_SECURITY_BREACH',
        'SESSION_ACTIVITY',
        'TOKEN_ROTATION_STARTED',
        'TOKEN_ROTATION_COMPLETED',
        'TOKEN_ROTATION_FAILED',
        'DEVICE_CHANGED',
        'DEVICE_VERIFICATION_STARTED',
        'DEVICE_VERIFICATION_COMPLETED',
        'DEVICE_VERIFICATION_FAILED'
    ))
);

-- Create indices for session audit log
CREATE INDEX IF NOT EXISTS idx_session_audit_session ON session_audit_log(session_id);
CREATE INDEX IF NOT EXISTS idx_session_audit_user ON session_audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_session_audit_action ON session_audit_log(action);
CREATE INDEX IF NOT EXISTS idx_session_audit_created ON session_audit_log(created_at);
CREATE INDEX IF NOT EXISTS idx_session_audit_retention ON session_audit_log(retention_expires_at);

-- Create function to update last_activity_at with optimization
CREATE OR REPLACE FUNCTION update_session_activity()
RETURNS TRIGGER AS $$
BEGIN
    -- Only update last_activity_at if more than 5 minutes have passed
    IF NEW.last_activity_update_at IS NULL OR
       CURRENT_TIMESTAMP - NEW.last_activity_update_at > INTERVAL '5 minutes' THEN
        NEW.last_activity_at = CURRENT_TIMESTAMP;
        NEW.last_activity_update_at = CURRENT_TIMESTAMP;
    END IF;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for optimized session activity tracking
CREATE TRIGGER session_activity_tracker
    BEFORE UPDATE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_session_activity();

-- Create function to automatically log session changes
CREATE OR REPLACE FUNCTION log_session_change()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO session_audit_log (
            session_id, user_id, action, details, ip_address, user_agent
        ) VALUES (
            NEW.id,
            NEW.user_id,
            'SESSION_CREATED',
            jsonb_build_object(
                'expires_at', NEW.expires_at,
                'device_id', NEW.device_id,
                'device_fingerprint', NEW.device_fingerprint
            ),
            NEW.ip_address,
            NEW.user_agent
        );
    ELSIF TG_OP = 'UPDATE' THEN
        -- Log session invalidation with specific reasons
        IF NEW.is_valid = false AND OLD.is_valid = true THEN
            INSERT INTO session_audit_log (
                session_id, user_id, action, details, ip_address, user_agent
            ) VALUES (
                NEW.id,
                NEW.user_id,
                CASE NEW.invalidated_reason
                    WHEN 'USER_LOGOUT' THEN 'SESSION_INVALIDATED_BY_USER'
                    WHEN 'ADMIN_ACTION' THEN 'SESSION_INVALIDATED_BY_ADMIN'
                    WHEN 'INACTIVITY_TIMEOUT' THEN 'SESSION_INVALIDATED_DUE_TO_INACTIVITY'
                    WHEN 'PASSWORD_CHANGED' THEN 'SESSION_INVALIDATED_PASSWORD_CHANGED'
                    WHEN 'SECURITY_BREACH' THEN 'SESSION_INVALIDATED_SECURITY_BREACH'
                    ELSE 'SESSION_EXPIRED'
                END,
                jsonb_build_object(
                    'reason', NEW.invalidated_reason,
                    'device_id', NEW.device_id,
                    'device_fingerprint', NEW.device_fingerprint
                ),
                NEW.ip_address,
                NEW.user_agent
            );
        END IF;

        -- Log token rotations
        IF NEW.token_hash != OLD.token_hash THEN
            INSERT INTO session_audit_log (
                session_id, user_id, action, details, ip_address, user_agent
            ) VALUES (
                NEW.id,
                NEW.user_id,
                'TOKEN_ROTATION_COMPLETED',
                jsonb_build_object(
                    'rotation_time', NEW.token_rotation_at,
                    'device_id', NEW.device_id
                ),
                NEW.ip_address,
                NEW.user_agent
            );
        END IF;

        -- Log device changes with fingerprint comparison
        IF NEW.device_id IS DISTINCT FROM OLD.device_id OR
           NEW.device_fingerprint IS DISTINCT FROM OLD.device_fingerprint THEN
            INSERT INTO session_audit_log (
                session_id, user_id, action, details, ip_address, user_agent
            ) VALUES (
                NEW.id,
                NEW.user_id,
                'DEVICE_CHANGED',
                jsonb_build_object(
                    'old_device', OLD.device_id,
                    'new_device', NEW.device_id,
                    'old_fingerprint', OLD.device_fingerprint,
                    'new_fingerprint', NEW.device_fingerprint
                ),
                NEW.ip_address,
                NEW.user_agent
            );
        END IF;
    END IF;
    RETURN NULL;
END;
$$ language 'plpgsql';

-- Create trigger for session audit logging
CREATE TRIGGER session_audit_logger
    AFTER INSERT OR UPDATE ON sessions
    FOR EACH ROW
    EXECUTE FUNCTION log_session_change();

-- Create function for session cleanup
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS INTEGER AS $$
DECLARE
    cleaned_count INTEGER;
BEGIN
    -- Invalidate expired sessions
    UPDATE sessions
    SET is_valid = false,
        invalidated_reason = 'TOKEN_EXPIRED'
    WHERE is_valid = true
    AND expires_at < CURRENT_TIMESTAMP;

    -- Delete sessions that have been invalid for more than 90 days
    WITH deleted AS (
        DELETE FROM sessions
        WHERE is_valid = false
        AND last_activity_at < CURRENT_TIMESTAMP - INTERVAL '90 days'
        RETURNING id
    )
    SELECT COUNT(*) INTO cleaned_count FROM deleted;

    -- Cleanup audit logs older than retention period
    DELETE FROM session_audit_log
    WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '90 days';

    RETURN cleaned_count;
END;
$$ language 'plpgsql';

/*
-- Down Migration (kommentiert, damit die Migration nicht sofort rückgängig gemacht wird)
-- DROP FUNCTION IF EXISTS cleanup_expired_sessions();
-- DROP TRIGGER IF EXISTS session_audit_logger ON sessions;
-- DROP FUNCTION IF EXISTS log_session_change();
-- DROP TRIGGER IF EXISTS session_activity_tracker ON sessions;
-- DROP FUNCTION IF EXISTS update_session_activity();
-- DROP TABLE IF EXISTS session_audit_log;
-- DROP TABLE IF EXISTS sessions;
-- DROP TYPE IF EXISTS session_invalidation_reason;
*/
