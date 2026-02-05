use llama_desktop_lib::models::chat::{ChatChoice, ChatMessage, ChatRequest, ChatResponse};
use serde_json::json;

#[test]
fn test_chat_message_creation() {
    let msg = ChatMessage {
        role: "user".to_string(),
        content: "Hello".to_string(),
    };

    assert_eq!(msg.role, "user");
    assert_eq!(msg.content, "Hello");
}

#[test]
fn test_chat_request_serialization() {
    let request = ChatRequest {
        model: "llama-3".to_string(),
        session_id: Some("session-123".to_string()),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Test".to_string(),
            },
        ],
        temperature: 0.7,
        top_p: 0.9,
        top_k: 40,
        max_tokens: 2048,
        stream: false,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("llama-3"));
    assert!(json.contains("session-123"));
}

#[test]
fn test_chat_request_deserialization() {
    let json = r#"{
        "model": "llama-3",
        "session_id": null,
        "messages": [{"role": "user", "content": "Hi"}],
        "temperature": 0.8,
        "top_p": 0.95,
        "top_k": 50,
        "max_tokens": 1024,
        "stream": true
    }"#;

    let request: ChatRequest = serde_json::from_str(json).unwrap();
    assert_eq!(request.model, "llama-3");
    assert_eq!(request.temperature, 0.8);
    assert_eq!(request.stream, true);
    assert!(request.session_id.is_none());
}

#[test]
fn test_chat_response_structure() {
    let response = ChatResponse {
        choices: vec![ChatChoice {
            message: ChatMessage {
                role: "assistant".to_string(),
                content: "Response".to_string(),
            },
            finish_reason: "stop".to_string(),
        }],
        usage: json!({"prompt_tokens": 10, "completion_tokens": 20}),
    };

    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.role, "assistant");
    assert_eq!(response.choices[0].finish_reason, "stop");
}

#[test]
fn test_chat_message_clone() {
    let msg = ChatMessage {
        role: "system".to_string(),
        content: "You are helpful".to_string(),
    };

    let cloned = msg.clone();
    assert_eq!(msg.role, cloned.role);
    assert_eq!(msg.content, cloned.content);
}

#[test]
fn test_chat_request_with_multiple_messages() {
    let request = ChatRequest {
        model: "test-model".to_string(),
        session_id: None,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "System prompt".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: "User message".to_string(),
            },
        ],
        temperature: 0.5,
        top_p: 1.0,
        top_k: 30,
        max_tokens: 512,
        stream: false,
    };

    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.messages[0].role, "system");
    assert_eq!(request.messages[1].role, "user");
}
