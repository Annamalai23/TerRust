//! Terminal buffer structures (grid and scrollback)

use super::cell::Cell;

/// Terminal grid representing the display buffer
#[derive(Debug, Clone)]
pub struct Grid {
    /// Number of columns
    pub columns: u16,
    /// Number of rows
    pub rows: u16,
    /// The grid cells
    pub cells: Vec<Vec<Cell>>,
}

impl Grid {
    /// Create a new grid with the specified dimensions
    pub fn new(columns: u16, rows: u16) -> Self {
        Self {
            columns,
            rows,
            cells: vec![vec![Cell::default(); columns as usize]; rows as usize],
        }
    }

    /// Resize the grid to new dimensions
    pub fn resize(&mut self, columns: u16, rows: u16) {
        let mut new_cells = vec![vec![Cell::default(); columns as usize]; rows as usize];

        let copy_rows = std::cmp::min(self.rows as usize, rows as usize);
        let copy_cols = std::cmp::min(self.columns as usize, columns as usize);

        for row in 0..copy_rows {
            for col in 0..copy_cols {
                new_cells[row][col] = self.cells[row][col].clone();
            }
        }

        self.columns = columns;
        self.rows = rows;
        self.cells = new_cells;
    }

    /// Get a cell at the specified position
    pub fn get(&self, column: u16, row: u16) -> Option<&Cell> {
        if column < self.columns && row < self.rows {
            Some(&self.cells[row as usize][column as usize])
        } else {
            None
        }
    }

    /// Set a cell at the specified position
    pub fn set(&mut self, column: u16, row: u16, cell: Cell) {
        if column < self.columns && row < self.rows {
            self.cells[row as usize][column as usize] = cell;
        }
    }

    /// Scroll the grid up by one row
    pub fn scroll_up(&mut self) {
        if self.rows > 1 {
            self.cells.remove(0);
            self.cells.push(vec![Cell::default(); self.columns as usize]);
        }
    }

    /// Clear the entire grid
    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                *cell = Cell::default();
            }
        }
    }

    /// Clear a specific row
    pub fn clear_row(&mut self, row: u16) {
        if row < self.rows {
            for cell in &mut self.cells[row as usize] {
                *cell = Cell::default();
            }
        }
    }

    /// Clear from cursor to end of line
    pub fn clear_to_end_of_line(&mut self, column: u16, row: u16) {
        if row < self.rows {
            let start = column as usize;
            for cell in &mut self.cells[row as usize][start..] {
                *cell = Cell::default();
            }
        }
    }

    /// Clear from cursor to beginning of line
    pub fn clear_to_start_of_line(&mut self, column: u16, row: u16) {
        if row < self.rows {
            for cell in &mut self.cells[row as usize][..=column as usize] {
                *cell = Cell::default();
            }
        }
    }

    /// Clear the entire line containing the cursor
    pub fn clear_line(&mut self, row: u16) {
        self.clear_row(row);
    }
}

/// Scrollback buffer for storing history
#[derive(Debug, Clone)]
pub struct ScrollbackBuffer {
    /// Maximum number of lines to keep
    max_lines: usize,
    /// The buffer lines (oldest first)
    lines: Vec<Vec<Cell>>,
    /// Current column count
    columns: u16,
}

impl ScrollbackBuffer {
    /// Create a new scrollback buffer
    pub fn new(max_lines: usize, columns: u16) -> Self {
        Self {
            max_lines,
            lines: Vec::with_capacity(max_lines),
            columns,
        }
    }

    /// Push a new line into the buffer
    pub fn push_line(&mut self, line: Vec<Cell>) {
        if line.len() > self.columns as usize {
            // Truncate
            let mut truncated = line;
            truncated.truncate(self.columns as usize);
            if self.lines.len() >= self.max_lines {
                self.lines.remove(0);
            }
            self.lines.push(truncated);
        } else if line.len() < self.columns as usize {
            // Extend
            let mut extended = line;
            extended.resize(self.columns as usize, Cell::default());
            if self.lines.len() >= self.max_lines {
                self.lines.remove(0);
            }
            self.lines.push(extended);
        } else {
            if self.lines.len() >= self.max_lines {
                self.lines.remove(0);
            }
            self.lines.push(line);
        }
    }

    /// Get the number of lines in the buffer
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Resize all lines to match the current column count
    pub fn resize(&mut self, columns: u16) {
        self.columns = columns;
        for line in &mut self.lines {
            if line.len() < columns as usize {
                line.resize(columns as usize, Cell::default());
            } else if line.len() > columns as usize {
                line.truncate(columns as usize);
            }
        }
    }

    /// Get a line from the buffer
    pub fn get_line(&self, index: usize) -> Option<&Vec<Cell>> {
        self.lines.get(index)
    }

    /// Get lines in reverse order (newest first)
    pub fn iter_rev(&self) -> impl Iterator<Item = &Vec<Cell>> {
        self.lines.iter().rev()
    }
}
