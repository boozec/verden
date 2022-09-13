use crate::errors::AppError;

use sqlx::postgres::PgPool;

/// Static variable used to manage the database connection. Called with value = None raises a panic
/// error.
static mut CONNECTION: Option<PgPool> = None;

/// Setup database connection. Get variable `DATABASE_URL` from the environment. Sqlx crate already
/// defines an error for environments without DATABASE_URL.
pub async fn setup() -> Result<(), AppError> {
    let database_url = &crate::config::CONFIG.database_url;

    unsafe {
        CONNECTION = Some(PgPool::connect(database_url).await?);
    }

    Ok(())
}

/// Get connection. Raises an error if `setup()` has not been called yet.
/// Managing static `CONNECTION` is an unsafe operation.
pub unsafe fn get_client() -> &'static PgPool {
    match &CONNECTION {
        Some(client) => client,
        None => panic!("Connection not established!"),
    }
}
