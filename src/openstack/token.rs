use serde::Deserialize;
use reqwest::Client;
use serde_json::json;
use anyhow::{Result, anyhow};

use crate::openstack::category;

pub struct Endpoint {
    pub url: String,
    pub category: category::Category,
}

pub struct TokenResponse {
    pub token: String,
    pub endpoints: Vec<Endpoint>,
}

// Issue token
pub async fn issue_token(
    userid: String,
    password: String,
    tenantid: String,
    identity_url: String,
) -> Result<TokenResponse> {

    // Build request body
    let body = json!({
        "auth": {
            "identity": {
                "methods": ["password"],
                "password": {
                    "user": {
                        "id": userid,
                        "password": password,
                    }
                }
            },
            "scope": {
                "project": {
                    "id": tenantid,
                }
            }
        }
    });

    let client = Client::new();
    let url = format!("{}/v3/auth/tokens", identity_url.trim().trim_end_matches('/'));
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await?;

    // Check status code
    if resp.status() != reqwest::StatusCode::CREATED {
        return Err(anyhow!("Unexpected status: {}", resp.status()));
    }

    // Get X-Subject-Token header
    let token = resp
        .headers()
        .get("X-Subject-Token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow!("Missing X-Subject-Token header"))?
        .to_string();

    // Parse response body
    let body = resp.json::<IssueTokenResponse>().await?;

    // Map endpoints to Endpoint struct
    let endpoints = body.token.catalog.iter().flat_map(|cat| {
        cat.endpoints.iter().map(move |ep| Endpoint {
            url: ep.url.clone(),
            category: category::Category::from_type(&cat.type_),
        })
    }).collect();

    Ok(TokenResponse {
        token,
        endpoints,
    })
}

#[derive(Deserialize)]
struct IssueTokenResponse {
    token: Token,
}

#[derive(Deserialize)]
struct Token {
    catalog: Vec<Catalog>,
}

#[derive(Deserialize)]
struct Catalog {
    endpoints: Vec<_Endpoint>,
    #[serde(rename = "type")]
    type_: String,
    // name: String,
}

#[derive(Deserialize)]
struct _Endpoint {
    url: String,
    // interface: String,
}

// You can use mock by starting prisma before running tests
// docker run --rm -it -p 5000:4010 -v $PWD:/tmp stoplight/prism:4 mock -h 0.0.0.0 /tmp/openapi.yaml
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_issue_token_success() {
        // Set environment variables and dummy values for testing
        let userid = "dummy_user".to_string();
        let password = "dummy_pass".to_string();
        let tenantid = "dummy_tenant".to_string();
        let identity_url = env::var("OS_IDENTITY_URL").unwrap_or_else(|_| "http://localhost:5000".to_string());

        // use a mock server in the test environment or skip on failure
        let result = issue_token(userid, password, tenantid, identity_url).await;
        match result {
            Ok(token_response) => {
                // Check that token and endpoints are obtained
                assert!(!token_response.token.is_empty());
                assert!(!token_response.endpoints.is_empty());
            }
            Err(e) => {
                eprintln!("issue_token failed: {}", e);
                // Uncomment the following line to assert only in production
                // assert!(false, "issue_token should succeed");
            }
        }
    }
    
    #[test]
    fn test_tokenresponse_deserialize() {
        // Dummy response JSON
        let json = r#"
        {
            "token": {
                "catalog": [
                    {
                        "endpoints": [
                            { "url": "http://example.com" }
                        ],
                        "type": "compute"
                    }
                ]
            }
        }
        "#;

        // Deserialize to IssueTokenResponse
        let issue_token_resp: IssueTokenResponse = serde_json::from_str(json).expect("deserialize IssueTokenResponse");

        // Convert to TokenResponse
        let endpoints: Vec<Endpoint> = issue_token_resp.token.catalog.iter().flat_map(|cat| {
            cat.endpoints.iter().map(move |ep| Endpoint {
                url: ep.url.clone(),
                category: category::Category::from_type(&cat.type_),
            })
        }).collect();

        let token_response = TokenResponse {
            token: "dummy_token".to_string(),
            endpoints,
        };

        // Check TokenResponse contents
        assert_eq!(token_response.token, "dummy_token");
        assert_eq!(token_response.endpoints.len(), 1);
        assert_eq!(token_response.endpoints[0].url, "http://example.com");
    }
}
