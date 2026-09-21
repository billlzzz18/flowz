pub mod workflow;
pub mod cron;
pub mod supervisor;

pub struct FlowzService {
    pub workflow: workflow::WorkflowService,
    pub cron: cron::CronService,
    pub supervisor: supervisor::SupervisorService,
}

impl FlowzService {
    pub fn new() -> Self {
        Self {
            workflow: workflow::WorkflowService::new(),
            cron: cron::CronService::new(),
            supervisor: supervisor::SupervisorService::new(),
        }
    }
}

impl Default for FlowzService {
    fn default() -> Self {
        Self::new()
    }
}