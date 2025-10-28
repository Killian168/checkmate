use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub user_id: String,
    pub rating: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User {
            user_id: "123".to_string(),
            rating: 1200,
        };
        assert_eq!(user.user_id, "123");
        assert_eq!(user.rating, 1200);
    }

    #[test]
    fn test_user_serialization() {
        let user = User {
            user_id: "456".to_string(),
            rating: 1000,
        };
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("\"user_id\":\"456\""));
        assert!(json.contains("\"rating\":1000"));
    }

    #[test]
    fn test_user_deserialization() {
        let json = r#"{"user_id":"789","rating":1500}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.user_id, "789");
        assert_eq!(user.rating, 1500);
    }

    #[test]
    fn test_user_clone() {
        let user1 = User {
            user_id: "clone".to_string(),
            rating: 1300,
        };
        let user2 = user1.clone();
        assert_eq!(user1.user_id, user2.user_id);
    }

    #[test]
    fn test_user_debug() {
        let user = User {
            user_id: "debug".to_string(),
            rating: 1400,
        };
        let debug_str = format!("{:?}", user);
        assert!(debug_str.contains("debug"));
    }
}
