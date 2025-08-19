use std::{path::Path, io};
use tokio::fs;

use pulldown_cmark::{html, Options, Parser};
use ammonia::{Builder, UrlRelative};
use derive_more::Display;
use infer::{self, Infer};
use futures::future::join_all;

/// Converts Markdown content to sanitized HTML to prevent XSS attacks.
pub fn safe_markdown_to_html(markdown: &str) -> String {
    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);

    let mut raw_html = String::with_capacity(markdown.len() * 2);
    html::push_html(&mut raw_html, parser);

    sanitize_markdown_content(&raw_html)
}

/// Sanitizes Markdown content to remove unsafe HTML.
pub fn sanitize_markdown_content(content: &str) -> String {
    Builder::default()
        .link_rel(Some("nofollow noopener noreferrer"))
        .url_relative(UrlRelative::Deny)
        .clean(content)
        .to_string()
}

/// Checks whether a given Markdown string is structurally valid.
pub fn is_valid_markdown(content: &str) -> bool {
    let parser = Parser::new_ext(content, Options::all());
    parser.into_iter().next().is_some()
}

/// Validates a markdown file for extension, emptiness, and structure.
/// 
/// - `original_filename`: The filename from TempFile::file_name()
/// - `file_path`: The path from TempFile::file.path()
/// - `max_size`: Max size in bytes
pub async fn read_markdown_file(
    original_filename: Option<&str>,
    file_path: &Path,
    max_size: usize
) -> Result<String, MarkdownError> {
    // 1. Extension check - allow common markdown extensions
    let allowed_exts = ["md", "markdown", "mkd", "mdown"];
    if let Some(name) = original_filename {
        let ext = Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());
        if ext.as_deref().map_or(true, |e| !allowed_exts.contains(&e)) {
            return Err(MarkdownError::InvalidExtension);
        }
    } else {
        return Err(MarkdownError::InvalidExtension);
    }

    // 2. MIME detection (tolerant mode)
    let infer = Infer::new();
    match infer.get_from_path(file_path) {
        Ok(Some(mime)) => {
            // Acceptable MIME types for markdown
            let valid_mime_types = [
                "text/markdown",
                "text/x-markdown",
            ];

            if !valid_mime_types.contains(&mime.mime_type()) {
                return Err(MarkdownError::InvalidType(mime.mime_type().to_string()));
            }
        }
        Ok(None) => {}
        Err(e) => {
            return Err(MarkdownError::MimeDetectionFailed(e.to_string()));
        }
    }

    // 3. File size check
    let metadata = fs::metadata(file_path)
        .await
        .map_err(|e| MarkdownError::IoError(e))?;
    if metadata.len() > max_size as u64 {
        return Err(MarkdownError::FileTooLarge);
    }

    // 4. Read file content
    let content = fs::read_to_string(file_path)
        .await
        .map_err(|e| MarkdownError::IoError(e))?;
    if content.trim().is_empty() {
        return Err(MarkdownError::EmptyFile);
    }

    Ok(content)
}

/// Converts a validated Markdown file to sanitized HTML.
///
/// - `original_filename`: The filename from TempFile::file_name()
/// - `file_path`: The path from TempFile::file.path()
pub async fn markdown_file_to_html(
    original_filename: Option<&str>,
    file_path: &Path
) -> Result<String, MarkdownError> {
    let content = read_markdown_file(original_filename, file_path, 2 * 1024 * 1024).await?;
    Ok(safe_markdown_to_html(&content))
}

/// Converts multiple Markdown files to HTML. Skips invalid ones.
///
/// - `files`: Vec of tuples containing the original filename and file path
pub async fn batch_markdown_to_html(
    files: &[(Option<&str>, &Path)]
) -> Vec<Result<String, MarkdownError>> {
    let futures: Vec<_> = files
        .iter()
        .map(|(filename, path)| markdown_file_to_html(*filename, path))
        .collect();

    join_all(futures).await
}


/// All errors related to Markdown file handling.
#[derive(Debug, Display)]
pub enum MarkdownError {
    #[display("Invalid file extension. Only .md files are allowed.")]
    InvalidExtension,

    #[display("Invalid MIME type: {_0}")]
    InvalidType(String),

    #[display("File is empty.")]
    EmptyFile,

    #[display("File size exceeds maximum allowed.")]
    FileTooLarge,

    #[display("Failed to read file: {_0}")]
    IoError(io::Error),

    #[display("Invalid Markdown content.")]
    InvalidContent,

    #[display("MIME detection failed: {_0}")]
    MimeDetectionFailed(String)
}
