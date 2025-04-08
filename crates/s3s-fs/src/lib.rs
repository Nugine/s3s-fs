#![allow(clippy::missing_errors_doc)]

mod error;

mod system;

pub use self::error::{Error, Result};
pub use self::system::System;
