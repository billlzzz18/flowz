use crate::domain::{
    ExecutionMode, FailurePolicy, FindingSeverity, ReducerSpec, RunBudget, RunRequest, WorkflowItem,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
    pub severity: FindingSeverity,
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

        // Document size bounds (DoS prevention)
        if self.nodes.len() > 1000 {
            findings.push(finding(
                "too_many_nodes",
                FindingSeverity::Error,
                "document exceeds maximum of 1000 nodes",
                None,
                None,
            ));
        }
        if self.edges.len() > 2000 {
            findings.push(finding(
                "too_many_edges",
                FindingSeverity::Error,
                "document exceeds maximum of 2000 edges",
                None,
                None,
            ));
        }
        let total_string_len: usize = self.id.len()
            + self.title.len()
            + self.nodes.iter().map(|n| n.id.len()).sum::<usize>()
            + self.edges.iter().map(|e| e.id.len() + e.source.len() + e.target.len()).sum::<usize>();
        if total_string_len > 1_000_000 {
            findings.push(finding(
                "document_too_large",
                FindingSeverity::Error,
                "document exceeds maximum string length of 1MB",
                None,
                None,
            ));
        }

        if self.id.trim().is_empty() {
            findings.push(finding(
                "document_id_required",
                FindingSeverity::Error,
                "document id is required",
                None,
                None,
            ));
        }
        if self.title.trim().is_empty() {
            findings.push(finding(
                "title_required",
                FindingSeverity::Error,
                "document title is required",
                None,
                None,
            ));
        }
        if self.nodes.is_empty() {
            findings.push(finding(
                "nodes_required",
                FindingSeverity::Error,
                "canvas must contain at least one node",
                None,
                None,
            ));
        }

        for node in &self.nodes {
            if !node_ids.insert(node.id.clone()) {
                findings.push(finding(
                    "duplicate_node_id",
                    FindingSeverity::Error,
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
                                FindingSeverity::Error,
                                "workflow item id must be unique",
                                Some(node.id.clone()),
                                None,
                            ));
                        }
                        if item.brief.trim().is_empty() {
                            findings.push(finding(
                                "item_brief_required",
                                FindingSeverity::Error,
                                "workflow item brief is required",
                                Some(node.id.clone()),
                                None,
                            ));
                        }
                        if item.prompt.trim().is_empty() {
                            findings.push(finding(
                                "item_prompt_required",
                                FindingSeverity::Error,
                                "workflow item prompt is required",
                                Some(node.id.clone()),
                                None,
                            ));
                        }
                        // Output schema validation (workflow validator requires it)
                        if item.schema.is_none() {
                            findings.push(finding(
                                "item_schema_required",
                                FindingSeverity::Error,
                                "workflow item output schema is required",
                                Some(node.id.clone()),
                                None,
                            ));
                        }
                    }
                    None => findings.push(finding(
                        "item_payload_required",
                        FindingSeverity::Error,
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
                            FindingSeverity::Error,
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
                FindingSeverity::Error,
                "canvas supports at most one reducer node",
                None,
                None,
            ));
        }

        // Cycle detection for edges
        if !findings.iter().any(|f| f.severity == FindingSeverity::Error) {
            if let Some(cycle_finding) = detect_cycles(&self.nodes, &self.edges) {
                findings.push(cycle_finding);
            }
        }

        for edge in &self.edges {
            if !node_ids.contains(&edge.source) || !node_ids.contains(&edge.target) {
                findings.push(finding(
                    "edge_node_not_found",
                    FindingSeverity::Error,
                    "edge source and target must reference existing nodes",
                    None,
                    Some(edge.id.clone()),
                ));
            }
            if edge.source == edge.target {
                findings.push(finding(
                    "self_edge",
                    FindingSeverity::Error,
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
                            FindingSeverity::Error,
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
        if findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error)
        {
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

        // Validate max_agent_calls against item count (minimum 1 call per item)
        let min_calls = items.len() as u32;
        let max_agent_calls = self.policy.max_agent_calls.map(|m| m.max(min_calls));

        Ok(RunRequest {
            items,
            reducer,
            mode: self.policy.mode.clone(),
            failure_policy: self.policy.failure_policy.clone(),
            max_concurrency: self.policy.max_concurrency,
            max_agent_calls,
            confirmation_required: self.policy.confirmation_required,
            run_budget: self.policy.run_budget.clone(),
        })
    }
}

fn finding(
    code: &str,
    severity: FindingSeverity,
    message: &str,
    node_id: Option<String>,
    edge_id: Option<String>,
) -> CanvasFinding {
    CanvasFinding {
        code: code.to_string(),
        severity,
        message: message.to_string(),
        node_id,
        edge_id,
    }
}

// Cycle detection for dependency edges
fn detect_cycles(nodes: &[CanvasNode], edges: &[CanvasEdge]) -> Option<CanvasFinding> {
    use std::collections::HashMap;

    let node_ids: Vec<&String> = nodes.iter().map(|n| &n.id).collect();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for id in &node_ids {
        adj.insert((*id).clone(), Vec::new());
    }
    for edge in edges {
        if let Some(targets) = adj.get_mut(&edge.source) {
            targets.push(edge.target.clone());
        }
    }

    let mut visited = HashMap::new();
    let mut rec_stack = HashMap::new();

    fn dfs(
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashMap<String, bool>,
        rec_stack: &mut HashMap<String, bool>,
    ) -> bool {
        visited.insert(node.to_string(), true);
        rec_stack.insert(node.to_string(), true);

        if let Some(neighbors) = adj.get(node) {
            for neighbor in neighbors {
                if *visited.get(neighbor).unwrap_or(&false) == false {
                    if dfs(neighbor, adj, visited, rec_stack) {
                        return true;
                    }
                } else if *rec_stack.get(neighbor).unwrap_or(&false) {
                    return true;
                }
            }
        }

        rec_stack.insert(node.to_string(), false);
        false
    }

    for node in &node_ids {
        if *visited.get(*node).unwrap_or(&false) == false {
            if dfs(node, &adj, &mut visited, &mut rec_stack) {
                return Some(finding(
                    "dependency_cycle",
                    FindingSeverity::Error,
                    "multi-node dependency cycle detected",
                    None,
                    None,
                ));
            }
        }
    }

    None
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

    #[test]
    fn item_without_schema_is_reported() {
        let mut doc = document();
        doc.nodes[0].item.as_mut().unwrap().schema = None;
        assert!(
            doc.validate()
                .iter()
                .any(|finding| finding.code == "item_schema_required")
        );
    }

    #[test]
    fn dependency_cycle_is_detected() {
        let mut doc = document();
        doc.nodes.push(CanvasNode {
            id: "node-2".to_string(),
            kind: CanvasNodeKind::Item,
            position: CanvasPosition { x: 1.0, y: 0.0 },
            item: Some(item("item-2")),
            reducer: None,
        });
        doc.nodes.push(CanvasNode {
            id: "node-3".to_string(),
            kind: CanvasNodeKind::Item,
            position: CanvasPosition { x: 2.0, y: 0.0 },
            item: Some(item("item-3")),
            reducer: None,
        });
        // Create cycle: node-1 -> node-2 -> node-3 -> node-1
        doc.edges.push(CanvasEdge {
            id: "edge-1".to_string(),
            source: "node-1".to_string(),
            target: "node-2".to_string(),
        });
        doc.edges.push(CanvasEdge {
            id: "edge-2".to_string(),
            source: "node-2".to_string(),
            target: "node-3".to_string(),
        });
        doc.edges.push(CanvasEdge {
            id: "edge-3".to_string(),
            source: "node-3".to_string(),
            target: "node-1".to_string(),
        });
        assert!(
            doc.validate()
                .iter()
                .any(|finding| finding.code == "dependency_cycle")
        );
    }

    #[test]
    fn max_agent_calls_is_at_least_item_count() {
        let mut doc = document();
        doc.policy.max_agent_calls = Some(1); // Less than item count (1)
        let request = doc.to_run_request().expect("should work with 1 item");
        assert_eq!(request.max_agent_calls, Some(1));

        doc.nodes.push(CanvasNode {
            id: "node-2".to_string(),
            kind: CanvasNodeKind::Item,
            position: CanvasPosition { x: 1.0, y: 0.0 },
            item: Some(item("item-2")),
            reducer: None,
        });
        let request = doc.to_run_request().expect("should bump max_agent_calls to item count");
        assert_eq!(request.max_agent_calls, Some(2));
    }
}
