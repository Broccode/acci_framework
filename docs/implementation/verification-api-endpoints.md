# Verification API Endpoints

This document outlines the API endpoints for the verification functionality in the ACCI Framework.

## Overview

The verification API provides endpoints for sending verification codes to users via Email or SMS and verifying those codes. These endpoints are part of the multi-factor authentication (MFA) flow and complement the existing TOTP implementation.

## Endpoints

### 1. Send Verification Code

**Endpoint:** `POST /auth/verify/send`

**Description:** Sends a verification code to a user via Email or SMS.

**Request Body:**
```json
{
  "user_id": "00000000-0000-0000-0000-000000000000",
  "verification_type": "email", // or "sms"
  "recipient": "user@example.com", // or phone number for SMS
  "tenant_id": "00000000-0000-0000-0000-000000000000",
  "session_token": "optional-session-token" // Optional
}
```

**Response (Success):**
```json
{
  "success": true,
  "data": {
    "success": true,
    "user_id": "00000000-0000-0000-0000-000000000000",
    "verification_type": "email"
  },
  "request_id": "123456789"
}
```

**Response (Error):**
```json
{
  "error": {
    "message": "Error message",
    "code": "ERROR_CODE",
    "status": 400
  },
  "request_id": "123456789"
}
```

**Error Codes:**
- `INVALID_USER_ID` (400): Invalid user ID format
- `INVALID_TENANT_ID` (400): Invalid tenant ID format
- `INVALID_VERIFICATION_TYPE` (400): Invalid verification type (must be "email" or "sms")
- `INVALID_SESSION` (401): Invalid session token
- `UNAUTHORIZED_SESSION` (403): Session does not belong to this user
- `RATE_LIMIT_EXCEEDED` (429): Rate limit exceeded
- `VALIDATION_ERROR` (400): General validation error
- `VERIFICATION_ERROR` (500): Server error during verification

### 2. Verify Code

**Endpoint:** `POST /auth/verify/code`

**Description:** Verifies a code sent to a user via Email or SMS.

**Request Body:**
```json
{
  "user_id": "00000000-0000-0000-0000-000000000000",
  "code": "123456", // The verification code
  "verification_type": "email", // or "sms"
  "tenant_id": "00000000-0000-0000-0000-000000000000",
  "session_token": "optional-session-token" // Optional, but recommended
}
```

**Response (Success):**
```json
{
  "success": true,
  "data": {
    "success": true,
    "user_id": "00000000-0000-0000-0000-000000000000",
    "verification_type": "email"
  },
  "request_id": "123456789"
}
```

**Response (Error):**
```json
{
  "error": {
    "message": "Error message",
    "code": "ERROR_CODE",
    "status": 400
  },
  "request_id": "123456789"
}
```

**Error Codes:**
- `INVALID_USER_ID` (400): Invalid user ID format
- `INVALID_TENANT_ID` (400): Invalid tenant ID format
- `INVALID_VERIFICATION_TYPE` (400): Invalid verification type (must be "email" or "sms")
- `INVALID_CODE` (400): Invalid verification code
- `CODE_EXPIRED` (400): Verification code has expired
- `TOO_MANY_ATTEMPTS` (400): Too many verification attempts
- `RATE_LIMIT_EXCEEDED` (429): Rate limit exceeded
- `VERIFICATION_ERROR` (500): Server error during verification

## Session Integration

If a `session_token` is provided:

1. When sending a verification code, the endpoint validates that the session belongs to the user.
2. When verifying a code:
   - On success, the session's MFA status is updated to `VERIFIED`.
   - On failure, the session's MFA status is updated to `FAILED`.

This allows the verification process to be integrated with the session management system, enabling proper MFA enforcement for protected resources.

## Rate Limiting

The verification endpoints include rate limiting to prevent abuse:

1. In-memory rate limiting: 3 requests per minute per user.
2. Database rate limiting: Tracking recent attempts and enforcing limits.

Users exceeding rate limits will receive a `429 Too Many Requests` response with the `RATE_LIMIT_EXCEEDED` error code.

## Implementation Notes

- The verification service supports both Email and SMS verification methods.
- Email verification uses either SMTP or SendGrid as the provider.
- SMS verification uses either Twilio or Vonage/Nexmo as the provider.
- Verification codes expire after a configurable period (default: 10 minutes).
- There's a maximum number of verification attempts (default: 5).
- All verification activities are tenant-aware for proper multi-tenant isolation.