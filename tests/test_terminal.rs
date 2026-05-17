use terrust::terminal::{Cell, Grid, ScrollbackBuffer, Attributes, Color, ColorName};

fn cell_with_char(c: char) -> Cell {
    Cell { character: c, foreground: None, background: None, attributes: Attributes::default() }
}

fn make_row(chars: &str) -> Vec<Cell> {
    chars.chars().map(cell_with_char).collect()
}

#[test]
fn test_grid_creation_and_access() {
    let mut grid = Grid::new(10, 5);
    assert_eq!(grid.columns, 10);
    assert_eq!(grid.rows, 5);
    assert_eq!(grid.cells.len(), 5);

    grid.set(0, 0, cell_with_char('X'));
    assert_eq!(grid.get(0, 0).unwrap().character, 'X');
    assert!(grid.get(10, 0).is_none());
}

#[test]
fn test_grid_resize_preserves_content() {
    let mut grid = Grid::new(10, 5);
    grid.set(0, 0, cell_with_char('A'));
    grid.set(9, 4, cell_with_char('B'));

    grid.resize(20, 10);
    assert_eq!(grid.get(0, 0).unwrap().character, 'A');
    assert_eq!(grid.get(9, 4).unwrap().character, 'B');
    assert_eq!(grid.columns, 20);
    assert_eq!(grid.rows, 10);
}

#[test]
fn test_grid_scroll_up_moves_content() {
    let mut grid = Grid::new(5, 3);
    grid.set(0, 0, cell_with_char('1'));
    grid.set(0, 1, cell_with_char('2'));

    grid.scroll_up();
    assert_eq!(grid.get(0, 0).unwrap().character, '2');
    assert_eq!(grid.get(0, 1).unwrap().character, ' ');
}

#[test]
fn test_grid_clear_operations() {
    let mut grid = Grid::new(5, 3);
    for col in 0..5 {
        grid.set(col, 0, cell_with_char('X'));
    }
    grid.clear_row(0);
    assert_eq!(grid.get(0, 0).unwrap().character, ' ');

    grid.set(0, 1, cell_with_char('Y'));
    grid.set(1, 1, cell_with_char('Z'));
    grid.clear_to_end_of_line(1, 1);
    assert_eq!(grid.get(0, 1).unwrap().character, 'Y');
    assert_eq!(grid.get(1, 1).unwrap().character, ' ');
}

#[test]
fn test_grid_clear_entire() {
    let mut grid = Grid::new(3, 3);
    grid.set(0, 0, cell_with_char('A'));
    grid.set(1, 1, cell_with_char('B'));
    grid.set(2, 2, cell_with_char('C'));
    grid.clear();
    assert_eq!(grid.get(0, 0).unwrap().character, ' ');
    assert_eq!(grid.get(2, 2).unwrap().character, ' ');
}

#[test]
fn test_scrollback_buffer_push_and_retrieve() {
    let mut sb = ScrollbackBuffer::new(10, 5);
    assert!(sb.is_empty());

    sb.push_line(make_row("hello"));
    sb.push_line(make_row("world"));
    assert_eq!(sb.len(), 2);
    assert!(!sb.is_empty());
}

#[test]
fn test_scrollback_buffer_max_lines() {
    let mut sb = ScrollbackBuffer::new(3, 5);
    sb.push_line(make_row("a"));
    sb.push_line(make_row("b"));
    sb.push_line(make_row("c"));
    sb.push_line(make_row("d"));
    assert_eq!(sb.len(), 3);
}

#[test]
fn test_scrollback_buffer_iter_rev() {
    let mut sb = ScrollbackBuffer::new(10, 10);
    sb.push_line(make_row("first"));
    sb.push_line(make_row("second"));

    let rev: Vec<String> = sb.iter_rev()
        .map(|row| row.iter().map(|c| c.character).collect::<String>())
        .collect();
    assert_eq!(rev[0].trim_end(), "second");
    assert_eq!(rev[1].trim_end(), "first");
}

#[test]
fn test_scrollback_buffer_resize() {
    let mut sb = ScrollbackBuffer::new(10, 5);
    sb.push_line(make_row("hello"));
    sb.resize(10);
    let line = sb.get_line(0).unwrap();
    assert_eq!(line.len(), 10);
}

#[test]
fn test_cell_default_is_space() {
    let cell = Cell::default();
    assert_eq!(cell.character, ' ');
    assert!(cell.foreground.is_none());
    assert!(cell.background.is_none());
}

#[test]
fn test_attributes_conversion() {
    let attrs = Attributes { bold: true, italic: true, ..Attributes::default() };
    let modifier = attrs.to_ratatui_modifier();
    assert!(modifier.contains(ratatui::style::Modifier::BOLD));
    assert!(modifier.contains(ratatui::style::Modifier::ITALIC));
}

#[test]
fn test_color_from_hex() {
    let color = Color::from_hex("#ff0000").unwrap();
    assert_eq!(color, Color::Rgb(255, 0, 0));

    let color = Color::from_hex("#f00").unwrap();
    assert_eq!(color, Color::Rgb(255, 0, 0));

    assert!(Color::from_hex("invalid").is_none());
}

#[test]
fn test_color_from_ansi_code() {
    let color = Color::from_ansi_code(0);
    assert_eq!(color, Color::Index(0));

    let color = Color::from_ansi_code(196);
    assert!(matches!(color, Color::Rgb(_, _, _)));
}

#[test]
fn test_color_to_ratatui() {
    let rgb = Color::Rgb(100, 150, 200);
    let ratatui_color = rgb.to_ratatui_color();
    assert_eq!(ratatui_color, ratatui::style::Color::Rgb(100, 150, 200));

    let named = Color::Named(ColorName::Red);
    assert_eq!(named.to_ratatui_color(), ratatui::style::Color::Red);
}

#[test]
fn test_shell_config_default() {
    let shell = terrust::terminal::ShellConfig::default();
    let expected = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    assert_eq!(shell.shell, expected);
}
