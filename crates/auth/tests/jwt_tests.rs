use acci_auth::utils::jwt::{JwtError, JwtUtils};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[tokio::test]
async fn test_jwt_creation_and_validation() {
    let secret = b"test-secret-key";
    let jwt_utils = JwtUtils::new(secret);
    let user_id = Uuid::new_v4();
    let email = "test@example.com";

    // Test token creation
    let token = jwt_utils
        .create_token(user_id, email)
        .expect("Failed to create token");
    assert!(!token.is_empty());

    // Test token validation
    let claims = jwt_utils
        .validate_token(&token)
        .expect("Failed to validate token");
    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.email, email);
}

#[tokio::test]
async fn test_expired_token() {
    let secret = b"test-secret-key";
    let jwt_utils = JwtUtils::new(secret);
    let user_id = Uuid::new_v4();
    let email = "test@example.com";

    // Create an expired token by manipulating the expiration time
    let now = OffsetDateTime::now_utc();
    let exp = now - Duration::hours(1); // Token expired 1 hour ago

    let claims = acci_auth::utils::jwt::Claims {
        sub: user_id,
        exp: exp.unix_timestamp(),
        iat: now.unix_timestamp(),
        email: email.to_string(),
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret),
    )
    .expect("Failed to create expired token");

    // Test expired token validation
    let result = jwt_utils.validate_token(&token);
    assert!(matches!(result, Err(JwtError::TokenExpired)));
}

#[tokio::test]
async fn test_invalid_token() {
    let secret = b"test-secret-key";
    let jwt_utils = JwtUtils::new(secret);

    // Test invalid token validation
    let result = jwt_utils.validate_token("invalid-token");
    assert!(matches!(result, Err(JwtError::TokenValidation(_))));
}
