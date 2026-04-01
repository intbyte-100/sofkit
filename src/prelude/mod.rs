//! Prelude module.
//!
//! This module re-exports small UI helpers and glue used across the crate.
//! The implementation is split into submodules under `src/prelude/`:
//!
//! - `box_wrapper`          -> `BoxWrapper` wrapper for `gtk::Box`
//! - `macros`               -> `hbox!` and `vbox!` macros
//! - `button_builder`       -> `ReactiveButtonBuilder`, `ButtonBuilderExt`, `button()`
//! - `state_ext`            -> `statefull()` and `StateHolderExt`
//! - `reactive_widgets`     -> reactive builders for `Label`, `Entry`, `CheckButton`
//!
//! The public surface is kept the same as before so existing `use crate::prelude::*;`
//! and usages in the rest of the codebase continue to work.

#[macro_use]
mod macros;

mod box_wrapper;
mod button_builder;
mod reactive_widgets;
mod state_ext;

pub use box_wrapper::BoxWrapper;
pub use button_builder::{ButtonBuilderExt, ReactiveButtonBuilder, button};
pub use reactive_widgets::{
    CheckButtonBuilderExt, EntryBuilderExt, LabelBuilderExt, ReactiveCheckButtonBuilder,
    ReactiveEntryBuilder, ReactiveLabelBuilder, check_button, entry, label,
};
pub use state_ext::{StateHolderExt, statefull};
