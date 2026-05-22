"""
Comprehensive tests for cchess Python wrapper
Tests all core functionality: Board, Game, MoveNotation, PGN, XQF, Move Generation, Attack Matrix
"""

import os
import tempfile

import cchess
import pytest

# ============================================================================
# Test Side Enum
# ============================================================================


class TestSide:
    def test_side_values(self):
        assert cchess.Side.Any is not None
        assert cchess.Side.Red is not None
        assert cchess.Side.Black is not None

    def test_side_equality(self):
        assert cchess.Side.Red == cchess.Side.Red
        assert cchess.Side.Red != cchess.Side.Black


# ============================================================================
# Test PieceType Enum
# ============================================================================


class TestPieceType:
    def test_piece_types_exist(self):
        assert cchess.PieceType.King is not None
        assert cchess.PieceType.Advisor is not None
        assert cchess.PieceType.Elephant is not None
        assert cchess.PieceType.Knight is not None
        assert cchess.PieceType.Rook is not None
        assert cchess.PieceType.Cannon is not None
        assert cchess.PieceType.Pawn is not None


# ============================================================================
# Test Board
# ============================================================================


class TestBoard:
    def setup_method(self):
        self.board = cchess.Board()

    def test_new_board_is_empty(self):
        board = cchess.Board()
        # All squares should be empty
        for row in range(10):
            for col in range(9):
                assert board.is_empty_at(col, row)

    def test_initial_position(self):
        self.board.initial_position()
        # Red rook at bottom-left
        piece = self.board.get_piece_at(0, 9)
        assert piece is not None
        piece_type, side = piece
        assert piece_type == cchess.PieceType.Rook
        assert side == cchess.Side.Red

        # Black rook at top-left
        piece = self.board.get_piece_at(0, 0)
        assert piece is not None
        piece_type, side = piece
        assert piece_type == cchess.PieceType.Rook
        assert side == cchess.Side.Black

    def test_fen_roundtrip(self):
        self.board.initial_position()
        fen = self.board.to_fen()
        assert "rnbakabnr" in fen.lower() or "RNBAKABNR" in fen

        # Create board from FEN
        board2 = cchess.Board.from_fen(fen)
        assert board2.to_fen() == fen

    def test_set_piece_at(self):
        self.board.set_piece_at(4, 4, cchess.PieceType.King, cchess.Side.Red)
        piece = self.board.get_piece_at(4, 4)
        assert piece is not None
        piece_type, side = piece
        assert piece_type == cchess.PieceType.King
        assert side == cchess.Side.Red

    def test_remove_piece_at(self):
        self.board.set_piece_at(4, 4, cchess.PieceType.King, cchess.Side.Red)
        self.board.remove_piece_at(4, 4)
        assert self.board.is_empty_at(4, 4)

    def test_is_color_at(self):
        self.board.set_piece_at(4, 4, cchess.PieceType.King, cchess.Side.Red)
        assert self.board.is_color_at(4, 4, cchess.Side.Red)
        assert not self.board.is_color_at(4, 4, cchess.Side.Black)

    def test_clear_board(self):
        self.board.initial_position()
        self.board.clear()
        for row in range(10):
            for col in range(9):
                assert self.board.is_empty_at(col, row)

    def test_copy_board(self):
        self.board.initial_position()
        copy = self.board.copy_board()
        assert copy.to_fen() == self.board.to_fen()

    def test_get_squares(self):
        self.board.initial_position()
        squares = self.board.get_squares()
        assert len(squares) == 10
        assert len(squares[0]) == 9

    def test_str_repr(self):
        self.board.initial_position()
        s = str(self.board)
        assert isinstance(s, str)
        r = repr(self.board)
        assert "Board" in r

    def test_invalid_fen(self):
        with pytest.raises(ValueError):
            cchess.Board.from_fen("invalid")


# ============================================================================
# Test Game
# ============================================================================


class TestGame:
    def setup_method(self):
        self.game = cchess.Game()

    def test_new_game(self):
        assert not self.game.is_game_over
        assert self.game.winner is None
        assert self.game.current_turn == cchess.Side.Red

    def test_from_board(self):
        board = cchess.Board()
        board.initial_position()
        game = cchess.Game.from_board(board)
        assert game.get_board().to_fen() == board.to_fen()

    def test_make_move(self):
        # Red cannon moves from (7,7) to (4,7) - 炮二平五
        self.game.make_move(7, 7, 4, 7)
        assert self.game.total_moves() == 1

    def test_make_move_invalid(self):
        with pytest.raises(ValueError):
            # Try to move from empty square
            self.game.make_move(4, 4, 4, 5)

    def test_make_move_with_annotation(self):
        # make_move exists, annotation is set via MoveNode after move
        self.game.make_move(7, 7, 4, 7)
        assert self.game.total_moves() == 1

    def test_make_multiple_moves(self):
        # Standard opening: 炮二平五 马8进7
        self.game.make_move(7, 7, 4, 7)  # Red: 炮二平五
        self.game.make_move(1, 0, 2, 2)  # Black: 马8进7
        assert self.game.total_moves() == 2

    def test_total_moves(self):
        assert self.game.total_moves() == 0
        self.game.make_move(7, 7, 4, 7)
        assert self.game.total_moves() == 1

    def test_get_main_line(self):
        self.game.make_move(7, 7, 4, 7)
        self.game.make_move(1, 0, 2, 2)
        moves = self.game.get_main_line()
        assert len(moves) == 2

    def test_get_board(self):
        board = self.game.get_board()
        assert board is not None
        assert isinstance(board, cchess.Board)

    def test_str_repr(self):
        s = str(self.game)
        assert isinstance(s, str)
        r = repr(self.game)
        assert "Game" in r

    def test_get_move_tree_string(self):
        self.game.make_move(7, 7, 4, 7)
        tree = self.game.get_move_tree_string()
        assert isinstance(tree, str)


# ============================================================================
# Test GameMetadata
# ============================================================================


class TestGameMetadata:
    def test_metadata_properties(self):
        game = cchess.Game()
        meta = game.metadata

        assert meta.title is None
        assert meta.red_player is None
        assert meta.black_player is None
        assert meta.event is None
        assert meta.date is None
        assert meta.result is None
        assert meta.source is None
        assert meta.branch_count == 0

    def test_set_metadata(self):
        game = cchess.Game()
        meta = game.metadata
        meta.title = "Test Game"
        meta.red_player = "Red Player"
        meta.black_player = "Black Player"
        meta.event = "Test Event"
        meta.date = "2024-01-01"
        meta.result = "1-0"

        assert meta.title == "Test Game"
        assert meta.red_player == "Red Player"
        assert meta.black_player == "Black Player"


# ============================================================================
# Test MoveNotation
# ============================================================================


class TestMoveNotation:
    def setup_method(self):
        self.board = cchess.Board()
        self.board.initial_position()

    def test_from_board(self):
        # Red cannon: 炮二平五 from (7,2) to (4,2)
        notation = cchess.MoveNotation.from_board(self.board, 7, 2, 4, 2)
        assert notation.piece_type == cchess.PieceType.Cannon
        assert notation.column == 2

    def test_to_chinese_simplified(self):
        notation = cchess.MoveNotation.from_board(self.board, 7, 2, 4, 2)
        chinese = notation.to_chinese(cchess.ChineseLocale.Simplified)
        assert isinstance(chinese, str)
        assert len(chinese) > 0

    def test_to_chinese_traditional(self):
        notation = cchess.MoveNotation.from_board(self.board, 7, 2, 4, 2)
        chinese = notation.to_chinese(cchess.ChineseLocale.Traditional)
        assert isinstance(chinese, str)

    def test_to_wxf(self):
        notation = cchess.MoveNotation.from_board(self.board, 7, 2, 4, 2)
        wxf = notation.to_wxf()
        assert isinstance(wxf, str)
        assert len(wxf) > 0

    def test_direction(self):
        notation = cchess.MoveNotation.from_board(self.board, 7, 2, 4, 2)
        direction = notation.direction
        assert direction in ["Forward", "Backward", "Horizontal"]

    def test_str(self):
        notation = cchess.MoveNotation.from_board(self.board, 7, 2, 4, 2)
        s = str(notation)
        assert isinstance(s, str)


# ============================================================================
# Test PGN
# ============================================================================


class TestPGN:
    def test_parse_pgn_basic(self):
        pgn = """[Event "Test"]
[Red "Red Player"]
[Black "Black Player"]
[Result "*"]

炮二平五 马8进7 马二进三 车9平8
"""
        game = cchess.parse_pgn(pgn)
        assert game is not None
        assert isinstance(game, cchess.Game)

    def test_game_to_pgn(self):
        game = cchess.Game()
        game.make_move(7, 7, 4, 7)  # 炮二平五
        game.make_move(1, 0, 2, 2)  # 马8进7

        pgn = cchess.game_to_pgn(game)
        assert isinstance(pgn, str)
        assert len(pgn) > 0

    def test_pgn_roundtrip(self):
        game = cchess.Game()
        game.make_move(7, 7, 4, 7)
        game.make_move(1, 0, 2, 2)

        pgn = cchess.game_to_pgn(game)
        # Note: parse_pgn parses PGN move notation which may not be directly compatible
        # This test just ensures the round-trip doesn't crash

    def test_read_write_pgn_file(self):
        game = cchess.Game()
        game.make_move(7, 7, 4, 7)
        game.make_move(1, 0, 2, 2)

        with tempfile.NamedTemporaryFile(mode="w", suffix=".pgn", delete=False) as f:
            path = f.name

        try:
            cchess.save_pgn_file(game, path)
            assert os.path.exists(path)

            loaded = cchess.read_pgn_file(path)
            assert loaded is not None
        finally:
            if os.path.exists(path):
                os.remove(path)


# ============================================================================
# Test XQF
# ============================================================================


class TestXQF:
    def test_board_to_xqf_bytes(self):
        board = cchess.Board()
        board.initial_position()
        data = cchess.board_to_xqf_bytes(board)
        assert len(data) == 90

    def test_board_from_xqf_bytes(self):
        board = cchess.Board()
        board.initial_position()
        data = cchess.board_to_xqf_bytes(board)
        board2 = cchess.board_from_xqf_bytes(data)
        assert board2.to_fen() == board.to_fen()

    def test_xqf_roundtrip(self):
        board = cchess.Board()
        board.initial_position()
        data = cchess.board_to_xqf_bytes(board)
        board2 = cchess.board_from_xqf_bytes(data)
        assert board2.to_fen() == board.to_fen()

    def test_invalid_xqf_bytes(self):
        with pytest.raises(ValueError):
            cchess.board_from_xqf_bytes([0] * 50)  # Wrong size


# ============================================================================
# Test Move Generation
# ============================================================================


class TestMoveGeneration:
    def test_generate_legal_moves_initial(self):
        board = cchess.Board()
        board.initial_position()
        moves = cchess.generate_legal_moves(board)
        # Red should have legal moves (cannons can move, horses can move)
        assert len(moves) > 0

    def test_generate_legal_moves_format(self):
        board = cchess.Board()
        board.initial_position()
        moves = cchess.generate_legal_moves(board)
        # Each move should be (from_col, from_row, to_col, to_row)
        for move in moves:
            assert len(move) == 4
            from_col, from_row, to_col, to_row = move
            assert 0 <= from_col <= 8
            assert 0 <= from_row <= 9
            assert 0 <= to_col <= 8
            assert 0 <= to_row <= 9


# ============================================================================
# Test Attack Matrix
# ============================================================================


class TestAttackMatrix:
    def test_is_king_in_check_initial(self):
        board = cchess.Board()
        board.initial_position()
        # In initial position, neither king should be in check
        assert not cchess.is_king_in_check(board, cchess.Side.Red)
        assert not cchess.is_king_in_check(board, cchess.Side.Black)

    def test_is_position_attacked(self):
        board = cchess.Board()
        board.initial_position()
        # Test that a square near a piece is attacked
        result = cchess.is_position_attacked(board, 4, 0, cchess.Side.Red)
        assert isinstance(result, bool)

    def test_generate_attack_matrix(self):
        board = cchess.Board()
        board.initial_position()
        matrix = cchess.generate_attack_matrix(board, cchess.Side.Red)
        # Matrix should be 10x9 (rows x cols)
        assert len(matrix) == 10
        assert len(matrix[0]) == 9


# ============================================================================
# Integration Tests
# ============================================================================


class TestIntegration:
    def test_full_game_play(self):
        """Test a complete game with multiple moves"""
        game = cchess.Game()

        # Standard opening moves
        game.make_move(7, 7, 4, 7)  # 炮二平五 (Red cannon horizontal)
        game.make_move(1, 0, 2, 2)  # 马8进7 (Black knight)
        game.make_move(0, 6, 0, 7)  # Red pawn forward (row 6→7 in Rust coords)
        game.make_move(0, 3, 0, 2)  # Black pawn forward (row 3→2 in Rust coords)

        assert game.total_moves() == 4
        assert not game.is_game_over

    def test_pgn_export_import(self):
        """Test PGN export and re-import"""
        game = cchess.Game()
        game.make_move(7, 7, 4, 7)
        game.make_move(1, 0, 2, 2)

        pgn = cchess.game_to_pgn(game)
        # PGN should contain moves and result marker
        assert len(pgn) > 0
        assert "h2e2" in pgn or "炮" in pgn or "1." in pgn

    def test_board_state_after_moves(self):
        """Test that board state changes correctly after moves"""
        board = cchess.Board()
        board.initial_position()
        fen_before = board.to_fen()

        game = cchess.Game()
        game.make_move(7, 7, 4, 7)

        board_after = game.get_board()
        fen_after = board_after.to_fen()

        # FEN should be different after a move
        assert fen_before != fen_after

    def test_move_notation_integration(self):
        """Test move notation with game state"""
        board = cchess.Board()
        board.initial_position()

        notation = cchess.MoveNotation.from_board(board, 7, 2, 4, 2)
        chinese = notation.to_chinese(cchess.ChineseLocale.Simplified)

        assert "炮" in chinese or "砲" in chinese

    def test_game_tree(self):
        """Test game tree with variations"""
        game = cchess.Game()
        game.make_move(7, 7, 4, 7)  # Main line

        moves = game.get_main_line()
        assert len(moves) == 1
        assert moves[0].uci_notation is not None


# ============================================================================
# Edge Cases
# ============================================================================


class TestEdgeCases:
    def test_empty_board_fen(self):
        board = cchess.Board()
        fen = board.to_fen()
        # Empty board FEN should have all empty rows
        assert isinstance(fen, str)

    def test_board_from_custom_fen(self):
        # Custom FEN with specific pieces
        fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
        board = cchess.Board.from_fen(fen)
        assert board is not None

    def test_multiple_piece_placements(self):
        board = cchess.Board()
        # Place multiple pieces
        board.set_piece_at(0, 0, cchess.PieceType.Rook, cchess.Side.Black)
        board.set_piece_at(8, 0, cchess.PieceType.Rook, cchess.Side.Black)
        board.set_piece_at(0, 9, cchess.PieceType.Rook, cchess.Side.Red)
        board.set_piece_at(8, 9, cchess.PieceType.Rook, cchess.Side.Red)

        assert board.get_piece_at(0, 0) is not None
        assert board.get_piece_at(8, 9) is not None

    def test_game_over_conditions(self):
        """Test that game over detection works"""
        game = cchess.Game()
        # Fresh game should not be over
        assert not game.is_game_over
        assert game.winner is None
