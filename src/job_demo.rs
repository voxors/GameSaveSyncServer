use tokio_util::sync::CancellationToken;

use crate::job_scheduler::Job;

#[derive(Debug)]
pub struct DemoJob;

impl Job for DemoJob {
    fn name(&self) -> &'static str {
        "DemoJob"
    }

    fn execute(
        &self,
        _cancellation_token: CancellationToken,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("potato");
        Ok(())
    }
}
