mod api;
mod app;
mod auth;
mod banner;
mod config;
mod db;
mod health;
mod users;

use anyhow::{Context, Result};
use config::Config;
use db::create_pool;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let config: Config = Config::from_env().context("loading application configuration")?;
    let pool: sqlx::Pool<sqlx::Postgres> = create_pool(&config.database_url)
        .await
        .context("connecting to PostgreSQL")?;

    let app = app::create_app(pool);

    let address = format!("{}:{}", config.host, config.port);

    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .with_context(|| format!("binding TCP listener to {address}"))?;

    banner::print(&config.host, config.port);

    axum::serve(listener, app)
        .await
        .context("serving HTTP requests")?;

    Ok(())
}
