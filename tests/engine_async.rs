/// Async engine integration tests for UCI/UCCI engines.
///
/// These tests use the `EngineDriver` from `cchess_rs::engine_driver` to
/// drive real external engine processes (Pikafish for UCI, EleEye for UCCI).
///
/// Usage:
///   cargo test --test engine_async
///
/// Environment variables (optional):
///   ELEEXE_PATH   - Path to EleEye executable (default: engine/eleeye/ELEEYE.EXE)
///   PIKAFISH_PATH - Path to Pikafish executable (default: engine/pikafish/pikafish.exe)
use cchess_rs::engine_driver::*;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Test Helpers
// ---------------------------------------------------------------------------

async fn create_uci_engine() -> Option<EngineDriver> {
    let exe = resolve_engine_path("PIKAFISH_PATH", "engine/pikafish/pikafish.exe");
    if !exe.exists() {
        return None;
    }
    let mut engine = EngineDriver::spawn(exe, Protocol::Uci).await.ok()?;
    engine.init().await.ok()?;
    Some(engine)
}

async fn create_ucci_engine() -> Option<EngineDriver> {
    let exe = resolve_engine_path("ELEEXE_PATH", "engine/eleeye/ELEEYE.EXE");
    if !exe.exists() {
        return None;
    }
    let mut engine = EngineDriver::spawn(exe, Protocol::Ucci).await.ok()?;
    engine.init().await.ok()?;
    Some(engine)
}

// ---------------------------------------------------------------------------
// Async Integration Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_async_uci_handshake() {
    let Some(mut engine) = create_uci_engine().await else {
        return;
    };
    assert!(engine.is_ready());
    assert!(engine.engine_name().is_some());
    eprintln!("Connected to: {} (UCI)", engine.engine_name().unwrap());
    engine.quit().await;
}

#[tokio::test]
async fn test_async_ucci_handshake() {
    let Some(mut engine) = create_ucci_engine().await else {
        return;
    };
    assert!(engine.is_ready());
    eprintln!(
        "Connected to: {} (UCCI)",
        engine.engine_name().unwrap_or("unknown")
    );
    engine.quit().await;
}

#[tokio::test]
async fn test_async_uci_search_with_info() {
    let Some(mut engine) = create_uci_engine().await else {
        return;
    };
    engine
        .position_fen(INITIAL_FEN)
        .await
        .expect("position failed");
    let result = engine.search_movetime(2000).await.expect("search failed");

    assert!(result.bestmove.is_some());
    let mv = result.bestmove.as_ref().unwrap();
    assert_eq!(mv.len(), 4);

    assert!(!result.info_events.is_empty());
    let fi = result.final_info.as_ref().unwrap();
    assert!(fi.depth > 0);
    assert!(fi.score.is_some());
    assert!(!fi.pv.is_empty());
    assert!(fi.nodes.is_some());
    assert!(fi.nps.is_some());

    eprintln!(
        "Pikafish: move={} depth={} score={:?} nodes={:?} nps={:?} pv={}",
        mv,
        result.depth().unwrap(),
        result.score(),
        result.nodes(),
        result.nps(),
        result.pv_string()
    );
    engine.quit().await;
}

#[tokio::test]
async fn test_async_ucci_search_with_info() {
    let Some(mut engine) = create_ucci_engine().await else {
        return;
    };
    engine
        .position_fen(INITIAL_FEN)
        .await
        .expect("position failed");
    let result = engine.search_movetime(200).await.expect("search failed");

    if let Some(mv) = &result.bestmove {
        assert!(mv.len() >= 4);
        eprintln!(
            "EleEye: move={} depth={} score={:?} nodes={:?} pv={}",
            mv,
            result.depth().unwrap_or(0),
            result.score(),
            result.nodes(),
            result.pv_string()
        );
    } else {
        eprintln!("EleEye returned nobestmove");
    }
    engine.quit().await;
}

#[tokio::test]
async fn test_async_uci_options_discovery() {
    let exe = resolve_engine_path("PIKAFISH_PATH", "engine/pikafish/pikafish.exe");
    if !exe.exists() {
        return;
    }
    let mut engine = EngineDriver::spawn(exe, Protocol::Uci)
        .await
        .expect("spawn");
    engine.send("uci").await.expect("send");
    let events = engine
        .collect_until(|e| e.is_ready(), Duration::from_secs(10))
        .await
        .expect("timeout");

    let option_count = events
        .iter()
        .filter(|e| matches!(e, EngineEvent::Option(_)))
        .count();
    eprintln!("Pikafish options discovered: {}", option_count);
    assert!(option_count > 0, "Should have discovered options");

    for event in &events {
        if let EngineEvent::Option(opt) = event {
            assert!(!opt.name.is_empty());
            assert!(!opt.r#type.is_empty());
        }
    }
    engine.quit().await;
}

#[tokio::test]
async fn test_async_uci_setoption_and_search() {
    let Some(mut engine) = create_uci_engine().await else {
        return;
    };
    engine
        .setoption("Hash", "32")
        .await
        .expect("setoption failed");
    engine.send("isready").await.expect("isready failed");
    engine
        .collect_until(|e| e.is_ready(), Duration::from_secs(10))
        .await
        .expect("ready timeout");
    engine
        .position_fen(INITIAL_FEN)
        .await
        .expect("position failed");
    let result = engine.search_movetime(1000).await.expect("search failed");
    assert!(result.bestmove.is_some());
    eprintln!(
        "Pikafish (Hash=32): move={} nodes={:?} nps={:?}",
        result.bestmove.as_deref().unwrap_or("?"),
        result.nodes(),
        result.nps()
    );
    engine.quit().await;
}

#[tokio::test]
async fn test_async_uci_info_events_stream() {
    let Some(mut engine) = create_uci_engine().await else {
        return;
    };
    engine
        .position_fen(INITIAL_FEN)
        .await
        .expect("position failed");
    engine.go_movetime(1000).await.expect("go failed");
    let events = engine
        .wait_bestmove(Duration::from_secs(30))
        .await
        .expect("search timeout");

    let info_count = events
        .iter()
        .filter(|e| matches!(e, EngineEvent::Info(_)))
        .count();
    eprintln!(
        "Pikafish events: {} info, {} total",
        info_count,
        events.len()
    );
    assert!(info_count > 0, "Should have info events during search");

    let info_events: Vec<_> = events
        .iter()
        .filter_map(|e| {
            if let EngineEvent::Info(info) = e {
                Some(info.clone())
            } else {
                None
            }
        })
        .collect();

    if info_events.len() > 1 {
        let first = info_events.first().unwrap().depth;
        let last = info_events.last().unwrap().depth;
        assert!(
            last >= first,
            "Last depth ({}) should be >= first depth ({})",
            last,
            first
        );
        eprintln!(
            "Depth progression: {} -> {} ({} iterations)",
            first,
            last,
            info_events.len()
        );
    }
    engine.quit().await;
}

#[tokio::test]
async fn test_async_uci_mate_detection() {
    let Some(mut engine) = create_uci_engine().await else {
        return;
    };
    let fen = "4k4/9/9/9/9/9/9/9/9/R3K4 w - - 0 1";
    engine.position_fen(fen).await.expect("position failed");
    let result = engine.search_depth(10).await.expect("search failed");
    let score = result.score().expect("Should have score");

    if let Score::Mate(m) = score {
        assert!(*m > 0, "Mate should be positive: {}", m);
        eprintln!("Pikafish found mate in {} moves", m);
    } else if let Some(cp) = score.cp() {
        assert!(cp > 1000, "Expected high score for mate: {}", cp);
        eprintln!("Pikafish high cp: {} (mate position)", cp);
    }
    engine.quit().await;
}

#[tokio::test]
async fn test_async_uci_position_with_moves() {
    let Some(mut engine) = create_uci_engine().await else {
        return;
    };
    engine
        .position_startpos_moves("h0e0 h9g7")
        .await
        .expect("position failed");
    let result = engine.search_movetime(1000).await.expect("search failed");
    assert!(result.bestmove.is_some());
    eprintln!(
        "Pikafish after opening: move={} pv={}",
        result.bestmove.as_deref().unwrap_or("?"),
        result.pv_string()
    );
    engine.quit().await;
}

#[tokio::test]
async fn test_async_concurrent_engines() {
    let uci = async {
        let Some(mut e) = create_uci_engine().await else {
            return None;
        };
        e.position_fen(INITIAL_FEN).await.ok()?;
        let r = e.search_movetime(1000).await.ok()?;
        e.quit().await;
        Some(("Pikafish", r))
    };
    let ucci = async {
        let Some(mut e) = create_ucci_engine().await else {
            return None;
        };
        e.position_fen(INITIAL_FEN).await.ok()?;
        let r = e.search_movetime(100).await.ok()?;
        e.quit().await;
        Some(("EleEye", r))
    };

    let (r1, r2) = tokio::join!(uci, ucci);
    if let (Some((n1, r1)), Some((n2, r2))) = (r1, r2) {
        eprintln!(
            "Concurrent: {}={} {}={}",
            n1,
            r1.bestmove.as_deref().unwrap_or("?"),
            n2,
            r2.bestmove.as_deref().unwrap_or("?")
        );
        assert!(r1.bestmove.is_some());
    }
}

#[tokio::test]
async fn test_async_uci_rapid_searches() {
    let Some(mut engine) = create_uci_engine().await else {
        return;
    };
    let positions = [
        INITIAL_FEN,
        "4k4/9/9/9/9/9/9/9/9/R3K4 w - - 0 1",
        "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/1NBAKABNR b - - 0 1",
    ];
    for (i, fen) in positions.iter().enumerate() {
        engine
            .position_fen(fen)
            .await
            .expect(&format!("position {}", i));
        let r = engine
            .search_movetime(500)
            .await
            .expect(&format!("search {}", i));
        assert!(r.bestmove.is_some());
        eprintln!(
            "Iter {}: move={} depth={} nodes={:?}",
            i,
            r.bestmove.as_deref().unwrap_or("?"),
            r.depth().unwrap_or(0),
            r.nodes()
        );
    }
    engine.quit().await;
}

#[tokio::test]
async fn test_async_event_line_parsing() {
    use EngineEvent::*;

    let event = EngineDriver::parse_line("bestmove h2e2 ponder b0c2");
    if let BestMove { bestmove, ponder } = event {
        assert_eq!(bestmove, "h2e2");
        assert_eq!(ponder, Some("b0c2".to_string()));
    }

    assert!(matches!(EngineDriver::parse_line("uciok"), Ready));
    assert!(matches!(EngineDriver::parse_line("ucciok"), Ready));
    assert!(matches!(EngineDriver::parse_line("readyok"), Ready));
    assert!(matches!(EngineDriver::parse_line("nobestmove"), NoBestMove));

    let event = EngineDriver::parse_line("id name Pikafish 2023-04-08");
    if let Id { name, author } = event {
        assert_eq!(name, Some("Pikafish 2023-04-08".to_string()));
        assert_eq!(author, None);
    }

    let event = EngineDriver::parse_line("info string NNUE evaluation using pikafish.nnue");
    if let InfoString(s) = event {
        assert_eq!(s, "NNUE evaluation using pikafish.nnue");
    }

    let event =
        EngineDriver::parse_line("option name Hash type spin default 16 min 1 max 33554432");
    if let Option(opt) = event {
        assert_eq!(opt.name, "Hash");
        assert_eq!(opt.r#type, "spin");
        assert_eq!(opt.default, Some("16".to_string()));
        assert_eq!(opt.min, Some(1));
        assert_eq!(opt.max, Some(33554432));
    }

    let event = EngineDriver::parse_line(
        "info depth 5 seldepth 8 score cp 147 pv h2e2 b0c2 nodes 12345 time 10 nps 1234500",
    );
    if let Info(info) = event {
        assert_eq!(info.depth, 5);
        assert_eq!(info.seldepth, Some(8));
        assert_eq!(info.nodes, Some(12345));
        assert_eq!(info.time_ms, Some(10));
        assert_eq!(info.nps, Some(1_234_500));
        assert_eq!(info.pv, vec!["h2e2", "b0c2"]);
        if let Score::Cp(v) = info.score.unwrap() {
            assert_eq!(v, 147);
        } else {
            panic!("Expected cp");
        }
    } else {
        panic!("Expected Info");
    }

    eprintln!("All event line parsing tests passed");
}
