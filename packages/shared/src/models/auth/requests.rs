use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_request_creation() {
        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };

        assert_eq!(request.email, "test@example.com");
        assert_eq!(request.password, "password123");
        assert_eq!(request.first_name, "John");
        assert_eq!(request.last_name, "Doe");
    }

    #[test]
    fn test_create_user_request_serialization() {
        let request = CreateUserRequest {
            email: "serialize@example.com".to_string(),
            password: "test_password".to_string(),
            first_name: "Serialize".to_string(),
            last_name: "Test".to_string(),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("serialize@example.com"));
        assert!(serialized.contains("test_password"));
        assert!(serialized.contains("Serialize"));
        assert!(serialized.contains("Test"));

        let deserialized: CreateUserRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.email, request.email);
        assert_eq!(deserialized.password, request.password);
        assert_eq!(deserialized.first_name, request.first_name);
        assert_eq!(deserialized.last_name, request.last_name);
    }

    #[test]
    fn test_login_request_creation() {
        let request = LoginRequest {
            email: "login@example.com".to_string(),
            password: "login_password".to_string(),
        };

        assert_eq!(request.email, "login@example.com");
        assert_eq!(request.password, "login_password");
    }

    #[test]
    fn test_login_request_serialization() {
        let request = LoginRequest {
            email: "login_serialize@example.com".to_string(),
            password: "serialized_password".to_string(),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("login_serialize@example.com"));
        assert!(serialized.contains("serialized_password"));

        let deserialized: LoginRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.email, request.email);
        assert_eq!(deserialized.password, request.password);
    }

    #[test]
    fn test_request_validation() {
        let create_request = CreateUserRequest {
            email: "valid@example.com".to_string(),
            password: "valid_password_123".to_string(),
            first_name: "Valid".to_string(),
            last_name: "User".to_string(),
        };

        assert!(create_request.email.contains('@'));
        assert!(create_request.password.len() >= 8);
        assert!(!create_request.first_name.is_empty());
        assert!(!create_request.last_name.is_empty());

        let login_request = LoginRequest {
            email: "login@example.com".to_string(),
            password: "login_pass".to_string(),
        };

        assert!(login_request.email.contains('@'));
        assert!(!login_request.password.is_empty());
    }

    #[test]
    fn test_request_clone_semantics() {
        let create_request = CreateUserRequest {
            email: "clone@example.com".to_string(),
            password: "clone_password".to_string(),
            first_name: "Clone".to_string(),
            last_name: "Test".to_string(),
        };

        let login_request = LoginRequest {
            email: "login_clone@example.com".to_string(),
            password: "login_clone_password".to_string(),
        };

        // Both structs should implement Clone and Debug via derive
        let cloned_create = create_request.clone();
        let cloned_login = login_request.clone();

        assert_eq!(create_request.email, cloned_create.email);
        assert_eq!(login_request.email, cloned_login.email);

        // Test debug formatting
        let create_debug = format!("{:?}", create_request);
        let login_debug = format!("{:?}", login_request);

        assert!(create_debug.contains("CreateUserRequest"));
        assert!(login_debug.contains("LoginRequest"));
    }

    #[test]
    fn test_request_field_types() {
        let create_request = CreateUserRequest {
            email: String::from("types@example.com"),
            password: String::from("types_password"),
            first_name: String::from("Types"),
            last_name: String::from("Test"),
        };

        let login_request = LoginRequest {
            email: String::from("login_types@example.com"),
            password: String::from("login_types_password"),
        };

        // Verify all fields are strings
        assert!(create_request.email.is_ascii());
        assert!(create_request.password.is_ascii());
        assert!(create_request.first_name.is_ascii());
        assert!(create_request.last_name.is_ascii());
        assert!(login_request.email.is_ascii());
        assert!(login_request.password.is_ascii());
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
