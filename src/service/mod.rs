pub mod cron;
pub mod decision;
pub mod supervisor;

#[derive(Default)]
pub struct FlowzService {
    pub cron: cron::CronService,
    pub supervisor: supervisor::SupervisorService,
    pub decision: decision::DecisionService,
}

impl FlowzService {
    pub fn new() -> Self {
        Self::default()
    }
}
