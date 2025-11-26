use crate::const_var::{MANIFEST_PATH, MANIFEST_URL};
use crate::file_system::write_bytes_to_tmp_file;
use crate::job_scheduler::Job;
use crate::ludusavi::yaml_import;
use async_trait::async_trait;
use reqwest::header::{ETAG, IF_NONE_MATCH};
use reqwest::{Client, Method, StatusCode};
use tokio::fs;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Default)]
pub struct LudusaviJob {
    etag: String,
}

#[async_trait]
impl Job for LudusaviJob {
    fn name(&self) -> &'static str {
        "Ludusavi Job"
    }

    async fn execute(
        &mut self,
        _cancellation_token: CancellationToken,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = Client::new();
        let response = client
            .request(Method::GET, MANIFEST_URL)
            .header(IF_NONE_MATCH, &self.etag)
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            self.etag = response
                .headers()
                .get(ETAG)
                .ok_or::<Box<dyn std::error::Error + Send + Sync>>(
                    format!("{} not found", ETAG).into(),
                )?
                .to_str()?
                .to_string();

            let bytes = response.bytes().await?;
            write_bytes_to_tmp_file(MANIFEST_PATH, &bytes).await?;
            yaml_import(MANIFEST_PATH).await?;
            let _ = fs::remove_file(MANIFEST_PATH).await;
        } else if response.status() != StatusCode::NOT_MODIFIED {
            response.error_for_status()?;
        }

        Ok(())
    }
}
