# cchess (Python) vs cchess-rs (Rust/PyO3) 功能对比表

> 最后更新: 2026-05-23
> Python cchess 版本: 1.25.5
> Rust cchess-rs 版本: 0.1.0

---

## 核心类型

| 功能 | Python `cchess` | Rust `cchess-rs` | 备注 |
|------|:---:|:---:|------|
| `Board` / `ChessBoard` | ✅ | ✅ `Board` | 等效 |
| `Game` | ✅ | ✅ `Game` | 等效 |
| `MoveNode` | ✅ | ✅ `MoveNode` | 等效 |
| `GameMetadata` | ✅ | ✅ `GameMetadata` | 等效 |
| `MoveNotation` | ❌ | ✅ | 🔴 Rust 独有 |
| `Piece` / `piece` | ✅ | ❌ | 🟡 Python 独有 |
| `Move` | ✅ | ❌ | 🟡 Python 独有 |

## 枚举

| 功能 | Python `cchess` | Rust `cchess-rs` | 备注 |
|------|:---:|:---:|------|
| `Side` (RED/BLACK/ANY) | ✅ `RED`/`BLACK` 常量 | ✅ `Side` 枚举 | 等效 |
| `PieceType` | ✅ 独立类 (King/Rook...) | ✅ `PieceType` 枚举 | 等效 |
| `ChineseLocale` | ❌ | ✅ `ChineseLocale` | 🔴 Rust 独有 |
| `MoveFormat` | ❌ | ✅ `MoveFormat` | 🔴 Rust 独有 |

## 棋盘方法 (Board)

| 功能 | Python `cchess` | Rust `cchess-rs` | 备注 |
|------|:---:|:---:|------|
| 初始局面 | ✅ `FULL_INIT_BOARD` | ✅ `full_init_board()` | 等效 |
| `from_fen()` / `to_fen()` | ✅ | ✅ | 等效 |
| `make_move()` | ✅ | ✅ | 等效 |
| `is_valid_move()` | ❌ | ✅ | 🔴 Rust 独有 |
| `is_checking_move()` | ❌ | ✅ | 🔴 Rust 独有 |
| `is_in_check()` | ❌ | ✅ | 🔴 Rust 独有 |
| `is_checkmate()` | ❌ | ✅ | 🔴 Rust 独有 |
| `create_moves()` | ❌ | ✅ | 🔴 Rust 独有 |
| `mirror()` / `flip()` | ✅ | ✅ | 等效 |
| `swap_colors()` / `is_mirror()` | ❌ | ✅ | 🔴 Rust 独有 |
| `get_king_pos()` | ❌ | ✅ | 🔴 Rust 独有 |
| `occupied()` | ❌ | ✅ | 🔴 Rust 独有 |
| `count_x_line_in()` / `count_y_line_in()` | ❌ | ✅ | 🔴 Rust 独有 |
| `x_line_in()` / `y_line_in()` | ❌ | ✅ | 🔴 Rust 独有 |
| `detect_move_pieces()` | ❌ | ✅ | 🔴 Rust 独有 |
| `get_fench_positions()` | ❌ | ✅ | 🔴 Rust 独有 |
| `move_iccs()` | ✅ | ✅ | 等效 |
| `move_text()` / `move_notation()` | ✅ | ✅ | 等效 |

## 引擎通信 (Engine)

| 功能 | Python `cchess` | Rust `cchess-rs` | 备注 |
|------|:---:|:---:|------|
| `EngineStatus` (9 states) | ✅ | ✅ | 等效 |
| `EngineError` / `CChessError` | ✅ | ✅ | 等效 |
| `EngineProcess` (同步引擎进程) | ❌ | ✅ | 🔴 Rust 独有 |
| `Engine` (线程基类) | ✅ `Thread` 封装 | ❌ | 🟡 Python 独有 |
| `UciEngine` / `UcciEngine` | ✅ 协议子类 | ❌ | 🟡 Python 独有 |
| `EngineManager` | ✅ | ✅ | 等效 |
| `FenCache` | ✅ | ✅ | 等效 |
| `action_mirror()` | ✅ | ✅ | 等效 |
| `play_move()` (同步便利函数) | ❌ | ✅ | 🔴 Rust 独有 |
| `analyse_position()` (同步便利函数) | ❌ | ✅ | 🔴 Rust 独有 |
| `EngineOption` | ❌ | ✅ | 🔴 Rust 独有 |
| `SearchInfo` | ✅ (dict) | ✅ (PyClass) | 等效 |
| `SearchResult` | ❌ | ✅ | 🔴 Rust 独有 |

## 引擎搜索方法

| 功能 | Python `cchess` | Rust `cchess-rs` | 备注 |
|------|:---:|:---:|------|
| `search_movetime()` | ✅ | ✅ | 等效 |
| `search_depth()` | ✅ | ✅ | 等效 |
| `setoption()` | ✅ | ✅ | 等效 |
| `position_fen()` | ✅ | ✅ | 等效 |
| `init()` | ✅ | ✅ | 等效 |
| `send()` / `read_until_any()` | ✅ | ✅ | 等效 |
| `quit()` | ✅ | ✅ | 等效 |
| `get_action()` (非阻塞) | ✅ | ❌ | 🟡 Python 独有 |
| `stop_thinking()` | ✅ | ❌ | 🟡 Python 独有 |
| `wait_for_ready()` | ✅ | ❌ | 🟡 Python 独有 |

## 攻击矩阵

| 功能 | Python `cchess` | Rust `cchess-rs` | 备注 |
|------|:---:|:---:|------|
| `generate_attack_matrix()` | ❌ | ✅ | 🔴 Rust 独有 |
| `is_position_attacked()` | ❌ | ✅ | 🔴 Rust 独有 |
| `is_king_in_check()` | ❌ | ✅ | 🔴 Rust 独有 |

## PGN / XQF 棋谱

| 功能 | Python `cchess` | Rust `cchess-rs` | 备注 |
|------|:---:|:---:|------|
| `read_pgn_file()` / `save_pgn_file()` | ✅ | ✅ | 等效 |
| `read_xqf_file()` / `write_xqf_file()` | ✅ | ✅ | 等效 |
| `parse_pgn()` / `game_to_pgn()` | ✅ | ✅ | 等效 |
| `board_to_xqf_bytes()` / `board_from_xqf_bytes()` | ❌ | ✅ | 🔴 Rust 独有 |
| `read_cbf()` / `read_cbr()` / `read_txt()` | ✅ | ❌ | 🟡 Python 独有 |
| `read_from_cbf()` / `read_from_cbl()` / `read_from_cbr()` | ✅ | ❌ | 🟡 Python 独有 |
| `read_from_pgn()` / `read_from_txt()` / `read_from_xqf()` | ✅ | ❌ | 🟡 Python 独有 |

## 走法生成

| 功能 | Python `cchess` | Rust `cchess-rs` | 备注 |
|------|:---:|:---:|------|
| `generate_legal_moves()` | ❌ | ✅ | 🔴 Rust 独有 |

## 常量 / 工具函数

| 功能 | Python `cchess` | Rust `cchess-rs` | 备注 |
|------|:---:|:---:|------|
| `initial_fen()` | ✅ `FULL_INIT_FEN` | ✅ `initial_fen()` | 等效 |
| `empty_fen()` / `empty_board()` | ✅ | ✅ | 等效 |
| `full_init_fen()` / `full_init_board()` | ✅ | ✅ | 等效 |
| `fen_mirror()` / `fen_flip()` / `fen_swap()` | ✅ | ✅ | 等效 |
| `fen_move_color()` | ✅ | ✅ | 等效 |
| `pos2iccs()` / `iccs2pos()` | ✅ | ✅ | 等效 |
| `iccs_mirror()` / `iccs_flip()` / `iccs_swap()` | ✅ | ✅ | 等效 |
| `iccs_list_mirror()` | ✅ | ✅ | 等效 |
| `get_fench_color()` / `fench_to_species()` | ❌ | ✅ | 🔴 Rust 独有 |
| `fench_to_text()` | ✅ | ❌ | 🟡 Python 独有 |
| `get_move_color()` | ✅ | ❌ | 🟡 Python 独有 |
| `get_fen_type()` / `get_fen_type_detail()` | ✅ | ❌ | 🟡 Python 独有 |
| `side_red()` / `side_black()` / `side_any()` | ✅ 常量 | ✅ 函数 | 等效 |
| `parse_info_line()` / `parse_info_lines()` | ✅ | ✅ | 等效 |
| `parse_bestmove_line()` | ❌ | ✅ | 🔴 Rust 独有 |
| `resolve_engine_path()` | ❌ | ✅ | 🔴 Rust 独有 |

## 异步支持

| 功能 | Python `cchess` | Rust `cchess-rs` | 备注 |
|------|:---:|:---:|------|
| `AsyncEngine` (asyncio) | ✅ | ❌ | 🟡 Python 独有 |
| `tokio` 驱动 | ❌ | ✅ (内部) | 🔴 Rust 内部使用 |

---

## 统计摘要

| 分类 | 数量 |
|------|------|
| ✅ **双方等效** | ~30 项 |
| 🔴 **Rust 独有** | ~32 项 |
| 🟡 **Python 独有** | ~14 项 |

## Python 独有但未在 Rust 实现的功能

### 引擎相关
- `Engine` 线程基类（`threading.Thread` + `queue.Queue` 架构）
- `UciEngine` / `UcciEngine` 协议子类
- `get_action()` 非阻塞拉取引擎动作
- `stop_thinking()` 停止思考
- `wait_for_ready()` 等待引擎就绪
- `AsyncEngine` (asyncio 异步引擎)

### 棋谱格式
- CBF 棋谱读取 (`read_cbf`, `read_from_cbf`)
- CBL 棋谱读取 (`read_from_cbl`)
- CBR 棋谱读取 (`read_cbr`, `read_from_cbr`)
- TXT 棋谱读取 (`read_txt`, `read_from_txt`)
- PGN 棋谱读取 (`read_pgn`, `read_from_pgn`)
- XQF 棋谱读取 (`read_xqf`, `read_from_xqf`)

### 工具函数
- `fench_to_text()` — 棋子文字转换
- `get_move_color()` — 获取走子方
- `get_fen_type()` / `get_fen_type_detail()` — FEN 类型判断

### 类型
- `Piece` / `piece` — 棋子类
- `Move` — 走法类
