/// 攻击矩阵测试
use cchess_rs::attack_matrix::*;
use cchess_rs::board::Board;
use cchess_rs::pieces::{PieceType, Side};

#[test]
fn test_attack_matrix_generation() {
    let mut board = Board::new();
    board.initial_position();

    // 生成 Red side 的攻击矩阵 (uppercase pieces)
    let red_matrix = generate_attack_matrix(&board, Side::Red);

    // 检查红方的马是否能攻击特定位置
    // 红马在(1,0)应该能攻击(2,2)和(0,2)
    // matrix[row][col], so (2,2) -> [2][2], (0,2) -> [2][0]
    assert!(!red_matrix[2][2].is_empty(), "红马应该能攻击(2,2)");
    assert!(!red_matrix[2][0].is_empty(), "红马应该能攻击(0,2)");

    // 生成 Black side 的攻击矩阵 (lowercase pieces)
    let black_matrix = generate_attack_matrix(&board, Side::Black);

    // 检查黑方的马是否能攻击特定位置
    // 黑马在(1,9)应该能攻击(2,7)和(0,7)
    assert!(!black_matrix[7][2].is_empty(), "黑马应该能攻击(2,7)");
    assert!(!black_matrix[7][0].is_empty(), "黑马应该能攻击(0,7)");
}

#[test]
fn test_is_position_attacked() {
    let mut board = Board::new();
    board.initial_position();

    // 测试红方攻击 (Side::Red = uppercase)
    // (2,2)应该被红马攻击
    assert!(
        is_position_attacked(&board, (2, 2), Side::Red),
        "(2,2)应该被红方攻击"
    );

    // (0,2)应该被红马攻击
    assert!(
        is_position_attacked(&board, (0, 2), Side::Red),
        "(0,2)应该被红方攻击"
    );

    // 测试黑方攻击 (Side::Black = lowercase)
    // (6,7)应该被黑马攻击 (黑马在(7,9)，日字移动到(6,7))
    assert!(
        is_position_attacked(&board, (6, 7), Side::Black),
        "(6,7)应该被黑方攻击"
    );

    // (2,7)应该被黑马攻击 (黑马在(1,9)，日字移动到(2,7))
    assert!(
        is_position_attacked(&board, (2, 7), Side::Black),
        "(2,7)应该被黑方攻击"
    );

    // 测试不被攻击的位置
    // 使用空棋盘确保没有棋子攻击
    let empty_board = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
    assert!(
        !is_position_attacked(&empty_board, (4, 5), Side::Red),
        "(4,5)在空棋盘不应该被红方攻击"
    );
    assert!(
        !is_position_attacked(&empty_board, (4, 5), Side::Black),
        "(4,5)在空棋盘不应该被黑方攻击"
    );
}

#[test]
fn test_get_attackers_to_position() {
    let mut board = Board::new();
    board.initial_position();

    // 获取攻击(2,2)的红方棋子 (Side::Red = uppercase)
    let attackers_red = get_attackers_to_position(&board, (2, 2), Side::Red);
    assert!(!attackers_red.is_empty(), "应该有红方棋子攻击(2,2)");

    // 检查是否有马在攻击者中
    let has_knight = attackers_red
        .iter()
        .any(|&(_col, _row, piece_type, color)| {
            piece_type == PieceType::Knight && color == Side::Red
        });
    assert!(has_knight, "攻击者中应该有红马");

    // 获取攻击(6,7)的黑方棋子 (Side::Black = lowercase)
    // 黑马在(7,9)可以日字移动到(6,7)
    let attackers_black = get_attackers_to_position(&board, (6, 7), Side::Black);
    assert!(!attackers_black.is_empty(), "应该有黑方棋子攻击(6,7)");

    // 检查是否有马在攻击者中
    let has_knight_black = attackers_black
        .iter()
        .any(|&(_col, _row, piece_type, color)| {
            piece_type == PieceType::Knight && color == Side::Black
        });
    assert!(has_knight_black, "攻击者中应该有黑马");
}

#[test]
fn test_is_king_in_check() {
    let mut board = Board::new();
    board.initial_position();

    // 初始局面将帅都不应该被将军
    assert!(
        !is_king_in_check(&board, Side::Red),
        "初始局面红方不应该被将军"
    );
    assert!(
        !is_king_in_check(&board, Side::Black),
        "初始局面黑方不应该被将军"
    );

    // 创建红方被将军的局面：黑车在(4,1)直接将军红帅(4,0)
    // Side::Red = uppercase (红方), Side::Black = lowercase (黑方)
    let mut check_board = Board::new();
    check_board.clear();
    check_board.set_fen(4, 0, 'K'); // 红帅 (uppercase = Red)
    check_board.set_fen(4, 9, 'k'); // 黑将 (lowercase = Black)
    check_board.set_fen(4, 1, 'r'); // 黑车将军 (lowercase = Black)

    assert!(
        is_king_in_check(&check_board, Side::Red),
        "红帅应该被黑车将军"
    );

    // 创建黑方被将军的局面：红车在(4,8)直接将军黑将(4,9)
    let mut check_board2 = Board::new();
    check_board2.clear();
    check_board2.set_fen(4, 0, 'K'); // 红帅
    check_board2.set_fen(4, 9, 'k'); // 黑将
    check_board2.set_fen(4, 8, 'R'); // 红车将军 (uppercase = Red)

    assert!(
        is_king_in_check(&check_board2, Side::Black),
        "黑将应该被红车将军"
    );

    // 创建马将军局面：黑马在(3,7)将军红帅(4,0)（日字：右1下2）
    let mut knight_check = Board::new();
    knight_check.clear();
    knight_check.set_fen(4, 0, 'K'); // 红帅
    knight_check.set_fen(4, 9, 'k'); // 黑将
    knight_check.set_fen(3, 2, 'n'); // 黑马在(3,2)将军红帅(4,0)

    assert!(
        is_king_in_check(&knight_check, Side::Red),
        "红帅应该被黑马将军"
    );

    // 创建炮将军局面：黑炮在(0,7)，炮架在(0,8)，红帅在(0,0)
    // 炮在col 0, row 7; 炮架在col 0, row 8; 红帅在col 0, row 0
    // 炮需要隔一个子才能吃，炮架在row 1, 红帅在row 0
    let mut cannon_check = Board::new();
    cannon_check.clear();
    cannon_check.set_fen(0, 0, 'K'); // 红帅
    cannon_check.set_fen(0, 9, 'k'); // 黑将
    cannon_check.set_fen(0, 2, 'c'); // 黑炮
    cannon_check.set_fen(0, 1, 'p'); // 炮架（在炮和目标之间）

    assert!(
        is_king_in_check(&cannon_check, Side::Red),
        "红帅应该被黑炮将军"
    );
}

#[test]
fn test_quick_attack_check() {
    let mut board = Board::new();
    board.initial_position();

    // 测试一个有效的攻击
    // 红马从(1,0)攻击(2,2)应该有效
    assert!(
        quick_attack_check(&board, (1, 0), (2, 2)),
        "红马应该能攻击(2,2)"
    );

    // 测试无效的攻击
    // 红马从(1,0)攻击(4,4)应该无效（距离太远）
    assert!(
        !quick_attack_check(&board, (1, 0), (4, 4)),
        "红马不应该能攻击(4,4)"
    );

    // 测试从空位置攻击
    assert!(
        !quick_attack_check(&board, (4, 4), (4, 5)),
        "空位置不应该能攻击"
    );
}

#[test]
fn test_king_attack_check() {
    let mut board = Board::new();
    board.clear();

    // 放置红帅和黑将在相邻位置
    // Side::Red = uppercase (红方), Side::Black = lowercase (黑方)
    board.set_piece_at(4, 0, PieceType::King, Side::Red); // 红帅
    board.set_piece_at(4, 1, PieceType::King, Side::Black); // 黑将

    // 生成 Red side 攻击矩阵 (uppercase)
    let red_matrix = generate_attack_matrix(&board, Side::Red);

    // 检查红帅是否能攻击黑将的位置
    let attacks_black_king = &red_matrix[1][4];
    assert!(!attacks_black_king.is_empty(), "红帅应该能攻击相邻的黑将");

    // 检查攻击者中是否有将
    let has_king = attacks_black_king
        .iter()
        .any(|&(_col, _row, piece_type, color)| {
            piece_type == PieceType::King && color == Side::Red
        });
    assert!(has_king, "攻击者中应该有红帅");
}

#[test]
fn test_advisor_attack_check() {
    let mut board = Board::new();
    board.clear();

    // 在九宫内放置红士 (Red palace = rows 0-2)
    board.set_piece_at(3, 0, PieceType::Advisor, Side::Red);

    // 生成 Red side 攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Red);

    // 检查红士能攻击的对角线位置
    assert!(
        !red_matrix[1][4].is_empty(),
        "红士应该能攻击(4,1)（对角线）"
    );

    // 检查红士不能攻击非对角线位置
    assert!(
        red_matrix[2][3].is_empty(),
        "红士不应该能攻击(3,2)（非对角线）"
    );
}

#[test]
fn test_elephant_attack_check() {
    let mut board = Board::new();
    board.clear();

    // 放置红象在己方阵地 (Red side = rows 0-4)
    board.set_piece_at(2, 0, PieceType::Elephant, Side::Red);

    // 生成 Red side 攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Red);

    // 检查红象能攻击的田字位置
    assert!(!red_matrix[2][4].is_empty(), "红象应该能攻击(4,2)（田字）");

    // 检查红象不能攻击过河位置 (Red can't go to row >= 5)
    assert!(red_matrix[6][2].is_empty(), "红象不应该能攻击过河位置(2,6)");
}

#[test]
fn test_rook_attack_check() {
    let mut board = Board::new();
    board.clear();

    // 放置红车
    board.set_piece_at(0, 0, PieceType::Rook, Side::Red);

    // 生成 Red side 攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Red);

    // 检查红车能攻击同一行的位置
    assert!(!red_matrix[0][8].is_empty(), "红车应该能攻击同一行(8,0)");

    // 检查红车能攻击同一列的位置
    assert!(!red_matrix[9][0].is_empty(), "红车应该能攻击同一列(0,9)");

    // 检查红车不能攻击对角线位置
    assert!(
        red_matrix[1][1].is_empty(),
        "红车不应该能攻击对角线位置(1,1)"
    );
}

#[test]
fn test_cannon_attack_check() {
    let mut board = Board::new();
    board.clear();

    // 放置红炮和炮架
    board.set_piece_at(0, 0, PieceType::Cannon, Side::Red);
    board.set_piece_at(0, 2, PieceType::Pawn, Side::Red); // 炮架
    board.set_piece_at(0, 4, PieceType::Pawn, Side::Black); // 目标

    // 生成 Red side 攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Red);

    // 检查红炮能攻击有炮架的目标
    assert!(
        !red_matrix[4][0].is_empty(),
        "红炮应该能攻击有炮架的目标(0,4)"
    );

    // 检查红炮不能攻击无炮架的目标 - 移除炮架
    board.set_fen(0, 2, '.'); // 移除炮架
    let red_matrix2 = generate_attack_matrix(&board, Side::Red);
    assert!(
        red_matrix2[4][0].is_empty(),
        "红炮不应该能攻击无炮架的目标(0,4)"
    );
}

#[test]
fn test_pawn_attack_check() {
    let mut board = Board::new();
    board.clear();

    // 放置红兵在没过河位置（row 3，红方阵地）
    // Red forward = increasing row, Red crosses river at row >= 5
    board.set_piece_at(4, 3, PieceType::Pawn, Side::Red);

    // 生成 Red side 攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Red);

    // 检查没过河的红兵只能前进攻击（红兵 forward=+1，row 3→4）
    assert!(
        !red_matrix[4][4].is_empty(),
        "没过河的红兵应该能前进攻击(4,4)"
    );

    // 放置红兵在过河位置
    board.set_piece_at(4, 6, PieceType::Pawn, Side::Red);
    let red_matrix2 = generate_attack_matrix(&board, Side::Red);

    // 检查过河的红兵能横向攻击
    assert!(
        !red_matrix2[6][5].is_empty(),
        "过河的红兵应该能横向攻击(5,6)"
    );
    assert!(
        !red_matrix2[6][3].is_empty(),
        "过河的红兵应该能横向攻击(3,6)"
    );
}
