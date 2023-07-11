//! This file is to ensure that the macro is expanded when clippy is run
//! with --no-default-features

use core::fmt::Display;

use super::declare_dyn_slice;

declare_dyn_slice!(Display, display_dyn_slice);
