use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect(database_url)
        .await
}

#[cfg(test)]
mod tests {
    use super::create_pool;

    #[tokio::test]
    async fn rejects_an_invalid_database_url() {
        let result: Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> =
            create_pool("not a database URL").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn rejects_an_unreachable_database() {
        let result: Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> =
            create_pool("postgres://postgres:postgres@127.0.0.1:0/users").await;

        assert!(result.is_err());
    }
}
