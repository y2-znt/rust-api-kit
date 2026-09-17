use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

pub fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .acquire_timeout(Duration::from_secs(2))
        .connect_lazy(database_url)
}

#[cfg(test)]
mod tests {
    use super::create_pool;

    #[test]
    fn rejects_an_invalid_database_url() {
        let result: Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> =
            create_pool("not a database URL");

        assert!(result.is_err());
    }
}
