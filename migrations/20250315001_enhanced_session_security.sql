-- Migration: 20250315001_enhanced_session_security
-- Description: Adds enhanced security features to session management including geolocation tracking, anomaly detection, and advanced fingerprinting

-- Up Migration

-- Add new session invalidation reasons
ALTER TYPE session_invalidation_reason ADD VALUE IF NOT EXISTS 'SUSPICIOUS_ACTIVITY';
ALTER TYPE session_invalidation_reason ADD VALUE IF NOT EXISTS 'SUSPICIOUS_LOCATION';
ALTER TYPE session_invalidation_reason ADD VALUE IF NOT EXISTS 'CONCURRENT_SESSION_LIMIT';

-- Create table for session locations
CREATE TABLE IF NOT EXISTS session_locations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    country_code VARCHAR(2) NOT NULL,
    region_code VARCHAR(10),
    city VARCHAR(100),
    timezone VARCHAR(50),
    ip_address INET NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    is_suspicious BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB,
    CONSTRAINT valid_coordinates CHECK (
        latitude BETWEEN -90 AND 90 AND
        longitude BETWEEN -180 AND 180
    )
);

CREATE INDEX IF NOT EXISTS idx_session_locations_session ON session_locations(session_id);
CREATE INDEX IF NOT EXISTS idx_session_locations_user ON session_locations(user_id);
CREATE INDEX IF NOT EXISTS idx_session_locations_ip ON session_locations(ip_address);
CREATE INDEX IF NOT EXISTS idx_session_locations_recorded ON session_locations(recorded_at);
CREATE INDEX IF NOT EXISTS idx_session_locations_country ON session_locations(country_code);
CREATE INDEX IF NOT EXISTS idx_session_locations_suspicious ON session_locations(is_suspicious) WHERE is_suspicious = true;

-- Create table for enhanced session fingerprints
CREATE TABLE IF NOT EXISTS enhanced_session_fingerprints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_fingerprint JSONB NOT NULL,
    network_info JSONB,
    browser_capabilities JSONB,
    canvas_fingerprint VARCHAR(255),
    webgl_fingerprint VARCHAR(255),
    audio_fingerprint VARCHAR(255),
    font_fingerprint VARCHAR(255),
    behavior_metrics JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_enhanced_fingerprints_session ON enhanced_session_fingerprints(session_id);
CREATE INDEX IF NOT EXISTS idx_enhanced_fingerprints_user ON enhanced_session_fingerprints(user_id);
CREATE INDEX IF NOT EXISTS idx_enhanced_fingerprints_canvas ON enhanced_session_fingerprints(canvas_fingerprint) 
    WHERE canvas_fingerprint IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_enhanced_fingerprints_webgl ON enhanced_session_fingerprints(webgl_fingerprint) 
    WHERE webgl_fingerprint IS NOT NULL;

-- Create table for session risk assessments
CREATE TABLE IF NOT EXISTS session_risk_assessments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    risk_score DECIMAL(5,4) NOT NULL CHECK (risk_score BETWEEN 0 AND 1),
    risk_factors JSONB NOT NULL,
    is_flagged BOOLEAN NOT NULL DEFAULT false,
    triggered_action BOOLEAN NOT NULL DEFAULT false,
    action_taken VARCHAR(50),
    metadata JSONB
);

CREATE INDEX IF NOT EXISTS idx_risk_assessments_session ON session_risk_assessments(session_id);
CREATE INDEX IF NOT EXISTS idx_risk_assessments_user ON session_risk_assessments(user_id);
CREATE INDEX IF NOT EXISTS idx_risk_assessments_score ON session_risk_assessments(risk_score);
CREATE INDEX IF NOT EXISTS idx_risk_assessments_flagged ON session_risk_assessments(is_flagged) 
    WHERE is_flagged = true;
CREATE INDEX IF NOT EXISTS idx_risk_assessments_triggered ON session_risk_assessments(triggered_action) 
    WHERE triggered_action = true;

-- Add more actions to session audit log constraint
ALTER TABLE session_audit_log DROP CONSTRAINT IF EXISTS valid_session_action;
ALTER TABLE session_audit_log ADD CONSTRAINT valid_session_action CHECK (action IN (
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
    'SESSION_INVALIDATED_SUSPICIOUS_ACTIVITY',
    'SESSION_INVALIDATED_SUSPICIOUS_LOCATION',
    'SESSION_INVALIDATED_CONCURRENT_LIMIT',
    'SESSION_ACTIVITY',
    'TOKEN_ROTATION_STARTED',
    'TOKEN_ROTATION_COMPLETED',
    'TOKEN_ROTATION_FAILED',
    'DEVICE_CHANGED',
    'DEVICE_VERIFICATION_STARTED',
    'DEVICE_VERIFICATION_COMPLETED',
    'DEVICE_VERIFICATION_FAILED',
    'LOCATION_TRACKED',
    'SUSPICIOUS_LOCATION_DETECTED',
    'FINGERPRINT_UPDATED',
    'RISK_ASSESSMENT_PERFORMED',
    'STEP_UP_AUTHENTICATION_REQUIRED',
    'CONCURRENT_SESSION_DETECTED',
    'OTHER_SESSIONS_TERMINATED'
));

-- Create function to track session locations
CREATE OR REPLACE FUNCTION track_session_location()
RETURNS TRIGGER AS $$
BEGIN
    -- This function would be called by an application function
    -- when a session's IP address is updated or when additional
    -- location information is provided
    
    INSERT INTO session_audit_log (
        session_id, user_id, action, details, ip_address, user_agent
    ) VALUES (
        NEW.session_id,
        NEW.user_id,
        'LOCATION_TRACKED',
        jsonb_build_object(
            'latitude', NEW.latitude,
            'longitude', NEW.longitude,
            'country_code', NEW.country_code,
            'city', NEW.city,
            'is_suspicious', NEW.is_suspicious
        ),
        NEW.ip_address::inet,
        NULL -- Would be populated by application
    );
    
    -- If the location is marked as suspicious, log it specially
    IF NEW.is_suspicious THEN
        INSERT INTO session_audit_log (
            session_id, user_id, action, details, ip_address, user_agent
        ) VALUES (
            NEW.session_id,
            NEW.user_id,
            'SUSPICIOUS_LOCATION_DETECTED',
            jsonb_build_object(
                'latitude', NEW.latitude,
                'longitude', NEW.longitude,
                'country_code', NEW.country_code,
                'city', NEW.city
            ),
            NEW.ip_address::inet,
            NULL -- Would be populated by application
        );
    END IF;
    
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for location tracking
CREATE TRIGGER session_location_tracker
    AFTER INSERT ON session_locations
    FOR EACH ROW
    EXECUTE FUNCTION track_session_location();

-- Create a function to find suspicious location changes
CREATE OR REPLACE FUNCTION detect_suspicious_locations(
    p_user_id UUID,
    p_max_distance_km DOUBLE PRECISION DEFAULT 500.0,
    p_time_threshold_minutes INTEGER DEFAULT 180  -- 3 hours
) RETURNS TABLE (
    current_location_id UUID,
    previous_location_id UUID,
    distance_km DOUBLE PRECISION,
    time_diff_minutes INTEGER,
    is_suspicious BOOLEAN
) AS $$
BEGIN
    RETURN QUERY WITH user_locations AS (
        SELECT 
            id,
            latitude,
            longitude,
            recorded_at,
            ROW_NUMBER() OVER (ORDER BY recorded_at DESC) as row_num
        FROM 
            session_locations
        WHERE 
            user_id = p_user_id
        ORDER BY 
            recorded_at DESC
    )
    SELECT 
        l1.id AS current_location_id,
        l2.id AS previous_location_id,
        -- Calculate distance using Pythagorean formula with Earth radius compensation
        -- This is an approximation but works for this purpose
        111.111 * SQRT(
            POW(ABS(l1.latitude - l2.latitude), 2) + 
            POW(ABS(l1.longitude - l2.longitude) * COS((l1.latitude + l2.latitude) * PI()/360), 2)
        ) AS distance_km,
        EXTRACT(EPOCH FROM (l1.recorded_at - l2.recorded_at))/60 AS time_diff_minutes,
        (
            111.111 * SQRT(
                POW(ABS(l1.latitude - l2.latitude), 2) + 
                POW(ABS(l1.longitude - l2.longitude) * COS((l1.latitude + l2.latitude) * PI()/360), 2)
            ) > p_max_distance_km AND
            EXTRACT(EPOCH FROM (l1.recorded_at - l2.recorded_at))/60 < p_time_threshold_minutes
        ) AS is_suspicious
    FROM 
        user_locations l1
    JOIN 
        user_locations l2
    ON 
        l1.row_num = l2.row_num - 1
    WHERE 
        l1.row_num <= 10;  -- Only check the last 10 locations
END;
$$ LANGUAGE plpgsql;

-- Function to enforce concurrent session limits
CREATE OR REPLACE FUNCTION enforce_concurrent_session_limits(
    p_user_id UUID,
    p_max_sessions INTEGER DEFAULT 5,
    p_exclude_session_id UUID DEFAULT NULL
) RETURNS INTEGER AS $$
DECLARE
    active_count INTEGER;
    excess_count INTEGER;
    invalidated_count INTEGER := 0;
BEGIN
    -- Count active sessions for this user
    SELECT COUNT(*) INTO active_count
    FROM sessions
    WHERE user_id = p_user_id
    AND is_valid = true
    AND (p_exclude_session_id IS NULL OR id != p_exclude_session_id);
    
    -- Calculate how many need to be invalidated
    excess_count := active_count - p_max_sessions;
    
    -- If we're over the limit, invalidate the oldest sessions
    IF excess_count > 0 THEN
        WITH oldest_sessions AS (
            SELECT id
            FROM sessions
            WHERE user_id = p_user_id
            AND is_valid = true
            AND (p_exclude_session_id IS NULL OR id != p_exclude_session_id)
            ORDER BY last_activity_at ASC
            LIMIT excess_count
        ),
        invalidated AS (
            UPDATE sessions
            SET is_valid = false,
                invalidated_reason = 'CONCURRENT_SESSION_LIMIT'
            FROM oldest_sessions
            WHERE sessions.id = oldest_sessions.id
            RETURNING sessions.id
        )
        SELECT COUNT(*) INTO invalidated_count FROM invalidated;
        
        -- Log the concurrent session limit action
        FOR i IN 1..invalidated_count LOOP
            INSERT INTO session_audit_log (
                session_id, 
                user_id, 
                action, 
                details
            ) VALUES (
                p_exclude_session_id,  -- Current session causing the limit
                p_user_id,
                'CONCURRENT_SESSION_DETECTED',
                jsonb_build_object(
                    'limit', p_max_sessions,
                    'actual', active_count,
                    'invalidated', invalidated_count
                )
            );
        END LOOP;
    END IF;
    
    RETURN invalidated_count;
END;
$$ LANGUAGE plpgsql;

-- Down Migration
/*
DROP FUNCTION IF EXISTS enforce_concurrent_session_limits(UUID, INTEGER, UUID);
DROP FUNCTION IF EXISTS detect_suspicious_locations(UUID, DOUBLE PRECISION, INTEGER);
DROP TRIGGER IF EXISTS session_location_tracker ON session_locations;
DROP FUNCTION IF EXISTS track_session_location();
DROP TABLE IF EXISTS session_risk_assessments;
DROP TABLE IF EXISTS enhanced_session_fingerprints;
DROP TABLE IF EXISTS session_locations;

-- Revert session_audit_log constraint
ALTER TABLE session_audit_log DROP CONSTRAINT IF EXISTS valid_session_action;
ALTER TABLE session_audit_log ADD CONSTRAINT valid_session_action CHECK (action IN (
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
));
*/ 