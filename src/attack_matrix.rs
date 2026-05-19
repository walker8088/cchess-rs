/// 攻击矩阵模块
/// 用于快速判断棋子是否能攻击某个位置
use crate::board::Board;
use crate::move_gen;
use crate::pieces::{Color, PieceType};

/// 攻击矩阵类型
/// 对于每个位置(9x10)，存储可以攻击该位置的所有棋子位置
pub type AttackMatrix = [[Vec<(usize, usize, PieceType, Color)>; 9]; 10];

/// 生成攻击矩阵
pub fn generate_attack_matrix(board: &Board, attacker_color: Color) -> AttackMatrix {
    let mut matrix: AttackMatrix = std::array::from_fn(|_| std::array::from_fn(|_| Vec::new()));

    // 遍历棋盘上的所有棋子
    for row in 0..10 {
        for col in 0..9 {
            if board.is_color_at(col, row, attacker_color) {
                if let Some(piece_type) = board.get_piece_type(col, row) {
                    // 获取这个棋子能攻击的所有位置
                    let attack_positions =
                        get_piece_attack_positions(board, piece_type, attacker_color, col, row);

                    // 将这些攻击位置添加到矩阵中
                    for (attack_col, attack_row) in attack_positions {
                        matrix[attack_row][attack_col].push((col, row, piece_type, attacker_color));
                    }
                }
            }
        }
    }

    matrix
}

/// 获取棋子能攻击的所有位置
/// 使用 move_gen::generate_piece_moves 来生成走法序列，避免重复实现
fn get_piece_attack_positions(
    board: &Board,
    piece_type: PieceType,
    color: Color,
    col: usize,
    row: usize,
) -> Vec<(usize, usize)> {
    let moves = move_gen::generate_piece_moves(board, piece_type, color, col, row);
    moves.iter().map(|m| (m.to_col, m.to_row)).collect()
}

/// 检查某个位置是否被攻击
pub fn is_position_attacked(
    board: &Board,
    position: (usize, usize),
    attacker_color: Color,
) -> bool {
    let (col, row) = position;
    let matrix = generate_attack_matrix(board, attacker_color);
    !matrix[row][col].is_empty()
}

/// 获取攻击某个位置的所有棋子
pub fn get_attackers_to_position(
    board: &Board,
    position: (usize, usize),
    attacker_color: Color,
) -> Vec<(usize, usize, PieceType, Color)> {
    let (col, row) = position;
    let matrix = generate_attack_matrix(board, attacker_color);
    matrix[row][col].clone()
}

/// 检查将军状态
pub fn is_king_in_check(board: &Board, king_color: Color) -> bool {
    // 找到将/帅的位置
    for row in 0..10 {
        for col in 0..9 {
            if board.is_color_at(col, row, king_color) {
                if let Some(piece_type) = board.get_piece_type(col, row) {
                    if piece_type == PieceType::King {
                        // 检查这个位置是否被敌方攻击
                        let opponent_color = if king_color == Color::Red {
                            Color::Black
                        } else {
                            Color::Red
                        };
                        return is_position_attacked(board, (col, row), opponent_color);
                    }
                }
            }
        }
    }
    false
}

/// 快速攻击检查 - 使用 move_gen::generate_piece_moves 验证走法合法性
pub fn quick_attack_check(board: &Board, from: (usize, usize), to: (usize, usize)) -> bool {
    let (from_col, from_row) = from;
    let (to_col, to_row) = to;

    // 检查起始位置是否有棋子
    if board.is_empty_at(from_col, from_row) {
        return false;
    }

    // 获取棋子信息和颜色
    let piece_char = board.get_fen(from_col, from_row);
    let Some(piece_type) = PieceType::from_fen(piece_char) else {
        return false;
    };
    let Some(color) = Color::from_fen(piece_char) else {
        return false;
    };

    // 使用 move_gen::generate_piece_moves 生成合法走法，检查目标位置是否在其中
    let moves = move_gen::generate_piece_moves(board, piece_type, color, from_col, from_row);
    moves
        .iter()
        .any(|m| m.to_col == to_col && m.to_row == to_row)
}
