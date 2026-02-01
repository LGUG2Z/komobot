#![warn(clippy::all)]

use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct LicenseCheckResponse {
    #[serde(rename = "hasValidSubscription")]
    pub has_valid_subscription: bool,
    pub student: Option<bool>,
}

pub enum LicenseStatus {
    ValidCommercial { platform: String },
    StudentLicense,
    Invalid,
}

const KOMOREBI_WINDOWS_URL: &str = "https://kw-icul.lgug2z.com";
const KOMOREBI_MAC_URL: &str = "https://km-icul.lgug2z.com";

async fn check_license_endpoint(
    client: &Client,
    base_url: &str,
    email: &str,
) -> Result<LicenseCheckResponse, Box<dyn std::error::Error + Send + Sync>> {
    let response = client
        .get(base_url)
        .query(&[("email", email)])
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let license_data: LicenseCheckResponse = response.json().await?;
    Ok(license_data)
}

pub async fn validate_license(client: &Client, email: &str) -> LicenseStatus {
    // Try komorebi-windows first
    if let Ok(response) = check_license_endpoint(client, KOMOREBI_WINDOWS_URL, email).await {
        if response.has_valid_subscription && response.student.is_none() {
            return LicenseStatus::ValidCommercial {
                platform: "komorebi-windows".to_string(),
            };
        }
        if response.student.is_some() {
            return LicenseStatus::StudentLicense;
        }
    }

    // Try komorebi-mac second
    if let Ok(response) = check_license_endpoint(client, KOMOREBI_MAC_URL, email).await {
        if response.has_valid_subscription && response.student.is_none() {
            return LicenseStatus::ValidCommercial {
                platform: "komorebi-mac".to_string(),
            };
        }
        if response.student.is_some() {
            return LicenseStatus::StudentLicense;
        }
    }

    LicenseStatus::Invalid
}
