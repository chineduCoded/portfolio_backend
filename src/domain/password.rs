use validator::ValidationError;
use zxcvbn::zxcvbn;

const MIN_LENGTH: usize = 8;
const MIN_STRENGTH_SCORE: u8 = 3;

/// Context-aware password validation strength
pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    if password.len() < MIN_LENGTH {
        let mut error = ValidationError::new("password_length");
        error.message = Some(format!("Must be at least {} characters", MIN_LENGTH).into());
        return Err(error);
    }

    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));
    let has_symbol = password.chars().any(|c| "!@#$%^&*".contains(c));

    if !(has_upper && has_digit && has_symbol) {
        let mut error = ValidationError::new("password_complexity");
        error.message = Some("Must include uppercase, number, and symbol".into());
        return Err(error);
    }

    let estimate = zxcvbn(password, &[]);
    let score = estimate.score() as u8;

    if score < MIN_STRENGTH_SCORE {
        let feedback = estimate.feedback()
            .and_then(|f| f.warning().map(|w| w.to_string()))
            .unwrap_or_else(|| "Password is too weak".to_string());

        let mut error = ValidationError::new("password_complexity");
        error.message = Some(format!("Must include uppercase, number, and symbol: {}", feedback).into());
        return Err(error);
    }

    Ok(())
}