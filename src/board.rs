/// Board module for Chinese Chess
use crate::pieces::{Color, PieceType};

/// Represents the game board
#[derive(Debug, Clone)]
pub struct Board {
    // 9x10 board representation using FEN characters
    // 0-8 for columns, 0-9 for rows
    pub squares: [[char; 9]; 10],
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    /// Create a new board with initial position
    pub fn new() -> Self {
        let mut squares = [['.'; 9]; 10];

        // Initialize starting position for Chinese Chess using FEN notation
        // Red pieces (bottom side, rows 0-2) - lowercase in FEN
        // Black pieces (top side, rows 7-9) - uppercase in FEN
        // FEN characters: k=King, a=Advisor, b=Elephant, n=Knight, r=Rook, c=Cannon, p=Pawn
        // Standard Chinese Chess starting position:
        // Row 0 (red back row): r n b a k a b n r
        // Row 2 (red cannons): . c . . . . . c .
        // Row 3 (red pawns): p . p . p . p . p
        // Row 6 (black pawns): P . P . P . P . P
        // Row 7 (black cannons): . C . . . . . C .
        // Row 9 (black back row): R N B A K A B N R

        // Red pieces (bottom side)
        // Row 0: Rooks, Knights, Elephants, Advisors, King, Advisors, Elephants, Knights, Rooks
        squares[0] = ['r', 'n', 'b', 'a', 'k', 'a', 'b', 'n', 'r'];

        // Row 2: Red cannons
        squares[2] = ['.', 'c', '.', '.', '.', '.', '.', 'c', '.'];

        // Row 3: Red pawns (every other column)
        squares[3] = ['p', '.', 'p', '.', 'p', '.', 'p', '.', 'p'];

        // Black pieces (top side)
        // Row 6: Black pawns (every other column)
        squares[6] = ['P', '.', 'P', '.', 'P', '.', 'P', '.', 'P'];

        // Row 7: Black cannons
        squares[7] = ['.', 'C', '.', '.', '.', '.', '.', 'C', '.'];

        // Row 9: Rooks, Knights, Elephants, Advisors, King, Advisors, Elephants, Knights, Rooks
        squares[9] = ['R', 'N', 'B', 'A', 'K', 'A', 'B', 'N', 'R'];

        Board { squares }
    }

    /// Create a board from a FEN string
    /// FEN format example: "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
    /// Only the board position part (before the first space) is parsed
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut squares = [['.'; 9]; 10];

        // Split FEN string to get only the board position part
        let board_part = fen.split_whitespace().next().unwrap_or(fen);
        let rows: Vec<&str> = board_part.split('/').collect();

        if rows.len() != 10 {
            return Err(format!(
                "Invalid FEN: expected 10 rows, got {}. FEN: {}",
                rows.len(),
                fen
            ));
        }

        for (row_idx, row_str) in rows.iter().enumerate() {
            let mut col_idx = 0;
            for c in row_str.chars() {
                if col_idx >= 9 {
                    return Err(format!("Row {} too long in FEN: {}", row_idx + 1, row_str));
                }

                if c.is_digit(10) {
                    // Digit represents empty squares
                    let empty_count = c.to_digit(10).unwrap() as usize;
                    for _ in 0..empty_count {
                        if col_idx >= 9 {
                            return Err(format!(
                                "Row {} exceeds 9 columns in FEN: {}",
                                row_idx + 1,
                                row_str
                            ));
                        }
                        squares[row_idx][col_idx] = '.';
                        col_idx += 1;
                    }
                } else {
                    // Piece character
                    squares[row_idx][col_idx] = c;
                    col_idx += 1;
                }
            }

            if col_idx != 9 {
                return Err(format!(
                    "Row {} has {} columns, expected 9 in FEN: {}",
                    row_idx + 1,
                    col_idx,
                    row_str
                ));
            }
        }

        Ok(Board { squares })
    }

    /// Convert the board to a FEN string
    /// Only includes the board position part
    pub fn to_fen(&self) -> String {
        let mut fen_parts = Vec::new();

        for row in 0..10 {
            let mut fen_row = String::new();
            let mut empty_count = 0;

            for col in 0..9 {
                let piece = self.squares[row][col];
                if piece == '.' {
                    empty_count += 1;
                } else {
                    if empty_count > 0 {
                        fen_row.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    fen_row.push(piece);
                }
            }

            if empty_count > 0 {
                fen_row.push_str(&empty_count.to_string());
            }

            fen_parts.push(fen_row);
        }

        fen_parts.join("/")
    }

    /// Clear the board (set all squares to empty)
    pub fn clear(&mut self) {
        for row in 0..10 {
            for col in 0..9 {
                self.squares[row][col] = '.';
            }
        }
    }

    /// Make a move on the board
    pub fn make_move(&mut self, from: (usize, usize), to: (usize, usize)) -> bool {
        let (from_col, from_row) = from;
        let (to_col, to_row) = to;

        // 验证坐标在棋盘范围内
        if from_col >= 9 || from_row >= 10 || to_col >= 9 || to_row >= 10 {
            return false;
        }

        // 检查起始位置是否有棋子
        if self.is_empty_at(from_col, from_row) {
            return false;
        }

        // 获取起始位置的棋子
        let moving_piece = self.get_fen(from_col, from_row);

        // 检查目标位置
        let target_piece = self.get_fen(to_col, to_row);

        if target_piece != '.' {
            // 目标位置有棋子
            // 检查是否为敌方棋子（通过大小写判断）
            let moving_is_lower = moving_piece.is_lowercase();
            let target_is_lower = target_piece.is_lowercase();

            // 相同颜色不能吃子
            if moving_is_lower == target_is_lower {
                return false;
            }

            // 检查是否为炮的特殊吃子规则（需要炮架）
            let is_cannon = moving_piece.to_ascii_lowercase() == 'c';
            if is_cannon {
                // 炮的吃子需要中间有一个炮架
                if !self.has_cannon_screen(from_col, from_row, to_col, to_row) {
                    return false;
                }
            }
        } else {
            // 目标位置为空
            // 如果是炮，移动到空位置时不能有炮架
            let is_cannon = moving_piece.to_ascii_lowercase() == 'c';
            if is_cannon {
                // 炮移动时空位不能有炮架
                if self.has_pieces_between(from_col, from_row, to_col, to_row) {
                    return false;
                }
            }
        }

        // 执行走法
        self.squares[to_row][to_col] = moving_piece;
        self.squares[from_row][from_col] = '.';

        true
    }

    /// 检查炮是否有炮架（用于吃子）
    pub fn has_cannon_screen(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
    ) -> bool {
        let dx = to_col as isize - from_col as isize;
        let dy = to_row as isize - from_row as isize;

        // 检查是否在同一行或同一列
        if dx != 0 && dy != 0 {
            return false; // 炮只能直线移动
        }

        let step_x = if dx == 0 { 0 } else { dx / dx.abs() };
        let step_y = if dy == 0 { 0 } else { dy / dy.abs() };

        let mut x = from_col as isize + step_x;
        let mut y = from_row as isize + step_y;
        let mut piece_count = 0;

        // 遍历中间位置
        while x != to_col as isize || y != to_row as isize {
            if self.squares[y as usize][x as usize] != '.' {
                piece_count += 1;
            }
            x += step_x;
            y += step_y;
        }

        // 炮吃子需要恰好有一个炮架
        piece_count == 1
    }

    /// 检查两个位置之间是否有棋子
    pub fn has_pieces_between(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
    ) -> bool {
        let dx = to_col as isize - from_col as isize;
        let dy = to_row as isize - from_row as isize;

        // 检查是否在同一行或同一列
        if dx != 0 && dy != 0 {
            return false; // 不在同一行或同一列
        }

        let step_x = if dx == 0 { 0 } else { dx / dx.abs() };
        let step_y = if dy == 0 { 0 } else { dy / dy.abs() };

        let mut x = from_col as isize + step_x;
        let mut y = from_row as isize + step_y;

        // 遍历中间位置
        while x != to_col as isize || y != to_row as isize {
            if self.squares[y as usize][x as usize] != '.' {
                return true;
            }
            x += step_x;
            y += step_y;
        }

        false
    }

    /// Get the FEN character at a specific position
    pub fn get_fen(&self, col: usize, row: usize) -> char {
        self.squares[row][col]
    }

    /// Set the FEN character at a specific position
    pub fn set_fen(&mut self, col: usize, row: usize, fen_char: char) {
        self.squares[row][col] = fen_char;
    }

    /// Check if a position contains a piece of specific color
    pub fn is_color_at(&self, col: usize, row: usize, color: Color) -> bool {
        let fen_char = self.get_fen(col, row);
        color.matches_fen(fen_char)
    }

    /// Get the piece type at a specific position
    pub fn get_piece_type(&self, col: usize, row: usize) -> Option<PieceType> {
        let fen_char = self.get_fen(col, row);
        PieceType::from_fen(fen_char)
    }

    /// Get the color at a specific position
    pub fn get_color_at(&self, col: usize, row: usize) -> Option<Color> {
        let fen_char = self.get_fen(col, row);
        Color::from_fen(fen_char)
    }

    /// Get the FEN character and color at a position
    pub fn get_fen_and_color(&self, col: usize, row: usize) -> (char, Option<Color>) {
        let fen_char = self.get_fen(col, row);
        let color = Color::from_fen(fen_char);
        (fen_char, color)
    }

    /// Check if a position is empty
    pub fn is_empty_at(&self, col: usize, row: usize) -> bool {
        self.get_fen(col, row) == '.'
    }

    /// Check if a position contains a piece (any color)
    pub fn has_piece_at(&self, col: usize, row: usize) -> bool {
        !self.is_empty_at(col, row)
    }

    /// Check if coordinates are within board bounds
    pub fn is_within_bounds(col: usize, row: usize) -> bool {
        col < 9 && row < 10
    }

    /// Create a copy of the board
    pub fn copy(&self) -> Board {
        let mut squares = [['.'; 9]; 10];

        for row in 0..10 {
            for col in 0..9 {
                squares[row][col] = self.squares[row][col];
            }
        }

        Board { squares }
    }

    /// Check if this board is equal to another board
    pub fn equals(&self, other: &Board) -> bool {
        for row in 0..10 {
            for col in 0..9 {
                if self.squares[row][col] != other.squares[row][col] {
                    return false;
                }
            }
        }
        true
    }

    /// Count the number of pieces on the board
    pub fn count_pieces(&self) -> usize {
        let mut count = 0;
        for row in 0..10 {
            for col in 0..9 {
                if self.squares[row][col] != '.' {
                    count += 1;
                }
            }
        }
        count
    }

    /// Count the number of pieces of a specific color
    pub fn count_color_pieces(&self, is_red: bool) -> usize {
        let mut count = 0;
        for row in 0..10 {
            for col in 0..9 {
                let fen_char = self.squares[row][col];
                if fen_char != '.' {
                    let is_lower = fen_char.is_lowercase();
                    if (is_red && is_lower) || (!is_red && !is_lower) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// Check if the board is empty
    pub fn is_empty(&self) -> bool {
        self.count_pieces() == 0
    }

    /// Get all positions that contain pieces
    pub fn get_all_piece_positions(&self) -> Vec<(usize, usize, char)> {
        let mut positions = Vec::new();
        for row in 0..10 {
            for col in 0..9 {
                let fen_char = self.squares[row][col];
                if fen_char != '.' {
                    positions.push((col, row, fen_char));
                }
            }
        }
        positions
    }

    /// Get positions of pieces of a specific color
    pub fn get_color_piece_positions(&self, is_red: bool) -> Vec<(usize, usize, char)> {
        let mut positions = Vec::new();
        for row in 0..10 {
            for col in 0..9 {
                let fen_char = self.squares[row][col];
                if fen_char != '.' {
                    let is_lower = fen_char.is_lowercase();
                    if (is_red && is_lower) || (!is_red && !is_lower) {
                        positions.push((col, row, fen_char));
                    }
                }
            }
        }
        positions
    }

    /// Check if a position is in the palace (九宫)
    pub fn is_in_palace(col: usize, row: usize, is_red: bool) -> bool {
        // Palace columns: 3, 4, 5 (0-indexed)
        if col < 3 || col > 5 {
            return false;
        }

        if is_red {
            // Red palace: rows 0, 1, 2
            row <= 2
        } else {
            // Black palace: rows 7, 8, 9
            row >= 7
        }
    }

    /// Check if a position is across the river (过河)
    pub fn is_across_river(row: usize, is_red: bool) -> bool {
        if is_red {
            // Red side: across river means row >= 5
            row >= 5
        } else {
            // Black side: across river means row <= 4
            row <= 4
        }
    }

    /// Get the distance between two positions
    pub fn distance(col1: usize, row1: usize, col2: usize, row2: usize) -> (isize, isize) {
        let dx = col2 as isize - col1 as isize;
        let dy = row2 as isize - row1 as isize;
        (dx, dy)
    }

    /// Get Manhattan distance between two positions
    pub fn manhattan_distance(col1: usize, row1: usize, col2: usize, row2: usize) -> usize {
        let dx = (col2 as isize - col1 as isize).abs() as usize;
        let dy = (row2 as isize - row1 as isize).abs() as usize;
        dx + dy
    }

    /// Get piece at position with type and color (通用方法)
    pub fn get_piece_at(&self, col: usize, row: usize) -> Option<(PieceType, Color)> {
        let fen_char = self.get_fen(col, row);
        if fen_char == '.' {
            return None;
        }

        let piece_type = PieceType::from_fen(fen_char)?;
        let color = Color::from_fen(fen_char)?;

        Some((piece_type, color))
    }

    /// Set piece at position (通用方法)
    pub fn set_piece_at(&mut self, col: usize, row: usize, piece_type: PieceType, color: Color) {
        let fen_char = match color {
            Color::Red => piece_type.to_fen_base(),
            Color::Black => piece_type.to_fen_base().to_ascii_uppercase(),
            Color::Any => '.', // Should not happen
        };
        self.set_fen(col, row, fen_char);
    }

    /// Remove piece at position (通用方法)
    pub fn remove_piece_at(&mut self, col: usize, row: usize) {
        self.set_fen(col, row, '.');
    }
}
