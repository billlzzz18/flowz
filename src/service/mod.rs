pub mod cron;
pub mod decision;
pub mod supervisor;
pub mod workflow;

pub struct FlowzService {
    pub workflow: workflow::WorkflowService,
    pub cron: cron::CronService,
    pub supervisor: supervisor::SupervisorService,
    pub decision: decision::DecisionService,
}

impl FlowzService {
    pub fn new() -> Result<Self, decider::DeciderError> {
        Ok(Self {
            workflow: workflow::WorkflowService::new(),
            cron: cron::CronService::new(),
            supervisor: supervisor::SupervisorService::new(),
            decision: decision::DecisionService::new()?,
        })
    }
}

impl Default for FlowzService {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            workflow: workflow::WorkflowService::new(),
            cron: cron::CronService::new(),
            supervisor: supervisor::SupervisorService::new(),
            decision: decision::DecisionService::with_backends(None, None),
        })
    }
}
