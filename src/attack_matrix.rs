/// 攻击矩阵模块
/// 用于快速判断棋子是否能攻击某个位置
use crate::board::Board;
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
fn get_piece_attack_positions(
    board: &Board,
    piece_type: PieceType,
    color: Color,
    col: usize,
    row: usize,
) -> Vec<(usize, usize)> {
    match piece_type {
        PieceType::King => get_king_attack_positions(board, color, col, row),
        PieceType::Advisor => get_advisor_attack_positions(board, color, col, row),
        PieceType::Elephant => get_elephant_attack_positions(board, color, col, row),
        PieceType::Knight => get_knight_attack_positions(board, color, col, row),
        PieceType::Rook => get_rook_attack_positions(board, color, col, row),
        PieceType::Cannon => get_cannon_attack_positions(board, color, col, row),
        PieceType::Pawn => get_pawn_attack_positions(board, color, col, row),
    }
}

/// 将/帅的攻击位置
fn get_king_attack_positions(
    board: &Board,
    color: Color,
    col: usize,
    row: usize,
) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // 上下左右

    for (dc, dr) in directions.iter() {
        let new_col = col as isize + dc;
        let new_row = row as isize + dr;

        // 将/帅必须保持在九宫内
        if new_col >= 3 && new_col <= 5 {
            if color == Color::Red && new_row >= 0 && new_row <= 2 {
                if new_col >= 0 && new_col <= 8 && new_row >= 0 && new_row <= 9 {
                    let new_col = new_col as usize;
                    let new_row = new_row as usize;

                    // 可以攻击空位或敌方棋子
                    if board.is_empty_at(new_col, new_row)
                        || !board.is_color_at(new_col, new_row, color)
                    {
                        positions.push((new_col, new_row));
                    }
                }
            } else if color == Color::Black && new_row >= 7 && new_row <= 9 {
                if new_col >= 0 && new_col <= 8 && new_row >= 0 && new_row <= 9 {
                    let new_col = new_col as usize;
                    let new_row = new_row as usize;

                    if board.is_empty_at(new_col, new_row)
                        || !board.is_color_at(new_col, new_row, color)
                    {
                        positions.push((new_col, new_row));
                    }
                }
            }
        }
    }

    positions
}

/// 士/仕的攻击位置
fn get_advisor_attack_positions(
    board: &Board,
    color: Color,
    col: usize,
    row: usize,
) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)]; // 四个对角线方向

    for (dc, dr) in directions.iter() {
        let new_col = col as isize + dc;
        let new_row = row as isize + dr;

        // 士/仕必须保持在九宫内
        if new_col >= 3 && new_col <= 5 {
            if color == Color::Red && new_row >= 0 && new_row <= 2 {
                if new_col >= 0 && new_col <= 8 && new_row >= 0 && new_row <= 9 {
                    let new_col = new_col as usize;
                    let new_row = new_row as usize;

                    if board.is_empty_at(new_col, new_row)
                        || !board.is_color_at(new_col, new_row, color)
                    {
                        positions.push((new_col, new_row));
                    }
                }
            } else if color == Color::Black && new_row >= 7 && new_row <= 9 {
                if new_col >= 0 && new_col <= 8 && new_row >= 0 && new_row <= 9 {
                    let new_col = new_col as usize;
                    let new_row = new_row as usize;

                    if board.is_empty_at(new_col, new_row)
                        || !board.is_color_at(new_col, new_row, color)
                    {
                        positions.push((new_col, new_row));
                    }
                }
            }
        }
    }

    positions
}

/// 象/相的攻击位置
fn get_elephant_attack_positions(
    board: &Board,
    color: Color,
    col: usize,
    row: usize,
) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    let directions = [(2, 2), (2, -2), (-2, 2), (-2, -2)]; // 田字对角线
    let blocks = [(1, 1), (1, -1), (-1, 1), (-1, -1)]; // 蹩腿点

    for i in 0..4 {
        let (dc, dr) = directions[i];
        let (bc, br) = blocks[i];
        let new_col = col as isize + dc;
        let new_row = row as isize + dr;
        let block_col = col as isize + bc;
        let block_row = row as isize + br;

        // 象/相不能过河
        if new_col >= 0 && new_col <= 8 && new_row >= 0 && new_row <= 9 {
            if color == Color::Red && new_row <= 4 {
                // 检查蹩腿点是否为空
                if board.is_empty_at(block_col as usize, block_row as usize) {
                    let new_col = new_col as usize;
                    let new_row = new_row as usize;

                    if board.is_empty_at(new_col, new_row)
                        || !board.is_color_at(new_col, new_row, color)
                    {
                        positions.push((new_col, new_row));
                    }
                }
            } else if color == Color::Black && new_row >= 5 {
                if board.is_empty_at(block_col as usize, block_row as usize) {
                    let new_col = new_col as usize;
                    let new_row = new_row as usize;

                    if board.is_empty_at(new_col, new_row)
                        || !board.is_color_at(new_col, new_row, color)
                    {
                        positions.push((new_col, new_row));
                    }
                }
            }
        }
    }

    positions
}

/// 马/傌的攻击位置
fn get_knight_attack_positions(
    board: &Board,
    color: Color,
    col: usize,
    row: usize,
) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    let moves_pattern = [
        (2, 1, 1, 0),    // 右2, 上1 - 蹩腿点 (1, 0)
        (2, -1, 1, 0),   // 右2, 下1 - 蹩腿点 (1, 0)
        (-2, 1, -1, 0),  // 左2, 上1 - 蹩腿点 (-1, 0)
        (-2, -1, -1, 0), // 左2, 下1 - 蹩腿点 (-1, 0)
        (1, 2, 0, 1),    // 上2, 右1 - 蹩腿点 (0, 1)
        (1, -2, 0, -1),  // 下2, 右1 - 蹩腿点 (0, -1)
        (-1, 2, 0, 1),   // 上2, 左1 - 蹩腿点 (0, 1)
        (-1, -2, 0, -1), // 下2, 左1 - 蹩腿点 (0, -1)
    ];

    for (dc, dr, bc, br) in moves_pattern.iter() {
        let new_col = col as isize + dc;
        let new_row = row as isize + dr;
        let block_col = col as isize + bc;
        let block_row = row as isize + br;

        if new_col >= 0 && new_col <= 8 && new_row >= 0 && new_row <= 9 {
            // 检查蹩腿点是否为空
            if board.is_empty_at(block_col as usize, block_row as usize) {
                let new_col = new_col as usize;
                let new_row = new_row as usize;

                if board.is_empty_at(new_col, new_row)
                    || !board.is_color_at(new_col, new_row, color)
                {
                    positions.push((new_col, new_row));
                }
            }
        }
    }

    positions
}

/// 车/俥的攻击位置
fn get_rook_attack_positions(
    board: &Board,
    color: Color,
    col: usize,
    row: usize,
) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // 上下左右

    for (dc, dr) in directions.iter() {
        let mut new_col = col as isize + dc;
        let mut new_row = row as isize + dr;

        while new_col >= 0 && new_col <= 8 && new_row >= 0 && new_row <= 9 {
            let new_col_u = new_col as usize;
            let new_row_u = new_row as usize;

            if board.is_empty_at(new_col_u, new_row_u) {
                // 空位，可以攻击
                positions.push((new_col_u, new_row_u));
                new_col += dc;
                new_row += dr;
            } else {
                // 有棋子
                if !board.is_color_at(new_col_u, new_row_u, color) {
                    // 敌方棋子，可以攻击
                    positions.push((new_col_u, new_row_u));
                }
                // 遇到棋子就停止
                break;
            }
        }
    }

    positions
}

/// 炮/砲的攻击位置
fn get_cannon_attack_positions(
    board: &Board,
    color: Color,
    col: usize,
    row: usize,
) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();
    let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // 上下左右

    for (dc, dr) in directions.iter() {
        let mut new_col = col as isize + dc;
        let mut new_row = row as isize + dr;
        let mut has_jumped = false;

        while new_col >= 0 && new_col <= 8 && new_row >= 0 && new_row <= 9 {
            let new_col_u = new_col as usize;
            let new_row_u = new_row as usize;

            if !has_jumped {
                // 还没有跳过棋子
                if board.is_empty_at(new_col_u, new_row_u) {
                    // 空位，可以移动和攻击
                    positions.push((new_col_u, new_row_u));
                    new_col += dc;
                    new_row += dr;
                } else {
                    // 遇到棋子，标记为已跳过
                    has_jumped = true;
                    new_col += dc;
                    new_row += dr;
                }
            } else {
                // 已经跳过棋子，只能吃子
                if !board.is_empty_at(new_col_u, new_row_u) {
                    if !board.is_color_at(new_col_u, new_row_u, color) {
                        // 敌方棋子，可以攻击
                        positions.push((new_col_u, new_row_u));
                    }
                    // 遇到棋子就停止
                    break;
                }
                new_col += dc;
                new_row += dr;
            }
        }
    }

    positions
}

/// 兵/卒的攻击位置
fn get_pawn_attack_positions(
    board: &Board,
    color: Color,
    col: usize,
    row: usize,
) -> Vec<(usize, usize)> {
    let mut positions = Vec::new();

    // 兵/卒前进方向
    let forward = if color == Color::Red { 1 } else { -1 };

    // 是否已经过河
    let crossed_river = if color == Color::Red {
        row >= 5
    } else {
        row <= 4
    };

    // 前进攻击
    let new_col = col as isize;
    let new_row = row as isize + forward;
    if new_row >= 0 && new_row <= 9 {
        let new_col = new_col as usize;
        let new_row = new_row as usize;

        if board.is_empty_at(new_col, new_row) || !board.is_color_at(new_col, new_row, color) {
            positions.push((new_col, new_row));
        }
    }

    // 过河后可以左右攻击
    if crossed_river {
        for dc in [-1, 1].iter() {
            let new_col = col as isize + dc;
            let new_row = row as isize;

            if new_col >= 0 && new_col <= 8 {
                let new_col = new_col as usize;
                let new_row = new_row as usize;

                if board.is_empty_at(new_col, new_row)
                    || !board.is_color_at(new_col, new_row, color)
                {
                    positions.push((new_col, new_row));
                }
            }
        }
    }

    positions
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

/// 快速攻击矩阵检查
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

    // 检查目标位置
    let target_char = board.get_fen(to_col, to_row);
    if target_char != '.' {
        // 目标位置有棋子
        let Some(target_color) = Color::from_fen(target_char) else {
            return false;
        };

        // 不能吃自己的棋子
        if target_color == color {
            return false;
        }
    }

    // 根据棋子类型检查攻击
    match piece_type {
        PieceType::King => check_king_attack(board, color, from_col, from_row, to_col, to_row),
        PieceType::Advisor => {
            check_advisor_attack(board, color, from_col, from_row, to_col, to_row)
        }
        PieceType::Elephant => {
            check_elephant_attack(board, color, from_col, from_row, to_col, to_row)
        }
        PieceType::Knight => check_knight_attack(board, from_col, from_row, to_col, to_row),
        PieceType::Rook => check_rook_attack(board, color, from_col, from_row, to_col, to_row),
        PieceType::Cannon => check_cannon_attack(board, color, from_col, from_row, to_col, to_row),
        PieceType::Pawn => check_pawn_attack(board, color, from_col, from_row, to_col, to_row),
    }
}

/// 检查将/帅的攻击
fn check_king_attack(
    _board: &Board,
    color: Color,
    from_col: usize,
    from_row: usize,
    to_col: usize,
    to_row: usize,
) -> bool {
    // 检查是否在九宫内
    if !Board::is_in_palace(to_col, to_row, color == Color::Red) {
        return false;
    }

    // 检查距离是否合法（只能移动一步）
    let dx = (to_col as isize - from_col as isize).abs();
    let dy = (to_row as isize - from_row as isize).abs();

    (dx == 1 && dy == 0) || (dx == 0 && dy == 1)
}

/// 检查士/仕的攻击
fn check_advisor_attack(
    _board: &Board,
    color: Color,
    from_col: usize,
    from_row: usize,
    to_col: usize,
    to_row: usize,
) -> bool {
    // 检查是否在九宫内
    if !Board::is_in_palace(to_col, to_row, color == Color::Red) {
        return false;
    }

    // 检查是否对角线移动一步
    let dx = (to_col as isize - from_col as isize).abs();
    let dy = (to_row as isize - from_row as isize).abs();

    dx == 1 && dy == 1
}

/// 检查象/相的攻击
fn check_elephant_attack(
    board: &Board,
    _color: Color,
    from_col: usize,
    from_row: usize,
    to_col: usize,
    to_row: usize,
) -> bool {
    // 检查是否过河
    if _color == Color::Red && to_row > 4 {
        return false; // 红象不能过河
    }
    if _color == Color::Black && to_row < 5 {
        return false; // 黑象不能过河
    }

    // 检查是否田字移动
    let dx = (to_col as isize - from_col as isize).abs();
    let dy = (to_row as isize - from_row as isize).abs();

    if dx != 2 || dy != 2 {
        return false;
    }

    // 检查蹩腿点
    let block_col = (from_col as isize + (to_col as isize - from_col as isize) / 2) as usize;
    let block_row = (from_row as isize + (to_row as isize - from_row as isize) / 2) as usize;

    board.is_empty_at(block_col, block_row)
}

/// 检查马/傌的攻击
fn check_knight_attack(
    _board: &Board,
    from_col: usize,
    from_row: usize,
    to_col: usize,
    to_row: usize,
) -> bool {
    let dx = (to_col as isize - from_col as isize).abs();
    let dy = (to_row as isize - from_row as isize).abs();

    // 检查是否是L形移动
    if !((dx == 1 && dy == 2) || (dx == 2 && dy == 1)) {
        return false;
    }

    // 检查蹩腿点
    let (block_col, block_row) = if dx == 2 {
        // 横着走，蹩腿点在横向中间
        let block_col = (from_col as isize + (to_col as isize - from_col as isize) / 2) as usize;
        (block_col, from_row)
    } else {
        // 竖着走，蹩腿点在纵向中间
        let block_row = (from_row as isize + (to_row as isize - from_row as isize) / 2) as usize;
        (from_col, block_row)
    };

    _board.is_empty_at(block_col, block_row)
}

/// 检查车/俥的攻击
fn check_rook_attack(
    board: &Board,
    _color: Color,
    from_col: usize,
    from_row: usize,
    to_col: usize,
    to_row: usize,
) -> bool {
    // 检查是否在同一行或同一列
    if from_col != to_col && from_row != to_row {
        return false;
    }

    // 检查中间是否有棋子阻挡
    if board.has_pieces_between(from_col, from_row, to_col, to_row) {
        return false;
    }

    true
}

/// 检查炮/砲的攻击
fn check_cannon_attack(
    board: &Board,
    _color: Color,
    from_col: usize,
    from_row: usize,
    to_col: usize,
    to_row: usize,
) -> bool {
    // 检查是否在同一行或同一列
    if from_col != to_col && from_row != to_row {
        return false;
    }

    let target_char = board.get_fen(to_col, to_row);
    let is_capture = target_char != '.';

    if is_capture {
        // 吃子需要炮架
        board.has_cannon_screen(from_col, from_row, to_col, to_row)
    } else {
        // 移动不能有棋子阻挡
        !board.has_pieces_between(from_col, from_row, to_col, to_row)
    }
}

/// 检查兵/卒的攻击
fn check_pawn_attack(
    _board: &Board,
    color: Color,
    from_col: usize,
    from_row: usize,
    to_col: usize,
    to_row: usize,
) -> bool {
    let forward = if color == Color::Red { 1 } else { -1 };
    let crossed_river = if color == Color::Red {
        from_row >= 5
    } else {
        from_row <= 4
    };

    let dx = (to_col as isize - from_col as isize).abs();
    let dy = to_row as isize - from_row as isize;

    if dx == 0 {
        // 前进
        dy == forward
    } else {
        // 左右移动（只能在过河后）
        crossed_river && dx == 1 && dy == 0
    }
}
