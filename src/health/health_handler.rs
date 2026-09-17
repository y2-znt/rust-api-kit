use std::{future::Future, time::Duration};

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;

use crate::{
    api::{ApiResponse, ok},
    app::AppState,
};

const DATABASE_CHECK_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, thiserror::Error)]
enum DatabaseHealthError {
    #[error("database health check timed out after {timeout:?}")]
    Timeout { timeout: Duration },

    #[error("database health check query failed")]
    Query(#[source] sqlx::Error),
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn health(
    State(state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<HealthResponse>>) {
    let database_check: Result<(), DatabaseHealthError> = check_database(
        sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&state.db),
        DATABASE_CHECK_TIMEOUT,
    )
    .await;

    match database_check {
        Ok(()) => (
            StatusCode::OK,
            ok(HealthResponse {
                status: "Health check ok",
            }),
        ),
        Err(error) => {
            eprintln!("database health check failed: {error:?}");

            (
                StatusCode::SERVICE_UNAVAILABLE,
                ok(HealthResponse {
                    status: "Health check degraded",
                }),
            )
        }
    }
}

async fn check_database<F>(query: F, timeout: Duration) -> Result<(), DatabaseHealthError>
where
    F: Future<Output = Result<i32, sqlx::Error>>,
{
    match tokio::time::timeout(timeout, query).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(DatabaseHealthError::Query(error)),
        Err(_) => Err(DatabaseHealthError::Timeout { timeout }),
    }
}

#[cfg(test)]
mod tests {
    use std::{future, time::Duration};

    use super::{DatabaseHealthError, check_database};

    #[tokio::test]
    async fn reports_a_database_timeout() {
        let result: Result<(), DatabaseHealthError> = check_database(
            future::pending::<Result<i32, sqlx::Error>>(),
            Duration::from_millis(1),
        )
        .await;

        assert!(matches!(result, Err(DatabaseHealthError::Timeout { .. })));
    }

    #[tokio::test]
    async fn preserves_a_database_query_error() {
        let result: Result<(), DatabaseHealthError> = check_database(
            future::ready(Err(sqlx::Error::PoolTimedOut)),
            Duration::from_secs(1),
        )
        .await;

        assert!(matches!(
            result,
            Err(DatabaseHealthError::Query(sqlx::Error::PoolTimedOut))
        ));
    }

    #[tokio::test]
    async fn accepts_a_successful_database_check() {
        let result: Result<(), DatabaseHealthError> =
            check_database(future::ready(Ok(1)), Duration::from_secs(1)).await;

        assert!(result.is_ok());
    }
}
