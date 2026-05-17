use terrust::ui::blocks::{Block, BlockManager, BlockType, BorderChars, get_block_style};
use terrust::ui::input::InputBar;
use terrust::ui::components::{Alignment, Layout, StyledText, ProgressBar, Breadcrumb, Component};
use terrust::config::ThemeConfig;
use terrust::terminal::{Cell, Attributes};

fn cell(c: char) -> Cell {
    Cell { character: c, foreground: None, background: None, attributes: Attributes::default() }
}

#[test]
fn test_block_lifecycle() {
    let mut manager = BlockManager::new(100, 1000);

    let id1 = manager.add_command_block("ls -la");
    let id2 = manager.add_output_block();

    assert_eq!(manager.len(), 2);
    assert!(manager.get_block(id1).is_some());
    assert!(manager.get_block(id2).is_some());
}

#[test]
fn test_block_types() {
    let cmd = Block::new(BlockType::Command);
    assert_eq!(cmd.block_type, BlockType::Command);
    assert!(!cmd.block_type.is_error());

    let err = Block::new(BlockType::Error);
    assert!(err.block_type.is_error());

    let ai = Block::new(BlockType::AI);
    assert!(ai.block_type.is_ai());
}

#[test]
fn test_block_pin_preservation() {
    let mut manager = BlockManager::new(2, 100);

    let pinned = manager.add_block(Block::new(BlockType::Output));
    manager.get_block_mut(pinned).unwrap().pin();

    let _id2 = manager.add_block(Block::new(BlockType::Output));
    let _id3 = manager.add_block(Block::new(BlockType::Output));

    assert_eq!(manager.len(), 2);
    assert!(manager.get_block(pinned).is_some());
    assert!(manager.pinned_blocks().len() == 1);
}

#[test]
fn test_block_content_and_metadata() {
    let content = vec![
        vec![cell('h'), cell('i')],
        vec![cell('t'), cell('h'), cell('e'), cell('r'), cell('e')],
    ];

    let block = Block::new(BlockType::Output)
        .with_title("Result")
        .with_content(content.clone())
        .with_command_metadata("echo hi".into(), "/tmp".into(), "/bin/bash".into());

    assert_eq!(block.title.unwrap(), "Result");
    assert_eq!(block.content.len(), 2);
    assert_eq!(block.command.unwrap(), "echo hi");
    assert_eq!(block.cwd.unwrap(), "/tmp");
}

#[test]
fn test_block_navigation() {
    let mut manager = BlockManager::new(100, 1000);
    let ids: Vec<_> = (0..5).map(|_| manager.add_block(Block::new(BlockType::Output))).collect();

    assert_eq!(manager.get_current_block().map(|b| b.id), Some(ids[4]));
    assert_eq!(manager.next_block(), Some(ids[0]));
    assert_eq!(manager.next_block(), Some(ids[1]));
    assert_eq!(manager.prev_block(), Some(ids[0]));
}

#[test]
fn test_block_clear_all_preserves_pinned() {
    let mut manager = BlockManager::new(100, 1000);
    let pinned = manager.add_block(Block::new(BlockType::Output));
    manager.get_block_mut(pinned).unwrap().pin();
    let _unpinned = manager.add_block(Block::new(BlockType::Output));

    manager.clear_all();
    assert_eq!(manager.len(), 1);
    assert!(manager.get_block(pinned).is_some());
}

#[test]
fn test_block_search() {
    let mut manager = BlockManager::new(100, 1000);
    let id = manager.add_block(
        Block::new(BlockType::Output)
            .with_content(vec![vec![cell('e'), cell('r'), cell('r')]])
    );

    let results = manager.search_blocks("err");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, id);
}

#[test]
fn test_block_scroll_operations() {
    let mut block = Block::new(BlockType::Output);
    block.height = 5;
    block.scroll_up(3);
    assert_eq!(block.scroll_offset, 3);
    block.scroll_down(1);
    assert_eq!(block.scroll_offset, 2);
    block.scroll_to_top();
    assert_eq!(block.scroll_offset, 0);
}

#[test]
fn test_block_collapse_expand() {
    let mut block = Block::new(BlockType::Output);
    assert!(!block.collapsed);
    block.collapse();
    assert!(block.collapsed);
    block.expand();
    assert!(!block.collapsed);
}

#[test]
fn test_input_bar_editing() {
    let mut input = InputBar::new("$ ");
    assert!(input.is_empty());

    input.insert_str("hello world");
    assert_eq!(input.get_content(), "hello world");

    input.move_cursor_to_start();
    input.move_word_right();
    input.delete_word();
    assert_eq!(input.get_content(), "hello ");
}

#[test]
fn test_input_bar_history_integration() {
    let mut input = InputBar::new("$ ");
    input.history_push("cargo build");
    input.history_push("cargo test");
    input.history_push("git push");

    assert_eq!(input.history.len(), 3);

    input.clear();
    assert!(input.history_up());
    assert_eq!(input.get_content(), "git push");
    assert!(input.history_up());
    assert_eq!(input.get_content(), "cargo test");
}

#[test]
fn test_input_bar_history_prevents_duplicates() {
    let mut input = InputBar::new("$ ");
    input.history_push("cmd");
    input.history_push("cmd");
    assert_eq!(input.history.len(), 1);
}

#[test]
fn test_input_bar_empty_history_skip() {
    let mut input = InputBar::new("$ ");
    input.history_push("");
    input.history_push("  ");
    input.history_push("real_cmd");
    assert_eq!(input.history.len(), 1);
}

#[test]
fn test_input_bar_command_parts() {
    let mut input = InputBar::new("$ ");
    input.insert_str("git commit -m 'fix'");
    let parts = input.get_command_parts();
    assert_eq!(parts.command.as_deref(), Some("git"));
    assert!(parts.has_args());
}

#[test]
fn test_input_bar_delete_operations() {
    let mut input = InputBar::new("$ ");
    input.insert_str("abcdef");

    input.move_cursor_to_end();
    input.delete_to_start();
    assert!(input.is_empty());

    input.insert_str("hello world");
    input.move_cursor_to_end();
    input.move_cursor_left();
    input.move_cursor_left();
    input.delete_to_end();
    assert_eq!(input.get_content(), "hello wor");
}

#[test]
fn test_component_helpers() {
    assert_eq!(Component::separator(5, "single"), "─────");
    assert_eq!(Component::separator(5, "double"), "═════");

    let truncated = Component::truncated_text("hello world", 8, "...");
    assert_eq!(truncated, "hello...");

    let right = Component::right_align_text("hi", 6);
    assert_eq!(right, "    hi");

    let center = Component::center_text("hi", 6);
    assert_eq!(center, "  hi  ");

    let status = Component::status_indicator("OK", true);
    assert!(status.contains("✓"));
}

#[test]
fn test_styled_text_builder() {
    let text = StyledText::new()
        .push_plain("Hello ")
        .push_bold("World")
        .push_italic("!")
        .push_fg("red", ratatui::style::Color::Red);

    assert_eq!(text.len(), 15);
    assert_eq!(text.parts.len(), 4);
    let spans = text.to_spans();
    assert_eq!(spans.len(), 4);
}

#[test]
fn test_progress_bar_rendering() {
    let bar = ProgressBar::new(50, 100, 20);
    assert_eq!(bar.percentage(), 50.0);
    assert_eq!(bar.filled_width(), 9);
    let rendered = bar.render();
    assert!(rendered.contains("50%"));
}

#[test]
fn test_breadcrumb_navigation() {
    let mut breadcrumb = Breadcrumb::new().push("home").push("projects").push("terrust");
    assert_eq!(breadcrumb.render(), "home > projects > terrust");
    assert_eq!(breadcrumb.pop(), Some("terrust".to_string()));
    assert_eq!(breadcrumb.len(), 2);
}

#[test]
fn test_layout_alignments() {
    let left = Layout::new().with_alignment(Alignment::Left);
    assert_eq!(left.align_text("hi", 5), "hi   ");

    let right = Layout::new().with_alignment(Alignment::Right);
    assert_eq!(right.align_text("hi", 5), "   hi");

    let center = Layout::new().with_alignment(Alignment::Center);
    assert_eq!(center.align_text("hi", 5), " hi  ");
}

#[test]
fn test_border_chars_styles() {
    let single = BorderChars::single();
    assert_eq!(single.top_left, "┌");

    let rounded = BorderChars::rounded();
    assert_eq!(rounded.top_left, "╭");

    let double = BorderChars::double();
    assert_eq!(double.top_left, "╔");

    let heavy = BorderChars::heavy();
    assert_eq!(heavy.top_left, "┏");
}

#[test]
fn test_get_block_style_with_theme() {
    let theme = ThemeConfig::tokyo_night();
    let style = get_block_style(BlockType::Command, &theme);
    assert!(style.show_border);
    assert_eq!(style.border_chars.top_left, "┌");

    let ai_style = get_block_style(BlockType::AI, &theme);
    assert_eq!(ai_style.border_chars.top_left, "╭");
}

#[test]
fn test_block_manager_filter_functions() {
    let mut manager = BlockManager::new(100, 1000);
    manager.add_block(Block::new(BlockType::Output));
    manager.add_block(Block::new(BlockType::Error));
    manager.add_block(Block::new(BlockType::AI));
    manager.add_block(Block::new(BlockType::Command));

    assert_eq!(manager.error_blocks().len(), 1);
    assert_eq!(manager.ai_blocks().len(), 1);
}

#[test]
fn test_input_bar_max_length_enforced() {
    let mut input = InputBar::new("$ ");
    let long = "a".repeat(5000);
    input.insert_str(&long);
    assert!(input.len() <= 4096);
}

#[test]
fn test_input_bar_cursor_position() {
    let mut input = InputBar::new("$ ");
    input.insert_str("hello");
    assert_eq!(input.cursor_position, 5);
    input.move_cursor_left();
    assert_eq!(input.cursor_position, 4);
    input.move_cursor_to_start();
    assert_eq!(input.cursor_position, 0);
    input.move_cursor_to_end();
    assert_eq!(input.cursor_position, 5);
}
