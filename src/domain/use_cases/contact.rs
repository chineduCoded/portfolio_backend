use crate::{
    entities::contact_me::{ContactMeListResponse, ContactMeMessage, ContactMeResponse, NewContactMeForm}, 
    errors::AppError, 
    repositories::contact_me::ContactMeRepository, utils::valid_uuid::valid_uuid
};
use validator::Validate;



pub struct ContactMeHandler<R>
where 
    R: ContactMeRepository,
{
    pub contact_repo: R,
}

impl<R> ContactMeHandler<R>
where 
    R: ContactMeRepository,
{
    pub fn new(contact_repo: R) -> Self {
        ContactMeHandler { contact_repo }
    }

    /// Handles the creation of a new contact message
    pub async fn create_contact_message(
        &self, 
        request: NewContactMeForm
    ) -> Result<ContactMeResponse, AppError> {
        request.validate()?;

        let new_msg = request.try_into()?;

        let id = self.contact_repo.create_contact_message(&new_msg).await?;

        Ok(ContactMeResponse {
            message: "Your message has been received.".to_string(),
            id,
        })
    }

    /// Retrieves a contact message by its ID
    pub async fn get_contact_message_by_id(&self, id: &str) -> Result<ContactMeMessage, AppError> {
        let valid_id = valid_uuid(id)?;

        let msg = self.contact_repo.get_contact_message_by_id(&valid_id).await?;

        Ok(msg)
    }

    /// Lists all contact messages
    pub async fn list_contact_messages(&self) -> Result<ContactMeListResponse, AppError> {
        let messages = self.contact_repo.list_contact_messages().await?;
        let total = self.contact_repo.count_contact_messages().await?;

        Ok(ContactMeListResponse {
            messages,
            total,
        })
    }

    /// Deletes a contact message by its ID
    pub async fn delete_contact_message(
        &self, 
        id: &str, 
        hard_delete: bool
    ) -> Result<(), AppError> {
        let valid_id = valid_uuid(id)?;

        match hard_delete {
            true => self.contact_repo.hard_delete_contact_message(&valid_id).await,
            false => self.contact_repo.soft_delete_contact_message(&valid_id).await,
        }.map_err(|e| match e {
            AppError::NotFound(_) => AppError::NotFound("Contact message not found".to_string()),
            _ => e,
        })
    }
}