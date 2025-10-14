use sqlx::PgPool;

#[derive(Clone)]
pub struct SqlxUserRepo {
    pub pool: PgPool,
}

#[derive(Clone)]
pub struct SqlxAboutMeRepo {
    pub pool: PgPool,
}

#[derive(Clone)]
pub struct SqlxBlogPostRepo {
    pub pool: PgPool,
}

#[derive(Clone)]
pub struct SqlxContactMeRepo {
    pub pool: PgPool,
}