use std::{
    collections::{HashSet, VecDeque},
    sync::Mutex,
};

use serde::{Deserialize, Serialize};

use crate::codex::{ConversationNotificationCandidate, ConversationState};

pub const DESKTOP_NOTIFICATION_SCHEMA_VERSION: u16 = 1;
const MAX_DELIVERED_NOTIFICATIONS: usize = 256;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopNotificationRequest {
    pub conversation_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesktopNotificationStatus {
    Sent,
    Foreground,
    Duplicate,
    Ineligible,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopNotificationResult {
    pub schema_version: u16,
    pub status: DesktopNotificationStatus,
}

impl DesktopNotificationResult {
    pub const fn new(status: DesktopNotificationStatus) -> Self {
        Self {
            schema_version: DESKTOP_NOTIFICATION_SCHEMA_VERSION,
            status,
        }
    }
}

#[derive(Default)]
pub struct DesktopNotificationService {
    state: Mutex<DesktopNotificationState>,
}

#[derive(Default)]
struct DesktopNotificationState {
    delivered_order: VecDeque<String>,
    delivered: HashSet<String>,
    reserved: HashSet<String>,
}

pub struct PreparedDesktopNotification {
    key: String,
    title: &'static str,
    body: &'static str,
}

impl PreparedDesktopNotification {
    pub const fn title(&self) -> &'static str {
        self.title
    }

    pub const fn body(&self) -> &'static str {
        self.body
    }
}

impl DesktopNotificationService {
    pub fn prepare(
        &self,
        candidate: ConversationNotificationCandidate,
    ) -> Result<Option<PreparedDesktopNotification>, ()> {
        let mut state = self.state.lock().map_err(|_| ())?;
        if state.delivered.contains(&candidate.key) || state.reserved.contains(&candidate.key) {
            return Ok(None);
        }
        let (title, body) = notification_copy(candidate.state).ok_or(())?;
        state.reserved.insert(candidate.key.clone());
        Ok(Some(PreparedDesktopNotification {
            key: candidate.key,
            title,
            body,
        }))
    }

    pub fn complete(&self, prepared: PreparedDesktopNotification) {
        if let Ok(mut state) = self.state.lock() {
            if !state.reserved.remove(&prepared.key) {
                return;
            }
            if state.delivered.insert(prepared.key.clone()) {
                state.delivered_order.push_back(prepared.key);
            }
            while state.delivered_order.len() > MAX_DELIVERED_NOTIFICATIONS {
                if let Some(oldest) = state.delivered_order.pop_front() {
                    state.delivered.remove(&oldest);
                }
            }
        }
    }

    pub fn restore(&self, prepared: PreparedDesktopNotification) {
        if let Ok(mut state) = self.state.lock() {
            state.reserved.remove(&prepared.key);
        }
    }
}

fn notification_copy(state: ConversationState) -> Option<(&'static str, &'static str)> {
    match state {
        ConversationState::WaitingForApproval => Some((
            "Codex task needs approval",
            "Return to QuireForge to review the request.",
        )),
        ConversationState::Completed => Some((
            "Codex task completed",
            "Return to QuireForge to review the result.",
        )),
        ConversationState::Blocked => Some((
            "Codex task needs attention",
            "Return to QuireForge to review the task.",
        )),
        ConversationState::Failed => Some((
            "Codex task stopped",
            "Return to QuireForge to review what happened.",
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(key: &str, state: ConversationState) -> ConversationNotificationCandidate {
        ConversationNotificationCandidate {
            key: key.to_owned(),
            state,
        }
    }

    #[test]
    fn reserves_fixed_privacy_safe_copy_once() {
        let service = DesktopNotificationService::default();
        let prepared = service
            .prepare(candidate(
                "terminal:opaque:completed",
                ConversationState::Completed,
            ))
            .expect("notification state must be available")
            .expect("first notification must reserve");
        assert_eq!(prepared.title(), "Codex task completed");
        assert_eq!(
            prepared.body(),
            "Return to QuireForge to review the result."
        );
        assert!(!prepared.title().contains("opaque"));
        service.complete(prepared);
        assert!(service
            .prepare(candidate(
                "terminal:opaque:completed",
                ConversationState::Completed
            ))
            .expect("notification state must be available")
            .is_none());
    }

    #[test]
    fn failed_delivery_can_be_retried_and_ineligible_states_fail_closed() {
        let service = DesktopNotificationService::default();
        let prepared = service
            .prepare(candidate(
                "approval:opaque",
                ConversationState::WaitingForApproval,
            ))
            .expect("notification state must be available")
            .expect("notification must reserve");
        service.restore(prepared);
        assert!(service
            .prepare(candidate(
                "approval:opaque",
                ConversationState::WaitingForApproval
            ))
            .expect("notification state must be available")
            .is_some());
        assert!(service
            .prepare(candidate("running:opaque", ConversationState::Running))
            .is_err());
    }

    #[test]
    fn result_contract_contains_only_closed_status() {
        let encoded = serde_json::to_value(DesktopNotificationResult::new(
            DesktopNotificationStatus::Foreground,
        ))
        .expect("result must serialize");
        assert_eq!(
            encoded,
            serde_json::json!({"schemaVersion": 1, "status": "foreground"})
        );
        assert!(
            serde_json::from_value::<DesktopNotificationRequest>(serde_json::json!({
                "conversationId": "018f0000-0000-7000-8000-000000000010",
                "title": "private task"
            }))
            .is_err()
        );
    }
}
