use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::types::CodexModel;

pub const MODEL_SELECTION_SCHEMA_VERSION: u16 = 1;
pub(crate) const MODEL_SELECTOR_TOOL_NAME: &str = "quireforge_model_selector";
const MAX_ALLOWED_MODELS: usize = 32;
const MAX_RATIONALE_BYTES: usize = 240;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelSelectionAvailability {
    Ready,
    RecommendationOnly,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelSelectionOwnership {
    Manual,
    Recommend,
    Automatic,
}

impl ModelSelectionOwnership {
    pub(crate) const fn as_storage_value(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Recommend => "recommend",
            Self::Automatic => "automatic",
        }
    }

    pub(crate) fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "manual" => Some(Self::Manual),
            "recommend" => Some(Self::Recommend),
            "automatic" => Some(Self::Automatic),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelSelectionProvenance {
    User,
    Codex,
}

impl ModelSelectionProvenance {
    pub(crate) const fn as_storage_value(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Codex => "codex",
        }
    }

    pub(crate) fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "user" => Some(Self::User),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelSelectionApplication {
    Manual,
    Recommendation,
    Automatic,
}

impl ModelSelectionApplication {
    pub(crate) const fn as_storage_value(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Recommendation => "recommendation",
            Self::Automatic => "automatic",
        }
    }

    pub(crate) fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "manual" => Some(Self::Manual),
            "recommendation" => Some(Self::Recommendation),
            "automatic" => Some(Self::Automatic),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelSelectionDiagnosticCode {
    InvalidRequest,
    ConversationNotFound,
    MetadataUnavailable,
    CatalogUnavailable,
    ModelUnavailable,
    ReasoningUnavailable,
    PolicyBlocked,
    ManualOwnership,
    UserLocked,
    RequestAlreadyMade,
    ControlUnavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelSelectionChoice {
    pub model_id: String,
    pub reasoning_effort: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelSelectionPolicy {
    pub ownership: ModelSelectionOwnership,
    pub user_locked: bool,
    pub allowed_model_ids: Vec<String>,
    pub reasoning_ceiling: Option<String>,
}

impl Default for ModelSelectionPolicy {
    fn default() -> Self {
        Self {
            ownership: ModelSelectionOwnership::Manual,
            user_locked: false,
            allowed_model_ids: Vec::new(),
            reasoning_ceiling: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingModelSelection {
    pub choice: ModelSelectionChoice,
    pub provenance: ModelSelectionProvenance,
    pub application: ModelSelectionApplication,
    pub rationale: String,
    pub requested_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSelectionSnapshot {
    pub schema_version: u16,
    pub availability: ModelSelectionAvailability,
    pub effective: ModelSelectionChoice,
    pub pending: Option<PendingModelSelection>,
    pub policy: ModelSelectionPolicy,
    pub diagnostic_code: Option<ModelSelectionDiagnosticCode>,
}

impl ModelSelectionSnapshot {
    pub(crate) fn ready(
        availability: ModelSelectionAvailability,
        effective: ModelSelectionChoice,
        pending: Option<PendingModelSelection>,
        policy: ModelSelectionPolicy,
    ) -> Self {
        Self {
            schema_version: MODEL_SELECTION_SCHEMA_VERSION,
            availability,
            effective,
            pending,
            policy,
            diagnostic_code: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PendingSelectionAction {
    Keep,
    Accept,
    Dismiss,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelSelectionUpdateRequest {
    pub conversation_id: String,
    pub choice: ModelSelectionChoice,
    pub policy: ModelSelectionPolicy,
    pub pending_action: PendingSelectionAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AgentSelectionRequest {
    pub choice: ModelSelectionChoice,
    pub rationale: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ModelSelectorArguments {
    Inspect,
    Request(AgentSelectionRequest),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct AgentVisibleModel {
    model_id: String,
    display_name: String,
    supported_reasoning_efforts: Vec<String>,
}

#[derive(Deserialize)]
#[serde(
    tag = "action",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum WireModelSelectorArguments {
    Inspect,
    Request {
        model_id: String,
        reasoning_effort: String,
        rationale: String,
    },
}

pub(crate) struct ModelSelectionService;

impl ModelSelectionService {
    pub(crate) fn parse_arguments(
        arguments: Value,
    ) -> Result<ModelSelectorArguments, ModelSelectionDiagnosticCode> {
        let arguments: WireModelSelectorArguments = serde_json::from_value(arguments)
            .map_err(|_| ModelSelectionDiagnosticCode::InvalidRequest)?;
        match arguments {
            WireModelSelectorArguments::Inspect => Ok(ModelSelectorArguments::Inspect),
            WireModelSelectorArguments::Request {
                model_id,
                reasoning_effort,
                rationale,
            } => {
                let choice = ModelSelectionChoice {
                    model_id,
                    reasoning_effort,
                };
                validate_choice_shape(&choice)?;
                let rationale = rationale.trim().to_owned();
                if rationale.is_empty()
                    || rationale.len() > MAX_RATIONALE_BYTES
                    || contains_unsafe_control(&rationale)
                {
                    return Err(ModelSelectionDiagnosticCode::InvalidRequest);
                }
                Ok(ModelSelectorArguments::Request(AgentSelectionRequest {
                    choice,
                    rationale,
                }))
            }
        }
    }

    pub(crate) fn validate_choice<'a>(
        choice: &ModelSelectionChoice,
        models: &'a [CodexModel],
    ) -> Result<&'a CodexModel, ModelSelectionDiagnosticCode> {
        validate_choice_shape(choice)?;
        let model = models
            .iter()
            .find(|model| model.id == choice.model_id)
            .ok_or(ModelSelectionDiagnosticCode::ModelUnavailable)?;
        if !model
            .supported_reasoning_efforts
            .contains(&choice.reasoning_effort)
        {
            return Err(ModelSelectionDiagnosticCode::ReasoningUnavailable);
        }
        Ok(model)
    }

    pub(crate) fn validate_policy(
        policy: &ModelSelectionPolicy,
        models: &[CodexModel],
        effective: &ModelSelectionChoice,
    ) -> Result<(), ModelSelectionDiagnosticCode> {
        Self::validate_policy_shape(policy, effective)?;
        if policy
            .allowed_model_ids
            .iter()
            .any(|model_id| !models.iter().any(|model| model.id == *model_id))
        {
            return Err(ModelSelectionDiagnosticCode::InvalidRequest);
        }
        Ok(())
    }

    pub(crate) fn validate_policy_shape(
        policy: &ModelSelectionPolicy,
        effective: &ModelSelectionChoice,
    ) -> Result<(), ModelSelectionDiagnosticCode> {
        if policy.allowed_model_ids.len() > MAX_ALLOWED_MODELS {
            return Err(ModelSelectionDiagnosticCode::InvalidRequest);
        }
        let mut unique = HashSet::with_capacity(policy.allowed_model_ids.len());
        for model_id in &policy.allowed_model_ids {
            if !valid_protocol_choice(model_id, 128) || !unique.insert(model_id.as_str()) {
                return Err(ModelSelectionDiagnosticCode::InvalidRequest);
            }
        }
        if let Some(ceiling) = policy.reasoning_ceiling.as_deref() {
            if reasoning_rank(ceiling).is_none() {
                return Err(ModelSelectionDiagnosticCode::InvalidRequest);
            }
        }
        if policy.ownership == ModelSelectionOwnership::Automatic {
            if policy.allowed_model_ids.is_empty() && policy.reasoning_ceiling.is_none() {
                return Err(ModelSelectionDiagnosticCode::InvalidRequest);
            }
            if !policy.allowed_model_ids.is_empty()
                && !policy.allowed_model_ids.contains(&effective.model_id)
            {
                return Err(ModelSelectionDiagnosticCode::InvalidRequest);
            }
        }
        validate_choice_shape(effective)
    }

    pub(crate) fn evaluate_agent_request(
        request: AgentSelectionRequest,
        policy: &ModelSelectionPolicy,
        models: &[CodexModel],
        pending: Option<&PendingModelSelection>,
    ) -> Result<(AgentSelectionRequest, ModelSelectionApplication), ModelSelectionDiagnosticCode>
    {
        Self::validate_choice(&request.choice, models)?;
        if pending.is_some_and(|pending| pending.provenance == ModelSelectionProvenance::User) {
            return Err(ModelSelectionDiagnosticCode::PolicyBlocked);
        }
        if policy.user_locked {
            return Err(ModelSelectionDiagnosticCode::UserLocked);
        }
        let application = match policy.ownership {
            ModelSelectionOwnership::Manual => {
                return Err(ModelSelectionDiagnosticCode::ManualOwnership);
            }
            ModelSelectionOwnership::Recommend => ModelSelectionApplication::Recommendation,
            ModelSelectionOwnership::Automatic => {
                if !policy.allowed_model_ids.is_empty()
                    && !policy.allowed_model_ids.contains(&request.choice.model_id)
                {
                    return Err(ModelSelectionDiagnosticCode::PolicyBlocked);
                }
                if let Some(ceiling) = policy.reasoning_ceiling.as_deref() {
                    if !reasoning_within_ceiling(&request.choice.reasoning_effort, ceiling) {
                        return Err(ModelSelectionDiagnosticCode::PolicyBlocked);
                    }
                }
                ModelSelectionApplication::Automatic
            }
        };
        Ok((request, application))
    }

    pub(crate) fn pending_still_applicable(
        pending: &PendingModelSelection,
        policy: &ModelSelectionPolicy,
        models: &[CodexModel],
    ) -> bool {
        if Self::validate_choice(&pending.choice, models).is_err() {
            return false;
        }
        match pending.application {
            ModelSelectionApplication::Manual => true,
            ModelSelectionApplication::Recommendation => false,
            ModelSelectionApplication::Automatic => {
                !policy.user_locked
                    && policy.ownership == ModelSelectionOwnership::Automatic
                    && (policy.allowed_model_ids.is_empty()
                        || policy.allowed_model_ids.contains(&pending.choice.model_id))
                    && policy.reasoning_ceiling.as_deref().is_none_or(|ceiling| {
                        reasoning_within_ceiling(&pending.choice.reasoning_effort, ceiling)
                    })
            }
        }
    }

    pub(crate) fn dynamic_tool_spec() -> Value {
        json!({
            "type": "function",
            "name": MODEL_SELECTOR_TOOL_NAME,
            "description": "Inspect QuireForge's normalized model selector state, then request at most one policy-compliant model/reasoning change for the next turn. The executing turn cannot replace itself. Treat returned policy and catalog values as authoritative.",
            "inputSchema": {
                "oneOf": [
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["action"],
                        "properties": {
                            "action": {"const": "inspect"}
                        }
                    },
                    {
                        "type": "object",
                        "additionalProperties": false,
                        "required": ["action", "modelId", "reasoningEffort", "rationale"],
                        "properties": {
                            "action": {"const": "request"},
                            "modelId": {"type": "string", "minLength": 1, "maxLength": 128},
                            "reasoningEffort": {"type": "string", "minLength": 1, "maxLength": 32},
                            "rationale": {"type": "string", "minLength": 1, "maxLength": 240}
                        }
                    }
                ]
            }
        })
    }

    pub(crate) fn inspection_text(
        snapshot: &ModelSelectionSnapshot,
        models: &[CodexModel],
    ) -> Result<String, ModelSelectionDiagnosticCode> {
        let catalog = models
            .iter()
            .map(|model| AgentVisibleModel {
                model_id: model.id.clone(),
                display_name: model.display_name.clone(),
                supported_reasoning_efforts: model.supported_reasoning_efforts.clone(),
            })
            .collect::<Vec<_>>();
        serde_json::to_string(&json!({
            "effective": snapshot.effective,
            "pending": snapshot.pending,
            "policy": snapshot.policy,
            "catalog": catalog,
            "rule": "A request can affect only the next turn and is revalidated before turn/start."
        }))
        .map_err(|_| ModelSelectionDiagnosticCode::ControlUnavailable)
    }
}

pub(crate) fn diagnostic_message(code: ModelSelectionDiagnosticCode) -> &'static str {
    match code {
        ModelSelectionDiagnosticCode::InvalidRequest => {
            "The selector request did not match the closed QuireForge schema."
        }
        ModelSelectionDiagnosticCode::ConversationNotFound => {
            "The app-owned conversation reference is unavailable."
        }
        ModelSelectionDiagnosticCode::MetadataUnavailable => {
            "QuireForge selector metadata is unavailable."
        }
        ModelSelectionDiagnosticCode::CatalogUnavailable => {
            "The fresh Codex model catalog is unavailable."
        }
        ModelSelectionDiagnosticCode::ModelUnavailable => {
            "The requested model is not in the advertised catalog."
        }
        ModelSelectionDiagnosticCode::ReasoningUnavailable => {
            "The requested reasoning effort is not advertised for that model."
        }
        ModelSelectionDiagnosticCode::PolicyBlocked => {
            "The requested change exceeds the automatic-selection policy."
        }
        ModelSelectionDiagnosticCode::ManualOwnership => {
            "Manual ownership does not permit Codex to stage a selector change."
        }
        ModelSelectionDiagnosticCode::UserLocked => {
            "The user lock prevents Codex from staging a selector change."
        }
        ModelSelectionDiagnosticCode::RequestAlreadyMade => {
            "Only one model-selection request is allowed per turn."
        }
        ModelSelectionDiagnosticCode::ControlUnavailable => {
            "The app-owned selector control is unavailable."
        }
    }
}

pub(crate) fn current_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(i64::MAX)
}

fn validate_choice_shape(
    choice: &ModelSelectionChoice,
) -> Result<(), ModelSelectionDiagnosticCode> {
    if !valid_protocol_choice(&choice.model_id, 128)
        || !valid_protocol_choice(&choice.reasoning_effort, 32)
    {
        return Err(ModelSelectionDiagnosticCode::InvalidRequest);
    }
    Ok(())
}

fn valid_protocol_choice(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:/-".contains(&byte))
}

fn contains_unsafe_control(value: &str) -> bool {
    value.chars().any(|character| {
        let code = character as u32;
        code <= 0x1f
            || (0x7f..=0x9f).contains(&code)
            || (0x200b..=0x200f).contains(&code)
            || (0x202a..=0x202e).contains(&code)
            || (0x2060..=0x206f).contains(&code)
            || code == 0xfeff
    })
}

fn reasoning_rank(value: &str) -> Option<u8> {
    match value {
        "none" => Some(0),
        "minimal" => Some(1),
        "low" => Some(2),
        "medium" => Some(3),
        "high" => Some(4),
        "xhigh" => Some(5),
        "max" => Some(6),
        "ultra" => Some(7),
        _ => None,
    }
}

fn reasoning_within_ceiling(value: &str, ceiling: &str) -> bool {
    reasoning_rank(value)
        .zip(reasoning_rank(ceiling))
        .is_some_and(|(value, ceiling)| value <= ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn models() -> Vec<CodexModel> {
        vec![
            CodexModel {
                id: "gpt-5.6-sol".to_owned(),
                display_name: "GPT-5.6 Sol".to_owned(),
                is_default: true,
                default_reasoning_effort: "medium".to_owned(),
                supported_reasoning_efforts: vec![
                    "medium".to_owned(),
                    "high".to_owned(),
                    "xhigh".to_owned(),
                ],
            },
            CodexModel {
                id: "gpt-5.6-terra".to_owned(),
                display_name: "GPT-5.6 Terra".to_owned(),
                is_default: false,
                default_reasoning_effort: "medium".to_owned(),
                supported_reasoning_efforts: vec!["medium".to_owned(), "high".to_owned()],
            },
        ]
    }

    fn automatic_policy() -> ModelSelectionPolicy {
        ModelSelectionPolicy {
            ownership: ModelSelectionOwnership::Automatic,
            user_locked: false,
            allowed_model_ids: vec!["gpt-5.6-sol".to_owned(), "gpt-5.6-terra".to_owned()],
            reasoning_ceiling: Some("high".to_owned()),
        }
    }

    #[test]
    fn parses_only_closed_inspect_and_request_arguments() {
        assert_eq!(
            ModelSelectionService::parse_arguments(json!({"action": "inspect"})),
            Ok(ModelSelectorArguments::Inspect)
        );
        assert!(matches!(
            ModelSelectionService::parse_arguments(json!({
                "action": "request",
                "modelId": "gpt-5.6-terra",
                "reasoningEffort": "high",
                "rationale": "Use the bounded lower-cost option."
            })),
            Ok(ModelSelectorArguments::Request(_))
        ));
        assert!(ModelSelectionService::parse_arguments(json!({
            "action": "request",
            "modelId": "gpt-5.6-terra",
            "reasoningEffort": "high",
            "rationale": "Use the bounded lower-cost option.",
            "prompt": "hidden"
        }))
        .is_err());
        assert!(ModelSelectionService::parse_arguments(json!({
            "action": "request",
            "modelId": "gpt-5.6-terra",
            "reasoningEffort": "high",
            "rationale": "bad\u{202e}text"
        }))
        .is_err());
    }

    #[test]
    fn enforces_manual_lock_allowlist_and_reasoning_ceiling() {
        let request = AgentSelectionRequest {
            choice: ModelSelectionChoice {
                model_id: "gpt-5.6-terra".to_owned(),
                reasoning_effort: "high".to_owned(),
            },
            rationale: "Use Terra for the next bounded turn.".to_owned(),
        };
        let mut policy = automatic_policy();
        assert_eq!(
            ModelSelectionService::evaluate_agent_request(
                request.clone(),
                &policy,
                &models(),
                None,
            )
            .map(|(_, application)| application),
            Ok(ModelSelectionApplication::Automatic)
        );

        policy.user_locked = true;
        assert_eq!(
            ModelSelectionService::evaluate_agent_request(
                request.clone(),
                &policy,
                &models(),
                None,
            ),
            Err(ModelSelectionDiagnosticCode::UserLocked)
        );

        policy.user_locked = false;
        policy.allowed_model_ids = vec!["gpt-5.6-sol".to_owned()];
        assert_eq!(
            ModelSelectionService::evaluate_agent_request(
                request.clone(),
                &policy,
                &models(),
                None,
            ),
            Err(ModelSelectionDiagnosticCode::PolicyBlocked)
        );

        policy.allowed_model_ids.push("gpt-5.6-terra".to_owned());
        let excessive = AgentSelectionRequest {
            choice: ModelSelectionChoice {
                reasoning_effort: "xhigh".to_owned(),
                ..request.choice
            },
            rationale: request.rationale,
        };
        assert_eq!(
            ModelSelectionService::evaluate_agent_request(excessive, &policy, &models(), None),
            Err(ModelSelectionDiagnosticCode::ReasoningUnavailable)
        );

        let manual_pending = PendingModelSelection {
            choice: ModelSelectionChoice {
                model_id: "gpt-5.6-sol".to_owned(),
                reasoning_effort: "high".to_owned(),
            },
            provenance: ModelSelectionProvenance::User,
            application: ModelSelectionApplication::Manual,
            rationale: "User selected the next turn.".to_owned(),
            requested_at_ms: 1,
        };
        assert_eq!(
            ModelSelectionService::evaluate_agent_request(
                AgentSelectionRequest {
                    choice: ModelSelectionChoice {
                        model_id: "gpt-5.6-terra".to_owned(),
                        reasoning_effort: "high".to_owned(),
                    },
                    rationale: "Do not replace the user's pending choice.".to_owned(),
                },
                &automatic_policy(),
                &models(),
                Some(&manual_pending),
            ),
            Err(ModelSelectionDiagnosticCode::PolicyBlocked)
        );
    }

    #[test]
    fn recommendation_never_auto_applies_and_manual_always_can() {
        let choice = ModelSelectionChoice {
            model_id: "gpt-5.6-terra".to_owned(),
            reasoning_effort: "high".to_owned(),
        };
        let policy = automatic_policy();
        let recommendation = PendingModelSelection {
            choice: choice.clone(),
            provenance: ModelSelectionProvenance::Codex,
            application: ModelSelectionApplication::Recommendation,
            rationale: "Consider Terra.".to_owned(),
            requested_at_ms: 1,
        };
        assert!(!ModelSelectionService::pending_still_applicable(
            &recommendation,
            &policy,
            &models()
        ));
        let manual = PendingModelSelection {
            application: ModelSelectionApplication::Manual,
            provenance: ModelSelectionProvenance::User,
            ..recommendation
        };
        assert!(ModelSelectionService::pending_still_applicable(
            &manual,
            &policy,
            &models()
        ));
    }
}
