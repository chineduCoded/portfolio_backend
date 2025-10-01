use std::borrow::Cow;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidateLength, ValidationErrors};

/// Represents optional field semantics in PATCH/UPDATE requests.
///
/// - `Unchanged` → field not touched
/// - `SetToNull` → explicitly null
/// - `SetToValue` → set to provided value
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptionField<T> {
    Unchanged,
    SetToNull,
    SetToValue(T),
}

impl<T> Default for OptionField<T> {
    fn default() -> Self {
        OptionField::Unchanged
    }
}

// ---------------------- Validation support ----------------------

impl<T> ValidateLength<u64> for OptionField<T>
where 
    T: ValidateLength<u64> 
{
    fn length(&self) -> Option<u64> {
        match self {
            OptionField::SetToValue(value) => value.length(),
            _ => None,
        }
    }
    fn validate_length(&self, min: Option<u64>, max: Option<u64>, equal: Option<u64>) -> bool {
        match self {
            OptionField::SetToValue(value) => value.validate_length(min, max, equal),
            _ => true,
        }
    }
}

impl<T: Validate> Validate for OptionField<T> {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            OptionField::SetToValue(value) => value.validate(),
            _ => Ok(()),
        }
    }
}

// ---------------------- Core helpers & conversions ----------------------

impl<T> OptionField<T> {
    /// Convert to nested option:
    /// - `None` → unchanged
    /// - `Some(None)` → set null
    /// - `Some(Some(T))` → set to value
    pub fn into_option(self) -> Option<Option<T>> {
        match self {
            Self::Unchanged => None,
            Self::SetToNull => Some(None),
            Self::SetToValue(v) => Some(Some(v)),
        }
    }

    /// Borrowed nested option:
    /// - `None` → unchanged
    /// - `Some(None)` → set null
    /// - `Some(Some(&T))` → set to value
    pub fn as_ref_option(&self) -> Option<Option<&T>> {
        match self {
            Self::Unchanged => None,
            Self::SetToNull => Some(None),
            Self::SetToValue(value) => Some(Some(value)),
        }
    }

    /// Mutable borrowed nested option:
    pub fn as_mut_option(&mut self) -> Option<Option<&mut T>> {
        match self {
            Self::Unchanged => None,
            Self::SetToNull => Some(None),
            Self::SetToValue(value) => Some(Some(value)),
        }
    }

    /// Transform inner value if `SetToValue`
    pub fn map_value<U, F: FnOnce(T) -> U>(self, f: F) -> OptionField<U> {
        match self {
            Self::Unchanged => OptionField::Unchanged,
            Self::SetToNull => OptionField::SetToNull,
            Self::SetToValue(v) => OptionField::SetToValue(f(v)),
        }
    }

    /// Transform by borrowing the inner value
    pub fn map_value_ref<U, F>(&self, f: F) -> OptionField<U>
    where
        F: FnOnce(&T) -> U,
    {
        match self {
            Self::Unchanged => OptionField::Unchanged,
            Self::SetToNull => OptionField::SetToNull,
            Self::SetToValue(v) => OptionField::SetToValue(f(v)),
        }
    }

    // ----------------- Ergonomics -----------------

    /// True when `Unchanged`.
    pub fn is_unchanged(&self) -> bool {
        matches!(self, Self::Unchanged)
    }

    /// True when `SetToNull`.
    pub fn is_set_to_null(&self) -> bool {
        matches!(self, Self::SetToNull)
    }

    /// If `SetToValue`, returns a reference to inner value.
    pub fn value_ref(&self) -> Option<&T> {
        if let Self::SetToValue(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// If `SetToValue`, returns a mutable reference to inner value.
    pub fn value_mut(&mut self) -> Option<&mut T> {
        if let Self::SetToValue(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// If `SetToValue`, consumes and returns inner value.
    pub fn take_value(self) -> Option<T> {
        if let Self::SetToValue(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Convert into `Option<T>` (what SQLx expects)
    pub fn flatten(self) -> Option<T> {
        match self {
            OptionField::SetToValue(v) => Some(v),
            _ => None
        }
    }

    /// Borrowed flatten for references
    pub fn flatten_ref(&self) -> Option<&T> {
        match self {
            OptionField::SetToValue(v) => Some(v),
            _ => None
        }
    }
}

// ---------------------- Type-specific convenience ----------------------

impl OptionField<String> {
    pub fn flatten_str(&self) -> Option<&str> {
        self.flatten_ref().map(|s| s.as_str())
    }
}

impl<T> OptionField<Vec<T>> {
    pub fn flatten_slice(&self) -> Option<&[T]> {
        self.flatten_ref().map(|v| v.as_slice())
    }
}

impl OptionField<bool> {
    pub fn flatten_bool(&self) -> Option<bool> {
        self.flatten_ref().copied()
    }
}

impl OptionField<DateTime<Utc>> {
    pub fn flatten_datetime(&self) -> Option<&DateTime<Utc>> {
        self.flatten_ref()
    }
}

// ---------------------- From conversions ----------------------

// From nested option into OptionField
impl<T> From<Option<Option<T>>> for OptionField<T> {
    fn from(opt: Option<Option<T>>) -> Self {
        match opt {
            None => OptionField::Unchanged,
            Some(None) => OptionField::SetToNull,
            Some(Some(v)) => OptionField::SetToValue(v),
        }
    }
}

impl<T> From<OptionField<T>> for Option<Option<T>> {
    fn from(of: OptionField<T>) -> Self {
        of.into_option()
    }
}

// ---------------------- Cow: safe helper methods (no lossy From impl) ----------------------

impl<'a, T> From<Option<Option<&'a T>>> for OptionField<Cow<'a, T>>
where
    T: 'a + ToOwned + ?Sized,
{
    fn from(opt: Option<Option<&'a T>>) -> Self {
        match opt {
            None => OptionField::Unchanged,
            Some(None) => OptionField::SetToNull,
            Some(Some(v)) => OptionField::SetToValue(Cow::Borrowed(v)),
        }
    }
}

/// NOTE:
/// We intentionally *do not* provide an automatic `From<OptionField<Cow<'a, T>>> for Option<Option<&'a T>>>`
/// because converting `Cow::Owned` into a `&'a T` may not be possible without extending lifetimes
/// (that would be lossy or unsafe). Instead, provide a safe borrow-view method tied to `&self`.
impl<'a, T> OptionField<Cow<'a, T>>
where
    T: 'a + ToOwned + ?Sized,
{
    /// Borrow-view of possibly-owned `Cow` value.
    ///
    /// Returns:
    /// - `None` -> `Unchanged`
    /// - `Some(None)` -> `SetToNull`
    /// - `Some(Some(&T))` -> `SetToValue` (works for both `Borrowed` and `Owned`, lifetime is tied to `&self`)
    pub fn as_ref_option_borrowed(&self) -> Option<Option<&T>> {
        match self {
            OptionField::Unchanged => None,
            OptionField::SetToNull => Some(None),
            OptionField::SetToValue(cow) => Some(Some(cow.as_ref())),
        }
    }

    /// Consume and produce `Option<Option<Cow<'a, T>>>`
    pub fn into_option_cow(self) -> Option<Option<Cow<'a, T>>> {
        match self {
            OptionField::Unchanged => None,
            OptionField::SetToNull => Some(None),
            OptionField::SetToValue(c) => Some(Some(c)),
        }
    }
}

// ---------------------- Aliases ----------------------

pub type PatchString = OptionField<String>;
pub type PatchVec<T> = OptionField<Vec<T>>;
pub type PatchDateTimeUtc = OptionField<DateTime<Utc>>;