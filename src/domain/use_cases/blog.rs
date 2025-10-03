use crate::{entities::blog_post::{BlogPost, BlogPostCreatedResponse, BlogPostInsert, NewBlogPostRequest, UpdateBlogPostRequest}, errors::AppError, repositories::blog_post::BlogPostRepository, utils::valid_uuid::valid_uuid};
use validator::Validate;


pub struct BlogPostHandler<R>
where
    R: BlogPostRepository,
{
    pub blog_post_repo: R,
}

impl<R> BlogPostHandler<R>
where
    R: BlogPostRepository,
{
    pub fn new(blog_post_repo: R) -> Self {
        BlogPostHandler { blog_post_repo }
    }

    /// Creates a new blog post with the provided data
    pub async fn create_blog_post(&self, post: NewBlogPostRequest) -> Result<BlogPostCreatedResponse, AppError> {
        let insert_post = BlogPostInsert::try_from(post)?;
        insert_post.validate()?;
        
        let id = self.blog_post_repo.create_blog_post(&insert_post).await?;
        
        let response = BlogPostCreatedResponse {
            id,
            slug: insert_post.slug.clone(),
            preview_url: format!("/blog/posts/{}", insert_post.slug.clone()),
            admin_url: format!("/admin/blog/posts/{}", insert_post.slug),
        };

        Ok(response)
    }

    /// Retrieves a blog post by its ID
    pub async fn get_blog_post_by_id(&self, post_id: &str) -> Result<BlogPost, AppError> {
       let valid_id = valid_uuid(post_id)?;
        self.blog_post_repo.get_blog_post_by_id(&valid_id).await
    }

    /// Retrieves all blog posts
    pub async fn get_all_blog_posts(&self, published_only: bool, page: u32, per_page: u32) -> Result<Vec<BlogPost>, AppError> {
        self.blog_post_repo.get_all_blog_posts(published_only, page, per_page).await
    }

    /// Retrieves recent blog posts limited by the specified number
    pub async fn get_recent_blog_posts(&self, limit: u32) -> Result<Vec<BlogPost>, AppError> {
        self.blog_post_repo.get_recent_blog_posts(limit).await
    }

    /// Updates an existing blog post
    pub async fn update_blog_post(
        &self,
        id: &str,
        post: &UpdateBlogPostRequest,
    ) -> Result<BlogPost, AppError> {
        post.validate()?;

        let valid_id = valid_uuid(id)?;

        self.blog_post_repo.update_blog_post(&valid_id, post).await
    }

    /// Publishes a blog post by its ID
    pub async fn publish_blog_post(
        &self, 
        id: &str
    ) -> Result<BlogPost, AppError> {
        let valid_id = valid_uuid(id)?;
        self.blog_post_repo.publish_blog_post(&valid_id).await
    }

    /// Deletes a blog post by its ID
    pub async fn delete_blog_post(
        &self, 
        id: &str,
        hard_delete: bool
    ) -> Result<(), AppError> {
        let valid_id = valid_uuid(id)?;
        
        match hard_delete {
            true => self.blog_post_repo.hard_delete_blog_post(&valid_id).await,
            false => self.blog_post_repo.soft_delete_blog_post(&valid_id).await
        }.map_err(|e| match e {
            AppError::NotFound(_) => AppError::NotFound("Blog post not found".to_string()),
            _ => e
        })
    }
} 