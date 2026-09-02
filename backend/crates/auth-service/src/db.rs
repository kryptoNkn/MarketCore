use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .connect(database_url)
        .await
}

pub async fn check_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await.map(|_| ())
}
