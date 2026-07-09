//! # app-todo-flex
//!
//! The todo application bundled with the flexbox-style layout library.
//!
//! All of the substance lives elsewhere: `app_todo_core` is the application,
//! `layout_flex` is the layout, and `app_shell` is the bundle glue. This file
//! only chooses the layout engine. The grid bundle is identical except for the
//! single type name below - that is the M3 demonstration in source form.

#![no_std]

app_shell::bundle!(layout_flex::FlexLayout, layout_flex::FlexLayout::new());
