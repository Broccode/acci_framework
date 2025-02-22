use acci_auth::utils::password::{check_password_strength, hash_password, verify_password};
use rstest::rstest;

#[rstest]
#[case("weak", &["user@example.com"], false)] // Score 0
#[case("password123", &["user@example.com"], false)] // Score 1
#[case("P@ssw0rd123!", &["user@example.com"], true)] // Score 2
#[case("Tr0ub4dour&3", &["user@example.com"], true)] // Score 3-4
fn test_password_strength(
    #[case] password: &str,
    #[case] user_inputs: &[&str],
    #[case] should_pass: bool,
) {
    let result = check_password_strength(password, user_inputs);
    assert_eq!(
        result.is_ok(),
        should_pass,
        "Password '{}' strength check failed",
        password
    );
}

#[tokio::test]
async fn test_password_hash_and_verify() {
    let password = "StrongP@ssw0rd";

    // Test password hashing
    let hash = hash_password(password).expect("Failed to hash password");
    assert!(!hash.is_empty());

    // Test password verification
    let is_valid = verify_password(password, &hash).expect("Failed to verify password");
    assert!(is_valid);

    // Test wrong password
    let is_valid = verify_password("WrongPassword", &hash).expect("Failed to verify password");
    assert!(!is_valid);
}
