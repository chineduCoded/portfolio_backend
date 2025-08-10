use sqlx::PgPool;

#[derive(Clone)]
pub struct SqlxUserRepo {
    pub pool: PgPool,
}

#[derive(Clone)]
pub struct SqlxAboutMeRepo {
    pub pool: PgPool,
}