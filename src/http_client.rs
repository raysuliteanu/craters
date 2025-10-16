use thiserror::Error;

#[derive(Debug, serde::Deserialize)]
pub struct Crates {
    crates: Vec<CrateInfo>,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize, Default)]
pub struct CrateInfo {
    pub id: String,
    pub name: String,
    pub versions: Option<Vec<String>>,
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub updated_at: String,
    pub created_at: String,
    pub downloads: u32,
    pub recent_downloads: u32,
    pub default_version: String,
    pub num_versions: u32,
    pub yanked: bool,
    pub max_version: String,
    pub newest_version: String,
    pub max_stable_version: String,
    pub description: String,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub repository: Option<String>,
    pub links: Links,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize, Default)]
pub struct Links {
    pub version_downloads: String,
    pub versions: String,
    pub owners: String,
    pub owner_team: String,
    pub owner_user: String,
    pub reverse_dependencies: String,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    ParseError(#[from] serde_json::Error),
}

pub struct HttpClient {
    client: reqwest::blocking::Client,
}

const URL: &str = "https://crates.io/api/v1/crates";

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl HttpClient {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .user_agent("craters/0.1.0 (https://github.com/raysuliteanu/craters)")
            .build()
            .expect("Failed to build HTTP client");

        HttpClient { client }
    }

    pub fn fetch_new_crates(&self) -> Result<Vec<CrateInfo>, ClientError> {
        let crate_info: Vec<CrateInfo> = self.sort("new")?;
        Ok(crate_info)
    }

    pub fn fetch_most_downloaded(&self) -> Result<Vec<CrateInfo>, ClientError> {
        let crate_info: Vec<CrateInfo> = self.sort("downloads")?;
        Ok(crate_info)
    }

    pub fn fetch_recent_updates(&self) -> Result<Vec<CrateInfo>, ClientError> {
        let crate_info: Vec<CrateInfo> = self.sort("recent-updates")?;
        Ok(crate_info)
    }

    fn sort(&self, param: &str) -> Result<Vec<CrateInfo>, ClientError> {
        let request = self.client.get(URL).query(&[("sort", param)]);
        let response = request.send().map_err(ClientError::RequestError)?;
        let text = response.text().map_err(ClientError::RequestError)?;
        let crates: Crates = serde_json::from_str(&text).map_err(ClientError::ParseError)?;
        Ok(crates.crates)
    }

    pub fn fetch_crate_info(&self, crate_name: &str) -> Result<CrateInfo, ClientError> {
        let url = format!("{}{}", URL, crate_name);
        let crate_info: CrateInfo = self.client.get(&url).send()?.json()?;
        Ok(crate_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_new_crates() {
        let client = HttpClient::new();
        let result = client.fetch_new_crates();

        match result {
            Ok(crates) => {
                println!("Successfully fetched {} crates", crates.len());
                assert!(!crates.is_empty(), "Should fetch at least one crate");

                // Print first crate for inspection
                if let Some(first) = crates.first() {
                    println!("First crate: {:#?}", first);
                }
            }
            Err(e) => {
                panic!("Failed to fetch new crates: {}", e);
            }
        }
    }

    #[test]
    fn test_deserialize_crates_response() {
        // Test with a sample JSON response structure
        let json = r#"{
            "crates": [
                {
                    "id": "test-crate",
                    "name": "test-crate",
                    "versions": null,
                    "keywords": null,
                    "categories": null,
                    "updated_at": "2024-01-01T00:00:00Z",
                    "created_at": "2024-01-01T00:00:00Z",
                    "downloads": 100,
                    "recent_downloads": 50,
                    "default_version": "0.1.0",
                    "num_versions": 1,
                    "yanked": false,
                    "max_version": "0.1.0",
                    "newest_version": "0.1.0",
                    "max_stable_version": "0.1.0",
                    "description": "A test crate",
                    "homepage": null,
                    "documentation": null,
                    "repository": null,
                    "links": {
                        "version_downloads": "/api/v1/crates/test-crate/downloads",
                        "versions": "/api/v1/crates/test-crate/versions",
                        "owners": "/api/v1/crates/test-crate/owners",
                        "owner_team": "/api/v1/crates/test-crate/owner_team",
                        "owner_user": "/api/v1/crates/test-crate/owner_user",
                        "reverse_dependencies": "/api/v1/crates/test-crate/reverse_dependencies"
                    }
                }
            ]
        }"#;

        let result: Result<Crates, _> = serde_json::from_str(json);
        match result {
            Ok(crates) => {
                assert_eq!(crates.crates.len(), 1);
                assert_eq!(crates.crates[0].name, "test-crate");
            }
            Err(e) => {
                panic!("Failed to deserialize: {}", e);
            }
        }
    }
}
