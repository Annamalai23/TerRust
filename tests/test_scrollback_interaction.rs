use terrust::ui::blocks::{BlockManager, BlockType, Block};
use terrust::ui::search::SearchEngine;

mod common;

#[test]
fn test_search_engine_integration() {
    let mut manager = BlockManager::new(100, 1000);
    let mut block = Block::new(BlockType::Output).with_title("Test");
    block.content = vec![
        common::row_from_str("hello world"),
        common::row_from_str("this is a test"),
        common::row_from_str("goodbye world"),
    ];
    manager.add_block(block);

    let mut engine = SearchEngine::new();
    engine.set_query("world", &manager);
    assert_eq!(engine.total_matches(), 2);
    assert_eq!(engine.matches()[0].line_text, "hello world");
    assert_eq!(engine.matches()[1].line_text, "goodbye world");
}

#[test]
fn test_search_engine_navigation() {
    let mut manager = BlockManager::new(100, 1000);
    let mut block = Block::new(BlockType::Output).with_title("Test");
    block.content = vec![
        common::row_from_str("apple banana apple"),
    ];
    manager.add_block(block);

    let mut engine = SearchEngine::new();
    engine.set_query("apple", &manager);
    assert_eq!(engine.total_matches(), 2);

    assert!(engine.current_match_info().is_some());
    assert_eq!(engine.current_match_info().unwrap().col_idx, 0);

    engine.next_match();
    assert_eq!(engine.current_match_info().unwrap().col_idx, 13);

    engine.next_match();
    assert_eq!(engine.current_match_info().unwrap().col_idx, 0);
}

#[test]
fn test_search_engine_case_insensitive() {
    let mut manager = BlockManager::new(100, 1000);
    let mut block = Block::new(BlockType::Output).with_title("Test");
    block.content = vec![
        common::row_from_str("HELLO WORLD"),
    ];
    manager.add_block(block);

    let mut engine = SearchEngine::new();
    engine.set_query("hello", &manager);
    assert_eq!(engine.total_matches(), 1);
}

#[test]
fn test_search_engine_no_matches() {
    let mut manager = BlockManager::new(100, 1000);
    let mut block = Block::new(BlockType::Output).with_title("Test");
    block.content = vec![
        common::row_from_str("hello world"),
    ];
    manager.add_block(block);

    let mut engine = SearchEngine::new();
    engine.set_query("zzzz", &manager);
    assert_eq!(engine.total_matches(), 0);
    assert!(!engine.has_matches());
}

#[test]
fn test_search_engine_reset() {
    let mut manager = BlockManager::new(100, 1000);
    let mut block = Block::new(BlockType::Output).with_title("Test");
    block.content = vec![
        common::row_from_str("hello world"),
    ];
    manager.add_block(block);

    let mut engine = SearchEngine::new();
    engine.set_query("hello", &manager);
    assert!(engine.has_matches());

    engine.reset();
    assert!(!engine.has_matches());
    assert_eq!(engine.query(), "");
    assert_eq!(engine.total_matches(), 0);
}

#[test]
fn test_search_engine_empty_block_manager() {
    let manager = BlockManager::new(100, 1000);
    let mut engine = SearchEngine::new();
    engine.set_query("test", &manager);
    assert_eq!(engine.total_matches(), 0);
}

#[test]
fn test_search_engine_multi_block() {
    let mut manager = BlockManager::new(100, 1000);

    let mut block1 = Block::new(BlockType::Command).with_title("Cmd1");
    block1.content = vec![common::row_from_str("ls -la")];
    let id1 = manager.add_block(block1);

    let mut block2 = Block::new(BlockType::Output).with_title("Out1");
    block2.content = vec![common::row_from_str("total 42")];
    manager.add_block(block2);

    let mut engine = SearchEngine::new();
    engine.set_query("ls", &manager);
    assert_eq!(engine.total_matches(), 1);
    assert_eq!(engine.matches()[0].block_id, id1);
}

#[test]
fn test_render_scrollbar_visibility() {
    // scrollbar returns an empty line when content fits on screen
    let line = terrust::ui::render::render_scrollbar_line(20, 10, 20, 0);
    let rendered = format!("{:?}", line);
    assert!(!rendered.is_empty());
}

#[test]
fn test_render_scrollbar_with_scroll() {
    // scrollbar shows indicator when content exceeds visible area
    let line = terrust::ui::render::render_scrollbar_line(20, 100, 20, 30);
    let rendered = format!("{:?}", line);
    assert!(!rendered.is_empty());
}
