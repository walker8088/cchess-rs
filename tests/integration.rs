// Integration tests for cchess-rs

#[cfg(test)]
mod tests {
    use cchess_rs::board::Board;
    use cchess_rs::game::Game;
    use cchess_rs::move_gen::generate_moves;
    use cchess_rs::pieces::{Color, PieceType};

    #[test]
    fn test_color_opposite() {
        assert_eq!(Color::Red.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::Red);
    }

    #[test]
    fn test_color_from_fen() {
        assert_eq!(Color::from_fen('k'), Some(Color::Red));
        assert_eq!(Color::from_fen('K'), Some(Color::Black));
        assert_eq!(Color::from_fen('.'), None);
        assert_eq!(Color::from_fen('x'), None);
    }

    #[test]
    fn test_piece_type_from_fen() {
        assert_eq!(PieceType::from_fen('k'), Some(PieceType::King));
        assert_eq!(PieceType::from_fen('a'), Some(PieceType::Advisor));
        assert_eq!(PieceType::from_fen('b'), Some(PieceType::Elephant));
        assert_eq!(PieceType::from_fen('n'), Some(PieceType::Knight));
        assert_eq!(PieceType::from_fen('r'), Some(PieceType::Rook));
        assert_eq!(PieceType::from_fen('c'), Some(PieceType::Cannon));
        assert_eq!(PieceType::from_fen('p'), Some(PieceType::Pawn));
        assert_eq!(PieceType::from_fen('K'), Some(PieceType::King)); // Uppercase also works
        assert_eq!(PieceType::from_fen('.'), None);
        assert_eq!(PieceType::from_fen('x'), None);
    }

    #[test]
    fn test_board_creation() {
        let board = Board::new();

        // Test that board is initialized with correct FEN characters
        // Red back row (row 0)
        assert_eq!(board.get_fen(0, 0), 'r'); // Red rook at col0, row0
        assert_eq!(board.get_fen(1, 0), 'n'); // Red knight at col1, row0
        assert_eq!(board.get_fen(2, 0), 'b'); // Red elephant at col2, row0
        assert_eq!(board.get_fen(3, 0), 'a'); // Red advisor at col3, row0
        assert_eq!(board.get_fen(4, 0), 'k'); // Red king at col4, row0
        assert_eq!(board.get_fen(5, 0), 'a'); // Red advisor at col5, row0
        assert_eq!(board.get_fen(6, 0), 'b'); // Red elephant at col6, row0
        assert_eq!(board.get_fen(7, 0), 'n'); // Red knight at col7, row0
        assert_eq!(board.get_fen(8, 0), 'r'); // Red rook at col8, row0

        // Red cannons (row 2)
        assert_eq!(board.get_fen(1, 2), 'c'); // Red cannon at col1, row2
        assert_eq!(board.get_fen(7, 2), 'c'); // Red cannon at col7, row2

        // Red pawns (row 3, every other column)
        assert_eq!(board.get_fen(0, 3), 'p'); // Red pawn at col0, row3
        assert_eq!(board.get_fen(2, 3), 'p'); // Red pawn at col2, row3
        assert_eq!(board.get_fen(4, 3), 'p'); // Red pawn at col4, row3
        assert_eq!(board.get_fen(6, 3), 'p'); // Red pawn at col6, row3
        assert_eq!(board.get_fen(8, 3), 'p'); // Red pawn at col8, row3

        // Black pawns (row 6, every other column)
        assert_eq!(board.get_fen(0, 6), 'P'); // Black pawn at col0, row6
        assert_eq!(board.get_fen(2, 6), 'P'); // Black pawn at col2, row6
        assert_eq!(board.get_fen(4, 6), 'P'); // Black pawn at col4, row6
        assert_eq!(board.get_fen(6, 6), 'P'); // Black pawn at col6, row6
        assert_eq!(board.get_fen(8, 6), 'P'); // Black pawn at col8, row6

        // Black cannons (row 7)
        assert_eq!(board.get_fen(1, 7), 'C'); // Black cannon at col1, row7
        assert_eq!(board.get_fen(7, 7), 'C'); // Black cannon at col7, row7

        // Black back row (row 9)
        assert_eq!(board.get_fen(0, 9), 'R'); // Black rook at col0, row9
        assert_eq!(board.get_fen(1, 9), 'N'); // Black knight at col1, row9
        assert_eq!(board.get_fen(2, 9), 'B'); // Black elephant at col2, row9
        assert_eq!(board.get_fen(3, 9), 'A'); // Black advisor at col3, row9
        assert_eq!(board.get_fen(4, 9), 'K'); // Black king at col4, row9
        assert_eq!(board.get_fen(5, 9), 'A'); // Black advisor at col5, row9
        assert_eq!(board.get_fen(6, 9), 'B'); // Black elephant at col6, row9
        assert_eq!(board.get_fen(7, 9), 'N'); // Black knight at col7, row9
        assert_eq!(board.get_fen(8, 9), 'R'); // Black rook at col8, row9

        // Test some empty squares
        assert_eq!(board.get_fen(0, 1), '.'); // Empty at col0, row1
        assert_eq!(board.get_fen(4, 1), '.'); // Empty at col4, row1
        assert_eq!(board.get_fen(1, 3), '.'); // Empty at col1, row3 (between pawns)
        assert_eq!(board.get_fen(3, 3), '.'); // Empty at col3, row3 (between pawns)
    }

    #[test]
    fn test_board_methods() {
        let board = Board::new();

        // Test is_color_at
        assert!(board.is_color_at(0, 0, Color::Red)); // Red rook at col0, row0
        assert!(board.is_color_at(0, 9, Color::Black)); // Black rook at col0, row9
        assert!(!board.is_color_at(0, 0, Color::Black)); // Not black

        // Test get_piece_type
        assert_eq!(board.get_piece_type(0, 0), Some(PieceType::Rook));
        assert_eq!(board.get_piece_type(4, 0), Some(PieceType::King));
        assert_eq!(board.get_piece_type(2, 0), Some(PieceType::Elephant));
        assert_eq!(board.get_piece_type(1, 0), Some(PieceType::Knight));
        assert_eq!(board.get_piece_type(3, 0), Some(PieceType::Advisor));
        assert_eq!(board.get_piece_type(0, 1), None); // Empty square

        // Test get_color_at
        assert_eq!(board.get_color_at(0, 0), Some(Color::Red));
        assert_eq!(board.get_color_at(0, 9), Some(Color::Black));
        assert_eq!(board.get_color_at(0, 1), None); // Empty square

        // Test is_empty_at and has_piece_at
        assert!(!board.is_empty_at(0, 0));
        assert!(board.has_piece_at(0, 0));
        assert!(board.is_empty_at(0, 1));
        assert!(!board.has_piece_at(0, 1));
    }

    #[test]
    fn test_game_creation() {
        let game = Game::new();
        assert_eq!(game.current_turn, Color::Red);
        assert_eq!(game.is_game_over, false);
        assert_eq!(game.winner, None);
    }

    #[test]
    fn test_move_generation() {
        let board = Board::new();
        let moves = generate_moves(&board, Color::Red);
        // Red should have some initial moves
        // At minimum, red pawns and cannons should have moves
        assert!(
            moves.len() > 0,
            "Red should have at least some initial moves"
        );

        // Test that we can generate moves for black too
        let black_moves = generate_moves(&board, Color::Black);
        assert!(
            black_moves.len() > 0,
            "Black should have at least some initial moves"
        );
    }

    #[test]
    fn test_board_from_fen() {
        // Test standard starting position FEN
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR";
        let board = Board::from_fen(fen).expect("Failed to parse FEN");

        // Verify some key pieces
        assert_eq!(board.get_fen(0, 0), 'r'); // Red rook
        assert_eq!(board.get_fen(4, 0), 'k'); // Red king
        assert_eq!(board.get_fen(0, 9), 'R'); // Black rook
        assert_eq!(board.get_fen(4, 9), 'K'); // Black king

        // Verify cannons
        assert_eq!(board.get_fen(1, 2), 'c'); // Red cannon
        assert_eq!(board.get_fen(1, 7), 'C'); // Black cannon

        // Verify empty squares
        assert_eq!(board.get_fen(0, 1), '.'); // Empty square
        assert_eq!(board.get_fen(4, 1), '.'); // Empty square

        // Test to_fen round trip
        let fen2 = board.to_fen();
        let board2 = Board::from_fen(&fen2).expect("Failed to parse generated FEN");

        // Verify boards are equal
        for row in 0..10 {
            for col in 0..9 {
                assert_eq!(
                    board.get_fen(col, row),
                    board2.get_fen(col, row),
                    "Boards differ at col={}, row={}",
                    col,
                    row
                );
            }
        }
    }

    #[test]
    fn test_board_to_fen() {
        let board = Board::new();
        let fen = board.to_fen();

        // Basic validation of FEN format
        assert!(fen.contains('/'), "FEN should contain row separators");
        assert_eq!(
            fen.matches('/').count(),
            9,
            "FEN should have 9 separators for 10 rows"
        );

        // Parse it back and verify
        let board2 = Board::from_fen(&fen).expect("Failed to parse generated FEN");

        for row in 0..10 {
            for col in 0..9 {
                assert_eq!(
                    board.get_fen(col, row),
                    board2.get_fen(col, row),
                    "Boards differ at col={}, row={}",
                    col,
                    row
                );
            }
        }
    }

    #[test]
    fn test_board_clear() {
        let mut board = Board::new();

        // Verify board is not empty initially
        assert!(board.has_piece_at(0, 0)); // Should have a rook

        // Clear the board
        board.clear();

        // Verify all squares are empty
        for row in 0..10 {
            for col in 0..9 {
                assert!(
                    board.is_empty_at(col, row),
                    "Square at col={}, row={} should be empty",
                    col,
                    row
                );
            }
        }

        // Verify FEN after clear
        let fen = board.to_fen();
        assert_eq!(fen, "9/9/9/9/9/9/9/9/9/9");
    }

    #[test]
    fn test_fen_error_handling() {
        // Test invalid row count
        let fen1 = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9"; // Only 9 rows
        assert!(Board::from_fen(fen1).is_err());

        // Test row too long
        let fen2 = "rnbakabnr/9/1c5c1/p1p1p1p1p1/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR"; // Row 4 has 10 columns
        assert!(Board::from_fen(fen2).is_err());

        // Test row too short
        let fen3 = "rnbakabnr/8/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR"; // Row 2 has 8 columns
        assert!(Board::from_fen(fen3).is_err());

        // Test invalid characters
        let fen4 = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNX"; // Invalid piece 'X'
                                                                                  // Note: We don't validate piece characters in from_fen, so this should succeed
                                                                                  // The validation happens in Color::from_fen and PieceType::from_fen
        assert!(Board::from_fen(fen4).is_ok());
    }

    #[test]
    fn test_board_make_move() {
        // Test 1: 基本移动 - 移动红方的兵
        let mut board = Board::new();
        // 红兵在(0, 3)，只能前进
        let result = board.make_move((0, 3), (0, 4));
        assert!(result, "红兵应该可以前进");

        // 验证移动后的棋盘状态
        assert_eq!(board.get_fen(0, 3), '.', "起始位置应该为空");
        assert_eq!(board.get_fen(0, 4), 'p', "目标位置应该有红兵");

        // Test 2: 无效移动 - 从空位置移动
        let result = board.make_move((0, 1), (0, 2));
        assert!(!result, "从空位置移动应该失败");

        // Test 3: 无效移动 - 坐标超出棋盘范围
        let result = board.make_move((0, 0), (10, 0));
        assert!(!result, "坐标超出范围应该失败");

        // Test 4: 不能吃自己的棋子
        let mut board2 = Board::new();
        // 红兵尝试吃自己的红车（不可能，但测试逻辑）
        let result = board2.make_move((0, 3), (0, 0));
        assert!(!result, "不能吃自己的棋子");

        // Test 5: 简单吃子测试 - 创建简单局面
        let mut board3 = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board3.set_fen(0, 0, 'p'); // 红兵
        board3.set_fen(0, 1, 'P'); // 黑卒

        // 红兵吃黑卒
        let result = board3.make_move((0, 0), (0, 1));
        assert!(result, "红兵应该可以吃黑卒");
        assert_eq!(board3.get_fen(0, 1), 'p', "目标位置应该有红兵");

        // Test 6: 炮的特殊规则 - 移动时不能有棋子阻挡
        let mut board4 = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board4.set_fen(0, 0, 'c'); // 红炮
        board4.set_fen(0, 1, 'p'); // 红兵阻挡

        // 炮移动时中间有棋子应该失败
        let result = board4.make_move((0, 0), (0, 2));
        assert!(!result, "炮移动时中间有棋子应该失败");

        // Test 7: 炮吃子需要炮架
        let mut board5 = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board5.set_fen(0, 0, 'c'); // 红炮
        board5.set_fen(0, 2, 'P'); // 黑卒（目标）

        // 炮吃子没有炮架应该失败
        let result = board5.make_move((0, 0), (0, 2));
        assert!(!result, "炮吃子没有炮架应该失败");

        // Test 8: 炮吃子有炮架应该成功
        let mut board6 = Board::from_fen("9/9/9/9/9/9/9/9/9/9").unwrap();
        board6.set_fen(0, 0, 'c'); // 红炮
        board6.set_fen(0, 1, 'p'); // 红兵（炮架）
        board6.set_fen(0, 2, 'P'); // 黑卒（目标）

        // 炮吃子有炮架应该成功
        let result = board6.make_move((0, 0), (0, 2));
        assert!(result, "炮吃子有炮架应该成功");
        assert_eq!(board6.get_fen(0, 2), 'c', "目标位置应该有红炮");
        assert_eq!(board6.get_fen(0, 1), 'p', "炮架应该还在原位");
    }

    #[test]
    fn test_board_helper_functions() {
        // Test is_within_bounds
        assert!(Board::is_within_bounds(0, 0), "(0,0)应该在棋盘内");
        assert!(Board::is_within_bounds(8, 9), "(8,9)应该在棋盘内");
        assert!(!Board::is_within_bounds(9, 0), "(9,0)应该在棋盘外");
        assert!(!Board::is_within_bounds(0, 10), "(0,10)应该在棋盘外");

        // Test copy
        let board1 = Board::new();
        let board2 = board1.copy();
        assert!(board1.equals(&board2), "复制后的棋盘应该相等");

        // Test equals
        let board3 = Board::new();
        let mut board4 = Board::new();
        assert!(board3.equals(&board4), "相同的棋盘应该相等");

        // 修改board4，然后应该不相等
        board4.set_fen(0, 0, '.');
        assert!(!board3.equals(&board4), "不同的棋盘应该不相等");

        // Test count_pieces
        let board5 = Board::new();
        let piece_count = board5.count_pieces();
        assert!(piece_count > 0, "初始棋盘应该有棋子");

        // Test count_color_pieces
        let red_count = board5.count_color_pieces(true);
        let black_count = board5.count_color_pieces(false);
        assert!(red_count > 0, "初始棋盘应该有红方棋子");
        assert!(black_count > 0, "初始棋盘应该有黑方棋子");
        assert_eq!(red_count + black_count, piece_count, "棋子总数应该匹配");

        // Test is_empty
        let mut board6 = Board::new();
        assert!(!board6.is_empty(), "初始棋盘不应该为空");
        board6.clear();
        assert!(board6.is_empty(), "清空后的棋盘应该为空");

        // Test get_all_piece_positions
        let board7 = Board::new();
        let positions = board7.get_all_piece_positions();
        assert_eq!(positions.len(), piece_count, "棋子位置数量应该匹配棋子总数");

        // Test get_color_piece_positions
        let red_positions = board7.get_color_piece_positions(true);
        let black_positions = board7.get_color_piece_positions(false);
        assert_eq!(red_positions.len(), red_count, "红方棋子位置数量应该匹配");
        assert_eq!(
            black_positions.len(),
            black_count,
            "黑方棋子位置数量应该匹配"
        );

        // Test is_in_palace
        assert!(Board::is_in_palace(3, 0, true), "(3,0)应该在红方九宫内");
        assert!(Board::is_in_palace(4, 1, true), "(4,1)应该在红方九宫内");
        assert!(Board::is_in_palace(5, 2, true), "(5,2)应该在红方九宫内");
        assert!(!Board::is_in_palace(2, 0, true), "(2,0)不应该在九宫内");
        assert!(!Board::is_in_palace(3, 3, true), "(3,3)不应该在红方九宫内");

        assert!(Board::is_in_palace(3, 7, false), "(3,7)应该在黑方九宫内");
        assert!(Board::is_in_palace(4, 8, false), "(4,8)应该在黑方九宫内");
        assert!(Board::is_in_palace(5, 9, false), "(5,9)应该在黑方九宫内");
        assert!(!Board::is_in_palace(2, 7, false), "(2,7)不应该在九宫内");
        assert!(!Board::is_in_palace(3, 6, false), "(3,6)不应该在黑方九宫内");

        // Test is_across_river
        assert!(Board::is_across_river(5, true), "第5行对红方来说是过河");
        assert!(Board::is_across_river(6, true), "第6行对红方来说是过河");
        assert!(!Board::is_across_river(4, true), "第4行对红方来说没过河");

        assert!(Board::is_across_river(4, false), "第4行对黑方来说是过河");
        assert!(Board::is_across_river(3, false), "第3行对黑方来说是过河");
        assert!(!Board::is_across_river(5, false), "第5行对黑方来说没过河");

        // Test distance
        let (dx, dy) = Board::distance(0, 0, 3, 2);
        assert_eq!(dx, 3, "x方向距离应该是3");
        assert_eq!(dy, 2, "y方向距离应该是2");

        // Test manhattan_distance
        let dist = Board::manhattan_distance(0, 0, 3, 2);
        assert_eq!(dist, 5, "曼哈顿距离应该是5");

        let dist2 = Board::manhattan_distance(4, 4, 4, 4);
        assert_eq!(dist2, 0, "相同位置的曼哈顿距离应该是0");
    }
}
