//! # app-todo-grid
//!
//! The todo application bundled with the fixed-grid layout library.
//!
//! Compare this file to `app-todo-flex/src/lib.rs`: they are identical except
//! for the layout type named below. Same app, same content tree, same bundle
//! glue, same host - different layout. That one-line difference is the entire
//! M3 demonstration.

#![no_std]

app_shell::bundle!(layout_grid::GridLayout, layout_grid::GridLayout::new());
