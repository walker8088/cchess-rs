/// PGN (Portable Game Notation) module for Chinese Chess
use crate::board::Board;
use crate::game::{Game, GameMetadata, MoveNode};
use crate::move_gen::generate_moves;
use crate::move_notation::{ChineseLocale, MoveNotation};
use crate::pieces::{PieceType, Side};
use std::collections::HashMap;

/// PGN format for moves
#[derive(Debug, Clone, PartialEq)]
pub enum PGNFormat {
    /// Chinese vertical line format (e.g., 炮二平五)
    Chinese,
    /// WXF format (e.g., C2.5)
    WXF,
    /// ICCS coordinate format (e.g., H2-E2)
    ICCS,
}

impl Default for PGNFormat {
    fn default() -> Self {
        PGNFormat::Chinese
    }
}

/// PGN game structure
#[derive(Debug, Clone, Default)]
pub struct PGNGame {
    /// Tags (key-value pairs)
    pub tags: HashMap<String, String>,
    /// Moves in the game (main line)
    pub moves: Vec<PGNMove>,
    /// Root moves (for game tree)
    pub root_moves: Vec<PGNMove>,
    /// Result
    pub result: String,
    /// Move format
    pub format: PGNFormat,
    /// FEN string (if any)
    pub fen: Option<String>,
    /// Comments
    pub comments: Vec<String>,
}

/// A single move in PGN
#[derive(Debug, Clone)]
pub struct PGNMove {
    /// Move number (ply count)
    pub move_number: u32,
    /// Original notation string
    pub notation: String,
    /// From position (col, row)
    pub from: Option<(usize, usize)>,
    /// To position (col, row)
    pub to: Option<(usize, usize)>,
    /// Comment/annotation
    pub comment: Option<String>,
    /// Variations
    pub variations: Vec<PGNMove>,
    /// Is this a red move?
    pub is_red: bool,
}

impl Default for PGNMove {
    fn default() -> Self {
        PGNMove {
            move_number: 0,
            notation: String::new(),
            from: None,
            to: None,
            comment: None,
            variations: Vec::new(),
            is_red: true,
        }
    }
}

/// PGN parser
pub struct PGNParser;

impl PGNParser {
    /// Parse a PGN string into a PGNGame
    pub fn parse(pgn: &str) -> Result<PGNGame, String> {
        let mut game = PGNGame::default();
        let mut in_moves = false;
        let mut moves_text = String::new();

        // First pass: separate tags from moves
        for line in pgn.lines() {
            let trimmed = line.trim();

            // Skip empty lines at the beginning
            if trimmed.is_empty() && !in_moves {
                continue;
            }

            // Check for tag
            if trimmed.starts_with('[') && !in_moves {
                if let Some((key, value)) = PGNParser::parse_tag(trimmed) {
                    match key.as_str() {
                        "Game" => {} // Always "Chinese Chess"
                        "Event" => {
                            game.tags.insert("Event".to_string(), value);
                        }
                        "Site" => {
                            game.tags.insert("Site".to_string(), value);
                        }
                        "Date" => {
                            game.tags.insert("Date".to_string(), value);
                        }
                        "Round" => {
                            game.tags.insert("Round".to_string(), value);
                        }
                        "Red" => {
                            game.tags.insert("Red".to_string(), value);
                        }
                        "Black" => {
                            game.tags.insert("Black".to_string(), value);
                        }
                        "Result" => {
                            game.result = value.clone();
                            game.tags.insert("Result".to_string(), value);
                        }
                        "RedTeam" => {
                            game.tags.insert("RedTeam".to_string(), value);
                        }
                        "BlackTeam" => {
                            game.tags.insert("BlackTeam".to_string(), value);
                        }
                        "Opening" => {
                            game.tags.insert("Opening".to_string(), value);
                        }
                        "Variation" => {
                            game.tags.insert("Variation".to_string(), value);
                        }
                        "ECO" | "ECCO" => {
                            game.tags.insert("ECCO".to_string(), value);
                        }
                        "FEN" => {
                            game.fen = Some(value.clone());
                            game.tags.insert("FEN".to_string(), value);
                        }
                        "Format" => {
                            game.format = match value.as_str() {
                                "WXF" => PGNFormat::WXF,
                                "ICCS" => PGNFormat::ICCS,
                                _ => PGNFormat::Chinese,
                            };
                            game.tags.insert("Format".to_string(), value);
                        }
                        _ => {
                            game.tags.insert(key, value);
                        }
                    };
                }
            } else if !trimmed.is_empty() {
                in_moves = true;
                moves_text.push_str(trimmed);
                moves_text.push(' ');
            }
        }

        // Parse moves
        if !moves_text.is_empty() {
            game.root_moves = PGNParser::parse_moves(&moves_text, &game.format)?;
        }

        Ok(game)
    }

    /// Parse a single tag line
    fn parse_tag(line: &str) -> Option<(String, String)> {
        if !line.starts_with('[') || !line.ends_with(']') {
            return None;
        }

        let content = &line[1..line.len() - 1];
        let parts: Vec<&str> = content.splitn(2, ' ').collect();

        if parts.len() == 2 {
            let key = parts[0].to_string();
            let value = parts[1].trim_matches('"').to_string();
            Some((key, value))
        } else {
            None
        }
    }

    /// Parse moves text into a list of PGNMove
    fn parse_moves(text: &str, format: &PGNFormat) -> Result<Vec<PGNMove>, String> {
        let mut moves: Vec<PGNMove> = Vec::new();
        let mut chars = text.chars().peekable();
        let mut current_move_number = 0u32;
        let mut is_red = true;

        // Skip leading separators and comments
        while let Some(&c) = chars.peek() {
            if c == '=' || c.is_whitespace() {
                chars.next();
            } else if c == '{' {
                // Skip comment
                chars.next();
                while let Some(c) = chars.next() {
                    if c == '}' {
                        break;
                    }
                }
            } else {
                break;
            }
        }

        while let Some(&c) = chars.peek() {
            // Skip whitespace
            if c.is_whitespace() {
                chars.next();
                continue;
            }

            // Skip result markers
            if c == '*' || c == '1' || c == '0' || c == '/' {
                // Check for result patterns: 1-0, 0-1, 1/2-1/2, *
                let mut result_text = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() || ch == '{' || ch == '(' {
                        break;
                    }
                    result_text.push(chars.next().unwrap());
                }
                // Result is the end of the game
                if result_text == "1-0"
                    || result_text == "0-1"
                    || result_text == "1/2-1/2"
                    || result_text == "*"
                {
                    break;
                }
                continue;
            }

            // Skip separators like ======
            if c == '=' {
                while let Some(&ch) = chars.peek() {
                    if ch == '=' {
                        chars.next();
                    } else {
                        break;
                    }
                }
                continue;
            }

            // Handle comments
            if c == '{' {
                chars.next(); // consume '{'
                let mut comment = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    comment.push(ch);
                }
                // Attach comment to the last move
                if let Some(last) = moves.last_mut() {
                    last.comment = Some(comment);
                }
                continue;
            }

            // Handle move number
            if c.is_ascii_digit() {
                let mut num_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if let Ok(num) = num_str.parse::<u32>() {
                    current_move_number = num;
                }
                // Skip the dot
                if let Some(&ch) = chars.peek() {
                    if ch == '.' {
                        chars.next();
                    }
                }
                continue;
            }

            // Handle variations
            if c == '(' {
                chars.next(); // consume '('
                let var_text = PGNParser::extract_balanced(&mut chars, '(', ')');
                if !var_text.is_empty() {
                    // Parse variation recursively
                    if let Ok(var_moves) = PGNParser::parse_moves(&var_text, format) {
                        if let Some(last) = moves.last_mut() {
                            last.variations.extend(var_moves);
                        }
                    }
                }
                continue;
            }

            // Handle ellipsis (...) for black moves
            if c == '.' {
                chars.next();
                continue;
            }

            // Parse the move notation
            let notation = PGNParser::extract_move_notation(&mut chars, format)?;
            if !notation.is_empty() {
                let pgn_move = PGNMove {
                    move_number: current_move_number,
                    notation: notation.clone(),
                    from: None,
                    to: None,
                    comment: None,
                    variations: Vec::new(),
                    is_red,
                };
                moves.push(pgn_move);

                // Toggle color
                if is_red {
                    // Red move completed, next is black (same move number)
                } else {
                    // Black move completed, increment move number
                    current_move_number += 1;
                }
                is_red = !is_red;
            }
        }

        Ok(moves)
    }

    /// Extract balanced text between delimiters
    fn extract_balanced<I: Iterator<Item = char>>(
        chars: &mut std::iter::Peekable<I>,
        open: char,
        close: char,
    ) -> String {
        let mut result = String::new();
        let mut depth = 1;

        while let Some(&c) = chars.peek() {
            if c == open {
                depth += 1;
                result.push(chars.next().unwrap());
            } else if c == close {
                depth -= 1;
                chars.next(); // consume close
                if depth == 0 {
                    break;
                }
                result.push(c);
            } else {
                result.push(chars.next().unwrap());
            }
        }

        result
    }

    /// Extract move notation from character stream
    fn extract_move_notation<I: Iterator<Item = char>>(
        chars: &mut std::iter::Peekable<I>,
        format: &PGNFormat,
    ) -> Result<String, String> {
        let mut notation = String::new();

        match format {
            PGNFormat::Chinese => {
                // Chinese notation: piece name + column + direction + distance
                // e.g., 炮二平五, 马8进7, 前炮退二
                while let Some(&c) = chars.peek() {
                    // Stop at whitespace, dots, braces, parentheses, or result markers
                    if c.is_whitespace()
                        || c == '.'
                        || c == '{'
                        || c == '('
                        || c == ')'
                        || c == '*'
                        || c == '1'
                        || c == '0'
                    {
                        break;
                    }
                    notation.push(chars.next().unwrap());
                }
            }
            PGNFormat::WXF => {
                // WXF notation: piece letter + column + direction + distance
                // e.g., C2.5, H8+7
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || c == '{' || c == '(' || c == ')' || c == '*' {
                        break;
                    }
                    // Check for result patterns
                    if c == '1' || c == '0' || c == '/' {
                        // Could be part of result or notation - just let it through
                    }
                    notation.push(chars.next().unwrap());
                }
            }
            PGNFormat::ICCS => {
                // ICCS notation: column+row-column+row
                // e.g., H2-E2, e7e5
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || c == '{' || c == '(' || c == ')' || c == '*' {
                        break;
                    }
                    notation.push(chars.next().unwrap());
                    // Stop after we have a complete coordinate
                    if notation.len() >= 5 && (notation.contains('-') || notation.len() == 4) {
                        break;
                    }
                }
            }
        }

        // Clean up trailing dots that aren't part of notation
        while notation.ends_with('.') && notation.len() > 1 {
            notation.pop();
        }

        Ok(notation)
    }
}

/// Convert PGN notation to coordinates
pub struct NotationConverter;

impl NotationConverter {
    /// Parse Chinese notation to get from/to coordinates
    /// Format: [qualifier][piece][column][direction][distance]
    /// e.g., 炮二平五, 马8进7, 前车进一
    pub fn parse_chinese(
        notation: &str,
        board: &Board,
        is_red: bool,
    ) -> Result<((usize, usize), (usize, usize)), String> {
        let chars: Vec<char> = notation.chars().collect();
        if chars.is_empty() {
            return Err("Empty notation".to_string());
        }

        let mut idx = 0;

        // Parse qualifier (optional)
        let _qualifier = if idx < chars.len()
            && (chars[idx] == '前' || chars[idx] == '中' || chars[idx] == '后')
        {
            let q = chars[idx];
            idx += 1;
            Some(q)
        } else {
            None
        };

        // Parse piece type
        if idx >= chars.len() {
            return Err("Missing piece type".to_string());
        }
        let piece_char = chars[idx];
        let piece_type = NotationConverter::char_to_piece_type(piece_char)?;
        idx += 1;

        // Parse column (Chinese number)
        if idx >= chars.len() {
            return Err("Missing column".to_string());
        }
        let col_char = chars[idx];
        let source_path = NotationConverter::parse_chinese_number(col_char)?;
        idx += 1;

        // Convert path to column (from Red's perspective, right to left)
        // path 1 = col 8, path 9 = col 0
        // Same formula for both Red and Black
        let source_col = 9 - source_path;

        // Parse direction
        if idx >= chars.len() {
            return Err("Missing direction".to_string());
        }
        let direction_char = chars[idx];
        let direction = match direction_char {
            '进' => "forward",
            '退' => "backward",
            '平' => "horizontal",
            _ => return Err(format!("Invalid direction: {}", direction_char)),
        };
        idx += 1;

        // Parse distance
        if idx >= chars.len() {
            return Err("Missing distance".to_string());
        }
        let dist_char = chars[idx];
        let distance = NotationConverter::parse_chinese_number(dist_char)?;

        // Find the piece on the board
        let source_row = NotationConverter::find_piece_on_column(
            board,
            source_col,
            &piece_type,
            is_red,
            None, // qualifier
        )?;

        // Calculate destination
        let (dest_col, dest_row) = NotationConverter::calculate_destination(
            piece_type, source_col, source_row, direction, distance, is_red,
        )?;

        Ok(((source_col, source_row), (dest_col, dest_row)))
    }

    /// Parse WXF notation to coordinates
    /// Format: [qualifier][piece][column][direction][distance]
    /// e.g., C2.5, H8+7, +C-2
    pub fn parse_wxf(
        notation: &str,
        board: &Board,
        is_red: bool,
    ) -> Result<((usize, usize), (usize, usize)), String> {
        let chars: Vec<char> = notation.chars().collect();
        if chars.is_empty() {
            return Err("Empty notation".to_string());
        }

        let mut idx = 0;

        // Handle prefix qualifier (+/-/. for 前/中/后)
        let _prefix_qualifier =
            if idx < chars.len() && (chars[idx] == '+' || chars[idx] == '-' || chars[idx] == '.') {
                let q = chars[idx];
                idx += 1;
                Some(q)
            } else {
                None
            };

        // Parse piece (letter or digit)
        if idx >= chars.len() {
            return Err("Missing piece".to_string());
        }
        let piece_char = chars[idx];
        let piece_type = NotationConverter::wxf_char_to_piece_type(piece_char)?;
        idx += 1;

        // Check if next char is also a piece identifier (for numeric piece notation)
        // e.g., "62.5" where 6=piece, 2=column
        let col_char = chars[idx];
        idx += 1;

        // Parse column (digit or letter a-i)
        let source_path = NotationConverter::parse_wxf_column(col_char)?;

        // Convert path to column (same for both Red and Black)
        let source_col = 9 - source_path;

        // Parse direction
        if idx >= chars.len() {
            return Err("Missing direction".to_string());
        }
        let dir_char = chars[idx];
        let direction = match dir_char {
            '+' => "forward",
            '-' => "backward",
            '.' => "horizontal",
            _ => return Err(format!("Invalid direction: {}", dir_char)),
        };
        idx += 1;

        // Parse distance
        if idx >= chars.len() {
            return Err("Missing distance".to_string());
        }
        let dist_char = chars[idx];
        let distance = dist_char.to_digit(10).ok_or("Invalid distance")? as usize;

        // Find the piece on the board
        let source_row =
            NotationConverter::find_piece_on_column(board, source_col, &piece_type, is_red, None)?;

        // Calculate destination
        let (dest_col, dest_row) = NotationConverter::calculate_destination(
            piece_type, source_col, source_row, direction, distance, is_red,
        )?;

        Ok(((source_col, source_row), (dest_col, dest_row)))
    }

    /// Parse ICCS notation to Rust coordinates
    /// Format: col_row-col_row (e.g., H2-E2)
    /// Columns: a-i (left to right)
    /// Rows: 0-9 (Red's perspective: 0=bottom, 9=top)
    /// Returns Rust coordinates (0=top/Black, 9=bottom/Red)
    pub fn parse_iccs(notation: &str) -> Result<((usize, usize), (usize, usize)), String> {
        let mv = crate::move_notation::try_parse_iccs_move(notation)?;
        Ok(((mv.from_col, mv.from_row), (mv.to_col, mv.to_row)))
    }

    /// Convert a FEN character to PieceType
    fn char_to_piece_type(c: char) -> Result<PieceType, String> {
        // Simplified and Traditional Chinese piece names
        // 帅/帥/将 = King, 仕/士 = Advisor, 相/象 = Elephant
        // 马/馬 = Knight, 车/車 = Rook, 炮/砲 = Cannon, 兵/卒 = Pawn
        match c {
            '帅' | '帥' | '将' | '將' => Ok(PieceType::King),
            '仕' | '士' => Ok(PieceType::Advisor),
            '相' | '象' => Ok(PieceType::Elephant),
            '马' | '馬' | '傌' => Ok(PieceType::Knight),
            '车' | '車' | '俥' => Ok(PieceType::Rook),
            '炮' | '砲' => Ok(PieceType::Cannon),
            '兵' | '卒' => Ok(PieceType::Pawn),
            _ => Err(format!("Unknown piece: {}", c)),
        }
    }

    /// Convert WXF character to PieceType
    fn wxf_char_to_piece_type(c: char) -> Result<PieceType, String> {
        match c.to_ascii_lowercase() {
            'k' | '1' => Ok(PieceType::King),
            'a' | '2' => Ok(PieceType::Advisor),
            'b' | 'e' | '3' => Ok(PieceType::Elephant),
            'n' | 'h' | '4' => Ok(PieceType::Knight),
            'r' | '5' => Ok(PieceType::Rook),
            'c' | '6' => Ok(PieceType::Cannon),
            'p' | '7' => Ok(PieceType::Pawn),
            _ => Err(format!("Unknown WXF piece: {}", c)),
        }
    }

    /// Parse Chinese number character to value (一=1, 二=2, ..., 九=9, １=1, ..., ９=9)
    fn parse_chinese_number(c: char) -> Result<usize, String> {
        match c {
            '一' | '１' | '1' => Ok(1),
            '二' | '２' | '2' => Ok(2),
            '三' | '３' | '3' => Ok(3),
            '四' | '４' | '4' => Ok(4),
            '五' | '５' | '5' => Ok(5),
            '六' | '６' | '6' => Ok(6),
            '七' | '７' | '7' => Ok(7),
            '八' | '８' | '8' => Ok(8),
            '九' | '９' | '9' => Ok(9),
            _ => Err(format!("Invalid Chinese number: {}", c)),
        }
    }

    /// Parse WXF column (digit 1-9 or letter a-i)
    fn parse_wxf_column(c: char) -> Result<usize, String> {
        // For WXF, columns are numbered 1-9 from right to left (Red's perspective)
        if let Some(d) = c.to_digit(10) {
            let val = d as usize;
            if val >= 1 && val <= 9 {
                return Ok(val);
            }
        }
        // Letters a-i (for multiple pieces on same column)
        match c.to_ascii_lowercase() {
            'a' => Ok(1),
            'b' => Ok(2),
            'c' => Ok(3),
            'd' => Ok(4),
            'e' => Ok(5),
            _ => Err(format!("Invalid WXF column: {}", c)),
        }
    }

    /// Find a piece on a specific column
    fn find_piece_on_column(
        board: &Board,
        col: usize,
        piece_type: &PieceType,
        is_red: bool,
        _qualifier: Option<char>,
    ) -> Result<usize, String> {
        // Fix: is_red = true → look for Side::Red (uppercase)
        let color = if is_red { Side::Red } else { Side::Black };

        // Find all pieces of this type and color on this column
        let mut positions: Vec<usize> = Vec::new();
        for row in 0..10 {
            if let Some(pt) = board.get_piece_type(col, row) {
                if pt == *piece_type && board.get_color_at(col, row) == Some(color) {
                    positions.push(row);
                }
            }
        }

        if positions.is_empty() {
            return Err(format!(
                "No {:?} ({}) found on column {}",
                piece_type,
                if is_red { "Red" } else { "Black" },
                col
            ));
        }

        // For most pieces, there should only be one on the column
        // If multiple, we need qualifier logic (simplified here)
        // For 帅/士/象, there's typically only one per column
        // For 兵, we need to determine which one based on position

        if positions.len() == 1 {
            return Ok(positions[0]);
        }

        // Multiple pieces on same column - sort by position
        // For Red: higher row = front, for Black: lower row = front
        if is_red {
            positions.sort_by(|a, b| b.cmp(a)); // Descending (front to back)
        } else {
            positions.sort(); // Ascending (front to back)
        }

        // Without qualifier, try to find the most reasonable one
        // For simplicity, return the first (front-most) piece
        Ok(positions[0])
    }

    /// Calculate destination position
    fn calculate_destination(
        piece_type: PieceType,
        source_col: usize,
        source_row: usize,
        direction: &str,
        distance: usize,
        is_red: bool,
    ) -> Result<(usize, usize), String> {
        let (dest_col, dest_row) = match direction {
            "horizontal" => {
                // 平: horizontal move, distance is target path number
                let target_path = distance;
                let target_col = 9 - target_path; // Same for both Red and Black
                (target_col, source_row)
            }
            "forward" => {
                match piece_type {
                    PieceType::King | PieceType::Rook | PieceType::Cannon | PieceType::Pawn => {
                        // distance is number of steps
                        let steps = distance;
                        let target_row = if is_red {
                            source_row + steps
                        } else {
                            source_row.wrapping_sub(steps)
                        };
                        (source_col, target_row)
                    }
                    PieceType::Knight | PieceType::Advisor | PieceType::Elephant => {
                        // distance is target path number
                        let target_path = distance;
                        let target_col = 9 - target_path; // Same for both Red and Black
                                                          // For diagonal moves, calculate row based on piece type
                        let target_row = match piece_type {
                            PieceType::Advisor => {
                                // 士 moves diagonally by 1
                                if is_red {
                                    source_row + 1
                                } else {
                                    source_row.wrapping_sub(1)
                                }
                            }
                            PieceType::Elephant => {
                                // 象 moves in 田 pattern (2 steps diagonally)
                                if is_red {
                                    source_row + 2
                                } else {
                                    source_row.wrapping_sub(2)
                                }
                            }
                            PieceType::Knight => {
                                // 马 moves in L-shape, row change is 1 or 2
                                // Determine based on column change
                                let col_diff = (target_col as isize - source_col as isize).abs();
                                if col_diff == 1 {
                                    // L-shape: 1 column, 2 rows
                                    if is_red {
                                        source_row + 2
                                    } else {
                                        source_row.wrapping_sub(2)
                                    }
                                } else {
                                    // L-shape: 2 columns, 1 row
                                    if is_red {
                                        source_row + 1
                                    } else {
                                        source_row.wrapping_sub(1)
                                    }
                                }
                            }
                            _ => return Err("Invalid piece for diagonal move".to_string()),
                        };
                        (target_col, target_row)
                    }
                }
            }
            "backward" => {
                match piece_type {
                    PieceType::King | PieceType::Rook | PieceType::Cannon | PieceType::Pawn => {
                        // distance is number of steps
                        let steps = distance;
                        let target_row = if is_red {
                            source_row.wrapping_sub(steps)
                        } else {
                            source_row + steps
                        };
                        (source_col, target_row)
                    }
                    PieceType::Knight | PieceType::Advisor | PieceType::Elephant => {
                        // distance is target path number
                        let target_path = distance;
                        let target_col = 9 - target_path; // Same for both Red and Black
                        let target_row = match piece_type {
                            PieceType::Advisor => {
                                if is_red {
                                    source_row.wrapping_sub(1)
                                } else {
                                    source_row + 1
                                }
                            }
                            PieceType::Elephant => {
                                if is_red {
                                    source_row.wrapping_sub(2)
                                } else {
                                    source_row + 2
                                }
                            }
                            PieceType::Knight => {
                                let col_diff = (target_col as isize - source_col as isize).abs();
                                if col_diff == 1 {
                                    if is_red {
                                        source_row.wrapping_sub(2)
                                    } else {
                                        source_row + 2
                                    }
                                } else {
                                    if is_red {
                                        source_row.wrapping_sub(1)
                                    } else {
                                        source_row + 1
                                    }
                                }
                            }
                            _ => return Err("Invalid piece for diagonal move".to_string()),
                        };
                        (target_col, target_row)
                    }
                }
            }
            _ => return Err(format!("Invalid direction: {}", direction)),
        };

        // Validate bounds
        if dest_col >= 9 || dest_row >= 10 {
            return Err(format!(
                "Destination out of bounds: ({}, {})",
                dest_col, dest_row
            ));
        }

        Ok((dest_col, dest_row))
    }

    /// Convert coordinate move to ICCS notation (with hyphen)
    /// Rust coordinates: (0=top/Black, 9=bottom/Red)
    /// ICCS coordinates: (0=bottom/Red, 9=top/Black)
    pub fn to_iccs(from: (usize, usize), to: (usize, usize)) -> String {
        let mv = crate::move_gen::Move::new(from.0, from.1, to.0, to.1);
        let iccs = crate::move_notation::format_iccs_move(&mv);
        // ICCS format in PGN uses hyphen: h2-e2
        format!("{}-{}", &iccs[0..2], &iccs[2..4])
    }

    fn index_to_iccs_col(col: usize) -> char {
        match col {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            8 => 'i',
            _ => '?',
        }
    }

    /// Convert Chinese notation to WXF format using MoveNotation as intermediate
    /// e.g., 炮二平五 -> C2.5, 马8进7 -> N8+7
    pub fn chinese_to_wxf(notation: &str, board: &Board, is_red: bool) -> Result<String, String> {
        // Step 1: Parse Chinese notation to get coordinates
        let ((from_col, from_row), (to_col, to_row)) =
            NotationConverter::parse_chinese(notation, board, is_red)?;

        // Step 2: Create MoveNotation from board move
        let move_notation =
            MoveNotation::from_board_move(board, (from_col, from_row), (to_col, to_row))?;

        // Step 3: Convert to WXF
        Ok(move_notation.to_wxf())
    }

    /// Convert WXF notation to Chinese format
    pub fn wxf_to_chinese(notation: &str, board: &Board, is_red: bool) -> Result<String, String> {
        // Step 1: Parse WXF notation to get coordinates
        let ((from_col, from_row), (to_col, to_row)) =
            NotationConverter::parse_wxf(notation, board, is_red)?;

        // Step 2: Create MoveNotation from board move
        let move_notation =
            MoveNotation::from_board_move(board, (from_col, from_row), (to_col, to_row))?;

        // Step 3: Convert to Chinese
        let locale = if is_red {
            ChineseLocale::Simplified
        } else {
            ChineseLocale::Simplified // Could also use Traditional
        };
        Ok(move_notation.to_chinese(locale))
    }
}

/// PGN Writer - converts Game to PGN format
pub struct PGNWriter;

impl PGNWriter {
    /// Convert a Game to PGN string
    pub fn write(game: &Game, format: PGNFormat) -> String {
        let mut pgn = String::new();

        // Write tags
        pgn.push_str(&format!("[Game \"Chinese Chess\"]\n"));
        if let Some(event) = &game.metadata.event {
            pgn.push_str(&format!("[Event \"{}\"]\n", event));
        }
        if let Some(red) = &game.metadata.red_player {
            pgn.push_str(&format!("[Red \"{}\"]\n", red));
        }
        if let Some(black) = &game.metadata.black_player {
            pgn.push_str(&format!("[Black \"{}\"]\n", black));
        }
        if let Some(date) = &game.metadata.date {
            pgn.push_str(&format!("[Date \"{}\"]\n", date));
        }
        if let Some(result) = &game.metadata.result {
            pgn.push_str(&format!("[Result \"{}\"]\n", result));
        }
        if let Some(opening) = game.metadata.extra.get("Opening") {
            pgn.push_str(&format!("[Opening \"{}\"]\n", opening));
        }
        if let Some(ecco) = game.metadata.extra.get("ECCO") {
            pgn.push_str(&format!("[ECCO \"{}\"]\n", ecco));
        }
        pgn.push('\n');

        // Write moves
        if let Some(current) = &game.current_node {
            let moves_text = PGNWriter::write_moves(current, &format, game);
            pgn.push_str(&moves_text);
        }

        // Write result
        let result = game.metadata.result.as_deref().unwrap_or("*");
        pgn.push_str(&format!(" {}", result));

        pgn
    }

    /// Write moves recursively
    fn write_moves(node: &MoveNode, format: &PGNFormat, game: &Game) -> String {
        let mut result = String::new();

        // Get move notation based on format
        let notation = PGNWriter::get_move_notation(node, format, game);
        let move_number = (node.move_number + 1) / 2 + 1; // Adjust for 1-based move numbers

        if node.move_number % 2 == 0 {
            // Red move
            result.push_str(&format!("{}. {}", move_number, notation));
        } else {
            // Black move
            result.push_str(&format!("... {}", notation));
        }

        // Add comment if any
        if let Some(comment) = &node.annotation {
            result.push_str(&format!(" {{{}}}", comment));
        }

        // Continue with main line
        if let Some(ref main) = node.main_line {
            result.push(' ');
            result.push_str(&PGNWriter::write_moves(main, format, game));
        }

        // Write variations
        for var in &node.variations {
            result.push_str(" (");
            result.push_str(&PGNWriter::write_moves(var, format, game));
            result.push(')');
        }

        result
    }

    /// Get move notation in specified format
    fn get_move_notation(node: &MoveNode, format: &PGNFormat, _game: &Game) -> String {
        match format {
            PGNFormat::ICCS => NotationConverter::to_iccs(node.from, node.to),
            PGNFormat::Chinese | PGNFormat::WXF => {
                // Use UCI notation as fallback
                node.uci_notation.clone()
            }
        }
    }
}

/// Convert PGNGame to Game
impl PGNGame {
    /// Convert to Game struct with board state
    pub fn to_game(&self) -> Result<Game, String> {
        // Create initial board
        let board = if let Some(fen) = &self.fen {
            Board::from_fen(fen)?
        } else {
            Board::new()
        };

        // Create game
        let mut game = Game::new();
        game.board = board.clone();
        game.metadata = GameMetadata {
            event: self.tags.get("Event").cloned(),
            red_player: self.tags.get("Red").cloned(),
            black_player: self.tags.get("Black").cloned(),
            date: self.tags.get("Date").cloned(),
            result: if self.result != "*" {
                Some(self.result.clone())
            } else {
                None
            },
            extra: self.tags.clone(),
            ..Default::default()
        };

        // Process moves
        let mut current_board = board;
        let mut is_red = true;
        let mut ply = 0u32;

        for pgn_move in &self.root_moves {
            let (from, to) = match self.format {
                PGNFormat::Chinese => {
                    NotationConverter::parse_chinese(&pgn_move.notation, &current_board, is_red)?
                }
                PGNFormat::WXF => {
                    NotationConverter::parse_wxf(&pgn_move.notation, &current_board, is_red)?
                }
                PGNFormat::ICCS => NotationConverter::parse_iccs(&pgn_move.notation)?,
            };

            // Validate and make the move
            let mut new_board = current_board.clone();
            if !new_board.make_move(from, to) {
                // Try to find the correct piece using move generation
                let color = if is_red { Side::Black } else { Side::Red };
                let moves = generate_moves(&current_board, color);

                // Find a move that matches the notation
                let found_move = moves.iter().find(|m| {
                    m.from_col == from.0
                        && m.from_row == from.1
                        && m.to_col == to.0
                        && m.to_row == to.1
                });

                if let Some(m) = found_move {
                    new_board.make_move((m.from_col, m.from_row), (m.to_col, m.to_row));
                } else {
                    return Err(format!(
                        "Invalid move: {} at ply {}",
                        pgn_move.notation, ply
                    ));
                }
            }

            // Create move node
            let uci = format!(
                "{}{}{}{}",
                NotationConverter::index_to_iccs_col(from.0),
                from.1,
                NotationConverter::index_to_iccs_col(to.0),
                to.1
            );

            let node = MoveNode::new(
                from,
                to,
                uci,
                new_board.clone(),
                if is_red { Side::Red } else { Side::Black },
                ply,
            );

            // Add to game tree
            if ply == 0 {
                game.root_moves.push(node.clone());
            }

            game.current_node = Some(node);
            current_board = new_board;
            is_red = !is_red;
            ply += 1;
        }

        // Set current turn
        game.current_turn = if is_red { Side::Black } else { Side::Red };

        Ok(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pgn_tags() {
        let pgn = r#"[Game "Chinese Chess"]
[Event "Test Event"]
[Red "Red Player"]
[Black "Black Player"]
[Result "1-0"]

1. 炮二平五 马８进７
2. 马二进三 1-0"#;

        let game = PGNParser::parse(pgn).unwrap();
        assert_eq!(game.tags.get("Event").unwrap(), "Test Event");
        assert_eq!(game.tags.get("Red").unwrap(), "Red Player");
        assert_eq!(game.tags.get("Black").unwrap(), "Black Player");
        assert_eq!(game.result, "1-0");
    }

    #[test]
    fn test_parse_chinese_notation_basic() {
        let mut board = Board::new();
        board.initial_position();

        // 炮二平五: Cannon on path 2 moves horizontally to path 5
        // Red's path 2 = col 7, path 5 = col 4
        // Red cannons (uppercase 'C') are at row 2 in the new coordinate system
        let result = NotationConverter::parse_chinese("炮二平五", &board, true);
        assert!(result.is_ok());
        let ((from_col, from_row), (to_col, _to_row)) = result.unwrap();
        assert_eq!(from_col, 7); // Path 2 = col 7
        assert_eq!(from_row, 2); // Red cannon starting row
        assert_eq!(to_col, 4); // Path 5 = col 4
    }

    #[test]
    fn test_parse_chinese_notation_black() {
        let mut board = Board::new();
        board.initial_position();

        // 马8进7: Black knight on path 8 moves forward to path 7
        // Black's path 8 = col 1, path 7 = col 2
        // Black knights (lowercase 'n') are at row 9 in new coords
        // Black forward = decreasing row
        let result = NotationConverter::parse_chinese("马8进7", &board, false);
        assert!(result.is_ok());
        let ((from_col, from_row), (to_col, to_row)) = result.unwrap();
        assert_eq!(from_col, 1);
        assert_eq!(from_row, 9); // Black knight starting row
        assert_eq!(to_col, 2);
        assert_eq!(to_row, 7); // Forward move for Black = decreasing row
    }

    #[test]
    fn test_parse_iccs_notation() {
        // ICCS "h2-e2" → internal coordinates (same system now)
        let result = NotationConverter::parse_iccs("h2-e2");
        assert!(result.is_ok());
        let ((from_col, from_row), (to_col, to_row)) = result.unwrap();
        assert_eq!(from_col, 7); // h = 7
        assert_eq!(from_row, 2); // ICCS row 2 = internal row 2
        assert_eq!(to_col, 4); // e = 4
        assert_eq!(to_row, 2); // ICCS row 2 = internal row 2
    }

    #[test]
    fn test_chinese_number_parsing() {
        assert_eq!(NotationConverter::parse_chinese_number('一').unwrap(), 1);
        assert_eq!(NotationConverter::parse_chinese_number('五').unwrap(), 5);
        assert_eq!(NotationConverter::parse_chinese_number('九').unwrap(), 9);
        assert_eq!(NotationConverter::parse_chinese_number('１').unwrap(), 1);
        assert_eq!(NotationConverter::parse_chinese_number('５').unwrap(), 5);
    }

    #[test]
    fn test_piece_type_conversion() {
        assert_eq!(
            NotationConverter::char_to_piece_type('炮').unwrap(),
            PieceType::Cannon
        );
        assert_eq!(
            NotationConverter::char_to_piece_type('马').unwrap(),
            PieceType::Knight
        );
        assert_eq!(
            NotationConverter::char_to_piece_type('車').unwrap(),
            PieceType::Rook
        );
        assert_eq!(
            NotationConverter::char_to_piece_type('卒').unwrap(),
            PieceType::Pawn
        );
    }

    #[test]
    fn test_full_pgn_game() {
        let pgn = r#"[Game "Chinese Chess"]
[Event "Test"]
[Red "Red"]
[Black "Black"]
[Result "*"]

1. 炮二平五 马２进３
2. 马二进三 马８进７"#;

        let game = PGNParser::parse(pgn).unwrap();
        assert_eq!(game.root_moves.len(), 4);
        assert_eq!(game.root_moves[0].notation, "炮二平五");
        assert_eq!(game.root_moves[1].notation, "马２进３");
    }

    #[test]
    fn test_pgn_with_comments() {
        let pgn = r#"[Game "Chinese Chess"]
[Result "*"]

1. 炮二平五 {good move} 马８进７"#;

        let game = PGNParser::parse(pgn).unwrap();
        assert_eq!(game.root_moves.len(), 2);
        assert_eq!(game.root_moves[0].comment.as_deref(), Some("good move"));
    }

    #[test]
    fn test_chinese_to_wxf_red_cannon() {
        let mut board = Board::new();
        board.initial_position();

        // 炮二平五 -> C2.5
        let result = NotationConverter::chinese_to_wxf("炮二平五", &board, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "C2.5");
    }

    #[test]
    fn test_chinese_to_wxf_red_pawn() {
        let mut board = Board::new();
        board.initial_position();

        // 兵五进一 -> P5+1
        let result = NotationConverter::chinese_to_wxf("兵五进一", &board, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "P5+1");
    }

    #[test]
    fn test_wxf_to_chinese_red() {
        let mut board = Board::new();
        board.initial_position();

        // C2.5 -> 炮二平五
        let result = NotationConverter::wxf_to_chinese("C2.5", &board, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "炮二平五");
    }

    #[test]
    fn test_chinese_wxf_roundtrip() {
        let mut board = Board::new();
        board.initial_position();

        // 中文 → WXF → 中文 往返测试
        let original = "炮二平五";
        let wxf = NotationConverter::chinese_to_wxf(original, &board, true).unwrap();
        let back = NotationConverter::wxf_to_chinese(&wxf, &board, true).unwrap();
        assert_eq!(original, back);
    }
}
