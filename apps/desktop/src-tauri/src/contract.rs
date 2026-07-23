use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopBootstrap {
    pub schema_version: u16,
    pub product: ProductIdentity,
    pub capabilities: Vec<CapabilitySummary>,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductIdentity {
    pub name: &'static str,
    pub tagline: &'static str,
    pub description: &'static str,
    pub identifier: &'static str,
    pub executable: &'static str,
    pub version: &'static str,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitySummary {
    pub id: &'static str,
    pub label: &'static str,
    pub state: CapabilityState,
    pub milestone: u16,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityState {
    Ready,
}

impl DesktopBootstrap {
    pub fn current() -> Self {
        Self {
            schema_version: 1,
            product: ProductIdentity {
                name: "QuireForge",
                tagline: "Build boldly. Work locally.",
                description: "An unofficial native Linux workspace for Codex",
                identifier: "io.github.codeframe78.QuireForge",
                executable: "quireforge",
                version: env!("CARGO_PKG_VERSION"),
            },
            capabilities: vec![
                CapabilitySummary {
                    id: "desktop-foundation",
                    label: "Desktop foundation",
                    state: CapabilityState::Ready,
                    milestone: 3,
                },
                CapabilitySummary {
                    id: "codex-runtime",
                    label: "Codex runtime adapter",
                    state: CapabilityState::Ready,
                    milestone: 4,
                },
                CapabilitySummary {
                    id: "codex-auth",
                    label: "Codex authentication",
                    state: CapabilityState::Ready,
                    milestone: 5,
                },
                CapabilitySummary {
                    id: "project-attachments",
                    label: "Local project attachments",
                    state: CapabilityState::Ready,
                    milestone: 6,
                },
                CapabilitySummary {
                    id: "conversation-runtime",
                    label: "Native conversation runtime",
                    state: CapabilityState::Ready,
                    milestone: 7,
                },
                CapabilitySummary {
                    id: "integrated-terminal",
                    label: "Integrated terminal",
                    state: CapabilityState::Ready,
                    milestone: 12,
                },
                CapabilitySummary {
                    id: "integration-center",
                    label: "Integration Center",
                    state: CapabilityState::Ready,
                    milestone: 14,
                },
                CapabilitySummary {
                    id: "safe-file-previews",
                    label: "Safe file previews",
                    state: CapabilityState::Ready,
                    milestone: 15,
                },
                CapabilitySummary {
                    id: "conversation-attachments",
                    label: "Conversation image attachments",
                    state: CapabilityState::Ready,
                    milestone: 15,
                },
                CapabilitySummary {
                    id: "desktop-integration",
                    label: "Reviewed desktop integration",
                    state: CapabilityState::Ready,
                    milestone: 15,
                },
                CapabilitySummary {
                    id: "scheduled-task-catalog",
                    label: "Read-only scheduled task catalog",
                    state: CapabilityState::Ready,
                    milestone: 17,
                },
                CapabilitySummary {
                    id: "agent-model-selection",
                    label: "Policy-bounded next-turn selection",
                    state: CapabilityState::Ready,
                    milestone: 18,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DesktopBootstrap;

    #[test]
    fn serialized_contract_matches_the_shared_fixture() {
        let actual = serde_json::to_value(DesktopBootstrap::current())
            .expect("desktop bootstrap must serialize");
        let expected: serde_json::Value =
            serde_json::from_str(include_str!("../../fixtures/desktop-bootstrap.json"))
                .expect("shared desktop bootstrap fixture must be valid JSON");

        assert_eq!(actual, expected);
    }
}
