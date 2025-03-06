# Advanced Authentication Implementation

## Overview

This document details the implementation of advanced authentication features for the ACCI Framework as part of Milestone 2. These enhancements expand upon the basic authentication implemented in Milestone 1, adding multi-factor authentication (MFA), risk-based authentication, enhanced session management, and OAuth2/OpenID Connect integration.

## Current Status (Pre-Implementation)

This is a planning document for Milestone 2. The implementation will follow the architecture and approaches described here.

## Multi-Factor Authentication

### TOTP Implementation

Time-based One-Time Password (TOTP) will be our primary second factor:

```rust
pub struct TotpService {
    secret_repository: Arc<dyn TotpSecretRepository>,
    config: TotpConfig,
}

pub struct TotpConfig {
    pub issuer: String,         // App name shown in authenticator apps
    pub algorithm: Algorithm,    // SHA1, SHA256, or SHA512
    pub digits: u32,            // Number of digits in code (usually 6)
    pub period: u64,            // Time step in seconds (usually 30)
    pub window_size: u64,       // Number of periods to check (+/-)
}

impl TotpService {
    /// Generate a new TOTP secret for a user
    pub async fn generate_totp_secret(
        &self, 
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> Result<TotpSecretInfo, AuthError> {
        // Generate cryptographically secure random bytes for the secret
        let mut rng = rand::thread_rng();
        let secret_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        
        // Encode as base32 for user entry into authenticator apps
        let secret = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret_bytes);
        
        // Generate a provisioning URI for QR code generation
        let uri = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm={}&digits={}&period={}",
            self.config.issuer,
            user_id,
            secret,
            urlencoding::encode(&self.config.issuer),
            self.config.algorithm.to_string(),
            self.config.digits,
            self.config.period
        );
        
        // Create recovery codes (one-time use backup codes)
        let recovery_codes = self.generate_recovery_codes()?;
        
        // Hash recovery codes before storing
        let hashed_recovery_codes = recovery_codes
            .iter()
            .map(|code| argon2::hash_encoded(
                code.as_bytes(),
                &generate_salt(),
                &argon2::Config::default(),
            ))
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| AuthError::InternalError(e.to_string()))?;
        
        // Store the secret and recovery codes
        let totp_secret = TotpSecret {
            user_id: user_id.clone(),
            tenant_id: tenant_id.clone(),
            secret,
            algorithm: self.config.algorithm.to_string(),
            digits: self.config.digits,
            period: self.config.period,
            recovery_codes: hashed_recovery_codes,
            created_at: Utc::now(),
            last_used_at: None,
            enabled: false,  // Not enabled until verified
        };
        
        self.secret_repository.save(&totp_secret).await?;
        
        // Return information needed for setup
        Ok(TotpSecretInfo {
            secret: totp_secret.secret,
            uri,
            recovery_codes,
        })
    }
    
    /// Verify a TOTP code provided by the user
    pub async fn verify_totp(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        code: &str,
    ) -> Result<bool, AuthError> {
        // Get the user's TOTP secret
        let totp_secret = self.secret_repository
            .get_by_user_id(user_id, tenant_id)
            .await?
            .ok_or_else(|| AuthError::MfaNotEnabled)?;
            
        // Verify if the TOTP code is valid
        let is_valid = self.verify_code(&totp_secret, code)?;
        
        // If valid, update last used time
        if is_valid {
            let mut updated_secret = totp_secret.clone();
            updated_secret.last_used_at = Some(Utc::now());
            updated_secret.enabled = true;  // Automatically enable on first successful verification
            self.secret_repository.save(&updated_secret).await?;
        }
        
        Ok(is_valid)
    }
    
    /// Generate recovery codes for backup access
    fn generate_recovery_codes(&self) -> Result<Vec<String>, AuthError> {
        let mut rng = rand::thread_rng();
        let mut codes = Vec::with_capacity(10);
        
        for _ in 0..10 {
            // Generate random bytes
            let random_bytes: Vec<u8> = (0..10).map(|_| rng.gen()).collect();
            
            // Encode with a custom alphabet for readability
            let alphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Removed similar-looking characters
            
            // Format as XXXX-XXXX-XXXX
            let chunks: Vec<String> = random_bytes
                .chunks(4)
                .map(|chunk| {
                    chunk.iter()
                        .map(|&b| alphabet.chars().nth((b as usize) % alphabet.len()).unwrap())
                        .collect()
                })
                .collect();
                
            let code = format!("{}-{}-{}", chunks[0], chunks[1], chunks[2]);
            codes.push(code);
        }
        
        Ok(codes)
    }
    
    /// Verify a TOTP code against the user's secret
    fn verify_code(&self, secret: &TotpSecret, code: &str) -> Result<bool, AuthError> {
        // Parse code as a number (removing spaces if present)
        let code = code.replace(" ", "");
        let code_num = code.parse::<u32>()
            .map_err(|_| AuthError::InvalidMfaCode)?;
            
        // Current time
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| AuthError::InternalError("Clock error".to_string()))?
            .as_secs();
            
        // Check current and adjacent time slots based on window size
        for i in -(self.config.window_size as i64)..=(self.config.window_size as i64) {
            let time_step = (now as i64 + i * secret.period as i64) as u64 / secret.period;
            
            // Calculate expected code for this time step
            if self.generate_code_for_time_step(secret, time_step)? == code_num {
                return Ok(true);
            }
        }
        
        // If no match found, check recovery codes
        self.verify_recovery_code(secret, &code).await
    }
    
    /// Generate TOTP code for a specific time step
    fn generate_code_for_time_step(&self, secret: &TotpSecret, time_step: u64) -> Result<u32, AuthError> {
        // Decode secret from base32
        let secret_bytes = base32::decode(
            base32::Alphabet::RFC4648 { padding: false },
            &secret.secret
        ).ok_or_else(|| AuthError::InternalError("Invalid secret format".to_string()))?;
        
        // Convert time step to bytes (big-endian)
        let time_bytes = time_step.to_be_bytes();
        
        // Create HMAC with the appropriate algorithm
        let mut mac = match secret.algorithm.as_str() {
            "SHA1" => Hmac::<Sha1>::new_from_slice(&secret_bytes)
                .map_err(|e| AuthError::InternalError(e.to_string()))?,
            "SHA256" => Hmac::<Sha256>::new_from_slice(&secret_bytes)
                .map_err(|e| AuthError::InternalError(e.to_string()))?,
            "SHA512" => Hmac::<Sha512>::new_from_slice(&secret_bytes)
                .map_err(|e| AuthError::InternalError(e.to_string()))?,
            _ => return Err(AuthError::InternalError("Unsupported algorithm".to_string())),
        };
        
        // Update MAC with time bytes
        mac.update(&time_bytes);
        
        // Finalize and get result
        let result = mac.finalize().into_bytes();
        
        // Get offset based on last nibble
        let offset = (result[result.len() - 1] & 0xf) as usize;
        
        // Get 4 bytes from result based on offset
        let binary = 
              ((result[offset] & 0x7f) as u32) << 24
            | ((result[offset + 1]) as u32) << 16
            | ((result[offset + 2]) as u32) << 8
            | ((result[offset + 3]) as u32);
            
        // Generate code of required length
        let code = binary % 10u32.pow(secret.digits);
        
        Ok(code)
    }
    
    /// Verify a recovery code
    async fn verify_recovery_code(&self, secret: &TotpSecret, code: &str) -> Result<bool, AuthError> {
        // Check each recovery code
        for (i, hashed_code) in secret.recovery_codes.iter().enumerate() {
            if argon2::verify_encoded(hashed_code, code.as_bytes())
                .map_err(|e| AuthError::InternalError(e.to_string()))?
            {
                // Recovery code is valid - now invalidate it
                let mut updated_secret = secret.clone();
                
                // Remove the used recovery code
                updated_secret.recovery_codes.remove(i);
                
                // Save updated recovery codes
                self.secret_repository.save(&updated_secret).await?;
                
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}
```

### SMS/Email Authentication

For alternative second factors, we'll implement SMS and email verification:

```rust
pub trait VerificationProvider: Send + Sync {
    async fn send_code(&self, recipient: &str, code: &str) -> Result<(), AuthError>;
}

pub struct SmsVerificationProvider {
    client: Arc<dyn SmsClient>,
    config: SmsConfig,
}

impl VerificationProvider for SmsVerificationProvider {
    async fn send_code(&self, phone_number: &str, code: &str) -> Result<(), AuthError> {
        // Format message
        let message = format!("{}: {} is your verification code", self.config.app_name, code);
        
        // Send SMS
        self.client.send_sms(phone_number, &message).await
            .map_err(|e| AuthError::ExternalServiceError(format!("SMS service error: {}", e)))
    }
}

pub struct EmailVerificationProvider {
    client: Arc<dyn EmailClient>,
    config: EmailConfig,
}

impl VerificationProvider for EmailVerificationProvider {
    async fn send_code(&self, email: &str, code: &str) -> Result<(), AuthError> {
        // Create email content
        let subject = format!("{} Verification Code", self.config.app_name);
        let body = format!("Your verification code is: {}. It will expire in {} minutes.", 
            code, self.config.expiration_minutes);
        
        // Send email
        self.client.send_email(email, &subject, &body).await
            .map_err(|e| AuthError::ExternalServiceError(format!("Email service error: {}", e)))
    }
}

pub struct VerificationService {
    repository: Arc<dyn VerificationRepository>,
    providers: HashMap<VerificationType, Arc<dyn VerificationProvider>>,
    config: VerificationConfig,
}

impl VerificationService {
    /// Generate and send a verification code
    pub async fn generate_and_send_code(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        verification_type: VerificationType,
        recipient: &str,
    ) -> Result<(), AuthError> {
        // Generate a random code
        let code = self.generate_code()?;
        
        // Calculate expiration time
        let expires_at = Utc::now() + chrono::Duration::minutes(self.config.expiration_minutes as i64);
        
        // Create verification record
        let verification = Verification {
            id: Uuid::new_v4(),
            user_id: user_id.clone(),
            tenant_id: tenant_id.clone(),
            verification_type,
            recipient: recipient.to_string(),
            code_hash: self.hash_code(&code)?,
            created_at: Utc::now(),
            expires_at,
            attempts: 0,
            verified: false,
        };
        
        // Store verification
        self.repository.save(&verification).await?;
        
        // Get the appropriate provider
        let provider = self.providers.get(&verification_type)
            .ok_or_else(|| AuthError::UnsupportedVerificationType)?;
            
        // Send the code
        provider.send_code(recipient, &code).await?;
        
        Ok(())
    }
    
    /// Verify a code provided by the user
    pub async fn verify_code(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        verification_type: VerificationType,
        recipient: &str,
        code: &str,
    ) -> Result<bool, AuthError> {
        // Get the verification record
        let verification = self.repository
            .get_active(user_id, tenant_id, &verification_type, recipient)
            .await?
            .ok_or_else(|| AuthError::VerificationNotFound)?;
            
        // Check if expired
        if Utc::now() > verification.expires_at {
            return Err(AuthError::VerificationExpired);
        }
        
        // Check if max attempts exceeded
        if verification.attempts >= self.config.max_attempts {
            return Err(AuthError::MaxAttemptsExceeded);
        }
        
        // Update attempts
        let mut updated = verification.clone();
        updated.attempts += 1;
        
        // Verify code
        let is_valid = argon2::verify_encoded(&verification.code_hash, code.as_bytes())
            .map_err(|e| AuthError::InternalError(e.to_string()))?;
            
        if is_valid {
            // Mark as verified
            updated.verified = true;
        }
        
        // Save updated verification
        self.repository.save(&updated).await?;
        
        Ok(is_valid)
    }
    
    /// Generate a random verification code
    fn generate_code(&self) -> Result<String, AuthError> {
        // Generate random digits
        let mut rng = rand::thread_rng();
        let code: String = (0..self.config.code_length)
            .map(|_| rng.gen_range(0..10).to_string())
            .collect();
            
        Ok(code)
    }
    
    /// Hash a verification code
    fn hash_code(&self, code: &str) -> Result<String, AuthError> {
        let salt = generate_salt();
        
        argon2::hash_encoded(
            code.as_bytes(),
            &salt,
            &argon2::Config::default(),
        )
        .map_err(|e| AuthError::InternalError(e.to_string()))
    }
}
```

### WebAuthn/FIDO2 Support

For passwordless authentication, we'll implement WebAuthn/FIDO2:

```rust
pub struct WebAuthnService {
    repository: Arc<dyn WebAuthnCredentialRepository>,
    webauthn: Webauthn,
}

impl WebAuthnService {
    pub fn new(
        repository: Arc<dyn WebAuthnCredentialRepository>,
        config: WebAuthnConfig,
    ) -> Result<Self, AuthError> {
        // Create WebAuthn configuration
        let rp_id = config.relying_party_id;
        let rp_origin = Url::parse(&config.relying_party_origin)
            .map_err(|e| AuthError::ConfigurationError(e.to_string()))?;
            
        let builder = WebauthnBuilder::new(rp_id, &rp_origin)
            .map_err(|e| AuthError::ConfigurationError(e.to_string()))?
            .rp_name(&config.relying_party_name);
            
        // Allow passkeys (resident keys)
        let builder = builder.allow_passkeys();
        
        // Set user verification preference
        let builder = match config.user_verification {
            UserVerificationPolicy::Required => builder.user_verification_required(),
            UserVerificationPolicy::Preferred => builder.user_verification_preferred(),
            UserVerificationPolicy::Discouraged => builder.user_verification_discouraged(),
        };
        
        // Finalize webauthn
        let webauthn = builder.build()
            .map_err(|e| AuthError::ConfigurationError(e.to_string()))?;
            
        Ok(Self {
            repository,
            webauthn,
        })
    }
    
    /// Start credential registration
    pub async fn start_registration(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        user_name: &str,
        user_display_name: &str,
    ) -> Result<RegistrationState, AuthError> {
        // Convert user ID to WebAuthn format
        let webauthn_user_id = user_id.as_bytes().to_vec();
        
        // Create a challenge for the user
        let (challenge, state) = self.webauthn
            .start_passkey_registration(
                webauthn_user_id,
                user_name,
                user_display_name,
                None, // No exclude credentials for now
            )
            .map_err(|e| AuthError::InternalError(e.to_string()))?;
            
        // Convert to serializable state
        let registration_state = RegistrationState {
            user_id: user_id.clone(),
            tenant_id: tenant_id.clone(),
            challenge: serde_json::to_value(challenge)
                .map_err(|e| AuthError::InternalError(e.to_string()))?,
            state: serde_json::to_value(state)
                .map_err(|e| AuthError::InternalError(e.to_string()))?,
            created_at: Utc::now(),
        };
        
        Ok(registration_state)
    }
    
    /// Finish credential registration
    pub async fn finish_registration(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        registration_state: RegistrationState,
        registration_response: &str,
    ) -> Result<WebAuthnCredential, AuthError> {
        // Parse the registration response
        let reg_response: RegisterPublicKeyCredential = serde_json::from_str(registration_response)
            .map_err(|e| AuthError::InvalidInput(format!("Invalid registration response: {}", e)))?;
            
        // Parse the state
        let state: PasskeyRegistration = serde_json::from_value(registration_state.state)
            .map_err(|e| AuthError::InternalError(format!("Failed to deserialize state: {}", e)))?;
            
        // Verify the registration
        let credential = self.webauthn
            .finish_passkey_registration(&reg_response, &state)
            .map_err(|e| AuthError::InvalidCredential(e.to_string()))?;
            
        // Create a webauthn credential record
        let cred = WebAuthnCredential {
            id: Uuid::new_v4(),
            user_id: user_id.clone(),
            tenant_id: tenant_id.clone(),
            credential_id: base64::encode(credential.cred_id().0.clone()),
            public_key: serde_json::to_string(&credential)
                .map_err(|e| AuthError::InternalError(e.to_string()))?,
            counter: credential.counter(),
            created_at: Utc::now(),
            last_used_at: None,
            name: None, // Optional name assigned by user
        };
        
        // Save credential
        self.repository.save(&cred).await?;
        
        Ok(cred)
    }
    
    /// Start authentication
    pub async fn start_authentication(
        &self,
        tenant_id: &TenantId,
    ) -> Result<AuthenticationState, AuthError> {
        // Get all credentials for the tenant
        let credentials = self.repository
            .get_all_for_tenant(tenant_id)
            .await?;
            
        // Convert to webauthn credentials
        let passkey_credentials: Vec<Passkey> = credentials
            .iter()
            .filter_map(|cred| {
                serde_json::from_str::<Passkey>(&cred.public_key).ok()
            })
            .collect();
            
        // Create an authentication challenge
        let (challenge, state) = self.webauthn
            .start_passkey_authentication(&passkey_credentials)
            .map_err(|e| AuthError::InternalError(e.to_string()))?;
            
        // Convert to serializable state
        let auth_state = AuthenticationState {
            tenant_id: tenant_id.clone(),
            challenge: serde_json::to_value(challenge)
                .map_err(|e| AuthError::InternalError(e.to_string()))?,
            state: serde_json::to_value(state)
                .map_err(|e| AuthError::InternalError(e.to_string()))?,
            created_at: Utc::now(),
        };
        
        Ok(auth_state)
    }
    
    /// Finish authentication
    pub async fn finish_authentication(
        &self,
        tenant_id: &TenantId,
        auth_state: AuthenticationState,
        auth_response: &str,
    ) -> Result<UserId, AuthError> {
        // Parse the authentication response
        let auth_response: PublicKeyCredential = serde_json::from_str(auth_response)
            .map_err(|e| AuthError::InvalidInput(format!("Invalid authentication response: {}", e)))?;
            
        // Parse the state
        let state: PasskeyAuthentication = serde_json::from_value(auth_state.state)
            .map_err(|e| AuthError::InternalError(format!("Failed to deserialize state: {}", e)))?;
            
        // Verify the authentication
        let auth_result = self.webauthn
            .finish_passkey_authentication(&auth_response, &state)
            .map_err(|e| AuthError::InvalidCredential(e.to_string()))?;
            
        // Find the credential in the database
        let credential_id = base64::encode(auth_result.cred_id().0.clone());
        let credential = self.repository
            .get_by_credential_id(&credential_id, tenant_id)
            .await?
            .ok_or_else(|| AuthError::CredentialNotFound)?;
            
        // Update credential counter
        let mut updated_credential = credential.clone();
        updated_credential.counter = auth_result.counter();
        updated_credential.last_used_at = Some(Utc::now());
        self.repository.save(&updated_credential).await?;
        
        Ok(credential.user_id)
    }
}
```

## Risk-Based Authentication

We'll implement a risk engine to assess authentication risk:

```rust
pub struct RiskEngine {
    repository: Arc<dyn RiskRepository>,
    config: RiskEngineConfig,
}

pub struct RiskScore {
    pub score: f64,                      // 0-100, higher = riskier
    pub factors: HashMap<String, f64>,   // Individual factor contributions
    pub requires_additional_verification: bool,
    pub suggested_actions: Vec<VerificationAction>,
}

pub enum VerificationAction {
    Mfa,
    EmailVerification,
    SmsVerification,
    SecurityQuestions,
    ManualReview,
}

impl RiskEngine {
    /// Assess authentication risk
    pub async fn assess_risk(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        auth_context: &AuthContext,
    ) -> Result<RiskScore, AuthError> {
        // Get user's risk profile
        let profile = self.repository
            .get_risk_profile(user_id, tenant_id)
            .await?
            .unwrap_or_else(|| self.create_default_profile(user_id, tenant_id));
            
        // Initial score is 0
        let mut score = 0.0;
        let mut factors = HashMap::new();
        
        // Check IP address
        score += self.assess_ip_risk(user_id, tenant_id, &profile, auth_context, &mut factors).await?;
        
        // Check device fingerprint
        score += self.assess_device_risk(user_id, tenant_id, &profile, auth_context, &mut factors).await?;
        
        // Check location
        score += self.assess_location_risk(user_id, tenant_id, &profile, auth_context, &mut factors).await?;
        
        // Check time patterns
        score += self.assess_time_risk(user_id, tenant_id, &profile, auth_context, &mut factors).await?;
        
        // Check behavioral biometrics
        if let Some(biometrics) = &auth_context.behavioral_biometrics {
            score += self.assess_biometric_risk(user_id, tenant_id, &profile, biometrics, &mut factors).await?;
        }
        
        // Determine required verification actions based on score
        let requires_additional_verification = score > self.config.verification_threshold;
        
        // Determine suggested actions
        let suggested_actions = if score > self.config.high_risk_threshold {
            vec![VerificationAction::Mfa, VerificationAction::EmailVerification]
        } else if score > self.config.medium_risk_threshold {
            vec![VerificationAction::Mfa]
        } else {
            Vec::new()
        };
        
        // Update risk profile with this authentication attempt
        let mut updated_profile = profile.clone();
        updated_profile.last_authentication = AuthAttempt {
            timestamp: Utc::now(),
            ip_address: auth_context.ip_address.clone(),
            device_fingerprint: auth_context.device_fingerprint.clone(),
            location: auth_context.geo_location.clone(),
            risk_score: score,
            successful: true, // Will be updated after authentication completes
        };
        self.repository.save_risk_profile(&updated_profile).await?;
        
        Ok(RiskScore {
            score,
            factors,
            requires_additional_verification,
            suggested_actions,
        })
    }
    
    /// Assess risk from IP address
    async fn assess_ip_risk(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        profile: &RiskProfile,
        auth_context: &AuthContext,
        factors: &mut HashMap<String, f64>,
    ) -> Result<f64, AuthError> {
        let mut score = 0.0;
        
        // Check if IP is in known IPs
        let is_known_ip = profile.known_ip_addresses
            .iter()
            .any(|known_ip| known_ip.ip_address == auth_context.ip_address);
            
        if !is_known_ip {
            // New IP address adds risk
            score += self.config.weights.new_ip;
            factors.insert("new_ip".to_string(), self.config.weights.new_ip);
            
            // Additional risk if this IP is known to be malicious
            if let Some(ip_reputation) = self.check_ip_reputation(&auth_context.ip_address).await? {
                if ip_reputation.is_malicious {
                    score += self.config.weights.malicious_ip;
                    factors.insert("malicious_ip".to_string(), self.config.weights.malicious_ip);
                }
            }
        }
        
        // Check for impossible travel
        if let (Some(last_auth), Some(current_location), Some(last_location)) = (
            profile.last_successful_authentication.as_ref(),
            auth_context.geo_location.as_ref(),
            profile.last_successful_authentication.as_ref().and_then(|a| a.location.as_ref()),
        ) {
            // Calculate time since last authentication
            let time_diff = Utc::now().signed_duration_since(last_auth.timestamp).num_hours();
            
            // Calculate distance between locations
            let distance_km = calculate_distance(
                last_location.latitude, last_location.longitude,
                current_location.latitude, current_location.longitude,
            );
            
            // Calculate expected travel speed in km/h
            let speed = if time_diff > 0 { distance_km / time_diff as f64 } else { f64::MAX };
            
            // If speed is unreasonably high (faster than typical flight speeds), consider it suspicious
            if speed > self.config.max_reasonable_speed_kmh {
                score += self.config.weights.impossible_travel;
                factors.insert("impossible_travel".to_string(), self.config.weights.impossible_travel);
            }
        }
        
        Ok(score)
    }
    
    /// Assess risk from device
    async fn assess_device_risk(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        profile: &RiskProfile,
        auth_context: &AuthContext,
        factors: &mut HashMap<String, f64>,
    ) -> Result<f64, AuthError> {
        let mut score = 0.0;
        
        // Check if device is in known devices
        let is_known_device = profile.known_devices
            .iter()
            .any(|device| device.fingerprint == auth_context.device_fingerprint);
            
        if !is_known_device {
            // New device adds risk
            score += self.config.weights.new_device;
            factors.insert("new_device".to_string(), self.config.weights.new_device);
        }
        
        // Check device fingerprint anomalies
        if let Some(anomalies) = self.check_device_anomalies(&auth_context.device_fingerprint).await? {
            if anomalies.spoofed_user_agent {
                score += self.config.weights.spoofed_user_agent;
                factors.insert("spoofed_user_agent".to_string(), self.config.weights.spoofed_user_agent);
            }
            
            if anomalies.bot_characteristics {
                score += self.config.weights.bot_characteristics;
                factors.insert("bot_characteristics".to_string(), self.config.weights.bot_characteristics);
            }
        }
        
        Ok(score)
    }
    
    /// Check IP reputation against threat intelligence
    async fn check_ip_reputation(&self, ip_address: &str) -> Result<Option<IpReputation>, AuthError> {
        // First check cache
        if let Some(reputation) = self.repository.get_ip_reputation(ip_address).await? {
            return Ok(Some(reputation));
        }
        
        // If not in cache, check with external service
        // This is a placeholder for actual threat intelligence integration
        let reputation = if ip_address.starts_with("192.168.") {
            // Placeholder logic - in reality would call a threat intelligence API
            Some(IpReputation {
                ip_address: ip_address.to_string(),
                score: 0.1,  // Low risk score for private IP
                is_malicious: false,
                categories: vec![],
                last_checked: Utc::now(),
            })
        } else {
            None
        };
        
        // Cache result if found
        if let Some(rep) = &reputation {
            self.repository.save_ip_reputation(rep).await?;
        }
        
        Ok(reputation)
    }
    
    // Other risk assessment methods follow similar patterns
}
```

## Enhanced Session Management

We'll improve session security with advanced features:

```rust
pub struct SessionService {
    repository: Arc<dyn SessionRepository>,
    config: SessionConfig,
}

impl SessionService {
    /// Create a new session with enhanced security
    pub async fn create_session(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        context: &SessionContext,
    ) -> Result<Session, AuthError> {
        // Generate session ID
        let session_id = SessionId::new();
        
        // Generate fingerprint from context
        let fingerprint = self.generate_fingerprint(context);
        
        // Generate tokens
        let (access_token, refresh_token) = self.generate_tokens(&session_id, user_id, tenant_id)?;
        
        // Set expiration based on context
        let expires_at = self.calculate_expiration(context);
        
        // Create session
        let session = Session {
            id: session_id,
            user_id: user_id.clone(),
            tenant_id: tenant_id.clone(),
            access_token,
            refresh_token,
            created_at: Utc::now(),
            expires_at,
            last_active_at: Utc::now(),
            fingerprint,
            ip_address: context.ip_address.clone(),
            user_agent: context.user_agent.clone(),
            geo_location: context.geo_location.clone(),
            risk_score: context.risk_score,
            device_info: context.device_info.clone(),
            mfa_completed: context.mfa_completed,
            parent_session_id: None,  // No parent session for new sessions
        };
        
        // Save session
        self.repository.save(&session).await?;
        
        // Check for suspicious concurrent sessions
        self.check_concurrent_sessions(user_id, tenant_id, &session).await?;
        
        Ok(session)
    }
    
    /// Generate a unique fingerprint from session context
    fn generate_fingerprint(&self, context: &SessionContext) -> String {
        // Create a fingerprint from multiple attributes
        let fingerprint_data = format!(
            "{}|{}|{}|{}",
            context.user_agent.as_deref().unwrap_or(""),
            context.ip_address,
            context.device_info.os.as_deref().unwrap_or(""),
            context.device_info.browser.as_deref().unwrap_or(""),
        );
        
        // Create SHA-256 hash
        let mut hasher = Sha256::new();
        hasher.update(fingerprint_data.as_bytes());
        let result = hasher.finalize();
        
        // Convert to hex string
        format!("{:x}", result)
    }
    
    /// Calculate session expiration based on context
    fn calculate_expiration(&self, context: &SessionContext) -> DateTime<Utc> {
        let now = Utc::now();
        
        if context.remember_me {
            // Longer session for "remember me"
            now + chrono::Duration::days(self.config.long_session_days as i64)
        } else if context.is_high_risk {
            // Shorter session for high risk
            now + chrono::Duration::minutes(self.config.high_risk_session_minutes as i64)
        } else if !context.mfa_completed {
            // Very short session for incomplete MFA
            now + chrono::Duration::minutes(self.config.mfa_pending_session_minutes as i64)
        } else {
            // Normal session length
            now + chrono::Duration::hours(self.config.normal_session_hours as i64)
        }
    }
    
    /// Validate a session token
    pub async fn validate_session(
        &self,
        access_token: &str,
        context: &SessionValidationContext,
    ) -> Result<SessionValidationResult, AuthError> {
        // Get session by token
        let session = self.repository
            .get_by_access_token(access_token)
            .await?
            .ok_or_else(|| AuthError::InvalidSession)?;
            
        // Check if session is expired
        if session.expires_at < Utc::now() {
            return Err(AuthError::SessionExpired);
        }
        
        // Generate current fingerprint
        let current_fingerprint = self.generate_fingerprint(&SessionContext {
            ip_address: context.ip_address.clone(),
            user_agent: context.user_agent.clone(),
            geo_location: context.geo_location.clone(),
            device_info: context.device_info.clone(),
            risk_score: 0.0,  // Not relevant for validation
            remember_me: false,  // Not relevant for validation
            is_high_risk: false,  // Will be determined
            mfa_completed: session.mfa_completed,  // Use session value
        });
        
        // Calculate fingerprint similarity (can use fuzzy matching for some tolerance)
        let fingerprint_matches = session.fingerprint == current_fingerprint;
        
        // Determine if additional verification is needed
        let requires_verification = if !fingerprint_matches {
            // Significant fingerprint change requires verification
            true
        } else if let (Some(session_location), Some(current_location)) = (
            session.geo_location.as_ref(),
            context.geo_location.as_ref(),
        ) {
            // Calculate distance between locations
            let distance_km = calculate_distance(
                session_location.latitude, session_location.longitude,
                current_location.latitude, current_location.longitude,
            );
            
            // Location changed significantly
            distance_km > self.config.location_change_threshold_km
        } else {
            false
        };
        
        // Update session with current activity
        if !requires_verification {
            let mut updated_session = session.clone();
            updated_session.last_active_at = Utc::now();
            
            // Only update fingerprint if it's a normal change (to avoid hijacking)
            if !fingerprint_matches && !requires_verification {
                updated_session.fingerprint = current_fingerprint;
            }
            
            self.repository.save(&updated_session).await?;
        }
        
        Ok(SessionValidationResult {
            session_id: session.id,
            user_id: session.user_id,
            tenant_id: session.tenant_id,
            is_valid: true,
            requires_verification,
            mfa_completed: session.mfa_completed,
        })
    }
    
    /// Check for suspicious concurrent sessions
    async fn check_concurrent_sessions(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        new_session: &Session,
    ) -> Result<(), AuthError> {
        // Get all active sessions for this user
        let active_sessions = self.repository
            .get_active_by_user_id(user_id, tenant_id)
            .await?;
            
        for existing_session in &active_sessions {
            // Skip the new session itself
            if existing_session.id == new_session.id {
                continue;
            }
            
            // Calculate time between sessions
            let time_diff = new_session.created_at
                .signed_duration_since(existing_session.created_at)
                .num_seconds()
                .abs();
                
            // If sessions are created very close in time but from different locations
            if time_diff < self.config.suspicious_session_time_threshold_seconds
                && existing_session.ip_address != new_session.ip_address
            {
                // Log suspicious activity
                // In a real implementation, this would alert security systems
                warn!(
                    "Suspicious concurrent sessions detected for user {} in tenant {}",
                    user_id, tenant_id
                );
                
                // Optionally terminate the older session or take other security measures
                if self.config.terminate_suspicious_sessions {
                    self.terminate_session(&existing_session.id).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get active sessions for a user with enhanced metadata
    pub async fn get_user_sessions(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> Result<Vec<SessionInfo>, AuthError> {
        // Get active sessions
        let sessions = self.repository
            .get_active_by_user_id(user_id, tenant_id)
            .await?;
            
        // Enrich with geo information
        let mut session_infos = Vec::with_capacity(sessions.len());
        
        for session in sessions {
            // Enrich with additional metadata
            let device_name = if let Some(device_info) = &session.device_info {
                format!(
                    "{} {} on {} {}",
                    device_info.browser.as_deref().unwrap_or("Unknown"),
                    device_info.browser_version.as_deref().unwrap_or(""),
                    device_info.os.as_deref().unwrap_or("Unknown"),
                    device_info.os_version.as_deref().unwrap_or(""),
                )
            } else {
                "Unknown device".to_string()
            };
            
            let location_name = if let Some(geo) = &session.geo_location {
                if let Some(city) = &geo.city {
                    format!("{}, {}", city, geo.country.as_deref().unwrap_or("Unknown"))
                } else {
                    geo.country.clone().unwrap_or_else(|| "Unknown location".to_string())
                }
            } else {
                "Unknown location".to_string()
            };
            
            session_infos.push(SessionInfo {
                id: session.id,
                created_at: session.created_at,
                expires_at: session.expires_at,
                last_active_at: session.last_active_at,
                device_name,
                location_name,
                ip_address: session.ip_address,
                is_current: false,  // Will be set by the caller
                is_mfa_completed: session.mfa_completed,
            });
        }
        
        Ok(session_infos)
    }
    
    /// Terminate all sessions except current
    pub async fn terminate_other_sessions(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        current_session_id: &SessionId,
    ) -> Result<usize, AuthError> {
        // Get all active sessions
        let sessions = self.repository
            .get_active_by_user_id(user_id, tenant_id)
            .await?;
            
        let mut terminated_count = 0;
        
        // Terminate all sessions except current
        for session in sessions {
            if session.id != *current_session_id {
                self.terminate_session(&session.id).await?;
                terminated_count += 1;
            }
        }
        
        Ok(terminated_count)
    }
    
    /// Refresh a session token
    pub async fn refresh_session(
        &self,
        refresh_token: &str,
        context: &SessionContext,
    ) -> Result<Session, AuthError> {
        // Get session by refresh token
        let session = self.repository
            .get_by_refresh_token(refresh_token)
            .await?
            .ok_or_else(|| AuthError::InvalidSession)?;
            
        // Check if session is expired
        if session.expires_at < Utc::now() {
            return Err(AuthError::SessionExpired);
        }
        
        // Generate current fingerprint
        let current_fingerprint = self.generate_fingerprint(context);
        
        // For refresh tokens, fingerprint should be very close to original
        if session.fingerprint != current_fingerprint {
            return Err(AuthError::SecurityValidationFailed);
        }
        
        // Create new session as child of current
        let new_session = Session {
            id: SessionId::new(),
            user_id: session.user_id.clone(),
            tenant_id: session.tenant_id.clone(),
            access_token: generate_token()?,
            refresh_token: generate_token()?,
            created_at: Utc::now(),
            expires_at: self.calculate_expiration(context),
            last_active_at: Utc::now(),
            fingerprint: current_fingerprint,
            ip_address: context.ip_address.clone(),
            user_agent: context.user_agent.clone(),
            geo_location: context.geo_location.clone(),
            risk_score: context.risk_score,
            device_info: context.device_info.clone(),
            mfa_completed: session.mfa_completed,  // Preserve MFA status
            parent_session_id: Some(session.id.clone()),
        };
        
        // Save new session
        self.repository.save(&new_session).await?;
        
        // Invalidate old session
        let mut old_session = session;
        old_session.expires_at = Utc::now();  // Expire immediately
        self.repository.save(&old_session).await?;
        
        Ok(new_session)
    }
}
```

## OAuth2/OIDC Integration

We'll implement OAuth2 and OpenID Connect:

```rust
pub struct OAuthProvider {
    config: OAuthConfig,
    client_repository: Arc<dyn OAuthClientRepository>,
    token_repository: Arc<dyn TokenRepository>,
    user_service: Arc<dyn UserService>,
}

impl OAuthProvider {
    /// Generate an authorization URL
    pub async fn generate_authorization_url(
        &self,
        client_id: &str,
        redirect_uri: &str,
        scope: &str,
        response_type: &str,
        state: &str,
        tenant_id: &TenantId,
        nonce: Option<&str>,
    ) -> Result<String, OAuthError> {
        // Validate client
        let client = self.client_repository
            .get_by_client_id(client_id, tenant_id)
            .await?
            .ok_or_else(|| OAuthError::InvalidClient("Unknown client".to_string()))?;
            
        // Validate redirect URI
        if !client.redirect_uris.contains(&redirect_uri.to_string()) {
            return Err(OAuthError::InvalidRedirectUri);
        }
        
        // Validate response type
        match response_type {
            "code" => {
                // Create authorization request
                let auth_request = AuthorizationRequest {
                    id: Uuid::new_v4(),
                    client_id: client_id.to_string(),
                    redirect_uri: redirect_uri.to_string(),
                    scope: scope.to_string(),
                    response_type: response_type.to_string(),
                    state: state.to_string(),
                    nonce: nonce.map(|s| s.to_string()),
                    tenant_id: tenant_id.clone(),
                    created_at: Utc::now(),
                    expires_at: Utc::now() + chrono::Duration::minutes(10),
                    code: None,
                    user_id: None,
                    is_authorized: false,
                };
                
                // Save request
                self.token_repository.save_authorization_request(&auth_request).await?;
                
                // Generate URL to authentication page (not the final authorization - that happens after login)
                let url = format!(
                    "{}/oauth/authorize?request_id={}",
                    self.config.authorization_endpoint,
                    auth_request.id
                );
                
                Ok(url)
            },
            "token" | "id_token" | "id_token token" => {
                // Implicit flow - not implemented in this example
                Err(OAuthError::UnsupportedResponseType)
            },
            _ => Err(OAuthError::UnsupportedResponseType),
        }
    }
    
    /// Process user authorization and generate code
    pub async fn authorize_request(
        &self,
        request_id: &Uuid,
        user_id: &UserId,
        tenant_id: &TenantId,
        scope_consent: &[String],
    ) -> Result<AuthorizationResponse, OAuthError> {
        // Get authorization request
        let mut auth_request = self.token_repository
            .get_authorization_request(request_id)
            .await?
            .ok_or_else(|| OAuthError::InvalidRequest("Unknown request".to_string()))?;
            
        // Check if request is expired
        if auth_request.expires_at < Utc::now() {
            return Err(OAuthError::ExpiredRequest);
        }
        
        // Check if request belongs to correct tenant
        if auth_request.tenant_id != *tenant_id {
            return Err(OAuthError::AccessDenied);
        }
        
        // Generate authorization code
        let code = generate_secure_token()?;
        
        // Update request
        auth_request.code = Some(code.clone());
        auth_request.user_id = Some(user_id.clone());
        auth_request.is_authorized = true;
        
        // Save updated request
        self.token_repository.save_authorization_request(&auth_request).await?;
        
        // Create response
        let response = AuthorizationResponse {
            redirect_uri: auth_request.redirect_uri,
            code: Some(code),
            state: auth_request.state,
            token: None,  // Only for implicit flow
            id_token: None,  // Only for implicit flow with OpenID Connect
        };
        
        Ok(response)
    }
    
    /// Exchange code for tokens
    pub async fn exchange_code(
        &self,
        code: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
        tenant_id: &TenantId,
    ) -> Result<TokenResponse, OAuthError> {
        // Validate client
        let client = self.client_repository
            .get_by_client_id(client_id, tenant_id)
            .await?
            .ok_or_else(|| OAuthError::InvalidClient("Unknown client".to_string()))?;
            
        // Validate client secret
        if !verify_secret(client_secret, &client.client_secret)? {
            return Err(OAuthError::InvalidClient("Invalid client credentials".to_string()));
        }
        
        // Get authorization request by code
        let auth_request = self.token_repository
            .get_authorization_request_by_code(code)
            .await?
            .ok_or_else(|| OAuthError::InvalidGrant("Invalid authorization code".to_string()))?;
            
        // Validate request is still valid
        if auth_request.expires_at < Utc::now() {
            return Err(OAuthError::InvalidGrant("Expired authorization code".to_string()));
        }
        
        // Validate client ID matches
        if auth_request.client_id != client_id {
            return Err(OAuthError::InvalidGrant("Client ID mismatch".to_string()));
        }
        
        // Validate redirect URI matches
        if auth_request.redirect_uri != redirect_uri {
            return Err(OAuthError::InvalidGrant("Redirect URI mismatch".to_string()));
        }
        
        // Get user
        let user = auth_request.user_id
            .as_ref()
            .ok_or_else(|| OAuthError::ServerError("No user associated with request".to_string()))?;
            
        let user_data = self.user_service
            .get_user(user, tenant_id)
            .await
            .map_err(|e| OAuthError::ServerError(format!("User service error: {}", e)))?
            .ok_or_else(|| OAuthError::ServerError("User not found".to_string()))?;
            
        // Parse scopes
        let scopes: Vec<&str> = auth_request.scope.split(' ').collect();
        
        // Generate access token
        let access_token = AccessToken {
            token: generate_secure_token()?,
            client_id: client_id.to_string(),
            user_id: user.clone(),
            tenant_id: tenant_id.clone(),
            scope: auth_request.scope.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),  // 1 hour token lifetime
        };
        
        // Generate refresh token if offline_access scope is requested
        let refresh_token = if scopes.contains(&"offline_access") {
            let token = RefreshToken {
                token: generate_secure_token()?,
                client_id: client_id.to_string(),
                user_id: user.clone(),
                tenant_id: tenant_id.clone(),
                scope: auth_request.scope.clone(),
                created_at: Utc::now(),
                expires_at: Utc::now() + chrono::Duration::days(30),  // 30 day refresh token
                access_token: access_token.token.clone(),
            };
            
            // Save refresh token
            self.token_repository.save_refresh_token(&token).await?;
            
            Some(token.token)
        } else {
            None
        };
        
        // Generate ID token if OpenID Connect
        let id_token = if scopes.contains(&"openid") {
            // Claims for ID token
            let mut claims = serde_json::Map::new();
            claims.insert("sub".to_string(), json!(user.to_string()));
            claims.insert("iss".to_string(), json!(self.config.issuer));
            claims.insert("aud".to_string(), json!(client_id));
            claims.insert("exp".to_string(), json!(access_token.expires_at.timestamp()));
            claims.insert("iat".to_string(), json!(Utc::now().timestamp()));
            
            // Add nonce if provided
            if let Some(nonce) = auth_request.nonce {
                claims.insert("nonce".to_string(), json!(nonce));
            }
            
            // Add user claims based on scopes
            if scopes.contains(&"profile") {
                claims.insert("name".to_string(), json!(user_data.name));
                // Add other profile claims
            }
            
            if scopes.contains(&"email") {
                claims.insert("email".to_string(), json!(user_data.email));
                claims.insert("email_verified".to_string(), json!(user_data.email_verified));
            }
            
            // Generate JWT
            let token = sign_jwt(&claims, &self.config.jwt_secret)?;
            Some(token)
        } else {
            None
        };
        
        // Save access token
        self.token_repository.save_access_token(&access_token).await?;
        
        // Invalidate authorization code to prevent reuse
        let mut updated_request = auth_request;
        updated_request.expires_at = Utc::now();  // Expire immediately
        self.token_repository.save_authorization_request(&updated_request).await?;
        
        // Create response
        let response = TokenResponse {
            access_token: access_token.token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,  // 1 hour in seconds
            refresh_token,
            id_token,
            scope: Some(auth_request.scope),
        };
        
        Ok(response)
    }
    
    /// Validate an access token
    pub async fn validate_token(
        &self,
        token: &str,
        tenant_id: &TenantId,
    ) -> Result<TokenValidationResult, OAuthError> {
        // Get token
        let access_token = self.token_repository
            .get_access_token(token)
            .await?
            .ok_or_else(|| OAuthError::InvalidToken)?;
            
        // Check if token is expired
        if access_token.expires_at < Utc::now() {
            return Err(OAuthError::TokenExpired);
        }
        
        // Check tenant
        if access_token.tenant_id != *tenant_id {
            return Err(OAuthError::AccessDenied);
        }
        
        // Get associated user
        let user = self.user_service
            .get_user(&access_token.user_id, tenant_id)
            .await
            .map_err(|e| OAuthError::ServerError(format!("User service error: {}", e)))?
            .ok_or_else(|| OAuthError::ServerError("User not found".to_string()))?;
            
        // Create result
        let result = TokenValidationResult {
            user_id: access_token.user_id,
            client_id: access_token.client_id,
            scope: access_token.scope,
            is_valid: true,
        };
        
        Ok(result)
    }
    
    /// Refresh an access token
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
        client_id: &str,
        client_secret: &str,
        tenant_id: &TenantId,
    ) -> Result<TokenResponse, OAuthError> {
        // Validate client
        let client = self.client_repository
            .get_by_client_id(client_id, tenant_id)
            .await?
            .ok_or_else(|| OAuthError::InvalidClient("Unknown client".to_string()))?;
            
        // Validate client secret
        if !verify_secret(client_secret, &client.client_secret)? {
            return Err(OAuthError::InvalidClient("Invalid client credentials".to_string()));
        }
        
        // Get refresh token
        let token = self.token_repository
            .get_refresh_token(refresh_token)
            .await?
            .ok_or_else(|| OAuthError::InvalidGrant("Invalid refresh token".to_string()))?;
            
        // Check if token is expired
        if token.expires_at < Utc::now() {
            return Err(OAuthError::InvalidGrant("Expired refresh token".to_string()));
        }
        
        // Check client association
        if token.client_id != client_id {
            return Err(OAuthError::InvalidGrant("Client mismatch".to_string()));
        }
        
        // Check tenant
        if token.tenant_id != *tenant_id {
            return Err(OAuthError::AccessDenied);
        }
        
        // Generate new access token
        let new_access_token = AccessToken {
            token: generate_secure_token()?,
            client_id: client_id.to_string(),
            user_id: token.user_id.clone(),
            tenant_id: tenant_id.clone(),
            scope: token.scope.clone(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
        };
        
        // Save new access token
        self.token_repository.save_access_token(&new_access_token).await?;
        
        // Update refresh token with reference to new access token
        let mut updated_token = token.clone();
        updated_token.access_token = new_access_token.token.clone();
        self.token_repository.save_refresh_token(&updated_token).await?;
        
        // Create response
        let response = TokenResponse {
            access_token: new_access_token.token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,  // 1 hour in seconds
            refresh_token: Some(refresh_token.to_string()),
            id_token: None,  // Not included in refresh flows
            scope: Some(token.scope.clone()),
        };
        
        Ok(response)
    }
}
```

## Audit Logging System

To maintain comprehensive security records:

```rust
pub struct AuditLogger {
    repository: Arc<dyn AuditLogRepository>,
    config: AuditConfig,
}

pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<UserId>,
    pub tenant_id: TenantId,
    pub request_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub operation: Option<String>,
    pub status: AuditEventStatus,
    pub details: serde_json::Value,
}

pub enum AuditEventType {
    Authentication,
    Authorization,
    UserManagement,
    TenantManagement,
    DataAccess,
    Configuration,
    SystemOperation,
}

impl AuditLogger {
    /// Log an authentication event
    pub async fn log_authentication_event(
        &self,
        event: AuthenticationEvent,
        tenant_id: &TenantId,
        context: &RequestContext,
    ) -> Result<(), LoggingError> {
        // Convert specific event to general audit event
        let audit_event = match event {
            AuthenticationEvent::LoginAttempt { user_id, success, error } => {
                let mut details = serde_json::Map::new();
                details.insert("success".to_string(), json!(success));
                
                if let Some(error) = error {
                    details.insert("error".to_string(), json!(error));
                }
                
                AuditEvent {
                    event_type: AuditEventType::Authentication,
                    timestamp: Utc::now(),
                    user_id: Some(user_id),
                    tenant_id: tenant_id.clone(),
                    request_id: context.request_id.clone(),
                    ip_address: context.ip_address.clone(),
                    user_agent: context.user_agent.clone(),
                    resource_id: None,
                    resource_type: None,
                    operation: Some("LOGIN".to_string()),
                    status: if success { AuditEventStatus::Success } else { AuditEventStatus::Failure },
                    details: serde_json::Value::Object(details),
                }
            },
            
            AuthenticationEvent::Logout { user_id, session_id } => {
                let mut details = serde_json::Map::new();
                details.insert("session_id".to_string(), json!(session_id.to_string()));
                
                AuditEvent {
                    event_type: AuditEventType::Authentication,
                    timestamp: Utc::now(),
                    user_id: Some(user_id),
                    tenant_id: tenant_id.clone(),
                    request_id: context.request_id.clone(),
                    ip_address: context.ip_address.clone(),
                    user_agent: context.user_agent.clone(),
                    resource_id: None,
                    resource_type: None,
                    operation: Some("LOGOUT".to_string()),
                    status: AuditEventStatus::Success,
                    details: serde_json::Value::Object(details),
                }
            },
            
            AuthenticationEvent::MfaAttempt { user_id, method, success, error } => {
                let mut details = serde_json::Map::new();
                details.insert("method".to_string(), json!(method));
                details.insert("success".to_string(), json!(success));
                
                if let Some(error) = error {
                    details.insert("error".to_string(), json!(error));
                }
                
                AuditEvent {
                    event_type: AuditEventType::Authentication,
                    timestamp: Utc::now(),
                    user_id: Some(user_id),
                    tenant_id: tenant_id.clone(),
                    request_id: context.request_id.clone(),
                    ip_address: context.ip_address.clone(),
                    user_agent: context.user_agent.clone(),
                    resource_id: None,
                    resource_type: None,
                    operation: Some("MFA_VERIFICATION".to_string()),
                    status: if success { AuditEventStatus::Success } else { AuditEventStatus::Failure },
                    details: serde_json::Value::Object(details),
                }
            },
            
            // Additional event types...
        };
        
        // Log the event
        self.log_event(audit_event).await
    }
    
    /// Log any audit event
    pub async fn log_event(&self, event: AuditEvent) -> Result<(), LoggingError> {
        // Check if event logging is enabled for this type
        if !self.should_log_event(&event) {
            return Ok(());
        }
        
        // Convert to database model
        let audit_log = AuditLog {
            id: Uuid::new_v4(),
            event_type: event.event_type.to_string(),
            timestamp: event.timestamp,
            user_id: event.user_id.map(|id| id.to_string()),
            tenant_id: event.tenant_id.to_string(),
            request_id: event.request_id,
            ip_address: event.ip_address,
            user_agent: event.user_agent,
            resource_id: event.resource_id,
            resource_type: event.resource_type,
            operation: event.operation,
            status: event.status.to_string(),
            details: event.details,
        };
        
        // Save to repository
        self.repository.save(&audit_log).await.map_err(|e| LoggingError::RepositoryError(e.to_string()))?;
        
        // Also emit structured log for real-time monitoring
        info!(
            event_type = %audit_log.event_type,
            user_id = ?audit_log.user_id,
            tenant_id = %audit_log.tenant_id,
            operation = ?audit_log.operation,
            status = %audit_log.status,
            "Audit event logged"
        );
        
        Ok(())
    }
    
    /// Determine if event should be logged based on configuration
    fn should_log_event(&self, event: &AuditEvent) -> bool {
        match &self.config.log_level {
            AuditLogLevel::None => false,
            AuditLogLevel::Error => matches!(event.status, AuditEventStatus::Failure),
            AuditLogLevel::Critical => {
                matches!(event.status, AuditEventStatus::Failure) && 
                matches!(event.event_type, AuditEventType::Authentication | AuditEventType::Authorization)
            },
            AuditLogLevel::All => true,
        }
    }
    
    /// Search audit logs
    pub async fn search_logs(
        &self,
        tenant_id: &TenantId,
        query: &AuditLogQuery,
    ) -> Result<AuditLogSearchResult, LoggingError> {
        // Verify tenant matches (for multi-tenant deployments)
        if query.tenant_id.is_some() && query.tenant_id.as_ref() != Some(tenant_id) {
            return Err(LoggingError::AccessDenied);
        }
        
        // Execute search on repository
        let (logs, total) = self.repository
            .search(tenant_id, query)
            .await
            .map_err(|e| LoggingError::RepositoryError(e.to_string()))?;
            
        Ok(AuditLogSearchResult {
            logs,
            total,
            page: query.page,
            page_size: query.page_size,
        })
    }
    
    /// Get logs for a specific user
    pub async fn get_user_logs(
        &self,
        tenant_id: &TenantId,
        user_id: &UserId,
        options: &AuditLogOptions,
    ) -> Result<Vec<AuditLog>, LoggingError> {
        // Create query for user logs
        let query = AuditLogQuery {
            tenant_id: Some(tenant_id.clone()),
            user_id: Some(user_id.clone()),
            event_types: options.event_types.clone(),
            start_date: options.start_date,
            end_date: options.end_date,
            status: options.status.clone(),
            resource_type: None,
            resource_id: None,
            operation: None,
            page: options.page,
            page_size: options.page_size,
        };
        
        // Execute search
        let (logs, _) = self.repository
            .search(tenant_id, &query)
            .await
            .map_err(|e| LoggingError::RepositoryError(e.to_string()))?;
            
        Ok(logs)
    }
}
```

## Conclusion

The advanced authentication implementation for Milestone 2 significantly enhances the security of the ACCI Framework through:

1. **Multi-Factor Authentication**:
   - TOTP Authenticator support
   - SMS/Email verification
   - WebAuthn/FIDO2 passwordless authentication

2. **Risk-Based Authentication**:
   - Contextual risk assessment
   - Progressive security challenges
   - Fraud detection capabilities

3. **Enhanced Session Management**:
   - Advanced session fingerprinting
   - Location-aware session validation
   - Concurrent session monitoring

4. **OAuth2/OIDC Integration**:
   - Full OAuth2 authorization server
   - OpenID Connect provider
   - External identity provider support

5. **Comprehensive Auditing**:
   - Immutable, structured audit events
   - Detailed context information
   - Flexible search and reporting

These implementations provide a robust security foundation that can be customized for each tenant's needs, enabling different security postures based on organizational requirements while maintaining a consistent core implementation.

## Next Steps

1. Implement tenant-specific security policy management
2. Add configurable brute force protection
3. Integrate with threat intelligence services
4. Implement automated security monitoring
5. Add penetration testing and fuzzing framework