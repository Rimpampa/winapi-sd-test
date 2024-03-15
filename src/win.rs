use std::{fmt::Display, num::NonZeroU32};

use windows_sys::Win32::Foundation::{GetLastError, WIN32_ERROR};

/// An error returned by a Windows API call
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error(NonZeroU32);

impl Error {
    /// Check if the last operation returned an error
    ///
    /// This function is a safe wrapper around the
    /// Windows API [`GetLastError`] function
    pub fn check() -> Result<()> {
        // SAFETY: how can this be unsafe?
        match NonZeroU32::new(unsafe { GetLastError() }) {
            Some(error) => Err(Self(error)),
            None => Ok(()),
        }
    }

    /// Return the last error that occurred
    ///
    /// This function is a safe wrapper around the
    /// Windows API [`GetLastError`] function
    /// and it is to be used when it's sure that
    /// there is an error (e.g. the return type
    /// of a function suggests so).
    ///
    /// # Panic
    ///
    /// This function panics if there is no error.
    pub fn get() -> Self {
        Self::check().unwrap_err()
    }

    /// Returns the actual error code
    pub fn into_raw(self) -> WIN32_ERROR {
        self.0.get()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: get a description of the error when possible
        write!(f, "WinAPI error {self:0x?}")
    }
}

pub type Result<T> = core::result::Result<T, Error>;

include!(concat!(env!("OUT_DIR"), "/impl_win_error.rs"));
