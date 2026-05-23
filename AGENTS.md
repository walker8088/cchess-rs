# AGENTS.md - AI 开发助手指南

本文档为 AI 开发助手提供参与 `cchess-rs` 项目开发的指导。`cchess-rs` 是一个用 Rust 实现的中国象棋库，提供完整的规则引擎、文件格式解析和 PyO3 Python 绑定。

## 项目概述

- **名称**: cchess-rs
- **描述**: 中国象棋 (Xiangqi) 的 Rust 实现
- **当前状态**: 核心引擎功能完整，Python 绑定完备，文件格式支持(PGN/XQF/CBR/CBL)
- **目标**: 完整的中国象棋规则实现，支持 AI 引擎和 UCI 协议
- **总体测试覆盖率**: 56.23%（核心模块 >77%）

## 项目结构

```
cchess-rs/
├── Cargo.toml              # 项目配置
├── AGENTS.md               # AI 开发指南
├── src/
│   ├── lib.rs              # 库入口点，导出所有模块
│   ├── main.rs             # 可执行入口点
│   ├── pieces.rs           # 棋子类型和颜色定义 (PieceType, Side)
│   ├── board.rs            # 棋盘逻辑（走法验证、合法性检查、攻击检测）
│   ├── game.rs             # 游戏状态和走法树管理
│   ├── move_gen.rs         # 走法生成（合法走法列表）
│   ├── move_notation.rs    # 走法中间表示（中文、WXF、ICCS 格式）
│   ├── attack_matrix.rs    # 攻击矩阵（快速位置攻击检测）
│   ├── pgn.rs              # PGN 格式解析和导出
│   ├── xqf.rs              # XQF 二进制格式解析和导出
│   ├── cbr.rs              # CBR/CBL 棋谱库格式解析
│   ├── engine.rs           # AI 引擎基础
│   ├── engine_driver.rs    # 引擎驱动（UCCI 协议通信）
│   ├── cbr/                # CBR 格式子模块
│   └── python/             # PyO3 Python 绑定 (14 个模块)
│       ├── mod.rs          # 模块注册和导出
│       ├── board.rs        # Board 绑定
│       ├── game.rs         # Game + GameMetadata 绑定
│       ├── move.rs         # MoveNode 绑定
│       ├── enums.rs        # 枚举类型绑定 (Side, PieceType, MoveFormat 等)
│       ├── move_notation.rs # MoveNotation 绑定
│       ├── movegen.rs      # 走法生成函数绑定
│       ├── fen_cache.rs    # 局面缓存绑定
│       ├── file_formats.rs # XQF/CBR/CBL 文件格式绑定
│       ├── pgn.rs          # PGN 函数绑定
│       ├── utils.rs        # 工具函数绑定
│       ├── engine_driver.rs# 引擎驱动绑定
│       ├── engine_manager.rs # 引擎管理器绑定
│       └── exceptions.rs   # 异常类型绑定
├── tests/
│   ├── integration.rs      # Rust 集成测试（103 个测试）
│   └── test_cchess_bindings.py  # Python 绑定测试（167 个测试）
└── tests/data/             # 测试数据文件 (.xqf, .pgn, .cbr, .cbl 等)
```

## 模块依赖关系

```
pieces.rs (基础类型, 无依赖)
    ↓
board.rs ──────────────────────→ move_notation.rs
    ↓                    ↙           ↘
move_gen.rs ←───────────┐             ↓
    ↓                   │     pgn.rs ←─┘
attack_matrix.rs        │
    ↓                   │
game.rs ────────────────┼─→ export_xqf → xqf.rs
    ↓                   │       ↕
cbr.rs ←────────────────┤  read_from (自动格式检测)
    ↓                   │
engine.rs ←─────────────┤
    ↓                   │
engine_driver.rs        │
    ↓                   │
python/ (PyO3 绑定层) ←─┘ (通过 inner 字段访问 Rust 类型)

python/ 内部依赖:
    mod.rs → enums.rs → board.rs, game.rs, move.rs, ...
    mod.rs → utils.rs, movegen.rs, pgn.rs, file_formats.rs
    mod.rs → engine_driver.rs, engine_manager.rs, exceptions.rs
```

### 颜色约定

```
Red pieces   = 大写 (K, R, N, B, A, C, P)   — rows 0-2 (底部)
Black pieces = 小写 (k, r, n, b, a, c, p)   — rows 7-9 (顶部)
Side::Red   → fen_char.is_uppercase()
Side::Black → fen_char.is_lowercase() && fen_char != '.'
```


## 开发优先级

### 已完成的核心功能
1. ✅ **棋盘初始位置** — `Board::new()` + `initial_position()` 实现标准初始布局
2. ✅ **走法验证** — `Board::make_move()` 支持所有棋子的完整规则验证（含飞将检查）
3. ✅ **将军检测** — `Game::check_game_over()` 支持将死、困毙检测
4. ✅ **特殊规则** — 炮的跳跃吃子、象不过河、士不出九宫、将帅照面
5. ✅ **攻击检测** — `board.rs::is_square_attacked_by()` 和 `move_gen.rs` 双实现

### 高优先级 (下一个发布)
1. **AI 引擎基础** — 局面评估函数和搜索算法
2. **UCI 协议支持** — UCI 命令解析和最佳走法计算

### 中优先级 (功能完善)
1. **走法生成优化**
   - 缓存合法走法
   - 预计算常用走法模式
2. **Python 绑定增强**
   - `GameMetadata` 设置后持久化问题 fix（当前 getter 返回副本）

### 低优先级 (扩展功能)
1. **性能基准测试**
2. **异步引擎支持**

## 开发规范

### 代码风格
- 遵循 Rust 官方代码风格
- 使用 `rustfmt` 格式化代码
- 使用 `clippy` 进行代码检查
- 添加适当的文档注释 (/// 和 //!)

### 测试要求
- 为每个新功能编写单元测试
- 测试用例应覆盖边界情况
- 使用 `#[cfg(test)]` 模块组织测试

### 错误处理
- 使用 `Result` 类型处理可恢复错误
- 使用适当的错误类型和错误信息
- 避免 panic，除非是程序无法恢复的错误

### 避免的代码模式
1. **攻击检测重复** — 不要重复实现棋子攻击检测；使用 `board.rs::is_square_attacked_by()`
2. **God Class** — `game.rs` 已从 1100 行缩减到 768 行；避免添加新职责
3. **直接访问 `board.squares`** — 必须通过 Board 的公开方法（`get_fen()`, `set_fen()` 等）
4. **枚举颜色约定混用** — 检查 `is_red` / `by_red` 参数与 `Side::Red` / `Side::Black` 的一致性
5. **`make_move` 的职责** — `Board::make_move()` 负责验证+执行；`Game::make_move()` 负责走法树管理

## 测试策略

### 单元测试位置
- 模块内部测试：放在 `#[cfg(test)]` 模块中
- 集成测试：放在 `tests/integration.rs`
- Python 绑定测试：`tests/test_cchess_bindings.py`

### 测试命令
```bash
cargo test --lib         # Rust 单元测试
cargo test --test integration  # Rust 集成测试
python -m pytest tests/test_cchess_bindings.py -q  # Python 绑定测试
cargo tarpaulin          # 覆盖率报告
```

### 测试覆盖率目标
- 核心功能（board, game, pieces, move_gen）：>80%
- 总体覆盖率：稳定提升
- 需要 Python 端 `pytest-cov` 测量 PyO3 绑定覆盖率（tarpaulin 对 PyO3 代码显示 0%）

## 贡献指南

### 提交前检查清单
- [ ] 代码通过 `cargo check`
- [ ] 代码通过 `cargo test`（Rust 单元/集成）
- [ ] 代码通过 `python -m pytest tests/test_cchess_bindings.py -q`
- [ ] 代码通过 `cargo clippy`
- [ ] 代码通过 `cargo fmt --check`
- [ ] 添加或更新了相关文档
- [ ] 添加了必要的测试用例

### 代码审查要点
1. **正确性**: 功能是否符合中国象棋规则
2. **安全性**: 是否避免 panic 和内存安全问题
3. **性能**: 是否有明显的性能问题
4. **可维护性**: 代码是否清晰易读
5. **测试覆盖**: 是否有足够的测试覆盖

## 已知问题和技术债务

### 当前已知问题
1. **PyGameMetadata getter 返回副本** — Python 侧 `game.metadata.title = "X"` 不会持久化，需要使用 `game.set_metadata()`
2. **PyO3 绑定覆盖率** — tarpaulin 显示 0%，需要用 Python 端工具测量
3. **`Game.current_node` 类型** — 使用 `Option<MoveNode>`（拥有所有权），应改用引用或 `Rc` 以避免克隆

### 技术债务
1. 缺少性能基准测试
2. `game.rs` 中的 `export_xqf` 直接依赖 `crate::xqf`（低耦合）
3. `python/enums.rs` 中有约 120 行机械重复的 `From` 实现，可用宏简化
4. 棋盘尺寸常量 `9` 和 `10` 散布在各模块中，应抽取为命名常量

## 开发工具建议

### 推荐工具
- `cargo-watch`: 文件变化时自动重建
- `cargo-udeps`: 检查未使用的依赖
- `cargo-audit`: 安全漏洞检查
- `cargo-tarpaulin`: Rust 代码覆盖率检查
- `pytest-cov`: Python 代码覆盖率检查

### 开发命令
```bash
# 开发环境
cargo watch -x check
cargo watch -x test

# 代码质量
cargo clippy -- -D warnings
cargo fmt --check

# 性能分析
cargo bench
cargo flamegraph --bin cchess-rs

# 文档生成
cargo doc --open

# 覆盖率
cargo tarpaulin --out Html
python -m pytest tests/test_cchess_bindings.py --cov=cchess_rs
```

## 中国象棋规则参考

### 棋盘布局
- 棋盘大小：9x10 格
- 红方 (下方)：坐标 (0,0) 到 (8,2)
- 黑方 (上方)：坐标 (0,7) 到 (8,9)
- 楚河汉界：第 4 和第 5 行之间

### 特殊规则
1. **将/帅**: 只能在九宫内移动，不能照面（飞将规则）
2. **士/仕**: 只能在九宫内斜线移动
3. **象/相**: 田字移动，不能过河，有蹩腿
4. **炮/砲**: 移动如车，吃子需隔一子（炮架）
5. **兵/卒**: 过河前只能前进，过河后可左右移动
6. **困毙**: 无子可走的一方算输（与国象不同）

## 紧急联系人/问题上报

如遇以下情况，应暂停开发并寻求指导：
- 发现严重的安全漏洞
- 遇到无法解决的编译或链接问题
- 需要修改项目架构的重大变更
- 涉及许可证或法律问题

---
*最后更新: 2026-05-23*
*文档维护: AI 开发助手*
