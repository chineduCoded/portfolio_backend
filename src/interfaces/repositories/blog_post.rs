use async_trait::async_trait;
use uuid::Uuid;
use sqlx::{self, PgPool, QueryBuilder};

use crate::{
    entities::{blog_post::{BlogPost, BlogPostInsert, UpdateBlogPostRequest}, option_fields::OptionField},
    errors::AppError,
    repositories::sqlx_repo::SqlxBlogPostRepo,
};

/// Helper to compute OFFSET safely from 1-based `page` and `per_page`.
fn page_offset(page: u32, per_page: u32) -> i64 {
    let page = page.saturating_sub(1);
    (page as i64) * (per_page as i64)
}

#[async_trait]
pub trait BlogPostRepository: Sync + Send {
    async fn create_blog_post(&self, post: &BlogPostInsert) -> Result<Uuid, AppError>;
    async fn get_blog_post_by_id(&self, id: &Uuid) -> Result<BlogPost, AppError>;
    async fn get_blog_post_by_slug(&self, slug: &str) -> Result<BlogPost, AppError>;
    async fn update_blog_post(&self, id: &Uuid, post: &UpdateBlogPostRequest) -> Result<BlogPost, AppError>;
    async fn get_all_blog_posts(&self, published_only: bool, page: u32, per_page: u32) -> Result<Vec<BlogPost>, AppError>;
    async fn publish_blog_post(&self, id: &Uuid) -> Result<BlogPost, AppError>;
    async fn count_blog_posts(&self, published_only: bool) -> Result<i64, AppError>;
    async fn get_recent_blog_posts(&self, limit: u32, published_only: bool) -> Result<Vec<BlogPost>, AppError>;
    async fn search_blog_posts(&self, query: &str) -> Result<Vec<BlogPost>, AppError>;
    async fn get_blog_posts_by_tag(&self, tag: &str) -> Result<Vec<BlogPost>, AppError>;
    async fn blog_post_exists_with_slug(&self, slug: &str, exclude_id: Option<Uuid>) -> Result<bool, AppError>;
    async fn soft_delete_blog_post(&self, id: &Uuid) -> Result<(), AppError>;
    async fn hard_delete_blog_post(&self, id: &Uuid) -> Result<(), AppError>;
}

impl SqlxBlogPostRepo {
    pub fn new(pool: PgPool) -> Self {
        SqlxBlogPostRepo { pool }
    }
}

#[async_trait]
impl BlogPostRepository for SqlxBlogPostRepo {
    async fn create_blog_post(&self, post: &BlogPostInsert) -> Result<Uuid, AppError> {
        let id: Uuid = sqlx::query_scalar!(
            r#"
            INSERT INTO blog_posts (
                title, slug, excerpt, content_markdown, cover_image_url, tags,
                seo_title, seo_description, published, published_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id
            "#,
            post.title,
            post.slug,
            post.excerpt,
            post.content_markdown,
            post.cover_image_url,
            post.tags as _,
            post.seo_title,
            post.seo_description,
            post.published,
            post.published_at,
            post.created_at,
            post.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("blog_posts_slug_key") {
                    return AppError::Conflict("Slug already exists".into());
                }
            }
            AppError::from(e)
        })?;

        Ok(id)
    }

    async fn get_blog_post_by_id(&self, id: &Uuid) -> Result<BlogPost, AppError> {
        let post = sqlx::query_as!(
            BlogPost,
            r#"
            SELECT * FROM blog_posts
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }

    async fn get_blog_post_by_slug(&self, slug: &str) -> Result<BlogPost, AppError> {
        let post = sqlx::query_as!(
            BlogPost,
            r#"
            SELECT * FROM blog_posts
            WHERE slug = $1 AND deleted_at IS NULL
            "#,
            slug
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(post)
    }

    async fn update_blog_post(&self, id: &Uuid, post: &UpdateBlogPostRequest) -> Result<BlogPost, AppError> {
        let current = self.get_blog_post_by_id(id).await?;

        let resolved_slug = resolve_slug_for_update(&post.slug, &post.title, &current.slug);

        // COALESCE used to preserve existing fields when Option::None is provided
        let updated_post = sqlx::query_as!(
            BlogPost,
            r#"
            UPDATE blog_posts SET
                title = COALESCE($1, title),
                slug = $2, -- Always set to resolved slug
                excerpt = COALESCE($3, excerpt),
                content_markdown = COALESCE($4, content_markdown),
                cover_image_url = COALESCE($5, cover_image_url),
                tags = COALESCE($6, tags),
                seo_title = COALESCE($7, seo_title),
                seo_description = COALESCE($8, seo_description),
                published = COALESCE($9, published),
                published_at = COALESCE($10, published_at),
                updated_at = NOW()
            WHERE id = $11 AND deleted_at IS NULL
            RETURNING *
            "#,
            post.title.flatten_str(),             
            resolved_slug,              
            post.excerpt.flatten_str(),           
            post.content_markdown.flatten_str(),  
            post.cover_image_url.flatten_str(),   
            post.tags.flatten_slice(),          
            post.seo_title.flatten_str(),         
            post.seo_description.flatten_str(),   
            post.published.flatten_bool(),
            post.published_at.flatten_datetime(),
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("blog_posts_slug_active_idx") {
                    return AppError::Conflict("Slug already exists".into());
                }
            }
            AppError::from(e)
        })?;

        Ok(updated_post)
    }

    async fn publish_blog_post(&self, id: &Uuid) -> Result<BlogPost, AppError> {
        let published_post = sqlx::query_as!(
            BlogPost,
            r#"
            UPDATE blog_posts SET
                published = TRUE,
                published_at = NOW(),
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(published_post)
    }

    async fn get_all_blog_posts(&self, published_only: bool, page: u32, per_page: u32) -> Result<Vec<BlogPost>, AppError> {
        let limit = per_page as i64;
        let offset = page_offset(page, per_page);

        let mut builder = QueryBuilder::new("SELECT * FROM blog_posts WHERE deleted_at IS NULL");

        if published_only {
            builder.push(" AND published = TRUE");
        }

        if published_only {
            builder.push(" ORDER BY published_at DESC NULLS LAST");
        } else {
            builder.push(" ORDER BY created_at DESC");
        }

        builder.push(" LIMIT ").push_bind(limit);
        builder.push(" OFFSET ").push_bind(offset);

        let query = builder.build_query_as::<BlogPost>();
        let posts: Vec<BlogPost> = query.fetch_all(&self.pool).await?;

        Ok(posts)
    }

    async fn count_blog_posts(&self, published_only: bool) -> Result<i64, AppError> {
        // Single query with the same filter predicate as listing
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM blog_posts
            WHERE deleted_at IS NULL
              AND ($1::boolean IS FALSE OR published = TRUE)
            "#
        )
        .bind(published_only)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn get_recent_blog_posts(
        &self, 
        limit: u32,
        published_only: bool
    ) -> Result<Vec<BlogPost>, AppError> {
        let mut builder = QueryBuilder::new(
            "SELECT * FROM blog_posts WHERE deleted_at IS NULL"
        );

        if published_only {
            builder.push(" AND published = TRUE");
        }

        builder.push(" ORDER BY published_at DESC NULLS LAST LIMIT ");
        builder.push_bind(limit as i64);

        let query = builder.build_query_as::<BlogPost>();
        let posts: Vec<BlogPost> = query.fetch_all(&self.pool).await?;

        Ok(posts)
    }

    async fn search_blog_posts(&self, query: &str) -> Result<Vec<BlogPost>, AppError> {
        let mut builder = QueryBuilder::new("SELECT * FROM blog_posts WHERE deleted_at IS NULL");


        builder.push(" AND (title ILIKE ").push_bind(format!("%{}%", query));
        builder.push(" OR content_markdown ILIKE ").push_bind(format!("%{}%", query));
        builder.push(") ORDER BY created_at DESC");


        let query = builder.build_query_as::<BlogPost>();
        let posts: Vec<BlogPost> = query.fetch_all(&self.pool).await?;


        Ok(posts)
    }

    async fn get_blog_posts_by_tag(&self, tag: &str) -> Result<Vec<BlogPost>, AppError> {
        let mut builder = QueryBuilder::new("SELECT * FROM blog_posts WHERE deleted_at IS NULL");


        builder.push(" AND tags @> ").push_bind(vec![tag]);
        builder.push(" ORDER BY created_at DESC");


        let query = builder.build_query_as::<BlogPost>();
        let posts: Vec<BlogPost> = query.fetch_all(&self.pool).await?;


        Ok(posts)
    }

    async fn blog_post_exists_with_slug(&self, slug: &str, exclude_id: Option<Uuid>) -> Result<bool, AppError> {
        // Use dynamic query + binds to avoid problematic casts with NULL
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM blog_posts
                WHERE slug = $1
                  AND deleted_at IS NULL
                  AND ($2 IS NULL OR id <> $2)
            )
            "#
        )
        .bind(slug)
        .bind(exclude_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn soft_delete_blog_post(&self, id: &Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE blog_posts
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Record not found".into()));
        }

        Ok(())
    }

    async fn hard_delete_blog_post(&self, id: &Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM blog_posts
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Record not found".into()));
        }

        Ok(())
    }
}

fn resolve_slug_for_update(
    slug_field: &OptionField<String>,
    title_field: &OptionField<String>,
    current_slug: &str,
) -> String {
    match slug_field {
        // User explicitly provided a new non-empty slug → use it
        OptionField::SetToValue(s) if !s.trim().is_empty() => s.clone(),

        // User explicitly gave empty string → regen from new title if provided
        OptionField::SetToValue(_) => {
            if let OptionField::SetToValue(new_title) = title_field {
                slug::slugify(new_title)
            } else {
                current_slug.to_string()
            }
        }

        // No slug provided → check if title changed, regenerate from new title
        OptionField::Unchanged | OptionField::SetToNull => {
            if let OptionField::SetToValue(new_title) = title_field {
                slug::slugify(new_title)
            } else {
                current_slug.to_string()
            }
        }
    }
}