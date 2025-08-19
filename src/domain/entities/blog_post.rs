use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};
use sqlx::types::Json;

use crate::utils::markdown::{safe_markdown_to_html, sanitize_markdown_content};

// ───── Constants ──────────────────────────────────────────────────────

const MIN_TITLE_LENGTH: u64 = 3;
const MAX_TITLE_LENGTH: u64 = 120;
const MIN_SLUG_LENGTH: u64 = 3;
const MAX_SLUG_LENGTH: u64 = 80;
const MIN_EXCERPT_LENGTH: u64 = 10;
const MAX_EXCERPT_LENGTH: u64 = 300;
const MAX_TAGS: usize = 10;
const MAX_TAG_LENGTH: usize = 30;

// ───── Database Models ───────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content_markdown: String,
    pub cover_image_url: Option<String>,
    pub tags: Option<Json<Vec<String>>>,       
    pub seo_title: Option<String>,       
    pub seo_description: Option<String>, 
    pub published: bool,                 
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Validate)]
pub struct BlogPostInsert {
    #[validate(
        length(min = MIN_TITLE_LENGTH, max = MAX_TITLE_LENGTH),
        custom(function = "validate_title")
    )]
    pub title: String,
    
    #[validate(
        length(min = MIN_SLUG_LENGTH, max = MAX_SLUG_LENGTH),
        custom(function = "validate_slug")
    )]
    pub slug: String,
    
    #[validate(
        length(min = MIN_EXCERPT_LENGTH, max = MAX_EXCERPT_LENGTH)
    )]
    pub excerpt: String,

    #[validate(length(min = 1, message = "Content cannot be empty"))]
    pub content_markdown: String,

    #[validate(url)]
    pub cover_image_url: Option<String>,

    #[validate(custom(function = "validate_tags"))]
    pub tags: Option<Json<Vec<String>>>,

    #[validate(length(max = MAX_TITLE_LENGTH))]
    pub seo_title: Option<String>,

    #[validate(length(max = MAX_EXCERPT_LENGTH))]
    pub seo_description: Option<String>,

    pub published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ───── API Response Models ──────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct BlogPostListResponse {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub cover_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BlogPostDetailResponse {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content_html: String,
    pub cover_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct BlogPostCreatedResponse {
    pub id: Uuid,
    pub slug: String,
    pub preview_url: String, // URL for previewing unpubished posts
    pub admin_url: String,   // URL for editing
}

// ───── Input & Validation ───────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewBlogPostRequest {
    #[validate(
        length(min = MIN_TITLE_LENGTH, max = MAX_TITLE_LENGTH),
        custom(function = "validate_title")
    )]
    pub title: String,

    #[validate(
        length(min = MIN_SLUG_LENGTH, max = MAX_SLUG_LENGTH),
        custom(function = "validate_slug")
    )]
    pub slug: String,

    #[validate(
        length(min = MIN_EXCERPT_LENGTH, max = MAX_EXCERPT_LENGTH)
    )]
    pub excerpt: String,

    #[validate(custom(function = "sanitize_markdown"))]
    pub content_markdown: String,

    #[validate(url)]
    pub cover_image_url: Option<String>,

    #[validate(custom(function = "validate_tags"))]
    pub tags: Option<Vec<String>>,

    #[validate(length(max = MAX_TITLE_LENGTH))]
    pub seo_title: Option<String>,

    #[validate(length(max = MAX_EXCERPT_LENGTH))]
    pub seo_description: Option<String>,

    pub published: bool,
    
    #[validate(custom(function = "validate_future_datetime"))]
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateBlogPostRequest {
    #[validate(
        length(min = MIN_TITLE_LENGTH, max = MAX_TITLE_LENGTH),
        custom(function = "validate_title")
    )]
    pub title: Option<String>,

    #[validate(
        length(min = "MIN_SLUG_LENGTH", max = "MAX_SLUG_LENGTH"),
        custom(function = "validate_slug")
    )]
    pub slug: Option<String>,

    #[validate(
        length(min = "MIN_EXCERPT_LENGTH", max = "MAX_EXCERPT_LENGTH")
    )]
    pub excerpt: Option<String>,

    #[validate(custom(function = "sanitize_markdown"))]
    pub content_markdown: Option<String>,

    #[validate(url)]
    pub cover_image_url: Option<Option<String>>, 

    #[validate(custom(function = "validate_tags"))]
    pub tags: Option<Option<Vec<String>>>,

    #[validate(length(max = MAX_TITLE_LENGTH))]
    pub seo_title: Option<Option<String>>,

    #[validate(length(max = MAX_EXCERPT_LENGTH))]
    pub seo_description: Option<Option<String>>,

    pub published: Option<bool>,
    
    #[validate(custom(function = "validate_future_datetime"))]
    pub published_at: Option<Option<DateTime<Utc>>>, 
}


// ───── Validation Helpers ───────────────────────────────────────────

fn validate_slug(slug: &str) -> Result<(), ValidationError> {
    let re = Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap();
    if !re.is_match(slug) {
        return Err(ValidationError::new(
            "Slug must be lowercase alphanumeric with hyphens"
        ));
    }
    Ok(())
}

fn validate_title(title: &str) -> Result<(), ValidationError> {
    if title.trim().len() != title.len() {
        return Err(ValidationError::new(
            "Title must not have leading/trailing whitespace"
        ));
    }
    Ok(())
}

fn validate_tags(tags: &[String]) -> Result<(), ValidationError> {
    if tags.len() > MAX_TAGS {
        return Err(ValidationError::new("Too many tags"));
    }

    for tag in tags {
        if tag.is_empty() || tag.len() > MAX_TAG_LENGTH {
            return Err(ValidationError::new("Invalid tag length"));
        }
        if !tag.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(ValidationError::new("Tags must be alphanumeric with hyphens"));
        }
    }

    Ok(())
}

fn validate_future_datetime(dt: &DateTime<Utc>) -> Result<(), ValidationError> {
    if *dt < Utc::now() {
        return Err(ValidationError::new(
            "Scheduled time must be in the future"
        ));
    }
    Ok(())
}

fn sanitize_markdown(content: &str) -> Result<(), ValidationError> {
    let sanitized = sanitize_markdown_content(content);

    if sanitized != content {
        return Err(ValidationError::new("Markdown contains unsafe HTML"));
    }

    Ok(())
}

// ───── Conversion Implementations ───────────────────────────────────

impl TryFrom<NewBlogPostRequest> for BlogPostInsert {
    type Error = ValidationErrors;

    fn try_from(value: NewBlogPostRequest) -> Result<Self, Self::Error> {
        value.validate()?;

        Ok(Self {
            title: value.title,
            slug: value.slug,
            excerpt: value.excerpt,
            content_markdown: value.content_markdown,
            cover_image_url: value.cover_image_url,
            tags: value.tags.map(Json),
            seo_title: value.seo_title,
            seo_description: value.seo_description,
            published: value.published,
            published_at: value.published_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

impl BlogPost {
    pub fn to_list_response(&self) -> BlogPostListResponse {
        BlogPostListResponse {
            id: self.id,
            title: self.title.clone(),
            slug: self.slug.clone(),
            excerpt: self.excerpt.clone(),
            cover_image_url: self.cover_image_url.clone(),
            tags: self.tags.as_ref().map(|t| t.0.clone()),
            published_at: self.published_at,
            updated_at: self.updated_at,
        }
    }

    pub fn to_detail_response(&self) -> BlogPostDetailResponse {
        BlogPostDetailResponse {
            id: self.id,
            title: self.title.clone(),
            slug: self.slug.clone(),
            excerpt: self.excerpt.clone(),
            content_html: safe_markdown_to_html(&self.content_markdown),
            cover_image_url: self.cover_image_url.clone(),
            tags: self.tags.as_ref().map(|t| t.0.clone()),
            seo_title: self.seo_title.clone(),
            seo_description: self.seo_description.clone(),
            published: self.published,
            published_at: self.published_at,
            updated_at: self.updated_at,
            created_at: self.created_at,
        }
    }
}

// ───── Helper Functions ─────────────────────────────────────────────