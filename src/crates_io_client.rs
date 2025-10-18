use crates_io_api::{AsyncClient, CrateResponse, Error as CratesIoError, Summary};

pub struct HttpClient {
    client: AsyncClient,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: AsyncClient::new(
                "craters/0.1.0 (https://github.com/raysuliteanu/craters)",
                std::time::Duration::from_millis(1000),
            )
            .expect("Failed to build Crates.io client"),
        }
    }

    pub async fn fetch_summary(&self) -> Result<Summary, CratesIoError> {
        let summary = self.client.summary().await?;
        Ok(summary)
    }

    pub async fn fetch_crate_info(&self, crate_name: &str) -> Result<CrateResponse, CratesIoError> {
        let crate_info = self.client.get_crate(crate_name).await?;
        Ok(crate_info)
    }
}
