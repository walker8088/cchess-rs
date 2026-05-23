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
        // Red pieces (红方, bottom side in board visualization)
        // Row 0: Rooks, Knights, Elephants, Advisors, King, Advisors, Elephants, Knights, Rooks
        self.squares[0] = ['R', 'N', 'B', 'A', 'K', 'A', 'B', 'N', 'R'];

        // Row 2: Red cannons (红方炮)
        self.squares[2] = ['.', 'C', '.', '.', '.', '.', '.', 'C', '.'];

        // Row 3: Red pawns (红方兵)
        self.squares[3] = ['P', '.', 'P', '.', 'P', '.', 'P', '.', 'P'];

        // Black pieces (黑方, top side in board visualization)
        // Row 6: Black pawns (黑方卒)
        self.squares[6] = ['p', '.', 'p', '.', 'p', '.', 'p', '.', 'p'];

        // Row 7: Black cannons (黑方砲)
        self.squares[7] = ['.', 'c', '.', '.', '.', '.', '.', 'c', '.'];

        // Row 9: Rooks, Knights, Elephants, Advisors, King, Advisors, Elephants, Knights, Rooks
        self.squares[9] = ['r', 'n', 'b', 'a', 'k', 'a', 'b', 'n', 'r'];
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
    /// Standard Xiangqi FEN: Black on top (row 9), Red on bottom (row 0)
    /// Only includes the board position part
    pub fn to_fen(&self) -> String {
        let mut fen_parts = Vec::new();

        // Standard FEN iterates from Black's back rank (row 9) to Red's (row 0)
        for row in (0..10).rev() {
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
        // Red (红方) palace: rows 0-2 (bottom)
        // Black (黑方) palace: rows 7-9 (top)
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
            // Red pawns move downward (increasing row, towards Black's side)
            if !crossed_river {
                // Before river: only forward
                dy == 1 && dx == 0
            } else {
                // After river: forward, left, or right (but not backward)
                dy >= 0
            }
        } else {
            // Black pawns move upward (decreasing row, towards Red's side)
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

    /// Mirror the board horizontally (left-right flip)
    pub fn mirror(&self) -> Board {
        let mut squares = [['.'; 9]; 10];
        for row in 0..10 {
            for col in 0..9 {
                squares[row][col] = self.squares[row][8 - col];
            }
        }
        Board { squares }
    }

    /// Flip the board vertically (up-down flip) + horizontal mirror
    /// This transforms from one side's perspective to the other
    pub fn flip(&self) -> Board {
        let mut squares = [['.'; 9]; 10];
        for row in 0..10 {
            for col in 0..9 {
                squares[row][col] = self.squares[9 - row][8 - col];
            }
        }
        Board { squares }
    }

    /// Swap piece colors (uppercase <-> lowercase)
    pub fn swap_colors(&self) -> Board {
        let mut squares = [['.'; 9]; 10];
        for row in 0..10 {
            for col in 0..9 {
                let c = self.squares[row][col];
                squares[row][col] = if c == '.' {
                    '.'
                } else if c.is_uppercase() {
                    c.to_ascii_lowercase()
                } else {
                    c.to_ascii_uppercase()
                };
            }
        }
        Board { squares }
    }

    /// Check if the board is horizontally symmetric
    pub fn is_mirror(&self) -> bool {
        for y in 0..10 {
            for x in 0..5 {
                if self.squares[y][x] != self.squares[y][8 - x] {
                    return false;
                }
            }
        }
        true
    }

    /// Find king position for given side. Red king='k' (lowercase), Black king='K' (uppercase)
    /// Note: In this codebase, Red=lowercase (rows 0-2), Black=uppercase (rows 7-9)
    pub fn find_king(&self, is_red: bool) -> Option<(usize, usize)> {
        let king_char = if is_red { 'k' } else { 'K' };
        let (min_row, max_row) = if is_red { (0, 2) } else { (7, 9) };
        for row in min_row..=max_row {
            for col in 3..=5 {
                if self.squares[row][col] == king_char {
                    return Some((col, row));
                }
            }
        }
        None
    }

    /// Find king position by side enum
    pub fn get_king_pos(&self, side: Side) -> Option<(usize, usize)> {
        match side {
            Side::Red => self.find_king(true),
            Side::Black => self.find_king(false),
            _ => None,
        }
    }

    /// Get occupied color at position: Some(Side) or None
    pub fn occupied(&self, col: usize, row: usize) -> Option<Side> {
        self.get_color_at(col, row)
    }

    /// Check if a move is valid (basic rules check)
    pub fn is_valid_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let (from_col, from_row) = from;
        let (to_col, to_row) = to;

        if from_col >= 9 || from_row >= 10 || to_col >= 9 || to_row >= 10 {
            return false;
        }

        if self.is_empty_at(from_col, from_row) {
            return false;
        }

        let moving_piece = self.get_fen(from_col, from_row);
        let target_piece = self.get_fen(to_col, to_row);

        if target_piece != '.' {
            let moving_side = Side::from_fen(moving_piece);
            let target_side = Side::from_fen(target_piece);
            if moving_side == target_side {
                return false;
            }
        }

        // Test the move on a copy (without fly-king check for basic validation)
        let mut test_board = self.copy();
        test_board.make_move(from, to)
    }

    /// Check if a move would result in check (将军)
    pub fn is_checking_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let (from_col, from_row) = from;
        let (to_col, to_row) = to;

        if from_col >= 9 || from_row >= 10 || to_col >= 9 || to_row >= 10 {
            return false;
        }

        let moving_piece = self.get_fen(from_col, from_row);
        let is_red = Side::from_fen(moving_piece) == Some(Side::Red);
        let opponent_side = if is_red { Side::Black } else { Side::Red };

        // Simulate the move
        let mut test_board = self.copy();
        test_board.squares[to_row][to_col] = moving_piece;
        test_board.squares[from_row][from_col] = '.';

        // Check if opponent king is under attack
        if let Some((kx, ky)) = test_board.get_king_pos(opponent_side) {
            test_board.is_under_attack(kx, ky, is_red)
        } else {
            false
        }
    }

    /// Check if a position is under attack by a given side
    fn is_under_attack(&self, col: usize, row: usize, by_red: bool) -> bool {
        // Check rook/cannon/king lines
        for &dx in &[-1isize, 0, 1] {
            for &dy in &[-1isize, 0, 1] {
                if (dx == 0) == (dy == 0) {
                    continue;
                }
                let mut screen_count = 0usize;
                let mut x = col as isize + dx;
                let mut y = row as isize + dy;
                while x >= 0 && x < 9 && y >= 0 && y < 10 {
                    let c = self.squares[y as usize][x as usize];
                    if c != '.' {
                        if screen_count == 0 {
                            let c_side = Side::from_fen(c);
                            let c_is_red = c_side == Some(Side::Red);
                            if c_is_red == by_red {
                                let pt = PieceType::from_fen(c);
                                if pt == Some(PieceType::Rook) || pt == Some(PieceType::King) {
                                    return true;
                                }
                                if pt == Some(PieceType::Cannon) && screen_count == 1 {
                                    return true;
                                }
                            }
                            screen_count += 1;
                        } else if screen_count == 1 {
                            let c_side = Side::from_fen(c);
                            let c_is_red = c_side == Some(Side::Red);
                            if c_is_red == by_red
                                && PieceType::from_fen(c) == Some(PieceType::Cannon)
                            {
                                return true;
                            }
                            break;
                        }
                    }
                    x += dx;
                    y += dy;
                }
            }
        }

        // Check knight moves
        let knight_moves = [
            ((1, 2), (0, 1)),
            ((1, -2), (0, -1)),
            ((-1, 2), (0, 1)),
            ((-1, -2), (0, -1)),
            ((2, 1), (1, 0)),
            ((2, -1), (1, 0)),
            ((-2, 1), (-1, 0)),
            ((-2, -1), (-1, 0)),
        ];
        for ((dx, dy), (bx, by)) in &knight_moves {
            let nx = col as isize + dx;
            let ny = row as isize + dy;
            if nx >= 0 && nx < 9 && ny >= 0 && ny < 10 {
                let c = self.squares[ny as usize][nx as usize];
                if PieceType::from_fen(c) == Some(PieceType::Knight) {
                    let c_is_red = Side::from_fen(c) == Some(Side::Red);
                    if c_is_red == by_red {
                        let bx_pos = (col as isize + bx, row as isize + by);
                        if bx_pos.0 >= 0 && bx_pos.0 < 9 && bx_pos.1 >= 0 && bx_pos.1 < 10 {
                            if self.squares[bx_pos.1 as usize][bx_pos.0 as usize] == '.' {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        // Check advisor moves
        for &dx in &[-1isize, 1] {
            for &dy in &[-1isize, 1] {
                let nx = col as isize + dx;
                let ny = row as isize + dy;
                if nx >= 0 && nx < 9 && ny >= 0 && ny < 10 {
                    let c = self.squares[ny as usize][nx as usize];
                    if PieceType::from_fen(c) == Some(PieceType::Advisor) {
                        let c_is_red = Side::from_fen(c) == Some(Side::Red);
                        if c_is_red == by_red {
                            return true;
                        }
                    }
                }
            }
        }

        // Check pawn
        let pawn_dirs = if by_red { 1isize } else { -1 };
        // Forward
        let fx = col as isize;
        let fy = row as isize + pawn_dirs;
        if fy >= 0 && fy < 10 {
            let c = self.squares[fy as usize][fx as usize];
            if PieceType::from_fen(c) == Some(PieceType::Pawn) {
                if Side::from_fen(c) == Some(if by_red { Side::Red } else { Side::Black }) {
                    return true;
                }
            }
        }
        // Side captures (only after crossing river)
        for &dc in &[-1isize, 1] {
            let sx = col as isize + dc;
            if sx >= 0 && sx < 9 {
                let c = self.squares[row as usize][sx as usize];
                if PieceType::from_fen(c) == Some(PieceType::Pawn) {
                    let c_is_red = Side::from_fen(c) == Some(Side::Red);
                    if c_is_red == by_red {
                        // Check if pawn has crossed river
                        let pawn_row = row;
                        let crossed = if by_red { pawn_row >= 5 } else { pawn_row <= 4 };
                        if crossed {
                            return true;
                        }
                    }
                }
            }
        }

        // Check elephant
        for &dx in &[-2isize, 2] {
            for &dy in &[-2isize, 2] {
                let nx = col as isize + dx;
                let ny = row as isize + dy;
                if nx >= 0 && nx < 9 && ny >= 0 && ny < 10 {
                    let c = self.squares[ny as usize][nx as usize];
                    if PieceType::from_fen(c) == Some(PieceType::Elephant) {
                        let c_is_red = Side::from_fen(c) == Some(Side::Red);
                        if c_is_red == by_red {
                            let crossed = if by_red { ny <= 4 } else { ny >= 5 };
                            if crossed {
                                // Check eye position
                                let eye_x = col as isize + dx / 2;
                                let eye_y = row as isize + dy / 2;
                                if eye_x >= 0 && eye_x < 9 && eye_y >= 0 && eye_y < 10 {
                                    if self.squares[eye_y as usize][eye_x as usize] == '.' {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Check if the king of the given side is in check
    pub fn is_in_check(&self, side: Side) -> bool {
        let is_red = side == Side::Red;
        if let Some((kx, ky)) = self.get_king_pos(side) {
            self.is_under_attack(kx, ky, !is_red)
        } else {
            false
        }
    }

    /// Check if the side is checkmated (in check and no legal moves)
    pub fn is_checkmate(&self, side: Side) -> bool {
        if !self.is_in_check(side) {
            return false;
        }
        // Try all possible moves for the side
        for from_row in 0..10 {
            for from_col in 0..9 {
                let fen = self.squares[from_row][from_col];
                if fen == '.' {
                    continue;
                }
                let piece_side = Side::from_fen(fen);
                if piece_side != Some(side) {
                    continue;
                }
                for to_row in 0..10 {
                    for to_col in 0..9 {
                        let mut test_board = self.copy();
                        if test_board.make_move((from_col, from_row), (to_col, to_row)) {
                            // After the move, check if our king is still in check
                            if !test_board.is_in_check(side) {
                                return false; // Found an escape move
                            }
                        }
                    }
                }
            }
        }
        true // No escape moves found
    }

    /// Check if a move would result in checkmate
    pub fn is_checkmate_move(
        &self,
        from: (usize, usize),
        to: (usize, usize),
        opponent_side: Side,
    ) -> bool {
        let mut test_board = self.copy();
        if !test_board.make_move(from, to) {
            return false;
        }
        test_board.is_checkmate(opponent_side)
    }

    /// Generate all legal moves for the given side
    pub fn create_moves(&self, side: Side) -> Vec<((usize, usize), (usize, usize))> {
        let mut moves = Vec::new();
        for from_row in 0..10 {
            for from_col in 0..9 {
                let fen = self.squares[from_row][from_col];
                if fen == '.' || Side::from_fen(fen) != Some(side) {
                    continue;
                }
                for to_row in 0..10 {
                    for to_col in 0..9 {
                        let mut test_board = self.copy();
                        if test_board.make_move((from_col, from_row), (to_col, to_row)) {
                            if !test_board.is_in_check(side) {
                                moves.push(((from_col, from_row), (to_col, to_row)));
                            }
                        }
                    }
                }
            }
        }
        moves
    }

    /// Count pieces between two positions on x-axis (same row, exclusive)
    pub fn count_x_line_in(&self, row: usize, from_col: usize, to_col: usize) -> usize {
        let min_c = from_col.min(to_col) + 1;
        let max_c = from_col.max(to_col);
        let mut count = 0;
        for c in min_c..max_c {
            if self.squares[row][c] != '.' {
                count += 1;
            }
        }
        count
    }

    /// Count pieces between two positions on y-axis (same col, exclusive)
    pub fn count_y_line_in(&self, col: usize, from_row: usize, to_row: usize) -> usize {
        let min_r = from_row.min(to_row) + 1;
        let max_r = from_row.max(to_row);
        let mut count = 0;
        for r in min_r..max_r {
            if self.squares[r][col] != '.' {
                count += 1;
            }
        }
        count
    }

    /// Get pieces on x-line between positions (exclusive)
    pub fn x_line_in(&self, row: usize, from_col: usize, to_col: usize) -> Vec<char> {
        let min_c = from_col.min(to_col) + 1;
        let max_c = from_col.max(to_col);
        (min_c..max_c).map(|c| self.squares[row][c]).collect()
    }

    /// Get pieces on y-line between positions (exclusive)
    pub fn y_line_in(&self, col: usize, from_row: usize, to_row: usize) -> Vec<char> {
        let min_r = from_row.min(to_row) + 1;
        let max_r = from_row.max(to_row);
        (min_r..max_r).map(|r| self.squares[r][col]).collect()
    }

    /// Get FEN with side to move (full FEN)
    pub fn to_full_fen(&self, side_to_move: Side) -> String {
        let board_fen = self.to_fen();
        let side_char = match side_to_move {
            Side::Red => 'w',
            Side::Black => 'b',
            _ => 'w',
        };
        format!("{} {}", board_fen, side_char)
    }

    /// Get all positions of a specific piece character
    pub fn get_fench_positions(&self, fen_char: char) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        for row in 0..10 {
            for col in 0..9 {
                if self.squares[row][col] == fen_char {
                    positions.push((col, row));
                }
            }
        }
        positions
    }

    /// Get all positions of pieces for a given color
    pub fn get_all_fench_positions(&self, color: Option<Side>) -> Vec<(char, usize, usize)> {
        let mut positions = Vec::new();
        for row in 0..10 {
            for col in 0..9 {
                let fen = self.squares[row][col];
                if fen == '.' {
                    continue;
                }
                match color {
                    None => positions.push((fen, col, row)),
                    Some(side) => {
                        if Side::from_fen(fen) == Some(side) {
                            positions.push((fen, col, row));
                        }
                    }
                }
            }
        }
        positions
    }

    /// Detect which pieces moved between two boards
    pub fn detect_move_pieces(&self, other: &Board) -> (Vec<(usize, usize)>, Vec<(usize, usize)>) {
        let mut from_positions = Vec::new();
        let mut to_positions = Vec::new();
        for row in 0..10 {
            for col in 0..9 {
                if self.squares[row][col] != other.squares[row][col] {
                    if self.squares[row][col] != '.' && other.squares[row][col] == '.' {
                        from_positions.push((col, row));
                    } else if self.squares[row][col] == '.' && other.squares[row][col] != '.' {
                        to_positions.push((col, row));
                    } else if self.squares[row][col] != '.' && other.squares[row][col] != '.' {
                        from_positions.push((col, row));
                        to_positions.push((col, row));
                    }
                }
            }
        }
        (from_positions, to_positions)
    }

    /// Create a move from the difference between two boards
    pub fn create_move_from_board(
        &self,
        other: &Board,
    ) -> Option<((usize, usize), (usize, usize))> {
        let (froms, tos) = self.detect_move_pieces(other);
        if froms.len() == 1 && tos.len() == 1 {
            Some((froms[0], tos[0]))
        } else if froms.is_empty() && tos.is_empty() {
            None
        } else {
            // Try to find the most likely move
            if !froms.is_empty() && !tos.is_empty() {
                Some((froms[0], tos[0]))
            } else {
                None
            }
        }
    }

    /// Pretty print board as text view
    pub fn print_view(&self) -> Vec<String> {
        let mut result = Vec::new();
        // Header
        result.push("  a   b   c   d   e   f   g   h   i".to_string());
        for row in (0..10).rev() {
            let mut line = format!("{}", row);
            for col in 0..9 {
                let c = self.squares[row][col];
                line.push_str(&format!("  {}", if c == '.' { '·' } else { c }));
            }
            result.push(line);
        }
        // Column numbers
        let mut nums = " ".to_string();
        for col in 0..9 {
            nums.push_str(&format!("  {}", col));
        }
        result.push(nums);
        result
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
        if !(3..=5).contains(&col) {
            return false;
        }
        if is_red {
            row <= 2
        } else {
            row >= 7
        }
    }

    /// Check if a position is across the river (过河)
    pub fn is_across_river(row: usize, is_red: bool) -> bool {
        if is_red {
            row >= 5
        } else {
            row <= 4
        }
    }

    /// Get piece at position with type and side
    pub fn get_piece_at(&self, col: usize, row: usize) -> Option<(PieceType, Side)> {
        let fen_char = self.get_fen(col, row);
        if fen_char == '.' {
            return None;
        }
        let piece_type = PieceType::from_fen(fen_char)?;
        let side = Side::from_fen(fen_char)?;
        Some((piece_type, side))
    }

    /// Set piece at position
    pub fn set_piece_at(&mut self, col: usize, row: usize, piece_type: PieceType, side: Side) {
        let fen_char = match side {
            Side::Black => piece_type.to_fen_base(),
            Side::Red => piece_type.to_fen_base().to_ascii_uppercase(),
            Side::Any => '.',
        };
        self.set_fen(col, row, fen_char);
    }

    /// Remove piece at position
    pub fn remove_piece_at(&mut self, col: usize, row: usize) {
        self.set_fen(col, row, '.');
    }

    /// Pop piece at position: get and remove the piece
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

    /// Generate move text representation
    pub fn move_text(
        &self,
        from: (usize, usize),
        to: (usize, usize),
        format: MoveFormat,
        traditional: bool,
    ) -> Result<String, String> {
        let notation = MoveNotation::from_board_move(self, from, to)?;
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

    /// Get move notation for a move
    pub fn move_notation(
        &self,
        from: (usize, usize),
        to: (usize, usize),
    ) -> Result<MoveNotation, String> {
        MoveNotation::from_board_move(self, from, to)
    }

    /// Make a move by ICCS notation (e.g., "e2e4")
    pub fn move_iccs(&mut self, iccs: &str) -> bool {
        if iccs.len() != 4 {
            return false;
        }
        let bytes = iccs.as_bytes();
        let from_col = (bytes[0] as char).to_lowercase().next().unwrap() as usize - 'a' as usize;
        let from_row = bytes[1].saturating_sub(b'0') as usize;
        let to_col = (bytes[2] as char).to_lowercase().next().unwrap() as usize - 'a' as usize;
        let to_row = bytes[3].saturating_sub(b'0') as usize;
        self.make_move((from_col, from_row), (to_col, to_row))
    }

    /// Flip board perspective
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

        // 测试红方车九进一（简体中文）- Red rook at (0,0) moves to (0,1)
        // Column 0 = 九路 (9 - 0 = 9)
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

        // 测试红方炮二平五 - Red cannon at (7,2) moves to (4,2)
        // Column 7 = 二路 (9 - 7 = 2), to col 4 = 五路
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

        // 测试黑方车1进1 - Black rook at (0,9) moves to (0,8)
        let result = board.move_text((0, 9), (0, 8), MoveFormat::Chinese, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("黑方车1进1: {}", text);
        // Black uses 车 for rook (same as Red in simplified), full-width numbers
        assert_eq!(text, "车９进１");

        // 测试黑方炮２平５ - Black cannon at (7,7) moves to (4,7)
        // Column 7 = 2路 (9-7=2), to col 4 = 5路
        let result = board.move_text((7, 7), (4, 7), MoveFormat::Chinese, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("黑方炮２平５: {}", text);
        // Black uses full-width numbers: ２平５
        assert_eq!(text, "炮２平５");
    }

    #[test]
    fn test_move_text_wxf() {
        let mut board = Board::new();
        board.initial_position();

        // 测试WXF格式：红方车九进一 - Red rook at (0,0), col 0 = file 9
        let result = board.move_text((0, 0), (0, 1), MoveFormat::WXF, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("WXF格式（红车）: {}", text);
        assert_eq!(text, "R9+1");

        // 测试WXF格式：黑方车9进1 - Black rook at (0,9), col 0 = file 9
        let result = board.move_text((0, 9), (0, 8), MoveFormat::WXF, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("WXF格式（黑车）: {}", text);
        assert_eq!(text, "R9+1");

        // 测试WXF格式：红方炮二平五 - Red cannon at (7,2)
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

        // 测试ICCS格式 - Red rook (0,0) → (0,1) = ICCS a0a1
        let result = board.move_text((0, 0), (0, 1), MoveFormat::ICCS, false);
        assert!(result.is_ok());
        let text = result.unwrap();
        println!("ICCS格式: {}", text);
        assert_eq!(text, "a0a1");

        // 测试ICCS格式 - Red cannon (7,2) → (7,3) = ICCS h2h3
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

        // 测试move_notation方法 - Red rook (0,0) → (0,1)
        let result = board.move_notation((0, 0), (0, 1));
        assert!(result.is_ok());
        let notation = result.unwrap();

        assert_eq!(notation.piece_type, PieceType::Rook);
        assert_eq!(notation.piece_color, Side::Red);
        assert_eq!(notation.column, 9); // col 0 = 九路
        assert_eq!(notation.direction, Direction::Forward);
        assert_eq!(notation.distance, 1);
    }

    #[test]
    fn test_flip_perspective() {
        let mut board = Board::new();
        board.initial_position();

        // 测试棋盘翻转
        let flipped = board.flip_perspective();

        // Red rook at (0,0) should flip to Black's top-right (8,9)
        assert_eq!(board.get_fen(0, 0), 'R'); // Red rook
        assert_eq!(flipped.get_fen(8, 9), 'R'); // Flipped

        // Black rook at (0,9) should flip to Red's bottom-right (8,0)
        assert_eq!(board.get_fen(0, 9), 'r'); // Black rook
        assert_eq!(flipped.get_fen(8, 0), 'r'); // Flipped

        // Center position test
        assert_eq!(board.get_fen(4, 4), '.'); // Center empty
        assert_eq!(flipped.get_fen(4, 5), '.'); // Flipped center empty
    }
}
