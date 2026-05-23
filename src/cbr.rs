/// CBR/CBL file format reader for Chinese chess game records.
///
/// CBR (CCBridge Record) is a binary format for storing single Chinese chess games.
/// CBL (CCBridge Library) is a container format that stores multiple CBR games in one file.
///
/// # Format Overview
///
/// ## CBR Header (2214 bytes)
/// - Magic: `CCBridge Record\0` (16 bytes)
/// - Title, Event, Red, Black player names (UTF-16-LE encoded)
/// - Game result
/// - Board position (90 bytes, one byte per square)
///
/// ## CBL Header (576 bytes)
/// - Magic: `CCBridgeLibrary\0` (16 bytes)
/// - Library name
/// - Games start at offset 101952, each 4096 bytes aligned
use encoding_rs::UTF_16LE;
use std::io::{self, Read};

use crate::board::Board;
use crate::game::{Game, GameMetadata, MoveNode};
use crate::pieces::{PieceType, Side};

/// CBR magic bytes
const CBR_MAGIC: &[u8] = b"CCBridge Record\x00";
/// CBL magic bytes
const CBL_MAGIC: &[u8] = b"CCBridgeLibrary\x00";

/// CBR header size
const CBR_HEADER_SIZE: usize = 2214;
/// CBL header size
const CBL_HEADER_SIZE: usize = 576;
/// CBL game data start offset
const CBL_GAME_OFFSET: usize = 101952;
/// CBL game block size
const CBL_BLOCK_SIZE: usize = 4096;

/// Piece codes in CBR format (Red pieces)
const PIECE_RED_ROOK: u8 = 0x11;
const PIECE_RED_KNIGHT: u8 = 0x12;
const PIECE_RED_BISHOP: u8 = 0x13;
const PIECE_RED_ADVISOR: u8 = 0x14;
const PIECE_RED_KING: u8 = 0x15;
const PIECE_RED_CANNON: u8 = 0x16;
const PIECE_RED_PAWN: u8 = 0x17;

/// Piece codes in CBR format (Black pieces)
const PIECE_BLACK_ROOK: u8 = 0x21;
const PIECE_BLACK_KNIGHT: u8 = 0x22;
const PIECE_BLACK_BISHOP: u8 = 0x23;
const PIECE_BLACK_ADVISOR: u8 = 0x24;
const PIECE_BLACK_KING: u8 = 0x25;
const PIECE_BLACK_CANNON: u8 = 0x26;
const PIECE_BLACK_PAWN: u8 = 0x27;

/// Game result mapping
const RESULT_MAP: [&str; 5] = ["*", "1-0", "0-1", "1/2-1/2", "1/2-1/2"];

/// Decode CBR piece byte to (PieceType, Side)
fn decode_piece(byte: u8) -> Option<(PieceType, Side)> {
    match byte {
        PIECE_RED_ROOK => Some((PieceType::Rook, Side::Red)),
        PIECE_RED_KNIGHT => Some((PieceType::Knight, Side::Red)),
        PIECE_RED_BISHOP => Some((PieceType::Elephant, Side::Red)),
        PIECE_RED_ADVISOR => Some((PieceType::Advisor, Side::Red)),
        PIECE_RED_KING => Some((PieceType::King, Side::Red)),
        PIECE_RED_CANNON => Some((PieceType::Cannon, Side::Red)),
        PIECE_RED_PAWN => Some((PieceType::Pawn, Side::Red)),
        PIECE_BLACK_ROOK => Some((PieceType::Rook, Side::Black)),
        PIECE_BLACK_KNIGHT => Some((PieceType::Knight, Side::Black)),
        PIECE_BLACK_BISHOP => Some((PieceType::Elephant, Side::Black)),
        PIECE_BLACK_ADVISOR => Some((PieceType::Advisor, Side::Black)),
        PIECE_BLACK_KING => Some((PieceType::King, Side::Black)),
        PIECE_BLACK_CANNON => Some((PieceType::Cannon, Side::Black)),
        PIECE_BLACK_PAWN => Some((PieceType::Pawn, Side::Black)),
        _ => None,
    }
}

/// Decode position from CBR format
/// CBR uses: col = p % 9, row = 9 - p / 9
fn decode_pos(p: u8) -> (usize, usize) {
    let col = (p % 9) as usize;
    let row = (9 - (p / 9)) as usize;
    (col, row)
}

/// Read a null-terminated UTF-16-LE string from bytes
fn read_utf16le_str(buf: &[u8]) -> String {
    let mut end = buf.len();
    // Find null terminator (two zero bytes)
    for i in (0..buf.len().saturating_sub(1)).step_by(2) {
        if buf[i] == 0 && buf[i + 1] == 0 {
            end = i;
            break;
        }
    }
    if end == 0 {
        return String::new();
    }
    let (decoded, _, _) = UTF_16LE.decode(&buf[..end]);
    decoded.to_string()
}

/// Read a UTF-16-LE string of fixed size
fn read_fixed_utf16le_str(buf: &[u8], size: usize) -> String {
    if buf.len() < size {
        return String::new();
    }
    read_utf16le_str(&buf[..size])
}

/// CBR buffer decoder for reading steps
struct CbrDecoder {
    buffer: Vec<u8>,
    index: usize,
}

impl CbrDecoder {
    fn new(buffer: Vec<u8>) -> Self {
        CbrDecoder { buffer, index: 0 }
    }

    fn is_end(&self) -> bool {
        self.index >= self.buffer.len().saturating_sub(1)
    }

    fn read_bytes(&mut self, size: usize) -> Option<Vec<u8>> {
        let start = self.index;
        let stop = (self.index + size).min(self.buffer.len());
        self.index = stop;
        if start >= self.buffer.len() {
            None
        } else {
            Some(self.buffer[start..stop].to_vec())
        }
    }

    fn read_str(&mut self, size: usize) -> String {
        if let Some(buf) = self.read_bytes(size) {
            read_utf16le_str(&buf)
        } else {
            String::new()
        }
    }

    fn read_i32(&mut self) -> Option<i32> {
        let bytes = self.read_bytes(4)?;
        if bytes.len() < 4 {
            return None;
        }
        Some(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
}

/// Read initial annotation from the decoder
fn read_init_info(decoder: &mut CbrDecoder) -> Option<String> {
    let a_len = decoder.read_i32()?;
    if a_len == 0 {
        return Some(String::new());
    }
    let annote_len = decoder.read_i32()?;
    Some(decoder.read_str(annote_len as usize))
}

/// Build UCI notation from positions
fn build_uci_notation(from_col: usize, from_row: usize, to_col: usize, to_row: usize) -> String {
    format!(
        "{}{}{}{}",
        (b'a' + from_col as u8) as char,
        from_row,
        (b'a' + to_col as u8) as char,
        to_row
    )
}

/// Count moves in the tree from root
fn count_moves_in_tree(root_moves: &[MoveNode]) -> usize {
    if root_moves.is_empty() {
        return 0;
    }
    let mut count = 0;
    // Get the first root move and follow its main line
    let mut current = &root_moves[0];
    while let Some(ref next) = current.main_line {
        count += 1;
        current = next;
    }
    count + 1 // +1 for the first root move
}

/// Continue reading steps from the current game state (main line continuation)
fn read_steps_continuation(decoder: &mut CbrDecoder, game: &mut Game, board: &mut Board) {
    if decoder.is_end() {
        return;
    }

    let step_info = match decoder.read_bytes(4) {
        Some(bytes) => bytes,
        None => return,
    };

    if step_info.len() < 4 || step_info == [0u8; 4] {
        return;
    }

    let step_mark = step_info[0];
    let step_from = step_info[2];
    let step_to = step_info[3];

    let has_next_move = (step_mark & 0x01) == 0;
    let has_var_step = (step_mark & 0x02) != 0;
    let annote_len = if (step_mark & 0x04) != 0 {
        decoder.read_i32().unwrap_or(0) as usize
    } else {
        0
    };

    let board_bak = board.copy();

    let (col_from, row_from) = decode_pos(step_from);
    let (col_to, row_to) = decode_pos(step_to);

    let annote = if annote_len > 0 {
        let s = decoder.read_str(annote_len);
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    } else {
        None
    };

    let fen_char = board.get_fen(col_from, row_from);
    if fen_char == '.' {
        return;
    }

    let side = match Side::from_fen(fen_char) {
        Some(s) => s,
        None => return,
    };

    if !board.is_valid_move((col_from, row_from), (col_to, row_to)) {
        return;
    }

    let mut new_board = board.copy();
    new_board.make_move((col_from, row_from), (col_to, row_to));

    let uci = build_uci_notation(col_from, row_from, col_to, row_to);
    let move_number = count_moves_in_tree(&game.root_moves) as u32;

    let mut move_node = MoveNode::new(
        (col_from, row_from),
        (col_to, row_to),
        uci,
        new_board.clone(),
        side.opposite(),
        move_number,
    );
    move_node.annotation = annote;

    // Add to the end of the main line
    if let Some(last) = game.root_moves.last_mut() {
        let mut current = last;
        while current.main_line.is_some() {
            current = current.main_line.as_mut().unwrap();
        }
        current.main_line = Some(Box::new(move_node));
    } else {
        game.root_moves.push(move_node);
    }

    *board = new_board;
    game.current_turn = side.opposite();

    // First, recursively read main line continuation
    if has_next_move {
        read_steps_continuation(decoder, game, board);
    }

    // Then, read variations (branching from the backed-up board state)
    if has_var_step {
        let mut var_board = board_bak;
        read_steps_variation(decoder, game, &mut var_board);
    }
}

/// Read variation steps - adds moves as variations branching from the current position
fn read_steps_variation(decoder: &mut CbrDecoder, game: &mut Game, board: &mut Board) {
    if decoder.is_end() {
        return;
    }

    let step_info = match decoder.read_bytes(4) {
        Some(bytes) => bytes,
        None => return,
    };

    if step_info.len() < 4 || step_info == [0u8; 4] {
        return;
    }

    let step_mark = step_info[0];
    let step_from = step_info[2];
    let step_to = step_info[3];

    let has_next_move = (step_mark & 0x01) == 0;
    let has_var_step = (step_mark & 0x02) != 0;
    let annote_len = if (step_mark & 0x04) != 0 {
        decoder.read_i32().unwrap_or(0) as usize
    } else {
        0
    };

    let board_bak = board.copy();

    let (col_from, row_from) = decode_pos(step_from);
    let (col_to, row_to) = decode_pos(step_to);

    let annote = if annote_len > 0 {
        let s = decoder.read_str(annote_len);
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    } else {
        None
    };

    let fen_char = board.get_fen(col_from, row_from);
    if fen_char == '.' {
        return;
    }

    let side = match Side::from_fen(fen_char) {
        Some(s) => s,
        None => return,
    };

    if !board.is_valid_move((col_from, row_from), (col_to, row_to)) {
        return;
    }

    let mut new_board = board.copy();
    new_board.make_move((col_from, row_from), (col_to, row_to));

    let uci = build_uci_notation(col_from, row_from, col_to, row_to);
    let move_number = count_moves_in_tree(&game.root_moves) as u32;

    let mut var_node = MoveNode::new(
        (col_from, row_from),
        (col_to, row_to),
        uci,
        new_board.clone(),
        side.opposite(),
        move_number,
    );
    var_node.annotation = annote;

    // Add as variation to the last move in the main line
    if let Some(last) = game.root_moves.last_mut() {
        let mut current = last;
        while current.main_line.is_some() {
            current = current.main_line.as_mut().unwrap();
        }
        current.add_variation(var_node);
    }

    *board = new_board;
    game.current_turn = side.opposite();

    // Recursively read continuation of this variation line
    if has_next_move {
        read_steps_variation(decoder, game, board);
    }

    // Read sub-variations
    if has_var_step {
        let mut sub_var_board = board_bak;
        read_steps_variation(decoder, game, &mut sub_var_board);
    }
}

/// Read a CBR file from a byte buffer.
///
/// Returns `None` if the buffer doesn't start with the CBR magic bytes.
pub fn read_from_cbr_buffer(contents: &[u8]) -> Option<Game> {
    if contents.len() < CBR_HEADER_SIZE {
        return None;
    }

    // Check magic
    if &contents[..16] != CBR_MAGIC {
        return None;
    }

    // Parse header fields
    // struct: <16s164s128s384s64s320s64s160s64s712sB35sB3sH2s90si
    // Cumulative offsets:
    //   0: magic (16)
    //  16: _is1 (164)
    // 180: title (128)
    // 308: _is2 (384)
    // 692: event (64)
    // 756: _is3 (320)
    //1076: red (64)
    //1140: _is_red (160)
    //1300: black (64)
    //1364: _is_black (712)
    //2076: game_result (B=1)
    //2077: _is4 (35)
    //2112: move_side (B=1)
    //2113: _is5 (3)
    //2116: steps (H=2)
    //2118: _is6 (2)
    //2120: boards (90)
    //2210: _is7 (i=4)
    // Total: 2214

    let title = read_fixed_utf16le_str(&contents[180..308], 128);
    let event = read_fixed_utf16le_str(&contents[692..756], 64);
    let red = read_fixed_utf16le_str(&contents[1076..1140], 64);
    let black = read_fixed_utf16le_str(&contents[1300..1364], 64);

    let game_result_byte = contents[2076];
    let result = RESULT_MAP
        .get(game_result_byte as usize)
        .copied()
        .unwrap_or("*");

    let move_side_byte = contents[2112];
    let boards_offset = 2120;

    // Build initial board
    let mut board = Board::new();

    // Boards are stored as: boards[y*9 + x] where y is row index and x is col index
    // The piece at boards[y*9+x] is at position (x, 9-y)
    for y in 0..10u8 {
        for x in 0..9u8 {
            let idx = boards_offset + (y * 9 + x) as usize;
            if idx < contents.len() {
                let v = contents[idx];
                if let Some((piece_type, side)) = decode_piece(v) {
                    let col = x as usize;
                    let row = (9 - y) as usize;
                    board.set_piece_at(col, row, piece_type, side);
                }
            }
        }
    }

    // Build game metadata
    let mut metadata = GameMetadata::default();
    metadata.source = Some("CBR".to_string());
    if !title.is_empty() {
        metadata.title = Some(title);
    }
    if !event.is_empty() {
        metadata.event = Some(event);
    }
    if !red.is_empty() {
        metadata.red_player = Some(red);
    }
    if !black.is_empty() {
        metadata.black_player = Some(black);
    }
    metadata.result = Some(result.to_string());

    let mut game = Game::from_board(board.clone());
    game.metadata = metadata;
    game.current_turn = if move_side_byte == 1 {
        Side::Red
    } else {
        Side::Black
    };

    // Read steps from offset 2214
    if contents.len() > CBR_HEADER_SIZE {
        let steps_data = contents[CBR_HEADER_SIZE..].to_vec();
        let mut decoder = CbrDecoder::new(steps_data);

        // Read initial annotation
        let _game_annotation = read_init_info(&mut decoder);

        if !decoder.is_end() {
            let mut working_board = board.copy();
            read_steps_continuation(&mut decoder, &mut game, &mut working_board);
        }
    }

    Some(game)
}

/// Read a CBR file from disk.
pub fn read_from_cbr(path: &str) -> io::Result<Option<Game>> {
    let mut file = std::fs::File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(read_from_cbr_buffer(&contents))
}

/// Read a CBL library file and return a list of games.
///
/// CBL files contain multiple CBR records, each 4096 bytes aligned.
pub fn read_from_cbl(path: &str) -> io::Result<Option<CblLibrary>> {
    let mut file = std::fs::File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(read_from_cbl_buffer(&contents))
}

/// Read a CBL library from a byte buffer.
pub fn read_from_cbl_buffer(contents: &[u8]) -> Option<CblLibrary> {
    if contents.len() < CBL_HEADER_SIZE {
        return None;
    }

    // Check magic
    if &contents[..16] != CBL_MAGIC {
        return None;
    }

    // Parse library name (offset 64, size 512)
    // struct: <16s44si512s
    // magic:16, _i1:44, book_count:4(i), lib_name:512
    let lib_name = read_fixed_utf16le_str(&contents[64..576], 512);

    let mut library = CblLibrary {
        name: lib_name,
        games: Vec::new(),
    };

    // Game data starts at offset 101952
    if contents.len() <= CBL_GAME_OFFSET {
        return Some(library);
    }

    let game_buffer = &contents[CBL_GAME_OFFSET..];

    // Find first CBR magic in the game buffer
    let mut search_start = 0;
    let mut found = false;
    for i in 0..game_buffer.len().saturating_sub(CBR_MAGIC.len()) {
        if &game_buffer[i..i + CBR_MAGIC.len()] == CBR_MAGIC {
            search_start = i;
            found = true;
            break;
        }
    }

    if !found {
        return Some(library);
    }

    let remaining = &game_buffer[search_start..];
    let block_count = remaining.len() / CBL_BLOCK_SIZE;

    for i in 0..block_count {
        let offset = i * CBL_BLOCK_SIZE;
        let block = &remaining[offset..];

        // Check if this block has CBR magic
        if block.len() < CBR_MAGIC.len() || &block[..CBR_MAGIC.len()] != CBR_MAGIC {
            continue;
        }

        if let Some(game) = read_from_cbr_buffer(block) {
            library.games.push(game);
        }
    }

    Some(library)
}

/// A CBL library containing multiple games.
pub struct CblLibrary {
    pub name: String,
    pub games: Vec<Game>,
}

impl CblLibrary {
    /// Create a new empty library.
    pub fn new(name: &str) -> Self {
        CblLibrary {
            name: name.to_string(),
            games: Vec::new(),
        }
    }

    /// Get the number of games in the library.
    pub fn game_count(&self) -> usize {
        self.games.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_piece() {
        assert_eq!(decode_piece(0x11), Some((PieceType::Rook, Side::Red)));
        assert_eq!(decode_piece(0x15), Some((PieceType::King, Side::Red)));
        assert_eq!(decode_piece(0x21), Some((PieceType::Rook, Side::Black)));
        assert_eq!(decode_piece(0x25), Some((PieceType::King, Side::Black)));
        assert_eq!(decode_piece(0x00), None);
        assert_eq!(decode_piece(0xFF), None);
    }

    #[test]
    fn test_decode_pos() {
        // Test position decoding: col = p % 9, row = 9 - p / 9
        assert_eq!(decode_pos(0), (0, 9)); // bottom-left
        assert_eq!(decode_pos(8), (8, 9)); // bottom-right
        assert_eq!(decode_pos(81), (0, 0)); // top-left
        assert_eq!(decode_pos(89), (8, 0)); // top-right
    }

    #[test]
    fn test_read_utf16le_str() {
        // "Hello" in UTF-16-LE with null terminator
        let buf: Vec<u8> = vec![
            0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00, 0x00, 0x00,
        ];
        assert_eq!(read_utf16le_str(&buf), "Hello");

        // Empty string
        let empty: Vec<u8> = vec![0x00, 0x00];
        assert_eq!(read_utf16le_str(&empty), "");
    }

    #[test]
    fn test_read_cbr_magic_check() {
        // Buffer too short
        assert!(read_from_cbr_buffer(&[0u8; 100]).is_none());

        // Wrong magic
        let mut buf = vec![0u8; CBR_HEADER_SIZE + 100];
        buf[..16].copy_from_slice(b"Wrong Magic\x00\x00\x00\x00\x00");
        assert!(read_from_cbr_buffer(&buf).is_none());
    }

    #[test]
    fn test_read_cbr_file() {
        // Test reading the test.cbr file
        let game = read_from_cbr("tests/data/test.cbr");
        match game {
            Ok(Some(g)) => {
                assert!(!g.root_moves.is_empty());
                assert!(g.metadata.title.is_some());
            }
            Ok(None) => panic!("Failed to parse test.cbr - returned None"),
            Err(e) => panic!("Failed to read test.cbr: {}", e),
        }
    }

    #[test]
    fn test_read_cbr_file2() {
        let game = read_from_cbr("tests/data/test2.cbr");
        match game {
            Ok(Some(g)) => {
                assert!(!g.root_moves.is_empty());
            }
            Ok(None) => panic!("Failed to parse test2.cbr - returned None"),
            Err(e) => panic!("Failed to read test2.cbr: {}", e),
        }
    }

    #[test]
    fn test_read_cbr_board_position() {
        let game = read_from_cbr("tests/data/test.cbr");
        if let Ok(Some(g)) = game {
            // The test.cbr starts with initial position
            let board = &g.board;
            // Check Red king (K at col 4, row 0)
            let fen = board.get_fen(4, 0);
            assert_eq!(fen, 'K', "Red king should be at (4, 0), got '{}'", fen);
            // Check Black king (k at col 4, row 9)
            let fen = board.get_fen(4, 9);
            assert_eq!(fen, 'k', "Black king should be at (4, 9), got '{}'", fen);
        }
    }

    #[test]
    fn test_read_cbr_moves() {
        let game = read_from_cbr("tests/data/test.cbr");
        if let Ok(Some(g)) = game {
            // Count main line moves only (not variations)
            let main_line_len = if let Some(first) = g.root_moves.first() {
                first.get_main_line().len()
            } else {
                0
            };
            // test.cbr should have 11 moves in main line
            assert!(
                main_line_len >= 11,
                "Expected at least 11 main line moves, got {}",
                main_line_len
            );

            // Check first move: (7, 2) -> (4, 2) which is 炮二平五
            if let Some(first) = g.root_moves.first() {
                assert_eq!(first.from, (7, 2), "First move from position mismatch");
                assert_eq!(first.to, (4, 2), "First move to position mismatch");
            }
        }
    }

    #[test]
    fn test_read_cbr_moves2() {
        let game = read_from_cbr("tests/data/test2.cbr");
        if let Ok(Some(g)) = game {
            let move_count = count_moves_in_tree(&g.root_moves);
            assert_eq!(move_count, 2, "Expected 2 moves, got {}", move_count);
        }
    }
}
