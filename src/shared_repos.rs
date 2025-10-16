use crate::{repositories::sqlx_repo::{SqlxAboutMeRepo, SqlxBlogPostRepo, SqlxContactMeRepo, SqlxUserRepo}};


#[derive(Clone)]
pub struct SharedRepositories {
    pub user_repo: SqlxUserRepo,
    pub about_repo: SqlxAboutMeRepo,
    pub blog_post_repo: SqlxBlogPostRepo,
    pub contact_repo: SqlxContactMeRepo,
}

impl SharedRepositories {
    pub fn new(pool: sqlx::PgPool) -> Self {
        let user_repo = SqlxUserRepo::new(pool.clone());
        let about_repo = SqlxAboutMeRepo::new(pool.clone());
        let blog_post_repo = SqlxBlogPostRepo::new(pool.clone());
        let contact_repo = SqlxContactMeRepo::new(pool.clone());
        
        SharedRepositories {
            user_repo,
            about_repo,
            blog_post_repo,
            contact_repo,
        }
    }
}