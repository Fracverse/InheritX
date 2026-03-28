use chrono::Utc;
use inheritx_backend::message_access_audit::{
    ActionCount, MessageAccessAction, MessageAccessLog, MessageAuditFilters, MessageAuditSummary,
    UserMessageActivity,
};
use uuid::Uuid;

#[test]
fn test_access_action_as_str() {
    assert_eq!(MessageAccessAction::Created.as_str(), "created");
    assert_eq!(MessageAccessAction::Viewed.as_str(), "viewed");
    assert_eq!(MessageAccessAction::Decrypted.as_str(), "decrypted");
    assert_eq!(MessageAccessAction::Delivered.as_str(), "delivered");
    assert_eq!(
        MessageAccessAction::DeliveryFailed.as_str(),
        "delivery_failed"
    );
    assert_eq!(MessageAccessAction::KeyRotated.as_str(), "key_rotated");
    assert_eq!(MessageAccessAction::KeyListed.as_str(), "key_listed");
    assert_eq!(MessageAccessAction::Deleted.as_str(), "deleted");
}

#[test]
fn test_access_action_serialization_roundtrip() {
    let actions = vec![
        MessageAccessAction::Created,
        MessageAccessAction::Viewed,
        MessageAccessAction::Decrypted,
        MessageAccessAction::Delivered,
        MessageAccessAction::DeliveryFailed,
        MessageAccessAction::KeyRotated,
        MessageAccessAction::KeyListed,
        MessageAccessAction::Deleted,
    ];

    for action in actions {
        let json = serde_json::to_value(action).unwrap();
        let deserialized: MessageAccessAction = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(deserialized, action);
        assert_eq!(json.as_str().unwrap(), action.as_str());
    }
}

#[test]
fn test_message_access_log_serialization() {
    let msg_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let log = MessageAccessLog {
        id: Uuid::new_v4(),
        message_id: Some(msg_id),
        user_id,
        action: "created".to_string(),
        ip_address: Some("10.0.0.1".to_string()),
        user_agent: Some("TestAgent/1.0".to_string()),
        metadata: serde_json::json!({"beneficiary_contact": "alice@example.com"}),
        created_at: Utc::now(),
    };

    let json = serde_json::to_value(&log).unwrap();
    assert_eq!(json["action"], "created");
    assert_eq!(json["user_id"], user_id.to_string());
    assert_eq!(json["message_id"], msg_id.to_string());
    assert_eq!(json["ip_address"], "10.0.0.1");
    assert_eq!(
        json["metadata"]["beneficiary_contact"],
        "alice@example.com"
    );
}

#[test]
fn test_message_access_log_with_no_message_id() {
    let log = MessageAccessLog {
        id: Uuid::new_v4(),
        message_id: None,
        user_id: Uuid::new_v4(),
        action: "key_rotated".to_string(),
        ip_address: None,
        user_agent: None,
        metadata: serde_json::json!({"key_version": 3}),
        created_at: Utc::now(),
    };

    let json = serde_json::to_value(&log).unwrap();
    assert!(json["message_id"].is_null());
    assert!(json["ip_address"].is_null());
    assert_eq!(json["action"], "key_rotated");
}

#[test]
fn test_audit_filters_all_none() {
    let filters = MessageAuditFilters {
        message_id: None,
        user_id: None,
        action: None,
        start_date: None,
        end_date: None,
        limit: None,
        offset: None,
    };

    assert!(filters.message_id.is_none());
    assert!(filters.user_id.is_none());
    assert!(filters.action.is_none());
    assert!(filters.start_date.is_none());
    assert!(filters.end_date.is_none());
    assert!(filters.limit.is_none());
    assert!(filters.offset.is_none());
}

#[test]
fn test_audit_filters_with_values() {
    let user_id = Uuid::new_v4();
    let msg_id = Uuid::new_v4();

    let filters = MessageAuditFilters {
        message_id: Some(msg_id),
        user_id: Some(user_id),
        action: Some("viewed".to_string()),
        start_date: Some(Utc::now()),
        end_date: Some(Utc::now()),
        limit: Some(50),
        offset: Some(10),
    };

    assert_eq!(filters.message_id.unwrap(), msg_id);
    assert_eq!(filters.user_id.unwrap(), user_id);
    assert_eq!(filters.action.as_deref(), Some("viewed"));
    assert_eq!(filters.limit, Some(50));
    assert_eq!(filters.offset, Some(10));
}

#[test]
fn test_action_count_serialization() {
    let counts = vec![
        ActionCount {
            action: "viewed".to_string(),
            count: 150,
        },
        ActionCount {
            action: "created".to_string(),
            count: 45,
        },
        ActionCount {
            action: "delivered".to_string(),
            count: 30,
        },
    ];

    let json = serde_json::to_value(&counts).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0]["action"], "viewed");
    assert_eq!(arr[0]["count"], 150);
    assert_eq!(arr[1]["action"], "created");
    assert_eq!(arr[2]["count"], 30);
}

#[test]
fn test_audit_summary_serialization() {
    let summary = MessageAuditSummary {
        total_events: 500,
        action_counts: vec![
            ActionCount {
                action: "viewed".to_string(),
                count: 200,
            },
            ActionCount {
                action: "created".to_string(),
                count: 150,
            },
            ActionCount {
                action: "delivered".to_string(),
                count: 100,
            },
            ActionCount {
                action: "key_rotated".to_string(),
                count: 50,
            },
        ],
        unique_users: 25,
        unique_messages: 80,
        first_event_at: Some(Utc::now()),
        last_event_at: Some(Utc::now()),
    };

    let json = serde_json::to_value(&summary).unwrap();
    assert_eq!(json["total_events"], 500);
    assert_eq!(json["unique_users"], 25);
    assert_eq!(json["unique_messages"], 80);
    assert_eq!(json["action_counts"].as_array().unwrap().len(), 4);
    assert!(json["first_event_at"].is_string());
    assert!(json["last_event_at"].is_string());
}

#[test]
fn test_audit_summary_empty() {
    let summary = MessageAuditSummary {
        total_events: 0,
        action_counts: vec![],
        unique_users: 0,
        unique_messages: 0,
        first_event_at: None,
        last_event_at: None,
    };

    let json = serde_json::to_value(&summary).unwrap();
    assert_eq!(json["total_events"], 0);
    assert!(json["action_counts"].as_array().unwrap().is_empty());
    assert!(json["first_event_at"].is_null());
}

#[test]
fn test_user_message_activity_serialization() {
    let user_id = Uuid::new_v4();
    let activity = UserMessageActivity {
        user_id,
        total_actions: 42,
        messages_created: 10,
        messages_viewed: 25,
        messages_delivered: 7,
        first_activity: Some(Utc::now()),
        last_activity: Some(Utc::now()),
    };

    let json = serde_json::to_value(&activity).unwrap();
    assert_eq!(json["user_id"], user_id.to_string());
    assert_eq!(json["total_actions"], 42);
    assert_eq!(json["messages_created"], 10);
    assert_eq!(json["messages_viewed"], 25);
    assert_eq!(json["messages_delivered"], 7);
    assert!(json["first_activity"].is_string());
}

#[test]
fn test_user_message_activity_no_activity() {
    let user_id = Uuid::new_v4();
    let activity = UserMessageActivity {
        user_id,
        total_actions: 0,
        messages_created: 0,
        messages_viewed: 0,
        messages_delivered: 0,
        first_activity: None,
        last_activity: None,
    };

    let json = serde_json::to_value(&activity).unwrap();
    assert_eq!(json["total_actions"], 0);
    assert!(json["first_activity"].is_null());
    assert!(json["last_activity"].is_null());
}

#[test]
fn test_action_equality() {
    assert_eq!(MessageAccessAction::Created, MessageAccessAction::Created);
    assert_ne!(MessageAccessAction::Created, MessageAccessAction::Viewed);
    assert_ne!(
        MessageAccessAction::Delivered,
        MessageAccessAction::DeliveryFailed
    );
}

#[test]
fn test_action_debug_format() {
    let action = MessageAccessAction::Decrypted;
    let debug = format!("{:?}", action);
    assert_eq!(debug, "Decrypted");
}

#[test]
fn test_action_clone() {
    let action = MessageAccessAction::KeyRotated;
    let cloned = action;
    assert_eq!(action, cloned);
}

#[test]
fn test_access_log_clone() {
    let log = MessageAccessLog {
        id: Uuid::new_v4(),
        message_id: Some(Uuid::new_v4()),
        user_id: Uuid::new_v4(),
        action: "viewed".to_string(),
        ip_address: Some("192.168.1.100".to_string()),
        user_agent: Some("Chrome/120".to_string()),
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    };

    let cloned = log.clone();
    assert_eq!(log.id, cloned.id);
    assert_eq!(log.action, cloned.action);
    assert_eq!(log.user_id, cloned.user_id);
}

#[test]
fn test_filters_deserialization_from_query_string() {
    let json = serde_json::json!({
        "action": "viewed",
        "limit": 50,
        "offset": 0
    });

    let filters: MessageAuditFilters = serde_json::from_value(json).unwrap();
    assert_eq!(filters.action.as_deref(), Some("viewed"));
    assert_eq!(filters.limit, Some(50));
    assert_eq!(filters.offset, Some(0));
    assert!(filters.message_id.is_none());
}

#[test]
fn test_metadata_with_complex_payload() {
    let log = MessageAccessLog {
        id: Uuid::new_v4(),
        message_id: Some(Uuid::new_v4()),
        user_id: Uuid::new_v4(),
        action: "delivered".to_string(),
        ip_address: Some("10.0.0.1".to_string()),
        user_agent: None,
        metadata: serde_json::json!({
            "beneficiary_contact": "bob@example.com",
            "delivery_method": "email",
            "key_version": 2,
            "attempt_number": 1,
            "success": true
        }),
        created_at: Utc::now(),
    };

    let json = serde_json::to_value(&log).unwrap();
    let meta = &json["metadata"];
    assert_eq!(meta["beneficiary_contact"], "bob@example.com");
    assert_eq!(meta["delivery_method"], "email");
    assert_eq!(meta["key_version"], 2);
    assert_eq!(meta["attempt_number"], 1);
    assert_eq!(meta["success"], true);
}
