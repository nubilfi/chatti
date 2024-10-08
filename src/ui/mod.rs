//! The UI module provides the user interface components for the Chatti application.
//!
//! This module contains various submodules that handle different aspects of the
//! user interface, including chat UI, input handling, markdown rendering, and UI state management.

// mod chat_ui;
// mod input_handler;
// mod markdown_renderer;
// mod spinner;
// mod ui_renderer;
// mod ui_state;
//
// pub use chat_ui::ChatUI;
// pub use ui_state::Action;

pub mod chat;
pub mod input_handler;
pub mod markdown_renderer;
pub mod renderer;
pub mod spinner;
pub mod state;

pub use chat::Interface;
pub use state::Action;
