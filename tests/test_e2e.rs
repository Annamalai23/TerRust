use terrust::terminal::{Cell, Grid};
use terrust::ui::blocks::{BlockManager, BlockType};

mod common;

#[test]
fn test_e2e_simple_command_lifecycle() {
    let mut grid = Grid::new(40, 10);
    let mut cursor_col = 0usize;
    let mut cursor_row = 0usize;

    fn place_text(
        grid: &mut Grid,
        data: &[u8],
        cursor_col: &mut usize,
        cursor_row: &mut usize,
    ) {
        for &byte in data {
            if byte == b'\n' {
                *cursor_col = 0;
                *cursor_row += 1;
                if *cursor_row >= grid.rows as usize {
                    grid.scroll_up();
                    *cursor_row = grid.rows as usize - 1;
                }
            } else if byte == b'\r' {
                *cursor_col = 0;
            } else {
                let cell = Cell {
                    character: byte as char,
                    foreground: None,
                    background: None,
                    attributes: terrust::terminal::Attributes::default(),
                };
                grid.set(*cursor_col as u16, *cursor_row as u16, cell);
                *cursor_col += 1;
                if *cursor_col >= grid.columns as usize {
                    *cursor_col = 0;
                    *cursor_row += 1;
                    if *cursor_row >= grid.rows as usize {
                        grid.scroll_up();
                        *cursor_row = grid.rows as usize - 1;
                    }
                }
            }
        }
    }

    // Stage 1: prompt
    place_text(&mut grid, b"$ ", &mut cursor_col, &mut cursor_row);
    // Stage 2: user command
    place_text(&mut grid, b"ls -la", &mut cursor_col, &mut cursor_row);
    // Stage 3: Enter
    place_text(&mut grid, b"\r\n", &mut cursor_col, &mut cursor_row);
    // Stage 4: output
    place_text(&mut grid, b"total 128\r\ndrwxr-xr-x  2 user staff    64 May 10 12:00 .\r\n-rw-r--r--  1 user staff   128 May 10 12:00 file.txt\r\n", &mut cursor_col, &mut cursor_row);
    // Stage 5: new prompt
    place_text(&mut grid, b"$ ", &mut cursor_col, &mut cursor_row);

    let text_lines: Vec<String> = grid
        .cells
        .iter()
        .map(|row| row.iter().map(|c| c.character).collect::<String>())
        .collect();

    let joined = text_lines.join("\n");
    assert!(
        joined.contains("ls -la"),
        "Grid should contain command. Content:\n{}",
        joined
    );
    assert!(
        joined.contains("total 128"),
        "Grid should contain output. Content:\n{}",
        joined
    );
    assert!(
        joined.contains("file.txt"),
        "Grid should contain file listing. Content:\n{}",
        joined
    );
}

#[test]
fn test_e2e_block_manager_integration() {
    let mut mgr = BlockManager::new(10, 1000);

    let cmd_block_id = mgr.add_command_block("echo hello");
    if let Some(block) = mgr.get_block_mut(cmd_block_id) {
        block.content = vec![common::row_from_str("echo hello")];
    }

    let output_block_id = mgr.add_output_block();
    if let Some(block) = mgr.get_block_mut(output_block_id) {
        block.content = vec![common::row_from_str("hello")];
        block.title = Some("Output".to_string());
    }

    let blocks = mgr.blocks();
    assert_eq!(blocks.len(), 2, "Should have 2 blocks");
    assert_eq!(blocks[0].block_type, BlockType::Command);
    assert_eq!(blocks[1].block_type, BlockType::Output);

    let cmd_text: String = blocks[0].content[0].iter().map(|c| c.character).collect();
    assert!(cmd_text.contains("echo hello"), "Command block content: {}", cmd_text);

    let out_text: String = blocks[1].content[0].iter().map(|c| c.character).collect();
    assert!(out_text.contains("hello"), "Output block content: {}", out_text);
}

#[test]
fn test_e2e_grid_sync_to_block() {
    let mut grid = Grid::new(40, 5);
    grid.set(0, 0, Cell {
        character: 'o',
        foreground: None,
        background: None,
        attributes: terrust::terminal::Attributes::default(),
    });
    grid.set(1, 0, Cell {
        character: 'k',
        foreground: None,
        background: None,
        attributes: terrust::terminal::Attributes::default(),
    });

    let mut mgr = BlockManager::new(10, 1000);
    let block_id = mgr.add_output_block();

    if let Some(block) = mgr.get_block_mut(block_id) {
        let mut content: Vec<Vec<Cell>> = grid.cells.clone();
        content.retain(|row: &Vec<Cell>| row.iter().any(|cell| cell.character != ' '));
        block.content = content;
        block.title = Some("Terminal Output".to_string());
    }

    let blocks = mgr.blocks();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].block_type, BlockType::Output);

    if !blocks[0].content.is_empty() {
        let text: String = blocks[0].content[0]
            .iter()
            .map(|c| c.character)
            .collect();
        assert_eq!(text.trim(), "ok");
    }
}

#[test]
fn test_e2e_command_with_metadata() {
    let mut mgr = BlockManager::new(10, 1000);
    let cmd = "git status";
    let cwd = "/home/user/project".to_string();
    let shell = "/bin/zsh".to_string();

    let block_id = mgr.add_command_block(cmd);
    if let Some(block) = mgr.get_block_mut(block_id) {
        block.content = vec![common::row_from_str(cmd)];
        block.command = Some(cmd.to_string());
        block.cwd = Some(cwd.clone());
        block.shell = Some(shell.clone());
    }

    let output_id = mgr.add_output_block();
    if let Some(block) = mgr.get_block_mut(output_id) {
        block.content = vec![
            common::row_from_str("On branch main"),
            common::row_from_str("Your branch is up to date with 'origin/main'."),
            common::row_from_str(""),
            common::row_from_str("nothing to commit, working tree clean"),
        ];
        block.title = Some("Output".to_string());
    }

    let blocks = mgr.blocks();
    assert_eq!(blocks.len(), 2);

    let cmd_block = &blocks[0];
    assert_eq!(cmd_block.block_type, BlockType::Command);
    assert_eq!(cmd_block.command.as_deref(), Some("git status"));
    assert_eq!(cmd_block.cwd.as_deref(), Some("/home/user/project"));
    assert_eq!(cmd_block.shell.as_deref(), Some("/bin/zsh"));

    let out_block = &blocks[1];
    assert_eq!(out_block.block_type, BlockType::Output);
    assert!(out_block.content.len() >= 4);

    let out_text: String = out_block.content[3].iter().map(|c| c.character).collect();
    assert!(out_text.contains("nothing to commit"));
}
