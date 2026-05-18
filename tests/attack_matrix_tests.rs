/// 攻击矩阵测试
#[cfg(test)]
mod tests {
    use cchess_rs::attack_matrix::*;
    use cchess_rs::board::Board;
    use cchess_rs::pieces::{Color, PieceType};

    #[test]
    fn test_attack_matrix_generation() {
        let board = Board::new();

        // 生成红方的攻击矩阵
        let red_matrix = generate_attack_matrix(&board, Color::Red);

        // 检查红方的马是否能攻击特定位置
        // 红马在(1,0)应该能攻击(2,2)和(0,2)
        assert!(!red_matrix[2][2].is_empty(), "红马应该能攻击(2,2)");
        assert!(!red_matrix[2][0].is_empty(), "红马应该能攻击(0,2)");

        // 生成黑方的攻击矩阵
        let black_matrix = generate_attack_matrix(&board, Color::Black);

        // 检查黑方的马是否能攻击特定位置
        // 黑马在(1,9)应该能攻击(2,7)和(0,7)
        assert!(!black_matrix[7][2].is_empty(), "黑马应该能攻击(2,7)");
        assert!(!black_matrix[7][0].is_empty(), "黑马应该能攻击(0,7)");
    }

    #[test]
    fn test_is_position_attacked() {
        let board = Board::new();

        // 测试红方攻击
        assert!(
            is_position_attacked(&board, (2, 2), Color::Red),
            "(2,2)应该被红方攻击"
        );
        assert!(
            is_position_attacked(&board, (0, 2), Color::Red),
            "(0,2)应该被红方攻击"
        );

        // 测试黑方攻击
        assert!(
            is_position_attacked(&board, (2, 7), Color::Black),
            "(2,7)应该被黑方攻击"
        );
        assert!(
            is_position_attacked(&board, (0, 7), Color::Black),
            "(0,7)应该被黑方攻击"
        );

        // 测试不应该被攻击的位置 - 使用空棋盘确保没有棋子攻击
        let empty_board = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        assert!(
            !is_position_attacked(&empty_board, (4, 4), Color::Red),
            "(4,4)不应该被红方攻击"
        );
        assert!(
            !is_position_attacked(&empty_board, (4, 4), Color::Black),
            "(4,4)不应该被黑方攻击"
        );
    }

    #[test]
    fn test_get_attackers_to_position() {
        let board = Board::new();

        // 获取攻击(2,2)的红方棋子
        let attackers = get_attackers_to_position(&board, (2, 2), Color::Red);
        assert!(!attackers.is_empty(), "应该有红方棋子攻击(2,2)");

        // 检查攻击者是否是红马
        let has_knight = attackers.iter().any(|&(col, row, piece_type, color)| {
            piece_type == PieceType::Knight && color == Color::Red
        });
        assert!(has_knight, "攻击者中应该有红马");
    }

    #[test]
    fn test_is_king_in_check() {
        let board = Board::new();

        // 初始局面，将/帅不应该被将军
        assert!(
            !is_king_in_check(&board, Color::Red),
            "初始红方不应该被将军"
        );
        assert!(
            !is_king_in_check(&board, Color::Black),
            "初始黑方不应该被将军"
        );

        // 创建一个将军的局面
        let mut board2 = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board2.set_fen(4, 0, 'k'); // 红将
        board2.set_fen(4, 2, 'R'); // 黑车

        // 红方应该被将军
        assert!(is_king_in_check(&board2, Color::Red), "红方应该被将军");

        // 移动黑车，将军解除
        board2.set_fen(4, 2, '.');
        board2.set_fen(5, 2, 'R');
        assert!(!is_king_in_check(&board2, Color::Red), "将军应该解除");
    }

    #[test]
    fn test_quick_attack_check() {
        let mut board = Board::new();

        // 测试红马攻击
        assert!(
            quick_attack_check(&board, (1, 0), (2, 2)),
            "红马应该能攻击(2,2)"
        );
        assert!(
            quick_attack_check(&board, (1, 0), (0, 2)),
            "红马应该能攻击(0,2)"
        );

        // 测试无效攻击
        assert!(
            !quick_attack_check(&board, (1, 0), (3, 3)),
            "红马不能攻击(3,3)"
        );

        // 测试红车攻击 - 创建一个简单局面
        let mut board2 = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board2.set_fen(0, 0, 'r'); // 红车在(0,0)
        assert!(
            quick_attack_check(&board2, (0, 0), (0, 4)),
            "红车应该能攻击(0,4)"
        );

        // 测试红炮攻击
        // 移动红炮到中间位置，设置炮架和目标
        let mut board2 = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board2.set_fen(0, 0, 'c'); // 红炮
        board2.set_fen(0, 1, 'p'); // 红兵（炮架）
        board2.set_fen(0, 2, 'P'); // 黑卒（目标）

        assert!(
            quick_attack_check(&board2, (0, 0), (0, 2)),
            "红炮应该能攻击黑卒"
        );

        // 没有炮架，不能攻击
        board2.set_fen(0, 1, '.');
        assert!(
            !quick_attack_check(&board2, (0, 0), (0, 2)),
            "没有炮架，红炮不能攻击"
        );
    }

    #[test]
    fn test_king_attack_check() {
        let mut board = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board.set_fen(4, 0, 'k'); // 红将

        // 红将应该在九宫内移动
        assert!(quick_attack_check(&board, (4, 0), (4, 1)), "红将应该能前进");
        assert!(quick_attack_check(&board, (4, 0), (3, 0)), "红将应该能左移");
        assert!(quick_attack_check(&board, (4, 0), (5, 0)), "红将应该能右移");

        // 不能出九宫
        assert!(
            !quick_attack_check(&board, (4, 0), (2, 0)),
            "红将不能出九宫"
        );
        assert!(
            !quick_attack_check(&board, (4, 0), (4, 3)),
            "红将不能出九宫"
        );
    }

    #[test]
    fn test_advisor_attack_check() {
        let mut board = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board.set_fen(4, 1, 'a'); // 红士在(4,1)，在九宫内

        // 红士应该能在九宫内对角线移动
        assert!(
            quick_attack_check(&board, (4, 1), (3, 0)),
            "红士应该能左上移动"
        );
        assert!(
            quick_attack_check(&board, (4, 1), (5, 0)),
            "红士应该能右上移动"
        );
        assert!(
            quick_attack_check(&board, (4, 1), (3, 2)),
            "红士应该能左下移动"
        );
        assert!(
            quick_attack_check(&board, (4, 1), (5, 2)),
            "红士应该能右下移动"
        );

        // 不能直线移动
        assert!(
            !quick_attack_check(&board, (4, 3), (4, 2)),
            "红士不能直线移动"
        );
        assert!(
            !quick_attack_check(&board, (4, 3), (4, 4)),
            "红士不能直线移动"
        );
    }

    #[test]
    fn test_elephant_attack_check() {
        let mut board = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board.set_fen(0, 0, 'b'); // 红象在(0,0)

        // 红象应该能田字移动
        assert!(
            quick_attack_check(&board, (0, 0), (2, 2)),
            "红象应该能移动到(2,2)"
        );

        // 检查蹩腿点
        board.set_fen(1, 1, 'p'); // 设置蹩腿点
        assert!(
            !quick_attack_check(&board, (0, 0), (2, 2)),
            "有蹩腿，红象不能移动"
        );

        // 清理蹩腿点
        board.set_fen(1, 1, '.');

        // 不能过河
        assert!(!quick_attack_check(&board, (0, 0), (2, 6)), "红象不能过河");
    }

    #[test]
    fn test_rook_attack_check() {
        let mut board = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board.set_fen(0, 0, 'r'); // 红车在(0,0)

        // 红车应该能直线移动
        assert!(
            quick_attack_check(&board, (0, 0), (0, 5)),
            "红车应该能向下移动"
        );
        assert!(
            quick_attack_check(&board, (0, 0), (5, 0)),
            "红车应该能向右移动"
        );

        // 设置障碍
        board.set_fen(0, 3, 'p'); // 设置障碍
        assert!(
            !quick_attack_check(&board, (0, 0), (0, 5)),
            "有障碍，红车不能通过"
        );

        // 清理障碍
        board.set_fen(0, 3, '.');

        // 不能对角线移动
        assert!(
            !quick_attack_check(&board, (0, 0), (3, 3)),
            "红车不能对角线移动"
        );
    }

    #[test]
    fn test_pawn_attack_check() {
        let mut board = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board.set_fen(4, 3, 'p'); // 红兵在(4,3)，没过河

        // 没过河只能前进
        assert!(quick_attack_check(&board, (4, 3), (4, 4)), "红兵应该能前进");
        assert!(
            !quick_attack_check(&board, (4, 3), (3, 3)),
            "没过河，红兵不能左右移动"
        );
        assert!(
            !quick_attack_check(&board, (4, 3), (5, 3)),
            "没过河，红兵不能左右移动"
        );

        // 过河后可以左右移动
        board.set_fen(4, 6, 'p'); // 红兵在(4,6)，已过河
        assert!(
            quick_attack_check(&board, (4, 6), (4, 7)),
            "过河红兵应该能前进"
        );
        assert!(
            quick_attack_check(&board, (4, 6), (3, 6)),
            "过河红兵应该能左移"
        );
        assert!(
            quick_attack_check(&board, (4, 6), (5, 6)),
            "过河红兵应该能右移"
        );
    }

    #[test]
    fn test_cannon_attack_check() {
        let mut board = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board.set_fen(0, 0, 'c'); // 红炮在(0,0)

        // 炮移动时不能有棋子阻挡
        assert!(
            quick_attack_check(&board, (0, 0), (0, 5)),
            "炮应该能移动到空位"
        );

        // 设置障碍
        board.set_fen(0, 3, 'p'); // 设置障碍
        assert!(
            !quick_attack_check(&board, (0, 0), (0, 5)),
            "有障碍，炮不能移动"
        );

        // 吃子需要炮架
        board.set_fen(0, 5, 'P'); // 黑卒在(0,5)
        assert!(
            quick_attack_check(&board, (0, 0), (0, 5)),
            "有炮架，炮应该能吃子"
        );

        // 清理炮架，不能吃子
        board.set_fen(0, 3, '.');
        assert!(
            !quick_attack_check(&board, (0, 0), (0, 5)),
            "没有炮架，炮不能吃子"
        );
    }
}
