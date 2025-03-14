-- Add MFA status to sessions table
ALTER TABLE sessions 
ADD COLUMN IF NOT EXISTS mfa_status VARCHAR(20) NOT NULL DEFAULT 'not_required';