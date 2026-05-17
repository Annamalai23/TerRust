//! UI module for TerRust
//!
//! Provides the user interface components including block-based output rendering,
//! command input handling, and reusable UI components.

pub mod blocks;
pub mod components;
pub mod input;
pub mod render;
pub mod search;

pub use blocks::{Block, BlockManager, BlockType};
pub use components::{Alignment, Component, Layout};
pub use input::InputBar;
