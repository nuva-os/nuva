/*
 * Nuva OS
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// ! DAP protocolfixedmeaning

/// DAP message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "request")]
    Request(Request),
    #[serde(rename = "response")]
    Response(Response),
    #[serde(rename = "event")]
    Event(Event),
}

/// DAP request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Request {
    pub seq: u64,
    pub command: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

/// DAP response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Response {
    pub request_seq: u64,
    pub success: bool,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

impl Response {
    pub fn success(request_seq: u64, body: impl Into<String>) -> Self {
        Self {
            request_seq,
            success: true,
            command: String::new(),
            message: None,
            body: Some(serde_json::Value::String(body.into())),
        }
    }

    pub fn error(request_seq: u64, message: impl Into<String>) -> Self {
        Self {
            request_seq,
            success: false,
            command: String::new(),
            message: Some(message.into()),
            body: None,
        }
    }
}

/// DAP event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

impl Event {
    pub fn new(event: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            body: None,
        }
    }

    pub fn with_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }
}

/// constantuseevent
pub mod events {
    use super::Event;

    pub fn initialized() -> Event {
        Event::new("initialized")
    }

    pub fn stopped(reason: &str, thread_id: u64) -> Event {
        Event::new("stopped").with_body(serde_json::json!({
            "reason": reason,
            "threadId": thread_id
        }))
    }

    pub fn continued(thread_id: u64) -> Event {
        Event::new("continued").with_body(serde_json::json!({
            "threadId": thread_id
        }))
    }

    pub fn exited(exit_code: i32) -> Event {
        Event::new("exited").with_body(serde_json::json!({
            "exitCode": exit_code
        }))
    }

    pub fn terminated() -> Event {
        Event::new("terminated")
    }

    pub fn output(category: &str, output: &str) -> Event {
        Event::new("output").with_body(serde_json::json!({
            "category": category,
            "output": output
        }))
    }

    pub fn breakpoint_changed(reason: &str, breakpoint: serde_json::Value) -> Event {
        Event::new("breakpoint").with_body(serde_json::json!({
            "reason": reason,
            "breakpoint": breakpoint
        }))
    }
}