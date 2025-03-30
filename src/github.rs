use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;

use anyhow::anyhow;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;

// GraphQL query to fetch sponsorship data
const SPONSORS_QUERY: &str = r#"
query($cursor: String) {
  viewer {
    sponsorshipsAsMaintainer(first: 100, includePrivate: true, activeOnly: false, after: $cursor) {
      pageInfo {
        hasNextPage
        endCursor
      }
      nodes {
        createdAt
        tier {
          monthlyPriceInDollars
          isOneTime
        }
        sponsorEntity {
          ... on User {
            login
          }
          ... on Organization {
            login
          }
        }
        isActive
      }
    }
  }
}
"#;

// Response structures for GraphQL queries
#[derive(Debug, Clone, Deserialize)]
pub struct GraphQLResponse {
    data: Data,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Data {
    viewer: Viewer,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Viewer {
    #[serde(rename = "sponsorshipsAsMaintainer")]
    sponsorships_as_maintainer: SponsorshipConnection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SponsorshipConnection {
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
    nodes: Vec<Sponsorship>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sponsorship {
    tier: Tier,
    #[serde(rename = "sponsorEntity")]
    sponsor_entity: SponsorEntity,
    #[serde(rename = "isActive")]
    is_active: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tier {
    #[serde(rename = "monthlyPriceInDollars")]
    monthly_price_in_dollars: f32,
    #[serde(rename = "isOneTime")]
    is_one_time: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SponsorEntity {
    login: String,
}

// GraphQL request structure
#[derive(Debug, Clone, Serialize)]
pub struct GraphQLRequest {
    query: String,
    variables: HashMap<String, Option<String>>,
}

pub enum SponsorLevel {
    OneDollar,
    FiveDollar,
    TenDollar,
    TwentyDollar,
    OneTime,
    Alumni,
    None,
}

impl Display for SponsorLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SponsorLevel::OneDollar => write!(f, "OneDollar"),
            SponsorLevel::FiveDollar => write!(f, "FiveDollar"),
            SponsorLevel::TenDollar => write!(f, "TenDollar"),
            SponsorLevel::TwentyDollar => write!(f, "TwentyDollar"),
            SponsorLevel::OneTime => write!(f, "OneTime"),
            SponsorLevel::Alumni => write!(f, "Alumni"),
            SponsorLevel::None => write!(f, "None"),
        }
    }
}

// Sponsor categories
#[derive(Default, Debug, Clone)]
pub struct SponsorLists {
    one_dollar_monthly: Vec<SponsorInfo>,
    five_dollar_monthly: Vec<SponsorInfo>,
    ten_dollar_monthly: Vec<SponsorInfo>,
    twenty_dollar_monthly: Vec<SponsorInfo>,
    current_one_time: Vec<SponsorInfo>,
    previous_sponsors: Vec<SponsorInfo>,
}

impl SponsorLists {
    pub fn total_active_count(&self) -> usize {
        self.one_dollar_monthly.len()
            + self.five_dollar_monthly.len()
            + self.ten_dollar_monthly.len()
            + self.twenty_dollar_monthly.len()
            + self.current_one_time.len()
    }

    pub fn level_for_user(&self, user: &str) -> SponsorLevel {
        for l in &self.one_dollar_monthly {
            if l.login == user {
                return SponsorLevel::OneDollar;
            }
        }

        for l in &self.five_dollar_monthly {
            if l.login == user {
                return SponsorLevel::FiveDollar;
            }
        }

        for l in &self.ten_dollar_monthly {
            if l.login == user {
                return SponsorLevel::TenDollar;
            }
        }

        for l in &self.twenty_dollar_monthly {
            if l.login == user {
                return SponsorLevel::TwentyDollar;
            }
        }

        for l in &self.previous_sponsors {
            if l.login == user {
                return SponsorLevel::Alumni;
            }
        }

        for l in &self.current_one_time {
            if l.login == user {
                return SponsorLevel::OneTime;
            }
        }

        SponsorLevel::None
    }
}

#[derive(Debug, Clone)]
pub struct SponsorInfo {
    login: String,
    amount: f32,
}

pub async fn fetch_all_sponsors(client: &Client) -> anyhow::Result<Vec<Sponsorship>> {
    let mut all_sponsors = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        let mut variables = HashMap::new();
        variables.insert("cursor".to_string(), cursor);

        let request = GraphQLRequest {
            query: SPONSORS_QUERY.to_string(),
            variables,
        };

        let response = client
            .post("https://api.github.com/graphql")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(anyhow!(
                "GitHub API error: Status {}, Body: {}",
                status,
                body
            ));
        }

        let response_data: GraphQLResponse = response.json().await?;
        let connection = &response_data.data.viewer.sponsorships_as_maintainer;

        // Add sponsors from current page to result
        all_sponsors.extend(connection.nodes.clone());

        // Check if there are more pages
        if !connection.page_info.has_next_page {
            break;
        }

        // Update cursor for next page
        cursor = connection.page_info.end_cursor.clone();
    }

    Ok(all_sponsors)
}

pub fn categorize_sponsors(sponsorships: Vec<Sponsorship>) -> SponsorLists {
    let mut lists = SponsorLists {
        one_dollar_monthly: Vec::new(),
        five_dollar_monthly: Vec::new(),
        ten_dollar_monthly: Vec::new(),
        twenty_dollar_monthly: Vec::new(),
        current_one_time: Vec::new(),
        previous_sponsors: Vec::new(),
    };

    for sponsorship in sponsorships {
        let sponsor_info = SponsorInfo {
            login: sponsorship.sponsor_entity.login.clone(),
            amount: sponsorship.tier.monthly_price_in_dollars,
        };

        if sponsorship.is_active {
            if sponsorship.tier.is_one_time {
                lists.current_one_time.push(sponsor_info);
            } else {
                // Categorize by monthly amount
                match sponsorship.tier.monthly_price_in_dollars as i32 {
                    1 => lists.one_dollar_monthly.push(sponsor_info),
                    5 => lists.five_dollar_monthly.push(sponsor_info),
                    10 => lists.ten_dollar_monthly.push(sponsor_info),
                    20 => lists.twenty_dollar_monthly.push(sponsor_info),
                    _ => {
                        // Handle other monthly amounts if needed
                        println!(
                            "Note: Sponsor {} has unconventional monthly amount: ${}",
                            sponsor_info.login, sponsor_info.amount
                        );
                    }
                }
            }
        } else {
            // Previous sponsors (no longer active)
            lists.previous_sponsors.push(sponsor_info);
        }
    }

    lists
}
