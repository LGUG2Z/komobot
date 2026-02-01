#![warn(clippy::all)]
mod github;
mod license;

use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use env_logger::Env;
use parking_lot::RwLock;
use poise::CreateReply;
use poise::serenity_prelude as serenity;
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::header::USER_AGENT;

use crate::github::SponsorLevel;
use crate::github::SponsorLists;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

lazy_static::lazy_static! {
    static ref DISCORD_TOKEN: String = {
        std::env::var("DISCORD_TOKEN").unwrap()
    };
    static ref GITHUB_TOKEN: String = {
        std::env::var("GITHUB_TOKEN").unwrap()
    };
    static ref GITHUB_CLIENT: Client = {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", *GITHUB_TOKEN)).unwrap(),
        );

        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("komobot"),
        );

        Client::builder()
            .default_headers(headers)
            .build()
            .unwrap()
    };
    static ref LICENSE_CLIENT: Client = {
        Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap()
    };
    static ref GITHUB_SPONSOR_DATA: Arc<RwLock<SponsorLists>> = {
        Arc::new(RwLock::new(SponsorLists::default()))
    };
    static ref LAST_UPDATED: Arc<RwLock<Instant>> = Arc::new(RwLock::new(Instant::now()));
}

const DATA_REFRESH_INTERVAL_MINS: u64 = 5;

const GITHUB_SPONSORS_TIER_S_ROLE_ID: &str = "1355695106195460397";
const GITHUB_SPONSORS_TIER_1_ROLE_ID: &str = "1355669933446664284";
const GITHUB_SPONSORS_TIER_2_ROLE_ID: &str = "1355670153722855474";
const GITHUB_SPONSORS_TIER_3_ROLE_ID: &str = "1355670200154067125";
const GITHUB_SPONSORS_ALUMNI_ROLE_ID: &str = "1355669718983250103";
const LICENSE_HOLDER_ROLE_ID: &str = "1327709840914780172";
const AUDIT_LOG_CHANNEL_ID: &str = "1355672655042183198";

/// Request a GitHub sponsor role on the server
#[poise::command(slash_command)]
async fn github_sponsor(
    ctx: Context<'_>,
    #[description = "Your GitHub username"] username: String,
) -> Result<(), Error> {
    log::info!("/github-sponsor: called by {}", ctx.author().name);

    let now = Instant::now();
    if now.duration_since(*LAST_UPDATED.read())
        > Duration::from_secs(DATA_REFRESH_INTERVAL_MINS * 60)
    {
        log::info!("/github-sponsor: using fresh data");
        let sponsors = github::fetch_all_sponsors(&GITHUB_CLIENT).await?;
        let sponsor_lists = github::categorize_sponsors(sponsors);
        *GITHUB_SPONSOR_DATA.write() = sponsor_lists;
    } else {
        log::info!("/github-sponsor: using cached data");
    }

    let user = ctx.author();
    let member = ctx
        .author_member()
        .await
        .ok_or("user is not a member of this server")?;

    let channel = (*AUDIT_LOG_CHANNEL_ID).parse::<serenity::ChannelId>()?;

    let (_total_active_count, level) = {
        let data = &*GITHUB_SPONSOR_DATA.read();
        (data.total_active_count(), data.level_for_user(&username))
    };

    let role_id = match level {
        SponsorLevel::OneDollar | SponsorLevel::OneTime => GITHUB_SPONSORS_TIER_3_ROLE_ID,
        SponsorLevel::FiveDollar => GITHUB_SPONSORS_TIER_2_ROLE_ID,
        SponsorLevel::TenDollar => GITHUB_SPONSORS_TIER_1_ROLE_ID,
        SponsorLevel::TwentyDollar => GITHUB_SPONSORS_TIER_S_ROLE_ID,
        SponsorLevel::Alumni => GITHUB_SPONSORS_ALUMNI_ROLE_ID,
        SponsorLevel::None => {
            ctx.send(
                CreateReply::default()
                    .content("You were not recognized as a current or previous Github Sponsor")
                    .ephemeral(true),
            )
            .await?;

            channel.say(ctx.http(), format!("github_sponsor: Discord user {} used {username}, but was not recognized as a current or previous GitHub sponsor", user.name)).await?;
            return Ok(());
        }
    };

    let role_name = match role_id {
        GITHUB_SPONSORS_TIER_S_ROLE_ID => "GitHub Sponsors: Tier S",
        GITHUB_SPONSORS_TIER_1_ROLE_ID => "GitHub Sponsors: Tier 1",
        GITHUB_SPONSORS_TIER_2_ROLE_ID => "GitHub Sponsors: Tier 2",
        GITHUB_SPONSORS_TIER_3_ROLE_ID => "GitHub Sponsors: Tier 3",
        GITHUB_SPONSORS_ALUMNI_ROLE_ID => "GitHub Sponsors: Alumni",
        _ => {
            channel
                .say(
                    ctx.http(),
                    "github_sponsor: There was an error mapping a role_id to a role_name",
                )
                .await?;
            return Ok(());
        }
    };

    log::info!(
        "/github-sponsor: assigning role name {role_name} to {}",
        ctx.author().name
    );

    let role_id = role_id.parse::<serenity::RoleId>()?;

    member.add_role(ctx.http(), role_id).await?;
    channel.say(ctx.http(), format!("github_sponsor: Discord user {} used {username} to self-assign the '{role_name}' role", user.name)).await?;
    ctx.send(
        CreateReply::default()
            .content(format!("You have self-assigned the '{role_name}' role"))
            .ephemeral(true),
    )
    .await?;
    ctx.send(
        CreateReply::default()
            .content("Use of this command is regularly audited")
            .ephemeral(true),
    )
    .await?;

    log::info!(
        "/github-sponsor: command for {} completed successfully",
        ctx.author().name
    );

    Ok(())
}

/// Request a license holder role on the server
#[poise::command(slash_command)]
async fn license_holder(
    ctx: Context<'_>,
    #[description = "Your email address from your Stripe purchase"] email: String,
) -> Result<(), Error> {
    log::info!("/license-holder: called by {}", ctx.author().name);

    let user = ctx.author();
    let member = ctx
        .author_member()
        .await
        .ok_or("user is not a member of this server")?;

    let channel = (*AUDIT_LOG_CHANNEL_ID).parse::<serenity::ChannelId>()?;

    let status = license::validate_license(&LICENSE_CLIENT, &email).await;

    match status {
        license::LicenseStatus::ValidCommercial { platform } => {
            log::info!(
                "/license-holder: assigning License Holder role to {}",
                ctx.author().name
            );

            let role_id = LICENSE_HOLDER_ROLE_ID.parse::<serenity::RoleId>()?;
            member.add_role(ctx.http(), role_id).await?;

            channel
                .say(
                    ctx.http(),
                    format!(
                        "license_holder: Discord user {} used email {} to self-assign the 'License Holder' role [validated via {}]",
                        user.name, email, platform
                    ),
                )
                .await?;

            ctx.send(
                CreateReply::default()
                    .content("You have self-assigned the 'License Holder' role")
                    .ephemeral(true),
            )
            .await?;

            ctx.send(
                CreateReply::default()
                    .content("Use of this command is regularly audited")
                    .ephemeral(true),
            )
            .await?;

            log::info!(
                "/license-holder: command for {} completed successfully",
                ctx.author().name
            );
        }
        license::LicenseStatus::StudentLicense => {
            ctx.send(
                CreateReply::default()
                    .content("Your email is associated with a student license. This command is only for commercial license holders.")
                    .ephemeral(true),
            )
            .await?;

            channel
                .say(
                    ctx.http(),
                    format!(
                        "license_holder: Discord user {} used email {}, but this is a student license (not eligible for commercial role)",
                        user.name, email
                    ),
                )
                .await?;
        }
        license::LicenseStatus::Invalid => {
            ctx.send(
                CreateReply::default()
                    .content("You were not recognized as a current or previous komorebi license holder. Please ensure you used the email address associated with your Stripe purchase.")
                    .ephemeral(true),
            )
            .await?;

            channel
                .say(
                    ctx.http(),
                    format!(
                        "license_holder: Discord user {} used email {}, but was not recognized as a current or previous license holder",
                        user.name, email
                    ),
                )
                .await?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("komobot=info")).init();

    log::info!("refreshing data");
    let sponsors = github::fetch_all_sponsors(&GITHUB_CLIENT).await?;
    let sponsor_lists = github::categorize_sponsors(sponsors);
    *GITHUB_SPONSOR_DATA.write() = sponsor_lists;
    log::info!("refreshed data");

    let intents = serenity::GatewayIntents::all();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![github_sponsor(), license_holder()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    log::info!("constructed framework");

    let client = serenity::ClientBuilder::new(&*DISCORD_TOKEN, intents)
        .framework(framework)
        .await;

    log::info!("starting bot");

    Ok(client?.start().await?)
}
