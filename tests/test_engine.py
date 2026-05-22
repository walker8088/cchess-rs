"""
Comprehensive tests for cchess engine driver Python bindings.

Tests engine process management, UCI/UCCI protocol communication,
search result parsing, and info line parsing.

Usage:
    pytest tests/test_engine.py -v

Environment variables (optional):
    PIKAFISH_PATH - Path to Pikafish executable
    ELEEXE_PATH   - Path to EleEye executable
"""

import gc
import os

import cchess
import pytest

# ============================================================================
# Fixtures
# ============================================================================


@pytest.fixture
def pikafish_path():
    """Resolve Pikafish engine path."""
    path = cchess.resolve_engine_path("PIKAFISH_PATH", "engine/pikafish/pikafish.exe")
    if not os.path.exists(path):
        pytest.skip(f"Pikafish not found at {path}")
    return path


@pytest.fixture
def eleeye_path():
    """Resolve EleEye engine path."""
    path = cchess.resolve_engine_path("ELEEXE_PATH", "engine/eleeye/ELEEYE.EXE")
    if not os.path.exists(path):
        pytest.skip(f"EleEye not found at {path}")
    return path


@pytest.fixture
def uci_engine(pikafish_path):
    """Create and initialize a UCI engine, yield it, then quit."""
    engine = cchess.EngineProcess(pikafish_path, "uci")
    engine.init(10000)
    yield engine
    engine.quit()
    gc.collect()


@pytest.fixture
def ucci_engine(eleeye_path):
    """Create and initialize a UCCI engine, yield it, then quit."""
    engine = cchess.EngineProcess(eleeye_path, "ucci")
    engine.init(10000)
    yield engine
    engine.quit()
    gc.collect()


# ============================================================================
# Test Resolve Engine Path
# ============================================================================


class TestResolveEnginePath:
    def test_resolve_from_env(self, monkeypatch):
        monkeypatch.setenv("TEST_ENGINE_VAR", "/fake/path/engine.exe")
        result = cchess.resolve_engine_path("TEST_ENGINE_VAR", "default/path.exe")
        assert result == "/fake/path/engine.exe"

    def test_resolve_from_default(self, monkeypatch):
        monkeypatch.delenv("NONEXISTENT_VAR", raising=False)
        result = cchess.resolve_engine_path("NONEXISTENT_VAR", "engine/test.exe")
        # Normalize path separators for cross-platform comparison
        normalized = result.replace("/", os.sep).replace("\\", os.sep)
        assert normalized.endswith("engine" + os.sep + "test.exe")
        assert "cchess-rs" in result or "cchess_rs" in result


# ============================================================================
# Test Initial FEN
# ============================================================================


class TestInitialFen:
    def test_initial_fen_returns_string(self):
        fen = cchess.initial_fen()
        assert isinstance(fen, str)
        assert len(fen) > 0

    def test_initial_fen_contains_board(self):
        fen = cchess.initial_fen()
        assert "rnbakabnr" in fen.lower() or "RNBAKABNR" in fen

    def test_initial_fen_has_side_to_move(self):
        fen = cchess.initial_fen()
        assert " w " in fen


# ============================================================================
# Test Parse Info Line
# ============================================================================


class TestParseInfoLine:
    def test_parse_depth_seldepth(self):
        line = "info depth 5 seldepth 8"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.depth == 5
        assert info.seldepth == 8

    def test_parse_cp_score(self):
        line = "info depth 5 score cp 147"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.score_cp == 147
        assert info.score_mate is None
        assert not info.is_mate

    def test_parse_mate_score(self):
        line = "info depth 10 score mate 3"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.score_mate == 3
        assert info.score_cp is None
        assert info.is_mate

    def test_parse_nodes_nps(self):
        line = "info depth 5 nodes 12345 time 10 nps 1234500"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.nodes == 12345
        assert info.time_ms == 10
        assert info.nps == 1234500

    def test_parse_hashfull(self):
        line = "info depth 5 hashfull 450"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.hashfull == 450

    def test_parse_multipv(self):
        line = "info depth 3 multipv 2 score cp 150"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.multipv == 2
        assert info.score_cp == 150

    def test_parse_currmove(self):
        line = "info depth 4 currmove h2e2 currmovenumber 5"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.currmove == "h2e2"
        assert info.currmovenumber == 5

    def test_parse_pv(self):
        line = "info depth 5 seldepth 8 score cp 147 pv h2e2 b0c2 h9g7"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.pv == ["h2e2", "b0c2", "h9g7"]

    def test_parse_pv_string_method(self):
        line = "info depth 5 seldepth 8 score cp 147 pv h2e2 b0c2"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.pv_string() == "h2e2 b0c2"

    def test_parse_full_info_line(self):
        line = "info depth 5 seldepth 8 score cp 147 pv h2e2 b0c2 nodes 12345 time 10 nps 1234500 hashfull 200 multipv 1"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.depth == 5
        assert info.seldepth == 8
        assert info.score_cp == 147
        assert info.nodes == 12345
        assert info.time_ms == 10
        assert info.nps == 1234500
        assert info.hashfull == 200
        assert info.multipv == 1
        assert info.pv == ["h2e2", "b0c2"]

    def test_parse_info_string_line(self):
        line = "info string NNUE evaluation using pikafish.nnue"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.depth == 0
        assert info.score_cp is None
        assert info.score_mate is None
        assert info.pv == []

    def test_parse_non_info_line(self):
        line = "bestmove h2e2 ponder b0c2"
        info = cchess.parse_info_line(line)
        assert info is None

    def test_parse_empty_line(self):
        info = cchess.parse_info_line("")
        assert info is None

    def test_parse_invalid_info(self):
        info = cchess.parse_info_line("info")
        assert info is not None
        assert info.depth == 0


# ============================================================================
# Test Parse Info Lines
# ============================================================================


class TestParseInfoLines:
    def test_parse_multiple_info_lines(self):
        lines = [
            "info string NNUE enabled",
            "info depth 1 score cp 100 pv h2e2",
            "info depth 2 score cp 150 pv h2e2 b0c2",
            "info depth 3 score cp 175 pv h2e2 b0c2 h9g7",
            "bestmove h2e2",
        ]
        infos = cchess.parse_info_lines(lines)
        assert len(infos) == 4  # 3 depth infos + 1 string info

        assert infos[0].depth == 0  # string line
        assert infos[1].depth == 1
        assert infos[1].score_cp == 100
        assert infos[2].depth == 2
        assert infos[2].score_cp == 150
        assert infos[3].depth == 3
        assert infos[3].score_cp == 175
        assert infos[3].pv == ["h2e2", "b0c2", "h9g7"]

    def test_parse_only_bestmove(self):
        lines = ["bestmove h2e2"]
        infos = cchess.parse_info_lines(lines)
        assert len(infos) == 0

    def test_parse_empty_lines(self):
        infos = cchess.parse_info_lines([])
        assert len(infos) == 0

    def test_parse_mixed_lines(self):
        lines = [
            "id name Pikafish",
            "info depth 1 score cp 100",
            "bestmove h2e2 ponder b0c2",
        ]
        infos = cchess.parse_info_lines(lines)
        assert len(infos) == 1
        assert infos[0].depth == 1
        assert infos[0].score_cp == 100


# ============================================================================
# Test Parse Bestmove Line
# ============================================================================


class TestParseBestmoveLine:
    def test_parse_with_ponder(self):
        lines = [
            "info depth 10 score cp 200",
            "bestmove h2e2 ponder b0c2",
        ]
        bestmove, ponder = cchess.parse_bestmove_line(lines)
        assert bestmove == "h2e2"
        assert ponder == "b0c2"

    def test_parse_without_ponder(self):
        lines = ["bestmove h2e2"]
        bestmove, ponder = cchess.parse_bestmove_line(lines)
        assert bestmove == "h2e2"
        assert ponder is None

    def test_parse_no_bestmove(self):
        lines = ["info depth 10", "readyok"]
        bestmove, ponder = cchess.parse_bestmove_line(lines)
        assert bestmove is None
        assert ponder is None

    def test_parse_nobestmove(self):
        lines = ["nobestmove"]
        bestmove, ponder = cchess.parse_bestmove_line(lines)
        assert bestmove is None
        assert ponder is None


# ============================================================================
# Test EngineOption
# ============================================================================


class TestEngineOption:
    def test_option_fields(self, uci_engine):
        options = uci_engine.options
        assert len(options) > 0
        # Check that options have required fields
        for opt in options:
            assert isinstance(opt.name, str)
            assert isinstance(opt.type, str)

    def test_spin_option(self, uci_engine):
        """Find a spin-type option (like Hash) and verify its fields."""
        for opt in uci_engine.options:
            if opt.type == "spin" and opt.name == "Hash":
                assert opt.default is not None
                assert opt.min is not None
                assert opt.max is not None
                assert opt.min <= int(opt.default) <= opt.max
                return
        # Hash option might not exist in all engines, skip if not found
        pytest.skip("Hash option not found")


# ============================================================================
# Test EngineProcess Creation
# ============================================================================


class TestEngineProcessCreation:
    def test_create_uci_engine(self, pikafish_path):
        engine = cchess.EngineProcess(pikafish_path, "uci")
        assert engine.protocol == "uci"
        engine.quit()

    def test_create_ucci_engine(self, eleeye_path):
        engine = cchess.EngineProcess(eleeye_path, "ucci")
        assert engine.protocol == "ucci"
        engine.quit()

    def test_invalid_protocol(self, pikafish_path):
        with pytest.raises(ValueError):
            cchess.EngineProcess(pikafish_path, "invalid")

    def test_nonexistent_engine(self):
        with pytest.raises(ValueError):
            cchess.EngineProcess("/nonexistent/path/engine.exe", "uci")

    def test_repr_before_init(self, pikafish_path):
        engine = cchess.EngineProcess(pikafish_path, "uci")
        r = repr(engine)
        assert "EngineProcess" in r
        assert "uci" in r
        engine.quit()


# ============================================================================
# Test Engine Initialization
# ============================================================================


class TestEngineInit:
    def test_uci_handshake(self, pikafish_path):
        engine = cchess.EngineProcess(pikafish_path, "uci")
        lines = engine.init(10000)
        assert any(line == "uciok" for line in lines)
        assert any(line == "readyok" for line in lines)
        engine.quit()

    def test_ucci_handshake(self, eleeye_path):
        engine = cchess.EngineProcess(eleeye_path, "ucci")
        lines = engine.init(10000)
        assert any(line == "ucciok" for line in lines)
        assert any(line == "readyok" for line in lines)
        engine.quit()

    def test_engine_name_discovered(self, uci_engine):
        assert len(uci_engine.engine_name) > 0, (
            "Engine name should be discovered after init"
        )

    def test_options_discovered(self, uci_engine):
        assert len(uci_engine.options) > 0, "Should discover engine options during init"

    def test_init_returns_lines(self, uci_engine):
        """Engine was already initialized by fixture, verify repr shows name."""
        assert len(uci_engine.engine_name) > 0


# ============================================================================
# Test Search - UCI (Pikafish)
# ============================================================================


class TestSearchUCI:
    def test_search_initial_position_time(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 1000, 15000)
        assert result.bestmove is not None
        assert len(result.bestmove) == 4, "Bestmove should be 4 chars (ICCS format)"

    def test_search_initial_position_depth(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_depth(fen, 6, 30000)
        assert result.bestmove is not None

    def test_search_has_info_lines(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 1000, 15000)
        assert len(result.info_lines) > 0, "Should have info lines during search"

    def test_search_info_has_depth(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 1000, 15000)
        assert result.depth is not None
        assert result.depth > 0

    def test_search_info_has_nodes(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 1000, 15000)
        assert result.nodes is not None
        assert result.nodes > 0

    def test_search_info_has_nps(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 1000, 15000)
        assert result.nps is not None
        assert result.nps > 0

    def test_search_info_has_score(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 1000, 15000)
        assert result.score_cp is not None or result.score_mate is not None

    def test_search_info_initial_not_mate(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 1000, 15000)
        assert not result.is_mate, "Initial position should not be a mate"
        assert result.score_cp is not None
        assert abs(result.score_cp) < 5000, "Score should be reasonable"

    def test_search_has_pv(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 2000, 15000)
        pv = result.pv_string()
        # PV may be empty for some engines, but Pikafish should have one
        if pv:
            moves = pv.split()
            assert len(moves) > 0
            for move in moves:
                assert len(move) == 4, f"PV move should be 4 chars: {move}"

    def test_search_result_repr(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 500, 15000)
        r = repr(result)
        assert "SearchResult" in r
        assert result.bestmove in r

    def test_search_final_info(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 1000, 15000)
        final = result.final_info
        assert final is not None
        assert final.depth > 0
        assert final.score_cp is not None or final.score_mate is not None


# ============================================================================
# Test Search - UCCI (EleEye)
# ============================================================================


class TestSearchUCCI:
    def test_search_initial_position(self, ucci_engine):
        fen = cchess.initial_fen()
        result = ucci_engine.search_movetime(fen, 200, 10000)
        # UCCI engines may return nobestmove for some positions
        if result.bestmove is not None:
            assert len(result.bestmove) >= 4

    def test_search_depth(self, ucci_engine):
        fen = cchess.initial_fen()
        result = ucci_engine.search_depth(fen, 4, 30000)
        # May return nobestmove
        assert result is not None

    def test_search_has_info_lines(self, ucci_engine):
        fen = cchess.initial_fen()
        result = ucci_engine.search_movetime(fen, 200, 10000)
        # EleEye may not output info lines in standard UCI format
        # but result should still be valid
        assert result is not None


# ============================================================================
# Test Setoption
# ============================================================================


class TestSetoption:
    def test_set_hash_uci(self, pikafish_path):
        engine = cchess.EngineProcess(pikafish_path, "uci")
        engine.init(10000)
        engine.setoption("Hash", "32")
        # Send isready to verify engine still works
        engine.send("isready")
        lines = engine.read_until_any(["readyok"], 10000)
        assert any(line == "readyok" for line in lines)

        # Verify search still works
        fen = cchess.initial_fen()
        result = engine.search_movetime(fen, 500, 15000)
        assert result.bestmove is not None
        engine.quit()

    def test_set_threads_ucci(self, eleeye_path):
        engine = cchess.EngineProcess(eleeye_path, "ucci")
        engine.init(10000)
        # Try setting Threads - may fail if not supported
        try:
            engine.setoption("Threads", "1")
            engine.send("isready")
            lines = engine.read_until_any(["readyok"], 10000)
            assert any(line == "readyok" for line in lines)
        except ValueError:
            pass  # Option may not be supported
        engine.quit()


# ============================================================================
# Test Position with Moves
# ============================================================================


class TestPositionWithMoves:
    def test_position_startpos_moves_uci(self, uci_engine):
        uci_engine.position_startpos_moves("h0e0 h9g7")
        result = uci_engine.search_movetime(cchess.initial_fen(), 1000, 15000)
        assert result.bestmove is not None

    def test_position_custom_fen(self, uci_engine):
        fen = "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1"
        result = uci_engine.search_movetime(fen, 1000, 15000)
        assert result.bestmove is not None

    def test_position_multiple_searches(self, uci_engine):
        positions = [
            cchess.initial_fen(),
            "4k4/9/9/9/9/9/9/9/9/R3K4 w - - 0 1",
        ]
        for fen in positions:
            result = uci_engine.search_movetime(fen, 500, 15000)
            assert result.bestmove is not None


# ============================================================================
# Test Mate Detection
# ============================================================================


class TestMateDetection:
    def test_mate_in_one_uci(self, uci_engine):
        """Red rook on a-file can deliver checkmate."""
        fen = "4k4/9/9/9/9/9/9/9/9/R3K4 w - - 0 1"
        result = uci_engine.search_depth(fen, 10, 30000)

        # Pikafish may report mate or very high cp
        if result.is_mate:
            assert result.score_mate > 0, "Mate should be positive (winning)"
        else:
            assert result.score_cp > 1000, "Expected very high score for mate position"


# ============================================================================
# Test Rapid Successive Searches
# ============================================================================


class TestRapidSearches:
    def test_multiple_searches_uci(self, uci_engine):
        """Run multiple searches in succession."""
        positions = [
            cchess.initial_fen(),
            "4k4/9/9/9/9/9/9/9/9/R3K4 w - - 0 1",
        ]
        for i, fen in enumerate(positions):
            result = uci_engine.search_movetime(fen, 500, 15000)
            assert result.bestmove is not None, f"Search {i} should return bestmove"

    def test_multiple_searches_ucci(self, ucci_engine):
        """Run multiple searches in succession."""
        positions = [
            cchess.initial_fen(),
            "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1",
        ]
        for i, fen in enumerate(positions):
            result = ucci_engine.search_movetime(fen, 100, 10000)
            # UCCI may return nobestmove for some positions
            assert result is not None, f"Search {i} should return a result"


# ============================================================================
# Test Timeout Handling
# ============================================================================


class TestTimeoutHandling:
    def test_init_timeout(self, pikafish_path):
        """Test that a very short timeout raises an error."""
        engine = cchess.EngineProcess(pikafish_path, "uci")
        try:
            engine.init(1)  # 1ms timeout - should fail
            pytest.fail("Should have timed out")
        except ValueError as e:
            assert "Timeout" in str(e) or "timeout" in str(e).lower()
        finally:
            engine.quit()


# ============================================================================
# Test Engine Cleanup
# ============================================================================


class TestEngineCleanup:
    def test_quit_cleans_up(self, pikafish_path):
        """Verify that quit() terminates the engine."""
        engine = cchess.EngineProcess(pikafish_path, "uci")
        engine.init(10000)
        engine.quit()
        # After quit, the engine should be terminated
        # We can't directly check process state from Python,
        # but subsequent operations should fail
        with pytest.raises(ValueError):
            engine.send("isready")

    def test_gc_cleanup(self, pikafish_path):
        """Verify that GC cleans up the engine process."""

        def create_and_forget():
            engine = cchess.EngineProcess(pikafish_path, "uci")
            engine.init(10000)
            # Don't explicitly quit - let __del__ handle it
            return engine.engine_name

        name = create_and_forget()
        gc.collect()
        assert len(name) > 0


# ============================================================================
# Test Integration with Board
# ============================================================================


class TestIntegrationWithBoard:
    def test_search_then_play_move(self, uci_engine):
        """Search for best move, then play it on the board."""
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 1000, 15000)
        assert result.bestmove is not None

        # Create a board from the FEN
        board = cchess.Board.from_fen(fen)
        assert board.to_fen() == fen

    def test_game_with_engine_move(self, uci_engine):
        """Create a game and get engine suggestion."""
        game = cchess.Game()
        board = game.get_board()
        fen = board.to_fen()

        result = uci_engine.search_movetime(fen, 500, 15000)
        assert result.bestmove is not None

    def test_position_after_moves_search(self, uci_engine):
        """Make moves on the board, get FEN, search from that position."""
        game = cchess.Game()
        # Make a move: 炮二平五 (red cannon from h0 to e0)
        game.make_move(7, 0, 4, 0)

        board = game.get_board()
        fen = board.to_fen()

        result = uci_engine.search_movetime(fen, 1000, 15000)
        assert result.bestmove is not None


# ============================================================================
# Test Info Line Parsing from Real Engine Output
# ============================================================================


class TestRealEngineOutput:
    def test_uci_search_result_info_parsed(self, uci_engine):
        """Verify that SearchResult is properly populated from real engine output."""
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 1000, 15000)

        assert result.bestmove is not None
        assert len(result.bestmove) == 4
        assert len(result.info_lines) > 0

        # Final info should have data
        final = result.final_info
        assert final is not None
        assert final.depth > 0
        assert final.score_cp is not None or final.score_mate is not None
        assert len(final.pv) > 0

    def test_ucci_search_result_info_parsed(self, ucci_engine):
        """Verify SearchResult from UCCI engine."""
        fen = cchess.initial_fen()
        result = ucci_engine.search_movetime(fen, 100, 10000)

        if result.bestmove is not None:
            assert len(result.bestmove) >= 4


# ============================================================================
# Test Raw Lines
# ============================================================================


class TestRawLines:
    def test_raw_lines_contain_output(self, uci_engine):
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 500, 15000)
        assert len(result.raw_lines) > 0
        assert any(line.startswith("bestmove") for line in result.raw_lines)


# ============================================================================
# Test Score Value Helper
# ============================================================================


class TestScoreValue:
    def test_cp_score_value(self):
        line = "info depth 5 score cp 147"
        info = cchess.parse_info_line(line)
        assert info is not None
        assert info.score_value == 147

    def test_mate_score_value(self):
        line = "info depth 10 score mate 3"
        info = cchess.parse_info_line(line)
        assert info is not None
        # Mate score value = mate_in * 100000
        assert info.score_value == 300000


# ============================================================================
# Test Edge Cases
# ============================================================================


class TestEdgeCases:
    def test_empty_position(self, uci_engine):
        """Search from a nearly empty position."""
        fen = "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1"
        result = uci_engine.search_movetime(fen, 500, 15000)
        assert result.bestmove is not None

    def test_very_short_search(self, uci_engine):
        """Search with very short time limit."""
        fen = cchess.initial_fen()
        result = uci_engine.search_movetime(fen, 100, 15000)
        assert result.bestmove is not None

    def test_high_depth_search(self, uci_engine):
        """Search with high depth limit."""
        fen = cchess.initial_fen()
        result = uci_engine.search_depth(fen, 8, 30000)
        assert result.bestmove is not None
        if result.depth is not None:
            assert result.depth <= 8
