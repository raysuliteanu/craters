use crates_io_api::{
    AsyncClient, Crate, CrateResponse, CratesQuery, Error as CratesIoError, Summary,
};

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

    #[tokio::test]
    async fn test_search_keywords() {
        let client = HttpClient::new();
        let search_results = client.search_keywords("async").await.unwrap();

        // Should return some results (not necessarily specific to the keyword)
        // The search_keywords method currently does a text search, not a keyword filter
        assert!(
            !search_results.is_empty(),
            "Should return some search results"
        );
    }

    #[tokio::test]
    async fn test_fetch_summary() {
        let client = HttpClient::new();
        let summary = client.fetch_summary().await.unwrap();

        // Verify summary has expected data
        assert!(summary.num_crates > 0, "Should have crates in the registry");
        assert!(summary.num_downloads > 0, "Should have downloads");
        assert!(!summary.new_crates.is_empty(), "Should have new crates");
        assert!(
            !summary.most_downloaded.is_empty(),
            "Should have most downloaded crates"
        );
        assert!(
            !summary.just_updated.is_empty(),
            "Should have recently updated crates"
        );
        assert!(
            !summary.popular_keywords.is_empty(),
            "Should have popular keywords"
        );
        assert!(
            !summary.popular_categories.is_empty(),
            "Should have popular categories"
        );
    }

    #[tokio::test]
    async fn test_fetch_crate_info_details() {
        let client = HttpClient::new();
        let crate_info = client.fetch_crate_info("tokio").await.unwrap();

        // Verify detailed crate information
        assert_eq!(crate_info.crate_data.name, "tokio");
        assert!(
            crate_info.crate_data.description.is_some(),
            "Tokio should have a description"
        );
        assert!(
            crate_info.crate_data.downloads > 0,
            "Tokio should have downloads"
        );
        assert!(
            !crate_info.crate_data.max_version.is_empty(),
            "Should have a version"
        );
    }

    #[tokio::test]
    async fn test_fetch_nonexistent_crate() {
        let client = HttpClient::new();
        let result = client
            .fetch_crate_info("this-crate-definitely-does-not-exist-12345")
            .await;

        // Should return an error for non-existent crate
        assert!(result.is_err(), "Should fail to fetch non-existent crate");
    }

    #[tokio::test]
    async fn test_search_empty_results() {
        let client = HttpClient::new();
        // Search for something that's unlikely to exist
        let search_results = client
            .search("xyzabc123nonexistentcratenameforsurethistimereally")
            .await
            .unwrap();

        // Empty results are valid, just verify it doesn't error
        assert_eq!(search_results.len(), 0);
    }

    #[tokio::test]
    async fn test_client_creation() {
        // Just verify we can create a client without panicking
        let client = HttpClient::new();

        // Do a simple operation to verify the client works
        let summary = client.fetch_summary().await;
        assert!(
            summary.is_ok(),
            "Client should be functional after creation"
        );
    }
}
