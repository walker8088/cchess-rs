//! XQF (Xiangqi File) format support
//!
//! This module provides functionality to read and write XQF files,
//! which are binary files used to store Chinese chess game records.
//!
//! Supports XQF version 1.0+ including multi-branch (variation) support
//! introduced in version 1.1 and above.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::board::Board;
use crate::game::Game;
use crate::pieces::{PieceType, Side};

// XQF format constants (from Python implementation)
const XQF_HEADER_SIZE: usize = 0x400; // 1024 bytes
const XQF_MOVE_FROM_OFFSET: u8 = 0x18;
const XQF_MOVE_TO_OFFSET: u8 = 0x20;
const XQF_STEP_FLAG_MASK: u8 = 0xE0;
const XQF_STEP_HAS_ANNO: u8 = 0x20;
const XQF_STEP_HAS_NEXT: u8 = 0x80;
const XQF_STEP_HAS_VAR: u8 = 0x40;

// Chess piece kinds in XQF order (16 pieces per side)
#[allow(dead_code)]
const CHESSMAN_KINDS: &str = "RNBAKABNRCCPPPPP";

// Game result mapping
const GAME_RESULT_MAP: [&str; 5] = ["*", "1-0", "0-1", "1-0", "*"];

/// XQF file header structure
#[derive(Debug, Clone)]
pub struct XqfHeader {
    /// File signature: "XQ$!"
    pub signature: [u8; 4],
    /// File version
    pub version: u16,
    /// File size
    pub file_size: u32,
    /// Game information offset
    pub game_info_offset: u32,
    /// Move data offset
    pub move_data_offset: u32,
    /// Reserved bytes
    pub reserved: [u8; 20],
}

/// XQF game information
#[derive(Debug, Clone)]
pub struct XqfGameInfo {
    /// Game title (max 64 bytes)
    pub title: String,
    /// Red player name (max 16 bytes)
    pub red_player: String,
    /// Black player name (max 16 bytes)
    pub black_player: String,
    /// Game time (in minutes)
    pub game_time: u16,
    /// Game date (YYYYMMDD)
    pub game_date: u32,
    /// Game result (0: unknown, 1: red win, 2: black win, 3: draw)
    pub result: u8,
    /// Game level (0-9)
    pub level: u8,
    /// Reserved bytes
    pub reserved: [u8; 110],
}

/// XQF move data
#[derive(Debug, Clone)]
pub struct XqfMove {
    /// From position (0-89)
    pub from: u8,
    /// To position (0-89)
    pub to: u8,
    /// Piece type at the destination
    pub piece_type: u8,
    /// Move flags
    pub flags: u8,
    /// Reserved bytes
    pub reserved: [u8; 2],
}

/// XQF move node for tree structure (supports variations)
#[derive(Debug, Clone)]
pub struct XqfMoveNode {
    /// The move at this node
    pub move_data: XqfMoveData,
    /// Annotation/comment for this move
    pub annotation: Option<String>,
    /// Main line continuation (next move in the primary variation)
    pub main_line: Option<Box<XqfMoveNode>>,
    /// Alternative variations (branches)
    pub variations: Vec<XqfMoveNode>,
}

/// Decoded move data from XQF format
#[derive(Debug, Clone)]
pub struct XqfMoveData {
    /// From position (0-89)
    pub from: u8,
    /// To position (0-89)
    pub to: u8,
    /// Piece type (FEN character)
    pub piece: Option<char>,
}

/// XQF decryption keys structure
#[derive(Debug, Clone)]
pub struct XqfKeys {
    /// Position encryption factor
    pub key_xy: u8,
    /// Move start position encryption factor
    pub key_xyf: u8,
    /// Move end position encryption factor
    pub key_xyt: u8,
    /// Annotation size encryption factor
    pub key_rmk_size: u16,
    /// Key bytes for decryption
    pub f_key_bytes: (u8, u8, u8, u8),
    /// 32-byte key array for step decryption
    pub f32_keys: Vec<u8>,
}

/// XQF game information (extended for multi-branch)
#[derive(Debug, Clone)]
pub struct XqfGameInfoExtended {
    /// Game title
    pub title: Option<String>,
    /// Red player name
    pub red_player: Option<String>,
    /// Black player name
    pub black_player: Option<String>,
    /// Event/match name
    pub event: Option<String>,
    /// Game result ("1-0", "0-1", "1/2-1/2", "*")
    pub result: String,
    /// XQF version
    pub version: u8,
    /// Game type
    pub game_type: u8,
    /// Source format
    pub source: String,
    /// Number of branches/variations
    pub branches: u32,
}

/// Complete XQF file with multi-branch support
#[derive(Debug, Clone)]
pub struct XqfFileWithVariations {
    /// Game information
    pub game_info: XqfGameInfoExtended,
    /// Initial board position
    pub initial_board: Board,
    /// Root move node (first move of the game)
    pub root_moves: Vec<XqfMoveNode>,
    /// XQF version
    pub version: u8,
    /// Whether the file was encrypted
    pub was_encrypted: bool,
}

/// XQF file structure
#[derive(Debug, Clone)]
pub struct XqfFile {
    /// File header
    pub header: XqfHeader,
    /// Game information
    pub game_info: XqfGameInfo,
    /// Moves
    pub moves: Vec<XqfMove>,
    /// Initial board state (optional)
    pub initial_board: Option<Board>,
}

/// Errors that can occur when reading/writing XQF files
#[derive(Debug)]
pub enum XqfError {
    /// I/O error
    Io(io::Error),
    /// Invalid file signature
    InvalidSignature,
    /// Invalid file version
    InvalidVersion,
    /// Invalid move data
    InvalidMoveData,
    /// Unsupported feature
    Unsupported,
    /// Other error with message
    Other(String),
}

impl std::fmt::Display for XqfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XqfError::Io(e) => write!(f, "I/O error: {}", e),
            XqfError::InvalidSignature => write!(f, "Invalid XQF file signature"),
            XqfError::InvalidVersion => write!(f, "Invalid XQF file version"),
            XqfError::InvalidMoveData => write!(f, "Invalid move data"),
            XqfError::Unsupported => write!(f, "Unsupported feature"),
            XqfError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<io::Error> for XqfError {
    fn from(err: io::Error) -> Self {
        XqfError::Io(err)
    }
}

impl Default for XqfHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl XqfHeader {
    /// Create a new XQF header
    pub fn new() -> Self {
        XqfHeader {
            signature: [b'X', b'Q', b'$', b'!'],
            version: 0x0100, // Version 1.0
            file_size: 0,
            game_info_offset: 0,
            move_data_offset: 0,
            reserved: [0; 20],
        }
    }

    /// Read header from file
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, XqfError> {
        let mut signature = [0u8; 4];
        reader.read_exact(&mut signature)?;

        if &signature != b"XQ$!" {
            return Err(XqfError::InvalidSignature);
        }

        let version = reader.read_u16::<LittleEndian>()?;
        let file_size = reader.read_u32::<LittleEndian>()?;
        let game_info_offset = reader.read_u32::<LittleEndian>()?;
        let move_data_offset = reader.read_u32::<LittleEndian>()?;

        let mut reserved = [0u8; 20];
        reader.read_exact(&mut reserved)?;

        Ok(XqfHeader {
            signature,
            version,
            file_size,
            game_info_offset,
            move_data_offset,
            reserved,
        })
    }

    /// Write header to file
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), XqfError> {
        writer.write_all(&self.signature)?;
        writer.write_u16::<LittleEndian>(self.version)?;
        writer.write_u32::<LittleEndian>(self.file_size)?;
        writer.write_u32::<LittleEndian>(self.game_info_offset)?;
        writer.write_u32::<LittleEndian>(self.move_data_offset)?;
        writer.write_all(&self.reserved)?;

        Ok(())
    }
}

impl Default for XqfGameInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl XqfGameInfo {
    /// Create new game info
    pub fn new() -> Self {
        XqfGameInfo {
            title: String::new(),
            red_player: String::new(),
            black_player: String::new(),
            game_time: 0,
            game_date: 0,
            result: 0,
            level: 0,
            reserved: [0; 110],
        }
    }

    /// Read game info from file
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, XqfError> {
        let mut title_buf = [0u8; 64];
        reader.read_exact(&mut title_buf)?;
        let title = String::from_utf8_lossy(&title_buf)
            .trim_end_matches('\0')
            .to_string();

        let mut red_player_buf = [0u8; 16];
        reader.read_exact(&mut red_player_buf)?;
        let red_player = String::from_utf8_lossy(&red_player_buf)
            .trim_end_matches('\0')
            .to_string();

        let mut black_player_buf = [0u8; 16];
        reader.read_exact(&mut black_player_buf)?;
        let black_player = String::from_utf8_lossy(&black_player_buf)
            .trim_end_matches('\0')
            .to_string();

        let game_time = reader.read_u16::<LittleEndian>()?;
        let game_date = reader.read_u32::<LittleEndian>()?;
        let result = reader.read_u8()?;
        let level = reader.read_u8()?;

        let mut reserved = [0u8; 110];
        reader.read_exact(&mut reserved)?;

        Ok(XqfGameInfo {
            title,
            red_player,
            black_player,
            game_time,
            game_date,
            result,
            level,
            reserved,
        })
    }

    /// Write game info to file
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), XqfError> {
        let mut title_buf = [0u8; 64];
        let title_bytes = self.title.as_bytes();
        let len = title_bytes.len().min(63);
        title_buf[..len].copy_from_slice(&title_bytes[..len]);
        writer.write_all(&title_buf)?;

        let mut red_player_buf = [0u8; 16];
        let red_player_bytes = self.red_player.as_bytes();
        let len = red_player_bytes.len().min(15);
        red_player_buf[..len].copy_from_slice(&red_player_bytes[..len]);
        writer.write_all(&red_player_buf)?;

        let mut black_player_buf = [0u8; 16];
        let black_player_bytes = self.black_player.as_bytes();
        let len = black_player_bytes.len().min(15);
        black_player_buf[..len].copy_from_slice(&black_player_bytes[..len]);
        writer.write_all(&black_player_buf)?;

        writer.write_u16::<LittleEndian>(self.game_time)?;
        writer.write_u32::<LittleEndian>(self.game_date)?;
        writer.write_u8(self.result)?;
        writer.write_u8(self.level)?;
        writer.write_all(&self.reserved)?;

        Ok(())
    }
}

impl XqfMove {
    /// Create a new move
    pub fn new(from: u8, to: u8, piece_type: u8, flags: u8) -> Self {
        XqfMove {
            from,
            to,
            piece_type,
            flags,
            reserved: [0; 2],
        }
    }

    /// Read a move from file
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, XqfError> {
        let from = reader.read_u8()?;
        let to = reader.read_u8()?;
        let piece_type = reader.read_u8()?;
        let flags = reader.read_u8()?;
        let mut reserved = [0u8; 2];
        reader.read_exact(&mut reserved)?;

        Ok(XqfMove {
            from,
            to,
            piece_type,
            flags,
            reserved,
        })
    }

    /// Write a move to file
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), XqfError> {
        writer.write_u8(self.from)?;
        writer.write_u8(self.to)?;
        writer.write_u8(self.piece_type)?;
        writer.write_u8(self.flags)?;
        writer.write_all(&self.reserved)?;

        Ok(())
    }

    /// Convert position index to coordinates
    pub fn to_coordinates(pos: u8) -> (usize, usize) {
        let row = (pos / 9) as usize;
        let col = (pos % 9) as usize;
        (col, row)
    }

    /// Convert coordinates to position index
    pub fn from_coordinates(col: usize, row: usize) -> u8 {
        (row * 9 + col) as u8
    }
}

impl XqfFile {
    /// Read an XQF file from a path
    pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<Self, XqfError> {
        let mut file = File::open(path)?;
        Self::read_from_reader(&mut file)
    }

    /// Read an XQF file from a reader
    pub fn read_from_reader<R: Read + Seek>(reader: &mut R) -> Result<Self, XqfError> {
        // Read header
        let header = XqfHeader::read(reader)?;

        // Seek to game info
        reader.seek(SeekFrom::Start(header.game_info_offset as u64))?;
        let game_info = XqfGameInfo::read(reader)?;

        // Seek to move data
        reader.seek(SeekFrom::Start(header.move_data_offset as u64))?;

        // Read moves until end of file or zero move
        let mut moves = Vec::new();
        loop {
            let move_data = XqfMove::read(reader);
            match move_data {
                Ok(mv) => {
                    // Check for end marker (all zeros)
                    if mv.from == 0 && mv.to == 0 && mv.piece_type == 0 && mv.flags == 0 {
                        break;
                    }
                    moves.push(mv);
                }
                Err(_) => break, // End of file or error
            }
        }

        // Try to read initial board if available
        let initial_board = if header.game_info_offset > 256 {
            // There might be an initial board after the header
            reader.seek(SeekFrom::Start(256))?;
            let mut board_buf = [0u8; 90];
            if reader.read_exact(&mut board_buf).is_ok() {
                match board_from_xqf(&board_buf) {
                    Ok(board) => Some(board),
                    Err(e) => return Err(e),
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(XqfFile {
            header,
            game_info,
            moves,
            initial_board,
        })
    }

    /// Write XQF file to a path
    pub fn write_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), XqfError> {
        let mut file = File::create(path)?;
        self.write_to_writer(&mut file)
    }

    /// Write XQF file to a writer
    pub fn write_to_writer<W: Write + Seek>(&self, writer: &mut W) -> Result<(), XqfError> {
        // Calculate offsets
        let header_size = 44; // 4 + 2 + 4 + 4 + 4 + 20 = 44 bytes
        let game_info_size = 200; // 64 + 16 + 16 + 2 + 4 + 1 + 1 + 110 = 200 bytes
        let move_size = 6; // 1 + 1 + 1 + 1 + 2 = 6 bytes per move

        // Update header with calculated values
        let mut header = self.header.clone();
        header.game_info_offset = header_size as u32;
        header.move_data_offset = (header_size + game_info_size) as u32;
        header.file_size =
            (header_size + game_info_size + (self.moves.len() * move_size) + 6) as u32; // +6 for end marker

        // Write header
        header.write(writer)?;

        // Write initial board if available
        if let Some(board) = &self.initial_board {
            match board_to_xqf(board) {
                Ok(board_buf) => {
                    writer.write_all(&board_buf)?;
                    // Pad to game info offset
                    let current_pos = header_size as u64 + 90;
                    if current_pos < header.game_info_offset as u64 {
                        let padding =
                            vec![0u8; (header.game_info_offset as u64 - current_pos) as usize];
                        writer.write_all(&padding)?;
                    }
                }
                Err(e) => return Err(e),
            }
        } else {
            // Pad to game info offset
            let padding = vec![0u8; (header.game_info_offset as u64 - header_size as u64) as usize];
            writer.write_all(&padding)?;
        }

        // Write game info
        writer.seek(SeekFrom::Start(header.game_info_offset as u64))?;
        self.game_info.write(writer)?;

        // Write moves
        writer.seek(SeekFrom::Start(header.move_data_offset as u64))?;
        for mv in &self.moves {
            mv.write(writer)?;
        }

        // Write end marker
        let end_marker = XqfMove::new(0, 0, 0, 0);
        end_marker.write(writer)?;

        Ok(())
    }

    /// Convert XQF file to Game
    pub fn to_game(&self) -> Result<Game, XqfError> {
        let game = if let Some(board) = &self.initial_board {
            Game::from_board(board.clone())
        } else {
            Game::new()
        };

        // Apply moves
        for mv in &self.moves {
            let (_from_col, _from_row) = XqfMove::to_coordinates(mv.from);
            let (_to_col, _to_row) = XqfMove::to_coordinates(mv.to);

            // Convert piece type from XQF format
            let _piece_type = match mv.piece_type {
                1 => PieceType::King,
                2 => PieceType::Advisor,
                3 => PieceType::Elephant,
                4 => PieceType::Knight,
                5 => PieceType::Rook,
                6 => PieceType::Cannon,
                7 => PieceType::Pawn,
                _ => return Err(XqfError::InvalidMoveData),
            };

            // TODO: Apply move to game
            // This would require a method to make a move from coordinates
        }

        Ok(game)
    }

    /// Create XQF file from Game
    pub fn from_game(
        game: &Game,
        title: &str,
        red_player: &str,
        black_player: &str,
    ) -> Result<Self, XqfError> {
        let header = XqfHeader::new();
        let mut game_info = XqfGameInfo::new();

        game_info.title = title.to_string();
        game_info.red_player = red_player.to_string();
        game_info.black_player = black_player.to_string();
        game_info.game_date = Self::current_date()?;

        // TODO: Extract moves from game and convert to XqfMove format
        let moves = Vec::new();

        Ok(XqfFile {
            header,
            game_info,
            moves,
            initial_board: Some(game.get_board().clone()),
        })
    }

    /// Get current date in YYYYMMDD format
    fn current_date() -> Result<u32, XqfError> {
        use chrono::prelude::*;
        let now = Local::now();
        let date = (now.year() as u32) * 10000 + (now.month() as u32) * 100 + (now.day() as u32);
        Ok(date)
    }
}

// XQF-specific conversion functions (not part of Board to maintain generality)

/// Convert board to XQF format byte array (90 bytes)
pub fn board_to_xqf(board: &Board) -> Result<[u8; 90], XqfError> {
    let mut data = [0u8; 90];

    // XQF format: row 0 = Black's back rank, row 9 = Red's back rank
    // Internal format: row 0 = Red's back rank, row 9 = Black's back rank
    // So we need to flip rows: xqf_row = 9 - internal_row
    for row in 0..10 {
        for col in 0..9 {
            let xqf_row = 9 - row;
            let index = xqf_row * 9 + col;

            if let Some((piece_type, side)) = board.get_piece_at(col, row) {
                let code = match (piece_type, side) {
                    (PieceType::King, Side::Black) => 1,
                    (PieceType::Advisor, Side::Black) => 2,
                    (PieceType::Elephant, Side::Black) => 3,
                    (PieceType::Knight, Side::Black) => 4,
                    (PieceType::Rook, Side::Black) => 5,
                    (PieceType::Cannon, Side::Black) => 6,
                    (PieceType::Pawn, Side::Black) => 7,
                    (PieceType::King, Side::Red) => 9,
                    (PieceType::Advisor, Side::Red) => 10,
                    (PieceType::Elephant, Side::Red) => 11,
                    (PieceType::Knight, Side::Red) => 12,
                    (PieceType::Rook, Side::Red) => 13,
                    (PieceType::Cannon, Side::Red) => 14,
                    (PieceType::Pawn, Side::Red) => 15,
                    _ => 0,
                };
                data[index] = code;
            }
        }
    }

    Ok(data)
}

/// Create board from XQF format byte array (90 bytes)
pub fn board_from_xqf(data: &[u8; 90]) -> Result<Board, XqfError> {
    let mut board = Board::new();
    board.clear();

    for i in 0..90 {
        let piece_code = data[i];
        if piece_code == 0 {
            continue;
        }

        let (col, row) = {
            let xqf_row = i / 9;
            let col = i % 9;
            // XQF row 0 = Black's back rank, internal row 0 = Red's back rank
            let internal_row = 9 - xqf_row;
            (col, internal_row)
        };

        let (piece_type, side) = match piece_code {
            1 => (PieceType::King, Side::Black),
            2 => (PieceType::Advisor, Side::Black),
            3 => (PieceType::Elephant, Side::Black),
            4 => (PieceType::Knight, Side::Black),
            5 => (PieceType::Rook, Side::Black),
            6 => (PieceType::Cannon, Side::Black),
            7 => (PieceType::Pawn, Side::Black),
            9 => (PieceType::King, Side::Red),
            10 => (PieceType::Advisor, Side::Red),
            11 => (PieceType::Elephant, Side::Red),
            12 => (PieceType::Knight, Side::Red),
            13 => (PieceType::Rook, Side::Red),
            14 => (PieceType::Cannon, Side::Red),
            15 => (PieceType::Pawn, Side::Red),
            _ => return Err(XqfError::InvalidMoveData),
        };

        board.set_piece_at(col, row, piece_type, side);
    }

    Ok(board)
}

// =============================================================================
// XQF 1.1+ Multi-branch Support
// =============================================================================

/// Decode XQF position to (col, row) coordinates
fn xqf_decode_pos(man_pos: u8) -> (u8, u8) {
    (man_pos / 10, man_pos % 10)
}

/// XQFKey structure for decryption
#[derive(Debug, Clone)]
struct XQFKey {
    key_xy: u8,
    key_xyf: u8,
    key_xyt: u8,
    key_rmk_size: u16,
    f_key_bytes: (u8, u8, u8, u8),
    f32_keys: Vec<u8>,
}

/// Initialize decryption keys
fn init_decrypt_key(buff_str: &[u8]) -> Result<XQFKey, XqfError> {
    if buff_str.len() < 13 {
        return Err(XqfError::Other("Key buffer too small".into()));
    }

    let head_key_mask = buff_str[0] as u32;
    let head_key_or_a = buff_str[5];
    let head_key_or_b = buff_str[6];
    let head_key_or_c = buff_str[7];
    let head_key_or_d = buff_str[8];
    let head_keys_sum = buff_str[9];
    let head_key_xy = buff_str[10];
    let head_key_xyf = buff_str[11];
    let head_key_xyt = buff_str[12];

    let mut keys = XQFKey {
        key_xy: 0,
        key_xyf: 0,
        key_xyt: 0,
        key_rmk_size: 0,
        f_key_bytes: (0, 0, 0, 0),
        f32_keys: vec![0; 32],
    };

    // Position encryption factor
    let b_key = head_key_xy as u32;
    keys.key_xy = (((((((b_key * b_key) * 3 + 9) * 3 + 8) * 2 + 1) * 3 + 8) * b_key) & 0xFF) as u8;

    // Move start encryption factor
    let b_key = head_key_xyf as u32;
    keys.key_xyf = (((((((b_key * b_key) * 3 + 9) * 3 + 8) * 2 + 1) * 3 + 8) * keys.key_xy as u32)
        & 0xFF) as u8;

    // Move end encryption factor
    let b_key = head_key_xyt as u32;
    keys.key_xyt = (((((((b_key * b_key) * 3 + 9) * 3 + 8) * 2 + 1) * 3 + 8) * keys.key_xyf as u32)
        & 0xFF) as u8;

    // Annotation size encryption factor
    let w_key = (head_keys_sum as u16) * 256 + keys.key_xy as u16;
    keys.key_rmk_size = ((w_key % 32000) + 767) & 0xFFFF;

    let b1 = ((head_keys_sum as u32 & head_key_mask) | head_key_or_a as u32) as u8;
    let b2 = ((keys.key_xy as u32 & head_key_mask) | head_key_or_b as u32) as u8;
    let b3 = ((keys.key_xyf as u32 & head_key_mask) | head_key_or_c as u32) as u8;
    let b4 = ((keys.key_xyt as u32 & head_key_mask) | head_key_or_d as u32) as u8;

    keys.f_key_bytes = (b1, b2, b3, b4);

    // Initialize F32Keys
    let base = b"[(C) Copyright Mr. Dong Shiwei.]";
    keys.f32_keys = base.iter().map(|&b| b).collect::<Vec<u8>>();
    for i in 0..keys.f32_keys.len() {
        let key_byte = match i % 4 {
            0 => keys.f_key_bytes.0,
            1 => keys.f_key_bytes.1,
            2 => keys.f_key_bytes.2,
            _ => keys.f_key_bytes.3,
        };
        keys.f32_keys[i] &= key_byte;
    }

    Ok(keys)
}

/// Decrypt step buffer
fn decode_xqf_buff(keys: &XQFKey, buff: &[u8]) -> Vec<u8> {
    let mut de_buff = buff.to_vec();
    let n_pos: usize = 0x400;

    for i in 0..buff.len() {
        let key_byte = keys.f32_keys[(n_pos + i) % 32];
        de_buff[i] = de_buff[i].wrapping_sub(key_byte);
    }

    de_buff
}

/// Initialize chess board from XQF man positions
fn init_chess_board(man_str: &[u8], version: u8, keys: Option<&XQFKey>) -> [u8; 32] {
    let mut tmp_man = [0xFFu8; 32];

    if let Some(keys) = keys {
        for i in 0..32 {
            if version >= 12 {
                let idx = ((keys.key_xy as usize + i + 1) & 0x1F) as usize;
                tmp_man[idx] = man_str[i];
            } else {
                tmp_man[i] = man_str[i];
            }
        }

        for i in 0..32 {
            tmp_man[i] = tmp_man[i].wrapping_sub(keys.key_xy);
            if tmp_man[i] > 89 {
                tmp_man[i] = 0xFF;
            }
        }
    } else {
        for i in 0..32 {
            if i < man_str.len() {
                tmp_man[i] = man_str[i];
            }
        }
    }

    tmp_man
}

/// Buffer decoder for reading step data
struct XQFBuffDecoder {
    buffer: Vec<u8>,
    index: usize,
}

impl XQFBuffDecoder {
    fn new(buffer: Vec<u8>) -> Self {
        XQFBuffDecoder { buffer, index: 0 }
    }

    fn read_bytes(&mut self, size: usize) -> Vec<u8> {
        let start = self.index;
        let stop = std::cmp::min(self.index + size, self.buffer.len());
        self.index = stop;
        self.buffer[start..stop].to_vec()
    }

    fn read_str(&mut self, size: usize) -> Option<String> {
        let buff = self.read_bytes(size);
        String::from_utf8(buff).ok()
    }

    fn read_int(&mut self) -> u32 {
        let data = self.read_bytes(4);
        if data.len() < 4 {
            return 0;
        }
        data[0] as u32
            + ((data[1] as u32) << 8)
            + ((data[2] as u32) << 16)
            + ((data[3] as u32) << 24)
    }

    #[allow(dead_code)]
    fn has_data(&self) -> bool {
        self.index < self.buffer.len()
    }
}

/// Build initial board from chess mans
fn build_xqf_board(chess_mans: &[u8; 32]) -> Board {
    let mut board = Board::new();
    let chessman_kinds = "RNBAKABNRCCPPPPP";
    let kinds: Vec<char> = chessman_kinds.chars().collect();

    for side_idx in 0..2 {
        for man_index in 0..16 {
            let man_pos = chess_mans[side_idx * 16 + man_index];
            if man_pos == 0xFF {
                continue;
            }

            let (col, row) = xqf_decode_pos(man_pos);
            let fen_ch = kinds[man_index];

            let piece_side = match fen_ch {
                'R' => (
                    PieceType::Rook,
                    if side_idx == 0 {
                        Side::Black
                    } else {
                        Side::Red
                    },
                ),
                'N' => (
                    PieceType::Knight,
                    if side_idx == 0 {
                        Side::Black
                    } else {
                        Side::Red
                    },
                ),
                'B' => (
                    PieceType::Advisor,
                    if side_idx == 0 {
                        Side::Black
                    } else {
                        Side::Red
                    },
                ),
                'A' => (
                    PieceType::Advisor,
                    if side_idx == 0 {
                        Side::Black
                    } else {
                        Side::Red
                    },
                ),
                'K' => (
                    PieceType::King,
                    if side_idx == 0 {
                        Side::Black
                    } else {
                        Side::Red
                    },
                ),
                'C' => (
                    PieceType::Cannon,
                    if side_idx == 0 {
                        Side::Black
                    } else {
                        Side::Red
                    },
                ),
                'P' => (
                    PieceType::Pawn,
                    if side_idx == 0 {
                        Side::Black
                    } else {
                        Side::Red
                    },
                ),
                _ => continue,
            };
            let (piece_type, piece_side) = piece_side;

            board.set_piece_at(col as usize, row as usize, piece_type, piece_side);
        }
    }

    board
}

/// Parse step info for low version (<= 0x0A)
fn parse_step_info_low_version(
    step_info: &mut [u8],
    buff_decoder: &mut XQFBuffDecoder,
) -> (bool, bool, u32) {
    let has_next_step = (step_info[2] & 0xF0) != 0;
    let has_var_step = (step_info[2] & 0x0F) != 0;
    let annote_len = buff_decoder.read_int();

    step_info[0] = step_info[0].wrapping_sub(XQF_MOVE_FROM_OFFSET);
    step_info[1] = step_info[1].wrapping_sub(XQF_MOVE_TO_OFFSET);

    (has_next_step, has_var_step, annote_len)
}

/// Parse step info for high version (> 0x0A)
fn parse_step_info_high_version(
    step_info: &mut [u8],
    buff_decoder: &mut XQFBuffDecoder,
    keys: &XQFKey,
) -> (bool, bool, u32) {
    step_info[2] &= XQF_STEP_FLAG_MASK;
    let has_next_step = (step_info[2] & XQF_STEP_HAS_NEXT) != 0;
    let has_var_step = (step_info[2] & XQF_STEP_HAS_VAR) != 0;
    let mut annote_len = 0u32;

    if (step_info[2] & XQF_STEP_HAS_ANNO) != 0 {
        annote_len = buff_decoder
            .read_int()
            .wrapping_sub(keys.key_rmk_size as u32);
    }

    step_info[0] = step_info[0]
        .wrapping_sub(XQF_MOVE_FROM_OFFSET)
        .wrapping_sub(keys.key_xyf);
    step_info[1] = step_info[1]
        .wrapping_sub(XQF_MOVE_TO_OFFSET)
        .wrapping_sub(keys.key_xyt);

    (has_next_step, has_var_step, annote_len)
}

/// Recursively read steps and build move tree
fn read_steps(
    buff_decoder: &mut XQFBuffDecoder,
    version: u8,
    keys: Option<&XQFKey>,
    board: &Board,
    branches: &mut u32,
) -> Option<XqfMoveNode> {
    let step_info_bytes = buff_decoder.read_bytes(4);
    if step_info_bytes.len() < 4 {
        return None;
    }

    let mut step_info = step_info_bytes;
    let board_bak = board.clone();

    let (has_next_step, has_var_step, annote_len) = if version <= 0x0A {
        parse_step_info_low_version(&mut step_info, buff_decoder)
    } else {
        if let Some(k) = keys {
            parse_step_info_high_version(&mut step_info, buff_decoder, k)
        } else {
            return None;
        }
    };

    let (from_col, from_row) = xqf_decode_pos(step_info[0]);
    let (to_col, to_row) = xqf_decode_pos(step_info[1]);

    let annote = if annote_len > 0 {
        buff_decoder.read_str(annote_len as usize)
    } else {
        None
    };

    let from_pos = from_row as usize * 9 + from_col as usize;
    let to_pos = to_row as usize * 9 + to_col as usize;

    let move_data = XqfMoveData {
        from: from_pos as u8,
        to: to_pos as u8,
        piece: None,
    };

    let mut node = XqfMoveNode {
        move_data,
        annotation: annote,
        main_line: None,
        variations: Vec::new(),
    };

    if has_next_step {
        if let Some(next_node) = read_steps(buff_decoder, version, keys, board, branches) {
            node.main_line = Some(Box::new(next_node));
        }
    }

    if has_var_step {
        if let Some(var_node) = read_steps(buff_decoder, version, keys, &board_bak, branches) {
            node.variations.push(var_node);
            *branches += 1;
        }
    }

    Some(node)
}

/// Read XQF file with multi-branch support
pub fn read_xqf_with_variations(path: &str) -> Result<XqfFileWithVariations, XqfError> {
    let contents = std::fs::read(path).map_err(XqfError::Io)?;
    read_xqf_from_bytes(&contents)
}

/// Read XQF from bytes with multi-branch support
pub fn read_xqf_from_bytes(contents: &[u8]) -> Result<XqfFileWithVariations, XqfError> {
    if contents.len() < XQF_HEADER_SIZE {
        return Err(XqfError::Other("File too small".into()));
    }

    if &contents[0..2] != b"XQ" {
        return Err(XqfError::InvalidSignature);
    }

    let version = contents[2];
    let crypt_keys = contents[3..16].to_vec();
    let uc_board = contents[16..48].to_vec();

    let keys = if version > 0x0A {
        Some(init_decrypt_key(&crypt_keys)?)
    } else {
        None
    };

    let chess_mans = init_chess_board(&uc_board, version, keys.as_ref());
    let initial_board = build_xqf_board(&chess_mans);

    let step_base_buff = if version > 0x0A {
        if let Some(k) = &keys {
            let decrypted = decode_xqf_buff(k, &contents[XQF_HEADER_SIZE..]);
            XQFBuffDecoder::new(decrypted)
        } else {
            XQFBuffDecoder::new(contents[XQF_HEADER_SIZE..].to_vec())
        }
    } else {
        XQFBuffDecoder::new(contents[XQF_HEADER_SIZE..].to_vec())
    };

    // Parse game info
    let mut offset = 48;
    let title_len = contents[offset] as usize;
    offset += 1;
    let title = if title_len > 0 && offset + title_len <= contents.len() {
        String::from_utf8(contents[offset..offset + title_len].to_vec()).ok()
    } else {
        None
    };
    offset += 64 + title_len;

    let red_name_len = contents[offset] as usize;
    offset += 1;
    let red_player = if red_name_len > 0 && offset + red_name_len <= contents.len() {
        String::from_utf8(contents[offset..offset + red_name_len].to_vec()).ok()
    } else {
        None
    };
    offset += red_name_len + 64;

    let black_name_len = contents[offset] as usize;
    offset += 1;
    let black_player = if black_name_len > 0 && offset + black_name_len <= contents.len() {
        String::from_utf8(contents[offset..offset + black_name_len].to_vec()).ok()
    } else {
        None
    };
    offset += black_name_len + 64;

    let uc_type = contents[offset];
    offset += 1;
    let uc_res = contents[offset];

    let result = if uc_res <= 4 {
        GAME_RESULT_MAP[uc_res as usize].to_string()
    } else {
        "*".to_string()
    };

    // Parse moves recursively
    let mut step_decoder = step_base_buff;
    let mut branches = 0u32;
    let mut root_moves = Vec::new();
    let current_board = initial_board.clone();

    while let Some(node) = read_steps(
        &mut step_decoder,
        version,
        keys.as_ref(),
        &current_board,
        &mut branches,
    ) {
        root_moves.push(node);
    }

    Ok(XqfFileWithVariations {
        game_info: XqfGameInfoExtended {
            title,
            red_player,
            black_player,
            event: None,
            result,
            version,
            game_type: uc_type,
            source: "XQF".to_string(),
            branches,
        },
        initial_board,
        root_moves,
        version,
        was_encrypted: version > 0x0A,
    })
}

/// Convert XqfFileWithVariations to Game
pub fn xqf_file_to_game(xqf_file: &XqfFileWithVariations) -> Result<Game, XqfError> {
    let mut game = Game::from_board(xqf_file.initial_board.clone());

    game.metadata.title = xqf_file.game_info.title.clone();
    game.metadata.red_player = xqf_file.game_info.red_player.clone();
    game.metadata.black_player = xqf_file.game_info.black_player.clone();
    game.metadata.result = Some(xqf_file.game_info.result.clone());
    game.metadata.source = Some(xqf_file.game_info.source.clone());
    game.metadata.branch_count = xqf_file.game_info.branches;

    for root_move in &xqf_file.root_moves {
        convert_move_node_to_game(root_move, &mut game, None)?;
    }

    Ok(game)
}

/// Recursively convert move nodes to game
fn convert_move_node_to_game(
    node: &XqfMoveNode,
    game: &mut Game,
    parent_ply: Option<u32>,
) -> Result<(), XqfError> {
    let from_col = node.move_data.from as usize % 9;
    let from_row = node.move_data.from as usize / 9;
    let to_col = node.move_data.to as usize % 9;
    let to_row = node.move_data.to as usize / 9;

    let from = (from_col, from_row);
    let to = (to_col, to_row);

    if let Some(ply) = parent_ply {
        let _ = game.make_variation(ply, from, to);
    } else {
        let _ = game.make_move(from, to);
    }

    if let Some(ref ann) = node.annotation {
        game.annotate_last_move(ann.clone());
    }

    if let Some(ref main) = node.main_line {
        let current_ply = game.get_current_ply() as u32;
        convert_move_node_to_game(main, game, Some(current_ply))?;
    }

    for var in &node.variations {
        let current_ply = game.get_current_ply() as u32;
        convert_move_node_to_game(var, game, Some(current_ply))?;
    }

    Ok(())
}

/// Write XQF file from game
pub fn write_xqf_from_game(_game: &Game, _path: &str) -> Result<(), XqfError> {
    Err(XqfError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_xqf_with_variations() {
        let result = read_xqf_with_variations("tests/data/game_varations.xqf");
        assert!(
            result.is_ok(),
            "Failed to read XQF file: {:?}",
            result.err()
        );

        let xqf_file = result.unwrap();

        // Verify game info
        println!("XQF Version: {}", xqf_file.version);
        println!("Was encrypted: {}", xqf_file.was_encrypted);
        println!("Title: {:?}", xqf_file.game_info.title);
        println!("Red player: {:?}", xqf_file.game_info.red_player);
        println!("Black player: {:?}", xqf_file.game_info.black_player);
        println!("Result: {}", xqf_file.game_info.result);
        println!("Branches: {}", xqf_file.game_info.branches);
        println!("Root moves count: {}", xqf_file.root_moves.len());

        // Verify we have moves
        assert!(
            !xqf_file.root_moves.is_empty(),
            "No moves found in XQF file"
        );

        // Verify branches count
        assert!(
            xqf_file.game_info.branches > 0,
            "Expected variations but found none"
        );

        // Print move tree
        print_move_tree(&xqf_file.root_moves, 0);
    }

    #[test]
    fn test_xqf_to_game() {
        let xqf_file = read_xqf_with_variations("tests/data/game_varations.xqf").unwrap();
        let game = xqf_file_to_game(&xqf_file);

        assert!(
            game.is_ok(),
            "Failed to convert XQF to game: {:?}",
            game.err()
        );

        let game = game.unwrap();
        println!("Game metadata:");
        println!("  Title: {:?}", game.metadata.title);
        println!("  Red: {:?}", game.metadata.red_player);
        println!("  Black: {:?}", game.metadata.black_player);
        println!("  Result: {:?}", game.metadata.result);
        println!("  Branch count: {}", game.metadata.branch_count);
        println!("  Total moves: {}", game.total_moves());
        println!("  Total variations: {}", game.total_variations());
    }

    fn print_move_tree(moves: &[XqfMoveNode], depth: usize) {
        let indent = "  ".repeat(depth);
        for (i, node) in moves.iter().enumerate() {
            if depth == 0 {
                println!("\n{}=== Root Move {} ===", indent, i + 1);
            }

            println!(
                "{}Move: {} -> {} (annotation: {:?})",
                indent, node.move_data.from, node.move_data.to, node.annotation
            );

            // Print variations
            for (j, var) in node.variations.iter().enumerate() {
                println!("{}  Variation {}:", indent, j + 1);
                print_move_tree(&[var.clone()], depth + 1);
            }

            // Print main line
            if let Some(ref main) = node.main_line {
                print_move_tree(&[main.as_ref().clone()], depth + 1);
            }
        }
    }
}
