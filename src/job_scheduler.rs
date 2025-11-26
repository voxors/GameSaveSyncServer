use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{sync::Mutex, task::JoinHandle};
use tokio_util::sync::CancellationToken;

#[async_trait]
pub trait Job: Send + Sync + Debug {
    fn name(&self) -> &'static str;
    async fn execute(
        &mut self,
        cancellation_token: CancellationToken,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[derive(Debug)]
struct JobEntry {
    pub interval: chrono::Duration,
    pub last_executed: DateTime<Utc>,
    pub cancellation_token: CancellationToken,
    pub job: Arc<Mutex<dyn Job>>,
    pub is_running: Arc<AtomicBool>,
}

#[derive(Debug)]
pub struct JobScheduler {
    jobs: Arc<Mutex<Vec<JobEntry>>>,
    scheduler_task_handle: Option<JoinHandle<()>>,
    cancellation_token: CancellationToken,
}

type ToRunType = (Arc<Mutex<dyn Job>>, CancellationToken, Arc<AtomicBool>);

fn collect_ready_jobs(jobs: &mut [JobEntry], task_cancel: &CancellationToken) -> Vec<ToRunType> {
    let now = Utc::now();
    jobs.iter_mut()
        .filter_map(|job_entry| {
            if job_entry.last_executed + job_entry.interval <= now {
                job_entry.last_executed = now;
                let child_token = task_cancel.child_token();
                job_entry.cancellation_token = child_token.clone();
                job_entry.is_running.store(true, Ordering::Relaxed);

                Some((
                    job_entry.job.clone(),
                    child_token,
                    job_entry.is_running.clone(),
                ))
            } else {
                None
            }
        })
        .collect()
}

async fn scheduler_loop(jobs: Arc<Mutex<Vec<JobEntry>>>, task_cancel: CancellationToken) {
    while !task_cancel.is_cancelled() {
        let mut guard = jobs.lock().await;
        let to_run = { collect_ready_jobs(&mut guard, &task_cancel) };

        for (job, token, is_running) in to_run {
            tokio::spawn(run_job(job, token, is_running));
        }

        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

async fn run_job(job: Arc<Mutex<dyn Job>>, token: CancellationToken, is_running: Arc<AtomicBool>) {
    if let Err(err) = job.lock().await.execute(token.clone()).await {
        eprintln!(
            "Error while executing job: {}, err: {}",
            job.lock().await.name(),
            err
        );
    }
    is_running.store(false, Ordering::Relaxed);
}

impl JobScheduler {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
            scheduler_task_handle: None,
            cancellation_token: CancellationToken::default(),
        }
    }

    pub fn start_scheduler(&mut self) {
        if self.scheduler_task_handle.is_none() {
            self.cancellation_token = CancellationToken::new();
            let task_cancel = self.cancellation_token.clone();
            let jobs = self.jobs.clone();
            self.scheduler_task_handle = Some(tokio::spawn(scheduler_loop(jobs, task_cancel)));
        } else {
            println!("Tried to start an already started scheduler")
        }
    }

    pub fn stop_scheduler(&mut self) {
        if self.scheduler_task_handle.is_some() {
            self.cancellation_token.cancel();
            self.scheduler_task_handle = None;
        } else {
            println!("Tried to stop an inactive scheduler")
        }
    }

    pub async fn add_job(&mut self, job: impl Job + 'static, interval: chrono::Duration) {
        self.jobs.lock().await.push(JobEntry {
            interval,
            last_executed: DateTime::<Utc>::MIN_UTC,
            job: Arc::new(Mutex::new(job)),
            cancellation_token: CancellationToken::default(),
            is_running: Arc::new(AtomicBool::default()),
        });
    }
}

impl Drop for JobScheduler {
    fn drop(&mut self) {
        self.stop_scheduler();
    }
}
