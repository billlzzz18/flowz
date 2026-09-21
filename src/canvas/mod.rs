use crate::domain::{
    ExecutionMode, FailurePolicy, ReducerSpec, RunBudget, RunRequest, WorkflowItem,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasDocument {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub nodes: Vec<CanvasNode>,
    #[serde(default)]
    pub edges: Vec<CanvasEdge>,
    #[serde(default)]
    pub policy: CanvasRunPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNode {
    pub id: String,
    pub kind: CanvasNodeKind,
    pub position: CanvasPosition,
    #[serde(default)]
    pub item: Option<WorkflowItem>,
    #[serde(default)]
    pub reducer: Option<ReducerSpec>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CanvasNodeKind {
    Item,
    Reducer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanvasPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanvasEdge {
    pub id: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasRunPolicy {
    #[serde(default)]
    pub mode: ExecutionMode,
    #[serde(default)]
    pub failure_policy: FailurePolicy,
    #[serde(default)]
    pub max_concurrency: Option<u32>,
    #[serde(default)]
    pub max_agent_calls: Option<u32>,
    #[serde(default)]
    pub confirmation_required: Option<bool>,
    #[serde(default)]
    pub run_budget: RunBudget,
}

impl Default for CanvasRunPolicy {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::default(),
            failure_policy: FailurePolicy::default(),
            max_concurrency: None,
            max_agent_calls: None,
            confirmation_required: None,
            run_budget: RunBudget::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanvasFinding {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub node_id: Option<String>,
    pub edge_id: Option<String>,
}

impl CanvasDocument {
    pub fn validate(&self) -> Vec<CanvasFinding> {
        let mut findings = Vec::new();
        let mut node_ids = HashSet::new();
        let mut item_ids = HashSet::new();
        let mut reducer_count = 0usize;

        if self.id.trim().is_empty() {
            findings.push(finding(
                "document_id_required",
                "error",
                "document id is required",
                None,
                None,
            ));
        }
        if self.title.trim().is_empty() {
            findings.push(finding(
                "title_required",
                "error",
                "document title is required",
                None,
                None,
            ));
        }
        if self.nodes.is_empty() {
            findings.push(finding(
                "nodes_required",
                "error",
                "canvas must contain at least one node",
                None,
                None,
            ));
        }

        for node in &self.nodes {
            if !node_ids.insert(node.id.clone()) {
                findings.push(finding(
                    "duplicate_node_id",
                    "error",
                    "node id must be unique",
                    Some(node.id.clone()),
                    None,
                ));
            }
            match node.kind {
                CanvasNodeKind::Item => match &node.item {
                    Some(item) => {
                        if !item_ids.insert(item.id.clone()) {
                            findings.push(finding(
                                "duplicate_item_id",
                                "error",
                                "workflow item id must be unique",
                                Some(node.id.clone()),
                                None,
                            ));
                        }
                        if item.brief.trim().is_empty() {
                            findings.push(finding(
                                "item_brief_required",
                                "error",
                                "workflow item brief is required",
                                Some(node.id.clone()),
                                None,
                            ));
                        }
                        if item.prompt.trim().is_empty() {
                            findings.push(finding(
                                "item_prompt_required",
                                "error",
                                "workflow item prompt is required",
                                Some(node.id.clone()),
                                None,
                            ));
                        }
                    }
                    None => findings.push(finding(
                        "item_payload_required",
                        "error",
                        "item node must contain an item payload",
                        Some(node.id.clone()),
                        None,
                    )),
                },
                CanvasNodeKind::Reducer => {
                    reducer_count += 1;
                    if node.reducer.is_none() {
                        findings.push(finding(
                            "reducer_payload_required",
                            "error",
                            "reducer node must contain a reducer payload",
                            Some(node.id.clone()),
                            None,
                        ));
                    }
                }
            }
        }

        if reducer_count > 1 {
            findings.push(finding(
                "multiple_reducers",
                "error",
                "canvas supports at most one reducer node",
                None,
                None,
            ));
        }

        for edge in &self.edges {
            if !node_ids.contains(&edge.source) || !node_ids.contains(&edge.target) {
                findings.push(finding(
                    "edge_node_not_found",
                    "error",
                    "edge source and target must reference existing nodes",
                    None,
                    Some(edge.id.clone()),
                ));
            }
            if edge.source == edge.target {
                findings.push(finding(
                    "self_edge",
                    "error",
                    "a node cannot connect to itself",
                    None,
                    Some(edge.id.clone()),
                ));
            }
        }

        for node in &self.nodes {
            if let Some(reducer) = &node.reducer {
                for input_id in &reducer.input_item_ids {
                    if !item_ids.contains(input_id) {
                        findings.push(finding(
                            "reducer_input_not_found",
                            "error",
                            "reducer input item was not found",
                            Some(node.id.clone()),
                            None,
                        ));
                    }
                }
            }
        }

        findings
    }

    pub fn to_run_request(&self) -> Result<RunRequest, Vec<CanvasFinding>> {
        let findings = self.validate();
        if findings.iter().any(|finding| finding.severity == "error") {
            return Err(findings);
        }

        let mut items = Vec::new();
        let mut reducer = None;
        for node in &self.nodes {
            match node.kind {
                CanvasNodeKind::Item => {
                    if let Some(item) = &node.item {
                        items.push(item.clone());
                    }
                }
                CanvasNodeKind::Reducer => reducer = node.reducer.clone(),
            }
        }

        Ok(RunRequest {
            items,
            reducer,
            mode: self.policy.mode.clone(),
            failure_policy: self.policy.failure_policy.clone(),
            max_concurrency: self.policy.max_concurrency,
            max_agent_calls: self.policy.max_agent_calls,
            confirmation_required: self.policy.confirmation_required,
            run_budget: self.policy.run_budget.clone(),
        })
    }
}

fn finding(
    code: &str,
    severity: &str,
    message: &str,
    node_id: Option<String>,
    edge_id: Option<String>,
) -> CanvasFinding {
    CanvasFinding {
        code: code.to_string(),
        severity: severity.to_string(),
        message: message.to_string(),
        node_id,
        edge_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EffortLevel, IterationBudget, SandboxMode, SubagentRole, TimeBudget};

    fn item(id: &str) -> WorkflowItem {
        WorkflowItem {
            id: id.to_string(),
            prompt: "Do the work".to_string(),
            brief: "Work item".to_string(),
            schema: Some(serde_json::json!({"type": "object"})),
            input_files: vec![],
            sandbox: SandboxMode::Isolated,
            effort_level: EffortLevel::Standard,
            max_duration_secs: None,
            time_budget: TimeBudget::default(),
            iteration_budget: IterationBudget::default(),
            role: SubagentRole::Leaf,
        }
    }

    fn document() -> CanvasDocument {
        CanvasDocument {
            id: "doc-1".to_string(),
            title: "Test".to_string(),
            nodes: vec![CanvasNode {
                id: "node-1".to_string(),
                kind: CanvasNodeKind::Item,
                position: CanvasPosition { x: 0.0, y: 0.0 },
                item: Some(item("item-1")),
                reducer: None,
            }],
            edges: vec![],
            policy: CanvasRunPolicy::default(),
        }
    }

    #[test]
    fn valid_document_converts_to_run_request() {
        let request = document()
            .to_run_request()
            .expect("document should be valid");
        assert_eq!(request.items.len(), 1);
        assert_eq!(request.items[0].id, "item-1");
    }

    #[test]
    fn duplicate_item_ids_are_reported() {
        let mut doc = document();
        doc.nodes.push(CanvasNode {
            id: "node-2".to_string(),
            kind: CanvasNodeKind::Item,
            position: CanvasPosition { x: 1.0, y: 1.0 },
            item: Some(item("item-1")),
            reducer: None,
        });
        assert!(
            doc.validate()
                .iter()
                .any(|finding| finding.code == "duplicate_item_id")
        );
    }

    #[test]
    fn missing_edge_endpoint_is_reported() {
        let mut doc = document();
        doc.edges.push(CanvasEdge {
            id: "edge-1".to_string(),
            source: "node-1".to_string(),
            target: "missing".to_string(),
        });
        assert!(
            doc.validate()
                .iter()
                .any(|finding| finding.code == "edge_node_not_found")
        );
    }
}
