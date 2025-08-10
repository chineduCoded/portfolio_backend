use crate::repositories::sqlx_repo::{SqlxAboutMeRepo, SqlxUserRepo};


#[derive(Clone)]
pub struct SharedRepositories {
    pub user_repo: SqlxUserRepo,
    pub about_repo: SqlxAboutMeRepo,
}

impl SharedRepositories {
    pub fn new(pool: sqlx::PgPool) -> Self {
        let user_repo = SqlxUserRepo::new(pool.clone());
        let about_repo = SqlxAboutMeRepo::new(pool.clone());
        
        SharedRepositories {
            user_repo,
            about_repo,
        }
    }
}