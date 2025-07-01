use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString, Error as Argon2Error
    },
    Argon2, Algorithm, Params, Version
};

use crate::errors::PasswordError;

pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15_000, 2, 1, None)
            .map_err(|e| PasswordError::InvalidParameters(e.to_string()))?
    );
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordError::HashingError(e.to_string()))
        .map(|hash| hash.to_string())
}

pub fn verify_password(
    password: &str,
    hashed: &str,
) -> Result<bool, PasswordError> {
    let parsed_hash = PasswordHash::new(hashed)
        .map_err(|e| PasswordError::InvalidHashFormat(e.to_string()))?;
    
    match Argon2::default().verify_password(
        password.as_bytes(),
        &parsed_hash,
    ) {
        Ok(()) => Ok(true),
        Err(Argon2Error::Password) => Ok(false),
        Err(e) => Err(PasswordError::VerificationError(e.to_string())),
    }
}