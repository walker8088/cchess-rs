/// Board module for Chinese Chess
use crate::move_notation::{ChineseLocale, MoveFormat, MoveNotation};
use crate::pieces::{PieceType, Side};

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
    /// Create a new empty board (all squares are empty)
    pub fn new() -> Self {
        let squares = [['.'; 9]; 10];
        Board { squares }
    }

    /// Set up the board with the standard Chinese Chess initial position
    pub fn initial_position(&mut self) {
        // Black pieces (黑方, top side in board visualization)
        // Row 0: Rooks, Knights, Elephants, Advisors, King, Advisors, Elephants, Knights, Rooks
        self.squares[0] = ['r', 'n', 'b', 'a', 'k', 'a', 'b', 'n', 'r'];

        // Row 2: Black cannons (黑方砲)
        self.squares[2] = ['.', 'c', '.', '.', '.', '.', '.', 'c', '.'];

        // Row 3: Black pawns (黑方卒)
        self.squares[3] = ['p', '.', 'p', '.', 'p', '.', 'p', '.', 'p'];

        // Red pieces (红方, bottom side in board visualization)
        // Row 6: Red pawns (红方兵)
        self.squares[6] = ['P', '.', 'P', '.', 'P', '.', 'P', '.', 'P'];

        // Row 7: Red cannons (红方炮)
        self.squares[7] = ['.', 'C', '.', '.', '.', '.', '.', 'C', '.'];

        // Row 9: Rooks, Knights, Elephants, Advisors, King, Advisors, Elephants, Knights, Rooks
        self.squares[9] = ['R', 'N', 'B', 'A', 'K', 'A', 'B', 'N', 'R'];
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

                if c.is_ascii_digit() {
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
    /// Validates piece-specific movement rules before executing
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
        let piece_type = match PieceType::from_fen(moving_piece) {
            Some(pt) => pt,
            None => return false,
        };
        let side = match Side::from_fen(moving_piece) {
            Some(s) => s,
            None => return false,
        };
        let is_red = side == Side::Red;

        // 检查目标位置 - 不能吃己方棋子
        let target_piece = self.get_fen(to_col, to_row);
        if target_piece != '.' {
            let target_side = match Side::from_fen(target_piece) {
                Some(s) => s,
                None => return false,
            };
            if side == target_side {
                return false;
            }
        }

        // 根据棋子类型验证走法
        let valid = match piece_type {
            PieceType::King => self.validate_king_move(from_col, from_row, to_col, to_row, is_red),
            PieceType::Advisor => {
                self.validate_advisor_move(from_col, from_row, to_col, to_row, is_red)
            }
            PieceType::Elephant => {
                self.validate_elephant_move(from_col, from_row, to_col, to_row, is_red)
            }
            PieceType::Knight => self.validate_knight_move(from_col, from_row, to_col, to_row),
            PieceType::Rook => self.validate_rook_move(from_col, from_row, to_col, to_row),
            PieceType::Cannon => {
                self.validate_cannon_move(from_col, from_row, to_col, to_row, target_piece != '.')
            }
            PieceType::Pawn => self.validate_pawn_move(from_col, from_row, to_col, to_row, is_red),
        };

        if !valid {
            return false;
        }

        // 执行走法（临时）
        self.squares[to_row][to_col] = moving_piece;
        self.squares[from_row][from_col] = '.';

        // 检查飞将规则（将帅不能照面）
        if self.kings_are_facing() {
            // 撤销走法
            self.squares[from_row][from_col] = moving_piece;
            self.squares[to_row][to_col] = target_piece;
            return false;
        }

        true
    }

    /// Validate King (将/帅) movement
    /// - Must stay within palace
    /// - Moves exactly one step orthogonally
    fn validate_king_move(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
        is_red: bool,
    ) -> bool {
        // Must stay within palace columns (3,4,5)
        if !(3..=5).contains(&to_col) {
            return false;
        }
        // Red palace: rows 0-2, Black palace: rows 7-9
        if is_red {
            if to_row > 2 {
                return false;
            }
        } else {
            if to_row < 7 {
                return false;
            }
        }
        // Must move exactly one step
        let dx = (to_col as isize - from_col as isize).abs();
        let dy = (to_row as isize - from_row as isize).abs();
        dx + dy == 1
    }

    /// Validate Advisor (士/仕) movement
    /// - Must stay within palace
    /// - Moves exactly one step diagonally
    fn validate_advisor_move(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
        is_red: bool,
    ) -> bool {
        // Must stay within palace columns (3,4,5)
        if !(3..=5).contains(&to_col) {
            return false;
        }
        // Must stay within palace rows
        if is_red {
            if to_row > 2 {
                return false;
            }
        } else {
            if to_row < 7 {
                return false;
            }
        }
        // Must move exactly one step diagonally
        let dx = (to_col as isize - from_col as isize).abs();
        let dy = (to_row as isize - from_row as isize).abs();
        dx == 1 && dy == 1
    }

    /// Validate Elephant (象/相) movement
    /// - Moves in 田 pattern (2 steps diagonally)
    /// - Cannot cross river
    /// - Can be blocked (蹩脚) at the center of 田
    fn validate_elephant_move(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
        is_red: bool,
    ) -> bool {
        let dx = to_col as isize - from_col as isize;
        let dy = to_row as isize - from_row as isize;

        // Must move exactly 2 steps diagonally (田)
        if dx.abs() != 2 || dy.abs() != 2 {
            return false;
        }

        // Cannot cross river: Red must stay in rows 0-4, Black must stay in rows 5-9
        if is_red && to_row > 4 {
            return false;
        }
        if !is_red && to_row < 5 {
            return false;
        }

        // Check for blocking piece at the center of 田
        let block_col = from_col as isize + dx / 2;
        let block_row = from_row as isize + dy / 2;
        if self.squares[block_row as usize][block_col as usize] != '.' {
            return false;
        }

        true
    }

    /// Validate Knight (马) movement
    /// - Moves in 日 pattern (one step orthogonally, then one step diagonally)
    /// - Can be blocked (蹩脚) at the first step
    fn validate_knight_move(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
    ) -> bool {
        let dx = to_col as isize - from_col as isize;
        let dy = to_row as isize - from_row as isize;
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();

        // Must move in 日 pattern: 1x2 or 2x1
        if !((abs_dx == 1 && abs_dy == 2) || (abs_dx == 2 && abs_dy == 1)) {
            return false;
        }

        // Check for blocking piece
        if abs_dx == 2 {
            // Horizontal 日, block is at adjacent horizontal position
            let block_col = from_col as isize + dx / 2;
            let block_row = from_row as isize;
            if self.squares[block_row as usize][block_col as usize] != '.' {
                return false;
            }
        } else {
            // Vertical 日, block is at adjacent vertical position
            let block_col = from_col as isize;
            let block_row = from_row as isize + dy / 2;
            if self.squares[block_row as usize][block_col as usize] != '.' {
                return false;
            }
        }

        true
    }

    /// Validate Rook (车) movement
    /// - Moves in straight lines (horizontal or vertical)
    /// - Cannot jump over pieces
    fn validate_rook_move(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
    ) -> bool {
        let dx = to_col as isize - from_col as isize;
        let dy = to_row as isize - from_row as isize;

        // Must move in straight line
        if dx != 0 && dy != 0 {
            return false;
        }

        // Cannot jump over pieces
        !self.has_pieces_between(from_col, from_row, to_col, to_row)
    }

    /// Validate Cannon (炮) movement
    /// - Moves in straight lines like Rook
    /// - Capturing requires exactly one piece between (炮架)
    fn validate_cannon_move(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
        is_capturing: bool,
    ) -> bool {
        let dx = to_col as isize - from_col as isize;
        let dy = to_row as isize - from_row as isize;

        // Must move in straight line
        if dx != 0 && dy != 0 {
            return false;
        }

        if is_capturing {
            // Capturing requires exactly one piece between
            self.has_cannon_screen(from_col, from_row, to_col, to_row)
        } else {
            // Moving without capturing: no pieces between
            !self.has_pieces_between(from_col, from_row, to_col, to_row)
        }
    }

    /// Validate Pawn (兵/卒) movement
    /// - Before crossing river: can only move forward one step
    /// - After crossing river: can move forward, left, or right one step
    fn validate_pawn_move(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
        is_red: bool,
    ) -> bool {
        let dx = to_col as isize - from_col as isize;
        let dy = to_row as isize - from_row as isize;
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();

        // Must move exactly one step
        if abs_dx + abs_dy != 1 {
            return false;
        }

        let crossed_river = Self::is_across_river(from_row, is_red);

        if is_red {
            // Red pawns move upward (increasing row)
            if !crossed_river {
                // Before river: only forward
                dy == 1 && dx == 0
            } else {
                // After river: forward, left, or right (but not backward)
                dy >= 0
            }
        } else {
            // Black pawns move downward (decreasing row)
            if !crossed_river {
                // Before river: only forward
                dy == -1 && dx == 0
            } else {
                // After river: forward, left, or right (but not backward)
                dy <= 0
            }
        }
    }

    /// Check if kings are facing each other (飞将规则)
    /// Kings on the same column with no pieces between them is illegal
    fn kings_are_facing(&self) -> bool {
        let mut red_king_pos: Option<(usize, usize)> = None;
        let mut black_king_pos: Option<(usize, usize)> = None;

        // Find both kings
        for row in 0..10 {
            for col in 0..9 {
                let fen = self.squares[row][col];
                if fen == 'k' {
                    red_king_pos = Some((col, row));
                } else if fen == 'K' {
                    black_king_pos = Some((col, row));
                }
            }
        }

        // Both kings must exist
        let (red_col, red_row) = match red_king_pos {
            Some(p) => p,
            None => return false,
        };
        let (black_col, black_row) = match black_king_pos {
            Some(p) => p,
            None => return false,
        };

        // Must be on the same column
        if red_col != black_col {
            return false;
        }

        // Check if there are any pieces between them
        let min_row = red_row.min(black_row) + 1;
        let max_row = red_row.max(black_row);

        for row in min_row..max_row {
            if self.squares[row][red_col] != '.' {
                return false; // There's a piece between, so not facing
            }
        }

        true // Kings are facing with no pieces between
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

    /// Count the number of pieces between two positions (exclusive)
    /// Returns 0 if not on the same row/column
    pub fn count_pieces_between(
        &self,
        from_col: usize,
        from_row: usize,
        to_col: usize,
        to_row: usize,
    ) -> usize {
        let dx = to_col as isize - from_col as isize;
        let dy = to_row as isize - from_row as isize;

        // Must be on same row or column
        if dx != 0 && dy != 0 {
            return 0;
        }

        let step_x = if dx == 0 { 0 } else { dx / dx.abs() };
        let step_y = if dy == 0 { 0 } else { dy / dy.abs() };

        let mut x = from_col as isize + step_x;
        let mut y = from_row as isize + step_y;
        let mut count = 0;

        while x != to_col as isize || y != to_row as isize {
            if self.squares[y as usize][x as usize] != '.' {
                count += 1;
            }
            x += step_x;
            y += step_y;
        }

        count
    }

    /// Get the FEN character at a specific position
    pub fn get_fen(&self, col: usize, row: usize) -> char {
        self.squares[row][col]
    }

    /// Set the FEN character at a specific position
    pub fn set_fen(&mut self, col: usize, row: usize, fen_char: char) {
        self.squares[row][col] = fen_char;
    }

    /// Check if a position contains a piece of specific side
    pub fn is_color_at(&self, col: usize, row: usize, side: Side) -> bool {
        let fen_char = self.get_fen(col, row);
        side.matches_fen(fen_char)
    }

    /// Get the piece type at a specific position
    pub fn get_piece_type(&self, col: usize, row: usize) -> Option<PieceType> {
        let fen_char = self.get_fen(col, row);
        PieceType::from_fen(fen_char)
    }

    /// Get the side at a specific position
    pub fn get_color_at(&self, col: usize, row: usize) -> Option<Side> {
        let fen_char = self.get_fen(col, row);
        Side::from_fen(fen_char)
    }

    /// Get the FEN character and side at a position
    pub fn get_fen_and_color(&self, col: usize, row: usize) -> (char, Option<Side>) {
        let fen_char = self.get_fen(col, row);
        let side = Side::from_fen(fen_char);
        (fen_char, side)
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

        for (row, row_squares) in squares.iter_mut().enumerate().take(10) {
            for (col, square) in row_squares.iter_mut().enumerate().take(9) {
                *square = self.squares[row][col];
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
        if !(3..=5).contains(&col) {
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
        let dx = (col2 as isize - col1 as isize).unsigned_abs();
        let dy = (row2 as isize - row1 as isize).unsigned_abs();
        dx + dy
    }

    /// Get piece at position with type and side (通用方法)
    pub fn get_piece_at(&self, col: usize, row: usize) -> Option<(PieceType, Side)> {
        let fen_char = self.get_fen(col, row);
        if fen_char == '.' {
            return None;
        }

        let piece_type = PieceType::from_fen(fen_char)?;
        let side = Side::from_fen(fen_char)?;

        Some((piece_type, side))
    }

    /// Set piece at position (通用方法)
    pub fn set_piece_at(&mut self, col: usize, row: usize, piece_type: PieceType, side: Side) {
        let fen_char = match side {
            Side::Black => piece_type.to_fen_base(),
            Side::Red => piece_type.to_fen_base().to_ascii_uppercase(),
            Side::Any => '.', // Should not happen
        };
        self.set_fen(col, row, fen_char);
    }

    /// Remove piece at position (通用方法)
    pub fn remove_piece_at(&mut self, col: usize, row: usize) {
        self.set_fen(col, row, '.');
    }

    /// Pop piece at position: get and remove the piece, returning its info
    /// Returns (fen_char, piece_type, side) if a piece was present, None if empty
    pub fn pop_piece_at(&mut self, col: usize, row: usize) -> Option<(char, PieceType, Side)> {
        let fen_char = self.get_fen(col, row);
        if fen_char == '.' {
            return None;
        }
        let piece_type = PieceType::from_fen(fen_char)?;
        let side = Side::from_fen(fen_char)?;
        self.remove_piece_at(col, row);
        Some((fen_char, piece_type, side))
    }

    /// 生成走法文本表示
    ///
    /// # 参数
    /// - `from`: 起始位置 (列, 行)
    /// - `to`: 目标位置 (列, 行)
    /// - `format`: 走法格式
    /// - `traditional`: 是否使用繁体中文（仅当format为Chinese时有效）
    ///
    /// # 返回值
    /// - `Ok(String)`: 走法文本
    /// - `Err(String)`: 错误信息
    pub fn move_text(
        &self,
        from: (usize, usize),
        to: (usize, usize),
        format: MoveFormat,
        traditional: bool,
    ) -> Result<String, String> {
        // 创建走法中间表示
        let notation = MoveNotation::from_board_move(self, from, to)?;

        // 根据格式生成文本
        match format {
            MoveFormat::Chinese => {
                let locale = if traditional {
                    ChineseLocale::Traditional
                } else {
                    ChineseLocale::Simplified
                };
                Ok(notation.to_chinese(locale))
            }
            MoveFormat::WXF => Ok(notation.to_wxf()),
            MoveFormat::ICCS => Ok(notation.to_iccs(from, to)),
        }
    }

    /// 获取走法中间表示
    pub fn move_notation(
        &self,
        from: (usize, usize),
        to: (usize, usize),
    ) -> Result<MoveNotation, String> {
        MoveNotation::from_board_move(self, from, to)
    }

    /// 翻转棋盘视角（用于测试）
    pub fn flip_perspective(&self) -> Board {
        let mut flipped = Board::new();

        for row in 0..10 {
            for col in 0..9 {
                let flipped_col = 8 - col;
                let flipped_row = 9 - row;
                flipped.squares[flipped_row][flipped_col] = self.squares[row][col];
            }
        }

        flipped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::move_notation::{Direction, MoveFormat};

    #[test]
    fn test_move_text_chinese() {
        let mut board = Board::new();
        board.initial_position();

        // 测试红方车九进一（简体中文）
        let result = board.move_text((0, 0), (0, 1), MoveFormat::Chinese, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("红方车九进一（简体）: {}", text);
        assert_eq!(text, "车九进一");

        // 测试红方车九进一（繁体中文）
        let result = board.move_text((0, 0), (0, 1), MoveFormat::Chinese, true);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("红方车九进一（繁体）: {}", text);
        assert_eq!(text, "車九進一");

        // 测试红方炮二平五
        let result = board.move_text((7, 2), (4, 2), MoveFormat::Chinese, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("红方炮二平五: {}", text);
        assert_eq!(text, "炮二平五");

        // 测试红方炮二平五（繁体）
        let result = board.move_text((7, 2), (4, 2), MoveFormat::Chinese, true);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("红方炮二平五（繁体）: {}", text);
        assert_eq!(text, "砲二平五");
    }

    #[test]
    fn test_move_text_black() {
        let mut board = Board::new();
        board.initial_position();

        // 测试黑方车9进1（注意：黑方使用全角数字）
        let result = board.move_text((0, 9), (0, 8), MoveFormat::Chinese, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("黑方车9进1: {}", text);
        // 注意：黑方数字应该是全角的，翻转后车是一路
        assert_eq!(text, "车９进１");

        // 测试黑方炮２平５
        let result = board.move_text((7, 7), (4, 7), MoveFormat::Chinese, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("黑方炮２平５: {}", text);
        assert_eq!(text, "炮２平５");
    }

    #[test]
    fn test_move_text_wxf() {
        let mut board = Board::new();
        board.initial_position();

        // 测试WXF格式：红方车九进一
        let result = board.move_text((0, 0), (0, 1), MoveFormat::WXF, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("WXF格式（红车）: {}", text);
        assert_eq!(text, "R9+1");

        // 测试WXF格式：黑方车9进1
        let result = board.move_text((0, 9), (0, 8), MoveFormat::WXF, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("WXF格式（黑车）: {}", text);

        // 测试WXF格式：红方炮二平五
        let result = board.move_text((7, 2), (4, 2), MoveFormat::WXF, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("WXF格式（红炮）: {}", text);
        assert_eq!(text, "C2.5");
    }

    #[test]
    fn test_move_text_iccs() {
        let mut board = Board::new();
        board.initial_position();

        // 测试ICCS格式（车九进一）
        let result = board.move_text((0, 0), (0, 1), MoveFormat::ICCS, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("ICCS格式: {}", text);
        assert_eq!(text, "a0a1");

        // 测试ICCS格式（炮二进一）
        let result = board.move_text((7, 2), (7, 3), MoveFormat::ICCS, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("ICCS格式: {}", text);
        assert_eq!(text, "h2h3");
    }

    #[test]
    fn test_move_text_invalid() {
        let board = Board::new();

        // 测试无效坐标
        let result = board.move_text((10, 10), (0, 0), MoveFormat::Chinese, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("超出棋盘范围"));

        // 测试空位置
        let result = board.move_text((4, 4), (4, 5), MoveFormat::Chinese, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("没有棋子"));
    }

    #[test]
    fn test_move_notation_method() {
        let mut board = Board::new();
        board.initial_position();

        // 测试move_notation方法
        let result = board.move_notation((0, 0), (0, 1));
        assert!(result.is_ok());
        let notation = result.unwrap();

        assert_eq!(notation.piece_type, PieceType::Rook);
        assert_eq!(notation.piece_color, Side::Black);
        assert_eq!(notation.column, 9);
        assert_eq!(notation.direction, Direction::Forward);
        assert_eq!(notation.distance, 1);
    }

    #[test]
    fn test_flip_perspective() {
        let mut board = Board::new();
        board.initial_position();

        // 测试棋盘翻转
        let flipped = board.flip_perspective();

        // 检查一些关键位置
        // 红方右下角的车应该翻转到黑方左上角
        assert_eq!(board.get_fen(0, 0), 'r'); // 红方车
        assert_eq!(flipped.get_fen(8, 9), 'r'); // 翻转后

        // 黑方左上角的车应该翻转到红方右下角
        assert_eq!(board.get_fen(0, 9), 'R'); // 黑方车
        assert_eq!(flipped.get_fen(8, 0), 'R'); // 翻转后

        // 中心位置测试
        assert_eq!(board.get_fen(4, 4), '.'); // 中心空位
        assert_eq!(flipped.get_fen(4, 5), '.'); // 翻转后中心空位
    }
}
