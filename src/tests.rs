use super::*;
use std::str::FromStr;

// Non-mocked tests

#[test]
fn test_config_clear_user_data() {
    let mut config = RTMConfig {
        api_key: Some("key123".to_string()),
        api_secret: Some("secret123".to_string()),
        token: Some("token123".to_string()),
        user: Some(User {
            id: "user1".to_string(),
            username: "testuser".to_string(),
            fullname: "Test User".to_string(),
        }),
    };

    config.clear_user_data();

    assert_eq!(config.api_key, Some("key123".to_string()));
    assert_eq!(config.api_secret, Some("secret123".to_string()));
    assert!(config.token.is_none());
    assert!(config.user.is_none());
}

#[test]
fn test_perms_includes() {
    assert!(Perms::Delete.includes(Perms::Read));
    assert!(Perms::Delete.includes(Perms::Write));
    assert!(Perms::Delete.includes(Perms::Delete));

    assert!(Perms::Write.includes(Perms::Read));
    assert!(Perms::Write.includes(Perms::Write));
    assert!(!Perms::Write.includes(Perms::Delete));

    assert!(Perms::Read.includes(Perms::Read));
    assert!(!Perms::Read.includes(Perms::Write));
    assert!(!Perms::Read.includes(Perms::Delete));
}

#[test]
fn test_perms_as_str() {
    assert_eq!(Perms::Read.as_str(), "read");
    assert_eq!(Perms::Write.as_str(), "write");
    assert_eq!(Perms::Delete.as_str(), "delete");
}

#[test]
fn test_perms_to_string() {
    assert_eq!(Perms::Read.to_string(), "read");
    assert_eq!(Perms::Write.to_string(), "write");
    assert_eq!(Perms::Delete.to_string(), "delete");
}

#[test]
fn test_perms_from_str() {
    assert_eq!(Perms::from_str("read").unwrap(), Perms::Read);
    assert_eq!(Perms::from_str("write").unwrap(), Perms::Write);
    assert_eq!(Perms::from_str("delete").unwrap(), Perms::Delete);
    assert!(Perms::from_str("invalid").is_err());
}

#[test]
fn test_task_time_left() {
    // Test a task that's due in the future
    let future_task = Task {
        id: "1".to_string(),
        due: Some(Utc::now() + Duration::hours(1)),
        has_due_time: true,
        deleted: None,
        added: Some(Utc::now() - Duration::days(1)),
        completed: None,
        priority: "1".to_string(),
    };

    match future_task.get_time_left() {
        TimeLeft::Remaining(seconds) => {
            assert!(seconds > 0);
            assert!(seconds <= 3600); // Due within an hour (3600 seconds)
        }
        _ => panic!("Expected a Remaining TimeLeft variant"),
    }

    // Test an overdue task
    let overdue_task = Task {
        id: "2".to_string(),
        due: Some(Utc::now() - Duration::hours(1)),
        has_due_time: true,
        deleted: None,
        added: Some(Utc::now() - Duration::days(1)),
        completed: None,
        priority: "1".to_string(),
    };

    match overdue_task.get_time_left() {
        TimeLeft::Overdue(seconds) => {
            assert!(seconds > 0);
            assert!(seconds <= 3600); // Overdue by an hour (3600 seconds)
        }
        _ => panic!("Expected an Overdue TimeLeft variant"),
    }

    // Test a completed task
    let completed_task = Task {
        id: "3".to_string(),
        due: Some(Utc::now() + Duration::hours(1)),
        has_due_time: true,
        deleted: None,
        added: Some(Utc::now() - Duration::days(1)),
        completed: Some(Utc::now()),
        priority: "1".to_string(),
    };

    match completed_task.get_time_left() {
        TimeLeft::Completed => {} // Expected behavior
        _ => panic!("Expected a Completed TimeLeft variant"),
    }

    // Test a task with no due date
    let no_due_task = Task {
        id: "4".to_string(),
        due: None,
        has_due_time: false,
        deleted: None,
        added: Some(Utc::now() - Duration::days(1)),
        completed: None,
        priority: "1".to_string(),
    };

    match no_due_task.get_time_left() {
        TimeLeft::NoDue => {} // Expected behavior
        _ => panic!("Expected a NoDue TimeLeft variant"),
    }

    // Test a deleted task
    let deleted_task = Task {
        id: "5".to_string(),
        due: Some(Utc::now() + Duration::hours(1)),
        has_due_time: true,
        deleted: Some(Utc::now()),
        added: Some(Utc::now() - Duration::days(1)),
        completed: None,
        priority: "1".to_string(),
    };

    match deleted_task.get_time_left() {
        TimeLeft::NoDue => {} // Expected behavior
        _ => panic!("Expected a NoDue TimeLeft variant for deleted task"),
    }
}

#[test]
fn test_bool_from_string() {
    use serde_json::json;

    #[derive(Deserialize, Debug)]
    struct TestBool {
        #[serde(deserialize_with = "bool_from_string")]
        value: bool,
    }

    let test_true = json!({"value": "1"});
    let test_false = json!({"value": "0"});
    let test_invalid = json!({"value": "invalid"});

    assert!(serde_json::from_value::<TestBool>(test_true).unwrap().value);
    assert!(
        !serde_json::from_value::<TestBool>(test_false)
            .unwrap()
            .value
    );
    assert!(serde_json::from_value::<TestBool>(test_invalid).is_err());
}

#[test]
fn test_empty_string_as_none() {
    use serde_json::json;

    #[derive(Deserialize, Debug)]
    struct TestOptional {
        #[serde(deserialize_with = "empty_string_as_none")]
        value: Option<String>,
    }

    let test_some = json!({"value": "something"});
    let test_empty = json!({"value": ""});
    let test_null = json!({"value": null});

    assert_eq!(
        serde_json::from_value::<TestOptional>(test_some)
            .unwrap()
            .value,
        Some("something".to_string())
    );
    assert_eq!(
        serde_json::from_value::<TestOptional>(test_empty)
            .unwrap()
            .value,
        None
    );
    assert_eq!(
        serde_json::from_value::<TestOptional>(test_null)
            .unwrap()
            .value,
        None
    );
}

#[test]
fn test_deser_tags() {
    use serde_json::json;

    #[derive(Deserialize, Debug)]
    struct TestTags {
        #[serde(deserialize_with = "deser_tags")]
        tags: Vec<String>,
    }

    let test_tags = json!({"tags": {"tag": ["work", "important"]}});
    let test_empty = json!({"tags": []});

    let result = serde_json::from_value::<TestTags>(test_tags).unwrap();
    assert_eq!(result.tags, vec!["work", "important"]);

    let result_empty = serde_json::from_value::<TestTags>(test_empty).unwrap();
    assert!(result_empty.tags.is_empty());
}

#[test]
fn test_deser_notes() {
    use serde_json::json;

    #[derive(Deserialize, Debug)]
    struct TestNotes {
        #[serde(deserialize_with = "deser_notes")]
        notes: Vec<RTMNote>,
    }

    let now = Utc::now();
    let now_string = now.to_rfc3339();

    let test_notes = json!({
        "notes": {
            "note": [
                {
                    "id": "note1",
                    "created": now_string,
                    "modified": now_string,
                    "title": "Note Title",
                    "$t": "Note Content"
                }
            ]
        }
    });

    let test_empty = json!({"notes": []});

    let result = serde_json::from_value::<TestNotes>(test_notes).unwrap();
    assert_eq!(result.notes.len(), 1);
    assert_eq!(result.notes[0].id, "note1");
    assert_eq!(result.notes[0].title, "Note Title");
    assert_eq!(result.notes[0].text, "Note Content");

    let result_empty = serde_json::from_value::<TestNotes>(test_empty).unwrap();
    assert!(result_empty.notes.is_empty());
}

// Test for API initialization (non-mocked version)
#[test]
fn test_api_initialization() {
    let api = API::new("test_key".to_string(), "test_secret".to_string());
    assert_eq!(api.api_key, "test_key");
    assert_eq!(api.api_secret, "test_secret");
    assert!(api.token.is_none());
    assert!(api.user.is_none());
}

// Test for RTMConfig and API configuration
#[test]
fn test_api_config() {
    // Create a new API
    let api = API::new("test_key".to_string(), "test_secret".to_string());

    // Convert to config
    let config = api.to_config();
    assert_eq!(config.api_key, Some("test_key".to_string()));
    assert_eq!(config.api_secret, Some("test_secret".to_string()));
    assert_eq!(config.token, None);
    assert_eq!(config.user, None);

    // Create an API from config
    let api_from_config = API::from_config(config);
    assert_eq!(api_from_config.api_key, "test_key");
    assert_eq!(api_from_config.api_secret, "test_secret");
    assert!(api_from_config.token.is_none());
    assert!(api_from_config.user.is_none());
}

#[test]
fn test_get_filter_extid() {
    let api = API::new("test_key".to_string(), "test_secret".to_string());
    let filter = api.get_filter_extid("my_external_id");
    assert_eq!(filter, "source:api:test_key:my_external_id");
}
