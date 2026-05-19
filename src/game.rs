/// Game module for Chinese Chess
use crate::board::Board;
use crate::pieces::Color;

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
    pub next_turn: Color,
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
    pub current_turn: Color,
    /// Whether the game is over
    pub is_game_over: bool,
    /// The winner of the game (if game is over)
    pub winner: Option<Color>,
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
        next_turn: Color,
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
        Game {
            board: Board::new(),
            current_turn: Color::Red, // Red always moves first in Chinese Chess
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
            // Create UCI notation
            let uci_notation = format!(
                "{}{}{}{}",
                (b'a' + from.0 as u8) as char,
                9 - from.1,
                (b'a' + to.0 as u8) as char,
                9 - to.1
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

            // Check for game over conditions (simplified - check if kings exist)
            self.check_game_over_simple();

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

        // Create UCI notation
        let uci_notation = format!(
            "{}{}{}{}",
            (b'a' + from.0 as u8) as char,
            9 - from.1,
            (b'a' + to.0 as u8) as char,
            9 - to.1
        );

        // Create new variation node
        let variation_node = MoveNode::new(
            from,
            to,
            uci_notation,
            new_board,
            if parent_ply.is_multiple_of(2) {
                Color::Red
            } else {
                Color::Black
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
            self.board = Board::new();
            return Ok(());
        }

        // Get the board state at the target ply
        let target_board = self.get_board_at_ply(ply);
        if let Some(board) = target_board {
            self.board = board;
            self.current_turn = if ply.is_multiple_of(2) {
                Color::Red
            } else {
                Color::Black
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
            board,
            current_turn: Color::Red,
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

    /// Simple game over check - check if kings exist
    fn check_game_over_simple(&mut self) {
        // Simplified: check if both kings still exist on the board
        let red_king_exists = self.find_king_position(Color::Red).is_some();
        let black_king_exists = self.find_king_position(Color::Black).is_some();

        if !red_king_exists {
            self.is_game_over = true;
            self.winner = Some(Color::Black);
        } else if !black_king_exists {
            self.is_game_over = true;
            self.winner = Some(Color::Red);
        }
    }

    /// Find king position (simplified)
    fn find_king_position(&self, color: Color) -> Option<(usize, usize)> {
        use crate::pieces::PieceType;
        for row in 0..10 {
            for col in 0..9 {
                if let Some((pt, c)) = self.board.get_piece_at(col, row) {
                    if pt == PieceType::King && c == color {
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
