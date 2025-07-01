use sqlx::PgPool;

#[derive(Clone)]
pub struct SqlxRepo {
    pub pool: PgPool,
}

impl SqlxRepo {
    pub fn new(pool: PgPool) -> Self {
        SqlxRepo { pool }
    }
}