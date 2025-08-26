//! # ANSIEscapeRS
//!
//! Unified API for ANSI escape code creation, parsing, and type-safe manipulation.

//!
//! ## Usage

//! All public types and functions are available directly from the crate root:

//!
//! ```rust

//! use ansiescapers::{creator::*, interpreter::*, types::*};

//! ```
//!
//! See the documentation for each type for details and examples.

#![allow(unused_imports)]

mod ansi_creator;

mod ansi_interpreter;

mod ansi_types;

pub(crate) mod creator {
    // Re-export all public items from creator
    pub use crate::ansi_escape::ansi_creator::*;
}

// Re-export all public items from types
pub(crate) mod types {
    pub use crate::ansi_escape::ansi_types::*;
}

// Re-export all public items from interpreter
pub(crate) mod interpreter {
    pub use crate::ansi_escape::ansi_interpreter::*;
}
