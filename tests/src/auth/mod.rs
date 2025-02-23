use crate::helpers::setup_test_db;
use acci_auth::{
    models::user::{User, UserError, UserRepository},
    repository::postgres::{PostgresUserRepository, RepositoryConfig},
};
use time::OffsetDateTime;
use uuid::Uuid;

#[tokio::test]
async fn test_user_crud_operations() -> Result<(), UserError> {
    // Setup test database
    let (_container, pool) = setup_test_db().await.unwrap();

    // Create repository instance
    let config = RepositoryConfig {
        database_url: "postgres://acci:acci@localhost:15432/acci_test".to_string(),
        ..Default::default()
    };
    let repo = PostgresUserRepository::new(config).await?;

    // Create test user
    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        last_login: None,
        is_active: true,
        is_verified: false,
    };

    // Test create
    repo.create(&user).await?;

    // Test find by email
    let found = repo.find_by_email(&user.email).await?;
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, user.id);

    // Test find by id
    let found = repo.find_by_id(user.id).await?;
    assert!(found.is_some());
    assert_eq!(found.unwrap().email, user.email);

    // Test update
    let mut updated_user = user.clone();
    updated_user.is_verified = true;
    repo.update(&updated_user).await?;

    let found = repo.find_by_id(user.id).await?;
    assert!(found.unwrap().is_verified);

    // Test delete
    repo.delete(user.id).await?;
    let found = repo.find_by_id(user.id).await?;
    assert!(found.is_none());

    Ok(())
}
