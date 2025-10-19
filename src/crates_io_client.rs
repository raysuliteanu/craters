use crates_io_api::{
    AsyncClient, Crate, CrateResponse, CratesQuery, Error as CratesIoError, Summary,
};

pub struct HttpClient {
    client: AsyncClient,
}

#[allow(dead_code)]
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

    pub async fn search(&self, search: &str) -> Result<Vec<Crate>, CratesIoError> {
        let search_results = self
            .search_crates(CratesQuery::builder().search(search).page_size(10).build())
            .await?;
        Ok(search_results)
    }

    pub async fn search_categories(&self, category: &str) -> Result<Vec<Crate>, CratesIoError> {
        let search_results = self
            .search_crates(
                CratesQuery::builder()
                    .category(category)
                    .page_size(10)
                    .build(),
            )
            .await?;
        Ok(search_results)
    }

    pub async fn search_keywords(&self, keyword: &str) -> Result<Vec<Crate>, CratesIoError> {
        let search_results = self
            .search_crates(CratesQuery::builder().search(keyword).page_size(10).build())
            .await?;
        Ok(search_results)
    }

    async fn search_crates(&self, query: CratesQuery) -> Result<Vec<Crate>, CratesIoError> {
        let search_results = self.client.crates(query).await?;
        Ok(search_results.crates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_categories() {
        let client = HttpClient::new();
        let search_results = client
            .search_categories("command-line-utilities")
            .await
            .unwrap();
        assert!(!search_results.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_crate_info() {
        let client = HttpClient::new();
        let crate_info = client.fetch_crate_info("serde").await.unwrap();
        assert_eq!(crate_info.crate_data.name, "serde");
    }

    #[tokio::test]
    async fn test_search() {
        let client = HttpClient::new();
        let search_results = client.search("serde").await.unwrap();
        assert!(!search_results.is_empty());
    }
}
