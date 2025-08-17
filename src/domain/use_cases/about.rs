use uuid::Uuid;
use validator::Validate;

use crate::{
    entities::about_me::{AboutMeCreatedResponse, AboutMeResponse, NewAboutMe, UpdateAboutMeRequest}, 
    errors::AppError, 
    repositories::about::AboutRepository, 
    utils::valid_uuid::valid_uuid
};


pub struct AboutHandler<R>
where 
    R: AboutRepository,
{
    pub about_repo: R,
}

impl<R> AboutHandler<R>
where 
    R: AboutRepository,
{
    pub fn new(about_repo: R) -> Self {
        AboutHandler { about_repo }
    }

    /// Creates the "About Me" content with the provided markdown and effective date
    pub async fn create_about_me(
        &self, 
        request: NewAboutMe
    ) -> Result<AboutMeCreatedResponse, AppError> {
        request.validate()?;

        let new_about_me = request.prepare_for_insert();

        let id = self.about_repo.create_about_me(&new_about_me).await?;

        let current_revision = self.about_repo.get_current_revision(new_about_me.effective_date).await?;

        Ok(AboutMeCreatedResponse {
            id,
            revision: current_revision,
            message: format!(
                "Created 'About Me' content with  revision {} and effective date {}",
                current_revision, new_about_me.effective_date
            ),
        })
    }

    /// Retrieves the current "About Me" content
    pub async fn get_about_me(&self) -> Result<AboutMeResponse, AppError> {
        self.about_repo.get_current_about_me().await
            .map_err(|e| match e {
                AppError::NotFound(_) => AppError::NotFound("About Me content not found".to_string()),
                _ => e,
            })
    }

    /// Updates the "About Me" content with new markdown and effective date
    pub async fn update_about_me_content(
        &self, 
        id: Uuid, 
        request: UpdateAboutMeRequest
    ) -> Result<AboutMeResponse, AppError> {
        request.validate()?;

        let valid_id = valid_uuid(&id.to_string())?;

        let current = self.about_repo.get_about_me_by_id(&valid_id).await?;

        if current.revision != request.expected_revision {
            return Err(AppError::Conflict("Revision mismatch".to_string()));
        }

        let updated = self.about_repo.update_about_me_content(id, &request.content_markdown).await?;

        Ok(updated.into())
    }

    /// Deletes the "About Me" content by ID
    pub async fn delete_about_me(
        &self, 
        id: Uuid,
        hard_delete: bool
    ) -> Result<(), AppError> {
        let valid_id = valid_uuid(&id.to_string())?;

        match hard_delete {
            true => self.about_repo.hard_delete_about_me(valid_id).await,
            false => self.about_repo.soft_delete_about_me(valid_id).await,
        }.map_err(|e| match e {
            AppError::NotFound(_) => AppError::NotFound("About Me content not found".to_string()),
            _ => e,
        })
    }
}