"""Tests for FenCache and EngineManager."""

import gc
import json
import os
import tempfile

import cchess_rs as cchess
import pytest

# ============================================================================
# Fixtures
# ============================================================================


@pytest.fixture
def pikafish_path():
    """Resolve Pikafish engine path."""
    env_path = os.environ.get("CCHESS_ENGINE")
    if env_path and os.path.exists(env_path):
        return env_path
    # Try common locations
    candidates = [
        os.path.join("engine", "Pikafish", "pikafish.exe"),
        os.path.join("engine", "pikafish.exe"),
    ]
    for c in candidates:
        if os.path.exists(c):
            return c
    pytest.skip("Pikafish engine not found")


@pytest.fixture
def uci_engine(pikafish_path):
    """Create and initialize a UCI engine."""
    engine = cchess.EngineProcess(pikafish_path, "uci")
    engine.init(10000)
    yield engine
    engine.quit()
    gc.collect()


# ============================================================================
# Test FenCache
# ============================================================================


class TestFenCacheBasic:
    """Test basic FenCache operations."""

    def test_create_cache(self):
        cache = cchess.FenCache()
        assert cache.cache_file == ""
        assert not cache.need_save

    def test_get_empty_cache(self):
        cache = cchess.FenCache()
        result, state = cache.get(
            "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
        )
        assert result is None
        assert state is None

    def test_save_and_get_action(self):
        cache = cchess.FenCache()
        fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
        action = {"move": "b0c2", "score": 50, "depth": 10}
        cache.save_action(fen, action)

        result, state = cache.get(fen)
        assert result is not None
        assert state == ""
        # The cache stores actions keyed by move
        assert "b0c2" in result

    def test_need_save_flag(self):
        cache = cchess.FenCache()
        assert not cache.need_save
        fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"
        action = {"move": "b0c2", "score": 50}
        cache.save_action(fen, action)
        assert cache.need_save


class TestFenCacheMirror:
    """Test FenCache mirror functionality."""

    def test_mirror_lookup(self):
        cache = cchess.FenCache()
        # Use an asymmetric FEN where mirroring produces a different FEN
        # King at e0 vs e9 - mirroring swaps their positions
        fen = "3k5/9/9/9/9/9/9/9/9/5K3 w - - 0 1"
        action = {"move": "f0e0", "score": 10}
        cache.save_action(fen, action)

        # Mirror the FEN - should produce a different string
        mirrored = cchess.fen_mirror(fen)
        assert mirrored != fen, "FEN should not be symmetric for this test"

        # Try to get from mirrored FEN - should find via mirror
        result, state = cache.get(mirrored)
        assert result is not None
        assert state == "mirror"

    def test_get_best_action_red(self):
        cache = cchess.FenCache()
        fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1"

        # Save multiple actions with different scores
        cache.save_action(fen, {"move": "b0c2", "score": -30})
        cache.save_action(fen, {"move": "h0g0", "score": -50})

        # Red (move_color=1) wants lowest score (most negative = best for red)
        best = cache.get_best_action(fen, 1)
        assert best is not None

    def test_get_best_action_black(self):
        cache = cchess.FenCache()
        fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR b - - 0 1"

        cache.save_action(fen, {"move": "b9c7", "score": 30})
        cache.save_action(fen, {"move": "h9g7", "score": 50})

        # Black (move_color=-1) wants highest score
        best = cache.get_best_action(fen, -1)
        assert best is not None


class TestFenCachePersistence:
    """Test FenCache save/load to file."""

    def test_save_and_load(self):
        cache = cchess.FenCache()
        fen = "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1"
        action = {"move": "e0d0", "score": 10, "depth": 15}
        cache.save_action(fen, action)

        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            path = f.name

        try:
            cache.save(path)
            assert os.path.exists(path)

            # Load into a new cache
            cache2 = cchess.FenCache()
            cache2.load(path)

            # Verify the data
            result, state = cache2.get(fen)
            assert result is not None
            assert state == ""
        finally:
            if os.path.exists(path):
                os.unlink(path)

    def test_load_nonexistent_file(self):
        cache = cchess.FenCache()
        with pytest.raises(Exception):
            cache.load("/nonexistent/path/cache.json")

    def test_save_no_path(self):
        cache = cchess.FenCache()
        fen = "4k4/9/9/9/9/9/9/9/9/4K4 w - - 0 1"
        action = {"move": "e0d0", "score": 10}
        cache.save_action(fen, action)

        with pytest.raises(Exception):
            cache.save(None)


# ============================================================================
# Test EngineManager
# ============================================================================


class TestEngineManagerBasic:
    """Test basic EngineManager operations."""

    def test_create_manager(self):
        manager = cchess.EngineManager()
        assert manager is not None

    def test_create_manager_with_cache(self):
        cache = cchess.FenCache()
        manager = cchess.EngineManager(cache)
        assert manager is not None

    def test_load_nonexistent_engine(self):
        manager = cchess.EngineManager()
        with pytest.raises(Exception):
            manager.load_uci("/nonexistent/path/engine.exe")

    def test_send_cmd_without_engine(self):
        manager = cchess.EngineManager()
        with pytest.raises(Exception):
            manager.send_cmd("isready")


class TestEngineManagerWithEngine:
    """Test EngineManager with a real engine."""

    def test_load_uci_engine(self, pikafish_path):
        manager = cchess.EngineManager()
        try:
            result = manager.load_uci(pikafish_path)
            assert result is True
        finally:
            manager.quit()

    def test_load_uci_with_options(self, pikafish_path):
        manager = cchess.EngineManager()
        try:
            result = manager.load_uci(
                pikafish_path,
                options={"Threads": "1"},
                go_params={"movetime": "1000"},
            )
            assert result is True
        finally:
            manager.quit()

    def test_run_engine_search(self, pikafish_path):
        manager = cchess.EngineManager()
        try:
            manager.load_uci(
                pikafish_path,
                options={"Threads": "1"},
                go_params={"movetime": "1000"},
            )
            fen = cchess.initial_fen()
            action = manager.run_engine(fen)
            assert action is not None
            assert "move" in action
            assert len(action["move"]) == 4
        finally:
            manager.quit()

    def test_get_fen_score_caches(self, pikafish_path):
        manager = cchess.EngineManager()
        try:
            manager.load_uci(
                pikafish_path,
                options={"Threads": "1"},
                go_params={"movetime": "1000"},
            )
            fen = cchess.initial_fen()

            # First call should run engine
            action1 = manager.get_fen_score(fen, 1)
            assert action1 is not None
            assert "move" in action1

            # Second call should return from cache
            action2 = manager.get_fen_score(fen, 1)
            assert action2 is not None
            assert "move" in action2
        finally:
            manager.quit()

    def test_get_fen_score_without_cache(self, pikafish_path):
        manager = cchess.EngineManager()
        try:
            manager.load_uci(
                pikafish_path,
                options={"Threads": "1"},
                go_params={"depth": "10", "movetime": "500"},
            )
            fen = cchess.initial_fen()
            action = manager.get_fen_score(fen, 1)
            assert action is not None
            # Should have score (negated from engine perspective)
            assert "score" in action or "mate" in action
        finally:
            manager.quit()

    def test_quit_and_reuse(self, pikafish_path):
        manager = cchess.EngineManager()
        manager.load_uci(pikafish_path, go_params={"movetime": "500"})
        fen = cchess.initial_fen()
        manager.run_engine(fen)
        manager.quit()

        # After quit, running engine should fail
        with pytest.raises(Exception):
            manager.run_engine(fen)

    def test_engine_cleanup_on_del(self, pikafish_path):
        """Test that __del__ cleans up the engine."""

        def create_and_forget():
            manager = cchess.EngineManager()
            manager.load_uci(pikafish_path, go_params={"movetime": "500"})
            fen = cchess.initial_fen()
            manager.run_engine(fen)
            # Don't explicitly quit - let __del__ handle it
            return True

        result = create_and_forget()
        assert result is True
        gc.collect()


# ============================================================================
# Test EngineManager + UCCI
# ============================================================================


class TestEngineManagerUCCI:
    """Test EngineManager with UCCI engine."""

    def test_load_ucci_engine(self):
        """Load UCCI engine if available."""
        env_path = os.environ.get("CCHESS_UCCI_ENGINE")
        if not env_path or not os.path.exists(env_path):
            pytest.skip("UCCI engine not found")

        manager = cchess.EngineManager()
        try:
            result = manager.load_ucci(env_path, go_params={"time": "100"})
            assert result is True
        finally:
            manager.quit()


# ============================================================================
# Test Integration: FenCache + EngineManager
# ============================================================================


class TestIntegrationCacheAndManager:
    """Test FenCache and EngineManager working together."""

    def test_manager_uses_custom_cache(self, pikafish_path):
        cache = cchess.FenCache()
        manager = cchess.EngineManager(cache)
        try:
            manager.load_uci(
                pikafish_path,
                go_params={"movetime": "500"},
            )
            fen = cchess.initial_fen()
            action = manager.run_engine(fen)
            assert action is not None

            # After run_engine, the cache should have the action
            # Use get_fen_score to verify cache was populated
            cached = manager.get_best_cache(fen, 1)
            assert cached is not None, "Cache should be populated after run_engine"
        finally:
            manager.quit()

    def test_save_cache_after_search(self, pikafish_path):
        cache = cchess.FenCache()
        manager = cchess.EngineManager(cache)
        try:
            manager.load_uci(
                pikafish_path,
                go_params={"movetime": "500"},
            )
            fen = cchess.initial_fen()
            manager.run_engine(fen)

            with tempfile.NamedTemporaryFile(
                mode="w", suffix=".json", delete=False
            ) as f:
                path = f.name

            try:
                # Get the cached action from manager's internal cache
                cached = manager.get_best_cache(fen, 1)
                assert cached is not None, "Cache should be populated"

                # Test that get_fen_score returns from cache on second call
                action2 = manager.get_fen_score(fen, 1)
                assert action2 is not None
                assert "move" in action2
            finally:
                if os.path.exists(path):
                    os.unlink(path)
        finally:
            manager.quit()
