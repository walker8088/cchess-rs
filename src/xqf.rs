//! XQF (Xiangqi File) format support
//!
//! This module provides functionality to read and write XQF files,
//! which are binary files used to store Chinese chess game records.

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::board::Board;
use crate::game::Game;
use crate::pieces::PieceType;

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

impl From<io::Error> for XqfError {
    fn from(err: io::Error) -> Self {
        XqfError::Io(err)
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
                match Board::from_xqf_board(&board_buf) {
                    Ok(board) => Some(board),
                    Err(e) => return Err(XqfError::Other(e)),
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
            match board.to_xqf_board() {
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
                Err(e) => return Err(XqfError::Other(e)),
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
