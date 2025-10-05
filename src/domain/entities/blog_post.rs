use std::borrow::Cow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};
use sqlx::types::Json;

use crate::{
    entities::option_fields::OptionField,
    utils::markdown::{safe_markdown_to_html, sanitize_markdown_content},
};

// ───── Constants ──────────────────────────────────────────────────────
const MIN_TITLE_LENGTH: u64 = 3;
const MAX_TITLE_LENGTH: u64 = 120;
const MIN_SLUG_LENGTH: u64 = 3;
const MAX_SLUG_LENGTH: u64 = 80;
const MIN_EXCERPT_LENGTH: u64 = 10;
const MAX_EXCERPT_LENGTH: u64 = 300;
const MAX_TAGS: u64 = 10;
const MAX_TAG_LENGTH: u64 = 30;


// ───── Database Models ───────────────────────────────────────────────


#[derive(Debug, sqlx::FromRow)]
pub struct BlogPostRow {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content_markdown: String,
    pub cover_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub content_markdown: String,
    pub cover_image_url: Option<String>,
    pub tags: Option<Vec<String>>,
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

    #[validate(length(min = MIN_EXCERPT_LENGTH, max = MAX_EXCERPT_LENGTH))]
    pub excerpt: String,

    #[validate(length(min = 1, message = "Content cannot be empty"))]
    pub content_markdown: String,

    #[validate(custom(function = "validate_optional_url"))]
    pub cover_image_url: Option<String>,

    #[validate(custom(function = "validate_tags"))]
    pub tags: Option<Vec<String>>,

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
    pub preview_url: String,
    pub admin_url: String,
}

// ───── Input & Validation Requests ──────────────────────────────────

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
    pub slug: Option<String>,

    #[validate(length(min = MIN_EXCERPT_LENGTH, max = MAX_EXCERPT_LENGTH))]
    pub excerpt: String,

    pub content_markdown: String,

    #[validate(custom(function = "validate_optional_url"))]
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

#[derive(Debug, Deserialize, Validate, Default)]
#[serde(default)]
pub struct UpdateBlogPostRequest {    
    #[validate(
        length(min = MIN_TITLE_LENGTH, max = MAX_TITLE_LENGTH),
        custom(function = "validate_optional_title")
    )]
    pub title: OptionField<String>,

    #[validate(
        length(min = MIN_SLUG_LENGTH, max = MAX_SLUG_LENGTH),
        custom(function = "validate_optional_slug")
    )]
    pub slug: OptionField<String>,

    #[validate(length(min = MIN_EXCERPT_LENGTH, max = MAX_EXCERPT_LENGTH))]
    pub excerpt: OptionField<String>,

    pub content_markdown: OptionField<String>,

    #[validate(custom(function = "validate_optional_url_field"))]
    pub cover_image_url: OptionField<String>,

    #[validate(custom(function = "validate_optional_tags"))]
    pub tags: OptionField<Vec<String>>,

    #[validate(length(max = MAX_TITLE_LENGTH))]
    pub seo_title: OptionField<String>,

    #[validate(length(max = MAX_EXCERPT_LENGTH))]
    pub seo_description: OptionField<String>,

    pub published: OptionField<bool>,

    #[validate(custom(function = "validate_optional_future_datetime"))]
    pub published_at: OptionField<DateTime<Utc>>,
}

// ───── Validation Helpers ───────────────────────────────────────────
pub fn validate_optional_url(url: &str) -> Result<(), ValidationError> {
    validate_url(url)
}

pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    match url::Url::parse(url) {
        Ok(parsed) => {
            if parsed.scheme() == "http" || parsed.scheme() == "https" {
                Ok(())
            } else {
                Err(new_validation_error("invalid_url_scheme", "URL must start with http:// or https://"))
            }
        }
        Err(_) => Err(new_validation_error("invalid_url", "Invalid URL format")),
    }
}


pub fn validate_future_datetime(dt: &DateTime<Utc>) -> Result<(), ValidationError> {
    if *dt < Utc::now() {
        return Err(new_validation_error("datetime_past", "Scheduled time must be in the future"));
    }
    Ok(())
}

pub fn validate_slug(slug: &str) -> Result<(), ValidationError> {
    if slug.is_empty() {
        return Err(new_validation_error("slug_empty", "Slug cannot be empty"));
    }
    if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(new_validation_error("slug_invalid_chars", "Slug must contain only lowercase letters, digits, or hyphens"));
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(new_validation_error("slug_edge_hyphen", "Slug must not start or end with a hyphen"));
    }
    if slug.contains("--") {
        return Err(new_validation_error("slug_double_hyphen", "Slug must not contain consecutive hyphens"));
    }
    Ok(())
}

pub fn validate_optional_title(value: &OptionField<String>) -> Result<(), ValidationError> {
    if let OptionField::SetToValue(title) = value {
        validate_title(title)?;
    }
    Ok(())
}

pub fn validate_optional_slug(value: &OptionField<String>) -> Result<(), ValidationError> {
    if let OptionField::SetToValue(slug) = value {
        validate_slug(slug)?;
    }
    Ok(())
}

pub fn validate_optional_url_field(value: &OptionField<String>) -> Result<(), ValidationError> {
    if let OptionField::SetToValue(url) = value {
        validate_url(url)?;
    }
    Ok(())
}

pub fn validate_optional_tags(value: &OptionField<Vec<String>>) -> Result<(), ValidationError> {
    if let OptionField::SetToValue(tags) = value {
        validate_tags(tags)?;
    }
    Ok(())
}

pub fn validate_optional_future_datetime(value: &OptionField<DateTime<Utc>>) -> Result<(), ValidationError> {
    if let OptionField::SetToValue(dt) = value {
        validate_future_datetime(dt)?;
    }
    Ok(())
}

pub fn validate_tags_json(tags: &Json<Vec<String>>) -> Result<(), ValidationError> {
    validate_tags(&tags.0)
}

pub fn validate_tags(tags: &[String]) -> Result<(), ValidationError> {
    if tags.len() > MAX_TAGS as usize {
        return Err(new_validation_error("too_many_tags", "Too many tags provided"));
    }
    for tag in tags {
        if tag.is_empty() || tag.len() > MAX_TAG_LENGTH as usize {
            return Err(new_validation_error("invalid_tag_length", "Tag length must be within allowed range"));
        }
        if !tag.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(new_validation_error("invalid_tag_chars", "Tags must be alphanumeric or hyphens"));
        }
    }
    Ok(())
}

pub fn validate_title(title: &str) -> Result<(), ValidationError> {
    if title.trim().len() != title.len() {
        return Err(new_validation_error("title_whitespace", "Title must not have leading or trailing whitespace"));
    }
    Ok(())
}

fn new_validation_error(code: &'static str, msg: &'static str) -> ValidationError {
    let mut err = ValidationError::new(code);
    err.message = Some(Cow::Borrowed(msg));
    err
}

// ───── Conversions ──────────────────────────────────────────────────
impl From<BlogPostRow> for BlogPost {
    fn from(row: BlogPostRow) -> Self {
        BlogPost {
            id: row.id,
            title: row.title,
            slug: row.slug,
            excerpt: row.excerpt,
            content_markdown: row.content_markdown,
            cover_image_url: row.cover_image_url,
            tags: row.tags,
            seo_title: row.seo_title,
            seo_description: row.seo_description,
            published: row.published,
            published_at: row.published_at,
            updated_at: row.updated_at,
            created_at: row.created_at,
            deleted_at: row.deleted_at,
        }
    }
}

impl TryFrom<NewBlogPostRequest> for BlogPostInsert {
    type Error = ValidationErrors;

    fn try_from(value: NewBlogPostRequest) -> Result<Self, Self::Error> {
        value.validate()?;
        let sanitized_content = sanitize_markdown_content(&value.content_markdown);

        // Generate slug if not provided
        let slug = match value.slug {
            Some(s) => s,
            None => {
                let generated = slug::slugify(&value.title);
                if generated.len() < MIN_SLUG_LENGTH as usize {
                    return Err({
                        let mut errors = ValidationErrors::new();
                        errors.add("slug", new_validation_error("slug_too_short", "Generated slug is too short; please provide a custom slug"));
                        errors
                    });
                }
                generated
            }
        };

        let insert = BlogPostInsert {
            title: value.title,
            slug,
            excerpt: value.excerpt,
            content_markdown: sanitized_content,
            cover_image_url: value.cover_image_url,
            tags: value.tags,
            seo_title: value.seo_title,
            seo_description: value.seo_description,
            published: value.published,
            published_at: value.published_at,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        insert.validate()?;
        Ok(insert)
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
            tags: self.tags.clone(),
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
            tags: self.tags.clone(),
            seo_title: self.seo_title.clone(),
            seo_description: self.seo_description.clone(),
            published: self.published,
            published_at: self.published_at,
            updated_at: self.updated_at,
            created_at: self.created_at,
        }
    }
}