-- Migration: 20250315003_create_session_mfa_enum
-- Description: Create enum for session MFA status tracking

-- Up Migration
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'session_mfa_status') THEN
        CREATE TYPE session_mfa_status AS ENUM (
            'NONE',
            'REQUIRED',
            'VERIFIED'
        );
    END IF;
END$$;

-- Add MFA status column to sessions table if not exists
DO $$
BEGIN
    IF NOT EXISTS(SELECT 1 FROM information_schema.columns 
                 WHERE table_name='sessions' AND column_name='mfa_status') THEN
        ALTER TABLE sessions ADD COLUMN mfa_status session_mfa_status NOT NULL DEFAULT 'NONE';
    END IF;
END$$;

-- Down Migration
/*
DROP TYPE IF EXISTS session_mfa_status CASCADE;
*/