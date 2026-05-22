/// Game module for Chinese Chess
use crate::board::Board;
use crate::pieces::Side;

/// Represents a move in the game tree
#[derive(Debug, Clone)]
pub struct MoveNode {
    /// From position (col, row)
    pub from: (usize, usize),
    /// To position (col, row)
    pub to: (usize, usize),
    /// Move in UCI notation (e.g., "e2e4")
    pub uci_notation: String,
    /// Move annotation/comment
    pub annotation: Option<String>,
    /// Board state after this move
    pub board_after: Board,
    /// Side to move after this move
    pub next_turn: Side,
    /// Main line continuation
    pub main_line: Option<Box<MoveNode>>,
    /// Variations (alternative lines)
    pub variations: Vec<MoveNode>,
    /// Move number (ply count)
    pub move_number: u32,
}

/// Represents a game with full move tree (supports variations)
pub struct Game {
    /// The current board state
    pub board: Board,
    /// The side whose turn it is
    pub current_turn: Side,
    /// Whether the game is over
    pub is_game_over: bool,
    /// The winner of the game (if game is over)
    pub winner: Option<Side>,
    /// Root moves of the game tree (first moves)
    pub root_moves: Vec<MoveNode>,
    /// Current position in the move tree
    pub current_node: Option<MoveNode>,
    /// Game metadata
    pub metadata: GameMetadata,
}

/// Game metadata (title, players, etc.)
#[derive(Debug, Clone, Default)]
pub struct GameMetadata {
    /// Game title
    pub title: Option<String>,
    /// Red player name
    pub red_player: Option<String>,
    /// Black player name
    pub black_player: Option<String>,
    /// Event name
    pub event: Option<String>,
    /// Date (YYYYMMDD)
    pub date: Option<String>,
    /// Result ("1-0", "0-1", "1/2-1/2", "*")
    pub result: Option<String>,
    /// Source format (e.g., "XQF", "PGN")
    pub source: Option<String>,
    /// Number of variations
    pub branch_count: u32,
    /// Additional custom metadata
    pub extra: std::collections::HashMap<String, String>,
}

impl MoveNode {
    /// Create a new move node
    pub fn new(
        from: (usize, usize),
        to: (usize, usize),
        uci_notation: String,
        board_after: Board,
        next_turn: Side,
        move_number: u32,
    ) -> Self {
        MoveNode {
            from,
            to,
            uci_notation,
            annotation: None,
            board_after,
            next_turn,
            main_line: None,
            variations: Vec::new(),
            move_number,
        }
    }

    /// Add a variation to this move
    pub fn add_variation(&mut self, variation: MoveNode) {
        self.variations.push(variation);
    }

    /// Get all moves in the main line from this node
    pub fn get_main_line(&self) -> Vec<&MoveNode> {
        let mut moves = Vec::new();
        let mut current = Some(self);
        while let Some(node) = current {
            moves.push(node);
            current = node.main_line.as_deref();
        }
        moves
    }

    /// Get total move count in main line
    pub fn count_moves(&self) -> usize {
        self.get_main_line().len()
    }

    /// Count all variations recursively
    pub fn count_variations(&self) -> u32 {
        let mut count = self.variations.len() as u32;
        for var in &self.variations {
            count += var.count_variations();
        }
        if let Some(ref main) = self.main_line {
            count += main.count_variations();
        }
        count
    }

    /// Get the last move in the main line
    pub fn get_last_move(&self) -> &MoveNode {
        let mut current = self;
        while let Some(ref next) = current.main_line {
            current = next.as_ref();
        }
        current
    }

    /// Get move in Chinese notation
    pub fn chinese_notation(&self, is_red: bool) -> String {
        let (from_col, from_row) = self.from;
        let (to_col, to_row) = self.to;

        // Simplified Chinese notation
        let col_names = ["一", "二", "三", "四", "五", "六", "七", "八", "九"];
        let _row_names = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];

        let from_name = if is_red {
            col_names[8 - from_col]
        } else {
            col_names[from_col]
        };

        let to_name = if is_red {
            col_names[8 - to_col]
        } else {
            col_names[to_col]
        };

        let direction = if is_red {
            if to_row > from_row {
                "进"
            } else if to_row < from_row {
                "退"
            } else {
                "平"
            }
        } else {
            if to_row < from_row {
                "进"
            } else if to_row > from_row {
                "退"
            } else {
                "平"
            }
        };

        let distance = if direction == "平" {
            to_name.to_string()
        } else {
            format!("{}", (to_row as isize - from_row as isize).abs())
        };

        format!("{}{}{}{}", from_name, direction, distance, to_name)
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// Create a new game with initial position
    pub fn new() -> Self {
        let mut board = Board::new();
        board.initial_position();
        Game {
            board,
            current_turn: Side::Red, // Red (红方) always moves first in Chinese Chess
            is_game_over: false,
            winner: None,
            root_moves: Vec::new(),
            current_node: None,
            metadata: GameMetadata::default(),
        }
    }

    /// Make a move and add to the main line
    pub fn make_move(&mut self, from: (usize, usize), to: (usize, usize)) -> Result<(), String> {
        if self.is_game_over {
            return Err("Game is already over".to_string());
        }

        // Validate and execute the move
        let mut new_board = self.board.copy();
        if new_board.make_move(from, to) {
            // Create UCI notation (ICCS-aligned: row 0 = Red bottom)
            let uci_notation = format!(
                "{}{}{}{}",
                (b'a' + from.0 as u8) as char,
                from.1,
                (b'a' + to.0 as u8) as char,
                to.1
            );

            // Calculate move number
            let move_number = self.get_current_ply() as u32;

            // Create new move node
            let new_node = MoveNode::new(
                from,
                to,
                uci_notation,
                new_board.clone(),
                self.current_turn.opposite(),
                move_number,
            );

            // Add to root or current node's main line
            if self.root_moves.is_empty() {
                // First move of the game
                self.root_moves.push(new_node);
                self.current_node = self.root_moves.last().cloned();
            } else {
                // Continue main line from last root move
                let last_idx = self.root_moves.len() - 1;
                let mut current = &mut self.root_moves[last_idx];
                while current.main_line.is_some() {
                    current = current.main_line.as_mut().unwrap();
                }
                current.main_line = Some(Box::new(new_node));
                self.current_node = current.main_line.as_deref().cloned();
            }

            // Update game state
            self.board = new_board;
            self.current_turn = self.current_turn.opposite();

            // Check for game over conditions
            self.check_game_over();

            Ok(())
        } else {
            Err("Invalid move".to_string())
        }
    }

    /// Make a move as a variation of a specific parent move
    pub fn make_variation(
        &mut self,
        parent_ply: u32,
        from: (usize, usize),
        to: (usize, usize),
    ) -> Result<(), String> {
        if self.is_game_over {
            return Err("Game is already over".to_string());
        }

        // Create new board for the variation
        let board_at_parent = if parent_ply == 0 {
            self.board.copy()
        } else {
            // Find the parent node and get its board state
            let parent_board = self.get_board_at_ply(parent_ply);
            match parent_board {
                Some(b) => b,
                None => return Err("Parent move not found".to_string()),
            }
        };

        let mut new_board = board_at_parent;
        if !new_board.make_move(from, to) {
            return Err("Invalid move for variation".to_string());
        }

        // Create UCI notation (ICCS-aligned: row 0 = Red bottom)
        let uci_notation = format!(
            "{}{}{}{}",
            (b'a' + from.0 as u8) as char,
            from.1,
            (b'a' + to.0 as u8) as char,
            to.1
        );

        // Create new variation node
        let variation_node = MoveNode::new(
            from,
            to,
            uci_notation,
            new_board,
            if parent_ply.is_multiple_of(2) {
                Side::Red
            } else {
                Side::Black
            },
            parent_ply,
        );

        // Add variation to parent
        self.add_variation_to_node(parent_ply, variation_node);
        self.metadata.branch_count += 1;

        Ok(())
    }

    /// Find a node by its ply number in the main line
    #[allow(dead_code)]
    fn find_node_by_ply(&mut self, ply: u32) -> Option<&mut MoveNode> {
        if ply == 0 && self.root_moves.is_empty() {
            return None;
        }

        if ply == 0 {
            return self.root_moves.last_mut();
        }

        // Traverse main line to find the node
        for root_move in &mut self.root_moves {
            if let Some(node) = Self::find_in_line(root_move, ply) {
                return Some(node);
            }
        }
        None
    }

    /// Helper to find node in a line
    fn find_in_line(node: &mut MoveNode, target_ply: u32) -> Option<&mut MoveNode> {
        if node.move_number == target_ply {
            return Some(node);
        }
        if let Some(ref mut main) = node.main_line {
            return Self::find_in_line(main, target_ply);
        }
        None
    }

    /// Get current ply (half-move count)
    pub fn get_current_ply(&self) -> usize {
        if self.root_moves.is_empty() {
            return 0;
        }
        // Count moves in the last main line
        let last_root = self.root_moves.last().unwrap();
        last_root.count_moves()
    }

    /// Add annotation to the last move
    pub fn annotate_last_move(&mut self, annotation: String) {
        if let Some(ref mut node) = self.current_node {
            node.annotation = Some(annotation);
        }
    }

    /// Get all moves in the main line
    pub fn get_main_line(&self) -> Vec<&MoveNode> {
        if self.root_moves.is_empty() {
            return Vec::new();
        }
        self.root_moves[0].get_main_line()
    }

    /// Get total move count
    pub fn total_moves(&self) -> usize {
        self.get_main_line().len()
    }

    /// Count all variations
    pub fn total_variations(&self) -> u32 {
        let mut count = 0;
        for root in &self.root_moves {
            count += root.count_variations();
        }
        count
    }

    /// Navigate to a specific move
    pub fn navigate_to_move(&mut self, ply: u32) -> Result<(), String> {
        if ply == 0 {
            self.current_node = None;
            let mut board = Board::new();
            board.initial_position();
            self.board = board;
            self.current_turn = Side::Red;
            return Ok(());
        }

        // Get the board state at the target ply
        let target_board = self.get_board_at_ply(ply);
        if let Some(board) = target_board {
            self.board = board;
            self.current_turn = if ply.is_multiple_of(2) {
                Side::Red
            } else {
                Side::Black
            };
            Ok(())
        } else {
            Err("Move number out of range".to_string())
        }
    }

    /// Get the move tree as a string representation
    pub fn get_move_tree_string(&self) -> String {
        let mut result = String::new();
        for (i, root) in self.root_moves.iter().enumerate() {
            if i > 0 {
                result.push_str("\n--- Alternative first move ---\n");
            }
            result.push_str(&Self::node_to_string(root, 0));
        }
        result
    }

    /// Convert a node and its variations to string
    fn node_to_string(node: &MoveNode, depth: usize) -> String {
        let mut result = String::new();
        let indent = "  ".repeat(depth);

        let move_num = if node.move_number % 2 == 1 {
            format!("{}. ", node.move_number.div_ceil(2))
        } else {
            format!("{}... ", node.move_number / 2)
        };

        result.push_str(&format!("{}{}{}\n", indent, move_num, node.uci_notation));

        if let Some(ref annotation) = node.annotation {
            result.push_str(&format!("{}  ; {}\n", indent, annotation));
        }

        // Print variations
        for (i, var) in node.variations.iter().enumerate() {
            result.push_str(&format!("{}Variation {}:\n", indent, i + 1));
            result.push_str(&Self::node_to_string(var, depth + 1));
        }

        // Print main line continuation
        if let Some(ref main) = node.main_line {
            result.push_str(&Self::node_to_string(main, depth));
        }

        result
    }

    /// Convert game to PGN format
    pub fn to_pgn(&self) -> String {
        let mut pgn = String::new();

        // Add metadata tags
        if let Some(ref title) = self.metadata.title {
            pgn.push_str(&format!("[Title \"{}\"]\n", title));
        }
        if let Some(ref red) = self.metadata.red_player {
            pgn.push_str(&format!("[Red \"{}\"]\n", red));
        }
        if let Some(ref black) = self.metadata.black_player {
            pgn.push_str(&format!("[Black \"{}\"]\n", black));
        }
        if let Some(ref result) = self.metadata.result {
            pgn.push_str(&format!("[Result \"{}\"]\n", result));
        }
        if let Some(ref event) = self.metadata.event {
            pgn.push_str(&format!("[Event \"{}\"]\n", event));
        }
        if let Some(ref date) = self.metadata.date {
            pgn.push_str(&format!("[Date \"{}\"]\n", date));
        }
        pgn.push('\n');

        // Add moves
        let moves = self.get_main_line();
        let mut move_text = String::new();

        for node in &moves {
            if node.move_number % 2 == 0 {
                if !move_text.is_empty() {
                    move_text.push(' ');
                }
                move_text.push_str(&format!(
                    "{}. {}",
                    node.move_number / 2 + 1,
                    node.uci_notation
                ));
            } else {
                move_text.push_str(&format!(" {}", node.uci_notation));
            }

            // Add annotation if present
            if let Some(ref ann) = node.annotation {
                move_text.push_str(&format!(" {{{}}}", ann));
            }
        }

        // Add result
        let result = self.metadata.result.as_deref().unwrap_or("*");
        move_text.push_str(&format!(" {}", result));

        pgn.push_str(&move_text);
        pgn
    }

    /// Get the PGN string directly
    pub fn get_pgn(&self) -> String {
        self.to_pgn()
    }

    /// Export game to PGN file
    pub fn export_pgn(&self, path: &str) -> Result<(), String> {
        std::fs::write(path, self.to_pgn()).map_err(|e| format!("Failed to write PGN: {}", e))
    }

    /// Export game to XQF file (if xqf module is available)
    pub fn export_xqf(&self, path: &str) -> Result<(), String> {
        // This will be implemented in xqf module
        crate::xqf::write_xqf_from_game(self, path)
            .map_err(|e| format!("Failed to write XQF: {}", e))
    }

    /// Read a game from a PGN file (auto-detects format by extension)
    pub fn read_from(path: &str) -> Result<Self, String> {
        if path.ends_with(".xqf") || path.ends_with(".XQF") {
            // Read XQF file
            let xqf = crate::xqf::read_xqf_with_variations(path)
                .map_err(|e| format!("Failed to read XQF: {}", e))?;
            crate::xqf::xqf_file_to_game(&xqf)
                .map_err(|e| format!("Failed to convert XQF to game: {}", e))
        } else {
            // Read PGN file (try UTF-8 first, then GBK)
            let bytes = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
            let content = match String::from_utf8(bytes.clone()) {
                Ok(s) => s,
                Err(_) => encoding_rs::GBK.decode(&bytes).0.into_owned(),
            };
            Self::from_pgn(&content)
        }
    }

    /// Parse a game from a PGN string
    pub fn from_pgn(pgn: &str) -> Result<Self, String> {
        use crate::pgn::PGNParser;
        let pgn_game = PGNParser::parse(pgn).map_err(|e| format!("Failed to parse PGN: {}", e))?;
        pgn_game
            .to_game()
            .map_err(|e| format!("Failed to convert PGN to game: {}", e))
    }

    /// Save game to a file (auto-detects format by extension)
    pub fn save_to(&self, path: &str) -> Result<(), String> {
        if path.ends_with(".xqf") || path.ends_with(".XQF") {
            self.export_xqf(path)
        } else {
            self.export_pgn(path)
        }
    }

    /// Dump all move lines as text (returns Vec<Vec<String>> for all variations)
    /// Each inner Vec is a complete move line with Chinese notation
    pub fn dump_text_moves(&self) -> Vec<Vec<String>> {
        let mut lines = Vec::new();
        for root_move in &self.root_moves {
            let mut line_moves = Vec::new();
            let mut current: Option<&MoveNode> = Some(root_move);
            // Track whose move it is for each ply
            // First move is always Red's move
            let mut is_red = true;
            while let Some(node) = current {
                line_moves.push(node.chinese_notation(is_red));
                current = node.main_line.as_deref();
                is_red = !is_red;
            }
            lines.push(line_moves);
        }
        if lines.is_empty() {
            // Return empty line if no moves
            lines.push(Vec::new());
        }
        lines
    }

    /// Verify that all moves in the game are legal
    pub fn verify_moves(&self) -> bool {
        // Re-play all moves from the initial position and check legality
        for root_move in &self.root_moves {
            let mut board = Board::new();
            board.initial_position();

            // Apply first move (always Red's first move)
            if !board.make_move(root_move.from, root_move.to) {
                return false;
            }

            // Follow main line
            let mut current = root_move.main_line.as_ref();
            while let Some(node) = current {
                if !board.make_move(node.from, node.to) {
                    return false;
                }
                current = node.main_line.as_ref();
            }
        }
        true
    }

    /// Get the current game state as a string
    pub fn display(&self) -> String {
        let moves = self.get_main_line();
        format!(
            "Current turn: {:?}\nGame over: {}\nWinner: {:?}\nTotal moves: {}\nVariations: {}",
            self.current_turn,
            self.is_game_over,
            self.winner,
            moves.len(),
            self.total_variations()
        )
    }

    /// Create a game from an existing board
    pub fn from_board(board: Board) -> Self {
        Game {
            board: board.clone(),
            current_turn: Side::Red,
            is_game_over: false,
            winner: None,
            root_moves: Vec::new(),
            current_node: None,
            metadata: GameMetadata::default(),
        }
    }

    /// Get a reference to the board
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    /// Check if a specific side's king is in check
    pub fn is_in_check(&self, side: Side) -> bool {
        let king_pos = match self.find_king_position(side) {
            Some(pos) => pos,
            None => return true, // King doesn't exist = in check (shouldn't happen)
        };
        self.is_square_attacked(king_pos.0, king_pos.1, side)
    }

    /// Check if a square is attacked by the opponent of `defending_color`
    fn is_square_attacked(&self, col: usize, row: usize, defending_side: Side) -> bool {
        use crate::pieces::PieceType;
        let attacking_side = defending_side.opposite();
        let is_attacking_red = attacking_side == Side::Red;

        // Iterate over all squares to find attacker pieces
        for from_row in 0..10 {
            for from_col in 0..9 {
                let fen = self.board.squares[from_row][from_col];
                if fen == '.' {
                    continue;
                }

                let attacker_side = Side::from_fen(fen);
                if attacker_side != Some(attacking_side) {
                    continue;
                }

                let piece_type = match PieceType::from_fen(fen) {
                    Some(pt) => pt,
                    None => continue,
                };

                // Check if this piece can attack the target square
                let can_attack = match piece_type {
                    PieceType::King => {
                        // King attacks adjacent squares within palace
                        let dx = (col as isize - from_col as isize).abs();
                        let dy = (row as isize - from_row as isize).abs();
                        dx + dy == 1 && Board::is_in_palace(col, row, is_attacking_red)
                    }
                    PieceType::Advisor => {
                        // Advisor attacks diagonally within palace
                        let dx = (col as isize - from_col as isize).abs();
                        let dy = (row as isize - from_row as isize).abs();
                        dx == 1 && dy == 1 && Board::is_in_palace(col, row, is_attacking_red)
                    }
                    PieceType::Elephant => {
                        // Elephant attacks in 田 pattern
                        let dx = col as isize - from_col as isize;
                        let dy = row as isize - from_row as isize;
                        if dx.abs() != 2 || dy.abs() != 2 {
                            false
                        } else {
                            // Check for blocking piece
                            let block_col = from_col as isize + dx / 2;
                            let block_row = from_row as isize + dy / 2;
                            // Cannot cross river
                            let crossed_river = if is_attacking_red { row > 4 } else { row < 5 };
                            if crossed_river {
                                false
                            } else {
                                self.board.squares[block_row as usize][block_col as usize] == '.'
                            }
                        }
                    }
                    PieceType::Knight => {
                        // Knight attacks in 日 pattern
                        let dx = col as isize - from_col as isize;
                        let dy = row as isize - from_row as isize;
                        let abs_dx = dx.abs();
                        let abs_dy = dy.abs();
                        if !((abs_dx == 1 && abs_dy == 2) || (abs_dx == 2 && abs_dy == 1)) {
                            false
                        } else {
                            // Check for blocking piece
                            if abs_dx == 2 {
                                let block_col = from_col as isize + dx / 2;
                                self.board.squares[from_row][block_col as usize] == '.'
                            } else {
                                let block_row = from_row as isize + dy / 2;
                                self.board.squares[block_row as usize][from_col] == '.'
                            }
                        }
                    }
                    PieceType::Rook => {
                        // Rook attacks in straight lines
                        let dx = col as isize - from_col as isize;
                        let dy = row as isize - from_row as isize;
                        if dx != 0 && dy != 0 {
                            false
                        } else {
                            !self.board.has_pieces_between(from_col, from_row, col, row)
                        }
                    }
                    PieceType::Cannon => {
                        // Cannon attacks in straight lines (can jump one piece to capture)
                        let dx = col as isize - from_col as isize;
                        let dy = row as isize - from_row as isize;
                        if dx != 0 && dy != 0 {
                            false
                        } else {
                            // Count pieces between
                            let pieces_between = self
                                .board
                                .count_pieces_between(from_col, from_row, col, row);
                            // For attacking an empty square or friendly piece: no pieces between
                            // For attacking enemy piece (capture): exactly one piece between
                            let target_fen = self.board.squares[row][col];
                            if target_fen == '.' {
                                pieces_between == 0
                            } else {
                                let target_side = Side::from_fen(target_fen);
                                if target_side == Some(attacking_side) {
                                    pieces_between == 0
                                } else {
                                    pieces_between == 1
                                }
                            }
                        }
                    }
                    PieceType::Pawn => {
                        // Pawn attacks one step forward
                        let dx = col as isize - from_col as isize;
                        let dy = row as isize - from_row as isize;
                        let abs_dx = dx.abs();
                        let abs_dy = dy.abs();
                        if abs_dx + abs_dy != 1 {
                            false
                        } else if is_attacking_red {
                            // Red pawn attacks forward (increasing row) or sideways after river
                            let crossed_river = Board::is_across_river(from_row, true);
                            if !crossed_river {
                                dy == 1 && dx == 0
                            } else {
                                dy >= 0 // forward or sideways
                            }
                        } else {
                            // Black pawn attacks forward (decreasing row) or sideways after river
                            let crossed_river = Board::is_across_river(from_row, false);
                            if !crossed_river {
                                dy == -1 && dx == 0
                            } else {
                                dy <= 0 // forward or sideways
                            }
                        }
                    }
                };

                if can_attack {
                    return true;
                }
            }
        }

        false
    }

    /// Check if a side has any legal moves
    fn has_legal_moves(&self, side: Side) -> bool {
        use crate::pieces::PieceType;
        let is_red = side == Side::Red;

        // Get all piece positions for this color
        let positions = self.board.get_color_piece_positions(is_red);

        for (from_col, from_row, fen_char) in &positions {
            let piece_type = match PieceType::from_fen(*fen_char) {
                Some(pt) => pt,
                None => continue,
            };

            // Generate all possible destination squares for this piece
            let possible_moves =
                self.get_possible_destinations(piece_type, *from_col, *from_row, is_red);

            for (to_col, to_row) in possible_moves {
                // Simulate the move and check if it leaves king in check
                let mut test_board = self.board.copy();
                if test_board.make_move((*from_col, *from_row), (to_col, to_row)) {
                    // Create test game to check if king is in check
                    let test_game = Game {
                        board: test_board,
                        current_turn: side.opposite(),
                        is_game_over: false,
                        winner: None,
                        root_moves: Vec::new(),
                        current_node: None,
                        metadata: GameMetadata::default(),
                    };
                    if !test_game.is_in_check(side) {
                        return true; // Found a legal move
                    }
                }
            }
        }

        false
    }

    /// Get possible destination squares for a piece (without full validation)
    fn get_possible_destinations(
        &self,
        piece_type: crate::pieces::PieceType,
        from_col: usize,
        from_row: usize,
        is_red: bool,
    ) -> Vec<(usize, usize)> {
        use crate::pieces::PieceType;
        let mut destinations = Vec::new();

        match piece_type {
            PieceType::King => {
                // Adjacent squares within palace
                for (dc, dr) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                    let nc = from_col as isize + dc;
                    let nr = from_row as isize + dr;
                    if (0..9).contains(&nc) && (0..10).contains(&nr) {
                        let nc = nc as usize;
                        let nr = nr as usize;
                        if Board::is_in_palace(nc, nr, is_red) {
                            destinations.push((nc, nr));
                        }
                    }
                }
            }
            PieceType::Advisor => {
                // Diagonal squares within palace
                for (dc, dr) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                    let nc = from_col as isize + dc;
                    let nr = from_row as isize + dr;
                    if (0..9).contains(&nc) && (0..10).contains(&nr) {
                        let nc = nc as usize;
                        let nr = nr as usize;
                        if Board::is_in_palace(nc, nr, is_red) {
                            destinations.push((nc, nr));
                        }
                    }
                }
            }
            PieceType::Elephant => {
                // 田 pattern (2 diagonal steps)
                for (dc, dr) in [(2, 2), (2, -2), (-2, 2), (-2, -2)] {
                    let nc = from_col as isize + dc;
                    let nr = from_row as isize + dr;
                    if (0..9).contains(&nc) && (0..10).contains(&nr) {
                        let nc = nc as usize;
                        let nr = nr as usize;
                        // Check river boundary
                        let can_reach = if is_red { nr <= 4 } else { nr >= 5 };
                        if can_reach {
                            destinations.push((nc, nr));
                        }
                    }
                }
            }
            PieceType::Knight => {
                // 日 pattern
                for (dc, dr) in [
                    (1, 2),
                    (1, -2),
                    (-1, 2),
                    (-1, -2),
                    (2, 1),
                    (2, -1),
                    (-2, 1),
                    (-2, -1),
                ] {
                    let nc = from_col as isize + dc;
                    let nr = from_row as isize + dr;
                    if (0..9).contains(&nc) && (0..10).contains(&nr) {
                        destinations.push((nc as usize, nr as usize));
                    }
                }
            }
            PieceType::Rook => {
                // All squares in straight lines
                // Horizontal
                for c in 0..9 {
                    if c != from_col {
                        destinations.push((c, from_row));
                    }
                }
                // Vertical
                for r in 0..10 {
                    if r != from_row {
                        destinations.push((from_col, r));
                    }
                }
            }
            PieceType::Cannon => {
                // All squares in straight lines
                // Horizontal
                for c in 0..9 {
                    if c != from_col {
                        destinations.push((c, from_row));
                    }
                }
                // Vertical
                for r in 0..10 {
                    if r != from_row {
                        destinations.push((from_col, r));
                    }
                }
            }
            PieceType::Pawn => {
                // One step in allowed directions
                if is_red {
                    // Forward
                    if from_row + 1 < 10 {
                        destinations.push((from_col, from_row + 1));
                    }
                    let crossed_river = Board::is_across_river(from_row, true);
                    if crossed_river {
                        // Left and right
                        if from_col > 0 {
                            destinations.push((from_col - 1, from_row));
                        }
                        if from_col < 8 {
                            destinations.push((from_col + 1, from_row));
                        }
                    }
                } else {
                    // Forward
                    if from_row > 0 {
                        destinations.push((from_col, from_row - 1));
                    }
                    let crossed_river = Board::is_across_river(from_row, false);
                    if crossed_river {
                        // Left and right
                        if from_col > 0 {
                            destinations.push((from_col - 1, from_row));
                        }
                        if from_col < 8 {
                            destinations.push((from_col + 1, from_row));
                        }
                    }
                }
            }
        }

        destinations
    }

    /// Check game over conditions: king captured, checkmate, or stalemate
    fn check_game_over(&mut self) {
        // Check if either king is missing
        let red_king_exists = self.find_king_position(Side::Red).is_some();
        let black_king_exists = self.find_king_position(Side::Black).is_some();

        if !red_king_exists {
            self.is_game_over = true;
            self.winner = Some(Side::Black);
            return;
        }
        if !black_king_exists {
            self.is_game_over = true;
            self.winner = Some(Side::Red);
            return;
        }

        // Check for checkmate or stalemate
        if !self.has_legal_moves(self.current_turn) {
            self.is_game_over = true;
            if self.is_in_check(self.current_turn) {
                // Checkmate: current side is in check with no legal moves
                self.winner = Some(self.current_turn.opposite());
            } else {
                // Stalemate: current side is not in check but has no legal moves
                // In Chinese Chess, stalemate is a win for the side that delivered it
                // (opposite of Western chess rules)
                self.winner = Some(self.current_turn.opposite());
            }
        }
    }

    /// Find king position (simplified)
    fn find_king_position(&self, side: Side) -> Option<(usize, usize)> {
        use crate::pieces::PieceType;
        for row in 0..10 {
            for col in 0..9 {
                if let Some((pt, s)) = self.board.get_piece_at(col, row) {
                    if pt == PieceType::King && s == side {
                        return Some((col, row));
                    }
                }
            }
        }
        None
    }

    /// Get board state at a specific ply number
    fn get_board_at_ply(&self, ply: u32) -> Option<Board> {
        if self.root_moves.is_empty() {
            return None;
        }

        // Traverse main line to find the node at the given ply
        let root = &self.root_moves[0];
        if ply == 0 {
            return Some(self.board.clone());
        }

        Self::find_board_in_line(root, ply - 1)
    }

    /// Helper to find board state in a line
    fn find_board_in_line(node: &MoveNode, target_ply: u32) -> Option<Board> {
        if node.move_number == target_ply {
            return Some(node.board_after.clone());
        }
        if let Some(ref main) = node.main_line {
            return Self::find_board_in_line(main, target_ply);
        }
        None
    }

    /// Add variation to a node at the given ply
    fn add_variation_to_node(&mut self, parent_ply: u32, variation: MoveNode) {
        if self.root_moves.is_empty() {
            return;
        }

        if parent_ply == 0 {
            if let Some(root) = self.root_moves.last_mut() {
                root.variations.push(variation);
            }
            return;
        }

        // Find the node at the given ply and add variation
        let root = &mut self.root_moves[0];
        Self::add_var_in_line(root, parent_ply - 1, variation);
    }

    /// Helper to add variation in a line
    fn add_var_in_line(node: &mut MoveNode, target_ply: u32, variation: MoveNode) {
        if node.move_number == target_ply {
            node.variations.push(variation);
            return;
        }
        if let Some(ref mut main) = node.main_line {
            Self::add_var_in_line(main, target_ply, variation);
        }
    }
}
