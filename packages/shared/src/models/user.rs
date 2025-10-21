use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "test@example.com".to_string(),
            "password123".to_string(),
            "John".to_string(),
            "Doe".to_string(),
        );

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password, "password123");
        assert_eq!(user.first_name, "John");
        assert_eq!(user.last_name, "Doe");
        assert_eq!(user.rating, 1200);
        assert!(!user.id.is_empty());
        assert!(user.created_at <= Utc::now());
    }

    #[test]
    fn test_user_id_uniqueness() {
        let user1 = User::new(
            "user1@example.com".to_string(),
            "password1".to_string(),
            "Alice".to_string(),
            "Smith".to_string(),
        );

        let user2 = User::new(
            "user2@example.com".to_string(),
            "password2".to_string(),
            "Bob".to_string(),
            "Johnson".to_string(),
        );

        assert_ne!(user1.id, user2.id);
    }

    #[test]
    fn test_user_serialization() {
        let user = User::new(
            "serialize@example.com".to_string(),
            "password".to_string(),
            "Serialize".to_string(),
            "Test".to_string(),
        );

        // Test serialization
        let serialized = serde_json::to_string(&user).unwrap();
        assert!(serialized.contains("serialize@example.com"));
        assert!(serialized.contains("Serialize"));
        assert!(serialized.contains("Test"));
        assert!(serialized.contains("1200"));

        // Test deserialization
        let deserialized: User = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.email, user.email);
        assert_eq!(deserialized.first_name, user.first_name);
        assert_eq!(deserialized.last_name, user.last_name);
        assert_eq!(deserialized.rating, user.rating);
    }

    #[test]
    fn test_user_default_rating() {
        let user = User::new(
            "rating@example.com".to_string(),
            "password".to_string(),
            "Rating".to_string(),
            "Test".to_string(),
        );

        assert_eq!(
            user.rating, 1200,
            "Default rating should be 1200 for chess platform"
        );
    }

    #[test]
    fn test_user_timestamp_ordering() {
        let user1 = User::new(
            "first@example.com".to_string(),
            "password".to_string(),
            "First".to_string(),
            "User".to_string(),
        );

        // Small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(1));

        let user2 = User::new(
            "second@example.com".to_string(),
            "password".to_string(),
            "Second".to_string(),
            "User".to_string(),
        );

        assert!(
            user1.created_at < user2.created_at,
            "User2 should be created after User1"
        );
    }

    #[test]
    fn test_user_email_validation() {
        let user = User::new(
            "valid.email@example.com".to_string(),
            "password".to_string(),
            "Valid".to_string(),
            "Email".to_string(),
        );

        assert!(user.email.contains('@'), "Email should contain @ symbol");
        assert!(user.email.contains('.'), "Email should contain domain");
    }

    #[test]
    fn test_user_password_storage() {
        let password = "secure_password_123";
        let user = User::new(
            "password@example.com".to_string(),
            password.to_string(),
            "Password".to_string(),
            "Test".to_string(),
        );

        assert_eq!(
            user.password, password,
            "Password should be stored as provided"
        );
    }

    #[test]
    fn test_user_name_fields() {
        let user = User::new(
            "name@example.com".to_string(),
            "password".to_string(),
            "FirstName".to_string(),
            "LastName".to_string(),
        );

        assert_eq!(user.first_name, "FirstName");
        assert_eq!(user.last_name, "LastName");
        assert_ne!(user.first_name, user.last_name);
    }

    #[test]
    fn test_multiple_users_same_credentials() {
        let email = "same@example.com";
        let password = "same_password";
        let first_name = "Same";
        let last_name = "User";

        let user1 = User::new(
            email.to_string(),
            password.to_string(),
            first_name.to_string(),
            last_name.to_string(),
        );

        let user2 = User::new(
            email.to_string(),
            password.to_string(),
            first_name.to_string(),
            last_name.to_string(),
        );

        // Same credentials but different IDs and timestamps
        assert_eq!(user1.email, user2.email);
        assert_eq!(user1.password, user2.password);
        assert_eq!(user1.first_name, user2.first_name);
        assert_eq!(user1.last_name, user2.last_name);
        assert_ne!(user1.id, user2.id);
        assert_ne!(user1.created_at, user2.created_at);
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub rating: i32,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(email: String, password: String, first_name: String, last_name: String) -> Self {
        User {
            id: Uuid::new_v4().to_string(),
            email,
            password,
            first_name,
            last_name,
            rating: 1200, // Default starting rating for chess platform
            created_at: Utc::now(),
        }
    }
}
