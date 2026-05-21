/// 攻击矩阵测试
use cchess_rs::attack_matrix::*;
use cchess_rs::board::Board;
use cchess_rs::pieces::{PieceType, Side};

#[test]
fn test_attack_matrix_generation() {
    let mut board = Board::new();
    board.initial_position();

    // 生成 lowercase side 的攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Black);

    // 检查红方的马是否能攻击特定位置
    // 红马在(1,0)应该能攻击(2,2)和(0,2)
    // matrix[row][col], so (2,2) -> [2][2], (0,2) -> [2][0]
    assert!(!red_matrix[2][2].is_empty(), "红马应该能攻击(2,2)");
    assert!(!red_matrix[2][0].is_empty(), "红马应该能攻击(0,2)");

    // 生成 uppercase side 的攻击矩阵
    let black_matrix = generate_attack_matrix(&board, Side::Red);

    // 检查黑方的马是否能攻击特定位置
    // 黑马在(1,9)应该能攻击(2,7)和(0,7)
    assert!(!black_matrix[7][2].is_empty(), "黑马应该能攻击(2,7)");
    assert!(!black_matrix[7][0].is_empty(), "黑马应该能攻击(0,7)");
}

#[test]
fn test_is_position_attacked() {
    let mut board = Board::new();
    board.initial_position();

    // 测试红方攻击
    // (2,2)应该被红马攻击
    assert!(
        is_position_attacked(&board, (2, 2), Side::Black),
        "(2,2)应该被红方攻击"
    );

    // (0,2)应该被红马攻击
    assert!(
        is_position_attacked(&board, (0, 2), Side::Black),
        "(0,2)应该被红方攻击"
    );

    // 测试黑方攻击
    // (2,7)应该被黑马攻击
    assert!(
        is_position_attacked(&board, (2, 7), Side::Red),
        "(2,7)应该被黑方攻击"
    );

    // (0,7)应该被黑马攻击
    assert!(
        is_position_attacked(&board, (0, 7), Side::Red),
        "(0,7)应该被黑方攻击"
    );

    // 测试不被攻击的位置
    // 使用空棋盘确保没有棋子攻击
    let empty_board = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
    assert!(
        !is_position_attacked(&empty_board, (4, 5), Side::Black),
        "(4,5)在空棋盘不应该被红方攻击"
    );
    assert!(
        !is_position_attacked(&empty_board, (4, 5), Side::Red),
        "(4,5)在空棋盘不应该被黑方攻击"
    );
}

#[test]
fn test_get_attackers_to_position() {
    let mut board = Board::new();
    board.initial_position();

    // 获取攻击(2,2)的红方棋子
    let attackers_red = get_attackers_to_position(&board, (2, 2), Side::Black);
    assert!(!attackers_red.is_empty(), "应该有红方棋子攻击(2,2)");

    // 检查是否有马在攻击者中
    let has_knight = attackers_red
        .iter()
        .any(|&(_col, _row, piece_type, color)| {
            piece_type == PieceType::Knight && color == Side::Black
        });
    assert!(has_knight, "攻击者中应该有红马");

    // 获取攻击(2,7)的黑方棋子
    let attackers_black = get_attackers_to_position(&board, (2, 7), Side::Red);
    assert!(!attackers_black.is_empty(), "应该有黑方棋子攻击(2,7)");

    // 检查是否有马在攻击者中
    let has_knight_black = attackers_black
        .iter()
        .any(|&(_col, _row, piece_type, color)| {
            piece_type == PieceType::Knight && color == Side::Red
        });
    assert!(has_knight_black, "攻击者中应该有黑马");
}

#[test]
fn test_is_king_in_check() {
    let mut board = Board::new();
    board.initial_position();

    // 初始局面将帅都不应该被将军
    assert!(
        !is_king_in_check(&board, Side::Black),
        "初始局面红方不应该被将军"
    );
    assert!(
        !is_king_in_check(&board, Side::Red),
        "初始局面黑方不应该被将军"
    );

    // TODO: 添加将军局面的测试
    // 需要创建一个将军的局面来测试

    // 创建红方被将军的局面：黑车在(4,1)直接将军红帅(4,0)
    let mut check_board = Board::new();
    check_board.clear();
    check_board.set_fen(4, 0, 'k'); // 红帅
    check_board.set_fen(4, 9, 'K'); // 黑将
    check_board.set_fen(4, 1, 'R'); // 黑车将军

    assert!(
        is_king_in_check(&check_board, Side::Black),
        "红帅应该被黑车将军"
    );

    // 创建黑方被将军的局面：红车在(4,8)直接将军黑将(4,9)
    let mut check_board2 = Board::new();
    check_board2.clear();
    check_board2.set_fen(4, 0, 'k'); // 红帅
    check_board2.set_fen(4, 9, 'K'); // 黑将
    check_board2.set_fen(4, 8, 'r'); // 红车将军

    assert!(
        is_king_in_check(&check_board2, Side::Red),
        "黑将应该被红车将军"
    );

    // 创建马将军局面：红马在(3,7)将军黑将(4,9)（日字：右1上2）
    let mut knight_check = Board::new();
    knight_check.clear();
    knight_check.set_fen(4, 9, 'K'); // 黑将
    knight_check.set_fen(4, 0, 'k'); // 红帅
    knight_check.set_fen(3, 7, 'n'); // 红马在(3,7)将军黑将(4,9)

    assert!(
        is_king_in_check(&knight_check, Side::Red),
        "黑将应该被红马将军"
    );

    // 创建炮将军局面：红炮在(0,7)，炮架在(0,8)，黑将在(0,9)
    let mut cannon_check = Board::new();
    cannon_check.clear();
    cannon_check.set_fen(0, 9, 'K'); // 黑将
    cannon_check.set_fen(0, 0, 'k'); // 红帅
    cannon_check.set_fen(0, 7, 'c'); // 红炮
    cannon_check.set_fen(0, 8, 'p'); // 炮架（在炮和目标之间）

    assert!(
        is_king_in_check(&cannon_check, Side::Red),
        "黑将应该被红炮将军"
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

    // 放置红将和黑帅在相邻位置
    board.set_piece_at(4, 0, PieceType::King, Side::Black);
    board.set_piece_at(4, 1, PieceType::King, Side::Red);

    // 生成 lowercase side 攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Black);

    // 检查红将是否能攻击黑帅的位置
    let attacks_black_king = &red_matrix[1][4];
    assert!(!attacks_black_king.is_empty(), "红将应该能攻击相邻的黑帅");

    // 检查攻击者中是否有将
    let has_king = attacks_black_king
        .iter()
        .any(|&(_col, _row, piece_type, color)| {
            piece_type == PieceType::King && color == Side::Black
        });
    assert!(has_king, "攻击者中应该有红将");
}

#[test]
fn test_advisor_attack_check() {
    let mut board = Board::new();
    board.clear();

    // 在九宫内放置红士
    board.set_piece_at(3, 0, PieceType::Advisor, Side::Black);

    // 生成 lowercase side 攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Black);

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

    // 放置红象在河边位置
    board.set_piece_at(2, 0, PieceType::Elephant, Side::Black);

    // 生成 lowercase side 攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Black);

    // 检查红象能攻击的田字位置
    assert!(!red_matrix[2][4].is_empty(), "红象应该能攻击(4,2)（田字）");

    // 检查红象不能攻击过河位置
    assert!(red_matrix[2][6].is_empty(), "红象不应该能攻击过河位置(6,2)");
}

#[test]
fn test_rook_attack_check() {
    let mut board = Board::new();
    board.clear();

    // 放置红车
    board.set_piece_at(0, 0, PieceType::Rook, Side::Black);

    // 生成 lowercase side 攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Black);

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
    board.set_piece_at(0, 0, PieceType::Cannon, Side::Black);
    board.set_piece_at(0, 2, PieceType::Pawn, Side::Black); // 炮架
    board.set_piece_at(0, 4, PieceType::Pawn, Side::Red); // 目标

    // 生成 lowercase side 攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Black);

    // 检查红炮能攻击有炮架的目标
    assert!(
        !red_matrix[4][0].is_empty(),
        "红炮应该能攻击有炮架的目标(0,4)"
    );

    // 检查红炮不能攻击无炮架的目标 - 移除炮架
    board.set_fen(0, 2, '.'); // 移除炮架
    let red_matrix2 = generate_attack_matrix(&board, Side::Black);
    assert!(
        red_matrix2[4][0].is_empty(),
        "红炮不应该能攻击无炮架的目标(0,4)"
    );
}

#[test]
fn test_pawn_attack_check() {
    let mut board = Board::new();
    board.clear();

    // 放置红兵在没过河位置
    board.set_piece_at(4, 3, PieceType::Pawn, Side::Black);

    // 生成 lowercase side 攻击矩阵
    let red_matrix = generate_attack_matrix(&board, Side::Black);

    // 检查没过河的红兵只能前进攻击
    assert!(
        !red_matrix[4][4].is_empty(),
        "没过河的红兵应该能前进攻击(4,4)"
    );

    // 放置红兵在过河位置
    board.set_piece_at(4, 6, PieceType::Pawn, Side::Black);
    let red_matrix2 = generate_attack_matrix(&board, Side::Black);

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
