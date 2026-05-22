# 引擎驱动架构文档

## 概述

`cchess-rs` 引擎驱动层提供了一套完整的异步引擎管理方案，支持通过 UCI/UCCI 协议与外部象棋引擎进程进行通信。该架构基于 Rust 的 `tokio` 异步运行时，实现了事件驱动的非阻塞 I/O 模型。

## 架构设计

### 系统总览

```
┌─────────────────────────────────────────────────────────────────┐
│                        应用层 (Application)                      │
│                                                                  │
│   ┌──────────┐  send()   ┌──────────────┐   search()   ┌──────┐ │
│   │ 测试用例  │ ────────► │ EngineDriver │ ◄─────────── │ 业务  │ │
│   │  GUI     │ ◄──────── │              │ ────────────► │ 逻辑  │ │
│   └──────────┘  recv()   └──────┬───────┘              └──────┘ │
│                                 │                                │
└─────────────────────────────────┼────────────────────────────────┘
                                  │
                    ┌─────────────┼─────────────┐
                    │   tokio::task (后台事件循环) │
                    │                             │
                    │  ┌─────────────────────┐    │
                    │  │    event_loop       │    │
                    │  │                     │    │
                    │  │  tokio::select! {   │    │
                    │  │    stdout → parse   │    │
                    │  │    stderr → log     │    │
                    │  │    stdin ← command  │    │
                    │  │  }                  │    │
                    │  └─────────────────────┘    │
                    │           │                 │
                    │     ┌─────┴─────┐           │
                    │     │ event_tx  │           │
                    │     └─────┬─────┘           │
                    └───────────┼─────────────────┘
                                │
                    ┌───────────┼─────────────────┐
                    │   OS 进程间通信 (stdin/stdout/stderr)
                    │           │                 │
              ┌─────┴─────┐     │           ┌─────┴─────┐
              │  stdin    │     │           │  stdout   │
              │  (write)  │     │           │  (read)   │
              └─────┬─────┘     │           └─────┬─────┘
                    │           │                 │
              ┌─────┴───────────┴─────────────────┴─────┐
              │          外部引擎进程 (Engine)            │
              │                                          │
              │  Pikafish (UCI) / EleEye (UCCI)          │
              │                                          │
              │  ┌──────────────────────────────────┐   │
              │  │  协议解析 → 走法生成 → 搜索 → 评估 │   │
              │  └──────────────────────────────────┘   │
              └──────────────────────────────────────────┘
```

### 核心组件

#### 1. EngineDriver - 引擎驱动主类

每个 `EngineDriver` 实例管理一个独立的引擎进程，拥有隔离的 stdin/stdout/stderr。

```rust
pub struct EngineDriver {
    /// 命令发送通道 (tx) - 写入引擎 stdin
    cmd_tx: mpsc::UnboundedSender<String>,
    /// 事件接收通道 (rx) - 读取引擎输出解析后的事件
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<EngineEvent>>>,
    /// 后台事件循环任务句柄
    handle: Option<tokio::task::JoinHandle<()>>,
    /// 协议类型 (UCI / UCCI)
    protocol: Protocol,
    /// 是否已完成初始化握手
    ready: bool,
    /// 引擎名称 (初始化后获取)
    engine_name: Option<String>,
}
```

**关键方法:**

| 方法 | 功能 | 协议 |
|------|------|------|
| `spawn(path, protocol)` | 启动引擎进程，创建后台事件循环 | UCI/UCCI |
| `init()` | 协议握手: `uci` → `uciok`, `isready` → `readyok` | UCI/UCCI |
| `send(cmd)` | 向引擎发送原始命令 | UCI/UCCI |
| `recv()` | 接收下一个事件 | - |
| `collect_until(predicate, timeout)` | 收集事件直到满足条件 | - |
| `wait_bestmove(timeout)` | 等待搜索完成 | - |
| `setoption(name, value)` | 设置引擎参数 | UCI/UCCI |
| `position_fen(fen)` | 设置局面 | UCI |
| `go_movetime(ms)` | 启动限时搜索 | UCI/UCCI |
| `go_depth(depth)` | 启动限深搜索 | UCI |
| `search_movetime(ms)` | 搜索并返回结构化结果 | UCI/UCCI |
| `search_depth(depth)` | 限深搜索并返回结构化结果 | UCI |
| `quit()` | 退出引擎进程 | UCI/UCCI |

#### 2. EngineEvent - 事件枚举

引擎输出的每一行都会被解析为强类型的 `EngineEvent`:

```rust
pub enum EngineEvent {
    /// 引擎进程已启动
    Started,
    /// 引擎标识信息: `id name <name>` / `id author <author>`
    Id { name: Option<String>, author: Option<String> },
    /// 引擎可用参数: `option name <name> type <type> ...`
    Option(EngineOption),
    /// 引擎就绪: `uciok` / `ucciok` / `readyok`
    Ready,
    /// 搜索信息行: `info depth ...`
    Info(SearchInfo),
    /// 最佳走法: `bestmove <move> [ponder <move>]`
    BestMove { bestmove: String, ponder: Option<String> },
    /// 无最佳走法: `nobestmove`
    NoBestMove,
    /// 信息字符串: `info string <text>`
    InfoString(String),
    /// 标准错误输出
    Stderr(String),
    /// 引擎进程退出
    Exited(i32),
}
```

#### 3. SearchInfo - 搜索信息结构

解析引擎 `info` 行的结构化数据:

```rust
pub struct SearchInfo {
    pub depth: u32,              // 搜索深度 (层)
    pub seldepth: Option<u32>,   // 选择性深度
    pub time_ms: Option<u64>,    // 搜索用时 (毫秒)
    pub nodes: Option<u64>,      // 搜索节点数
    pub nps: Option<u64>,        // 节点速度 (节点/秒)
    pub hashfull: Option<u32>,   // 哈希表使用率 (‰)
    pub multipv: Option<u32>,    // 多 PV 线编号
    pub score: Option<Score>,    // 局面评估分数
    pub currmove: Option<String>,// 当前正在搜索的走法
    pub currmovenumber: Option<u32>, // 当前走法序号
    pub pv: Vec<String>,         // 主要变化线 (Principal Variation)
}
```

#### 4. Score - 评估分数

```rust
pub enum Score {
    Cp(i64),   // 厘兵分 (正数 = 当前方优势)
    Mate(i32), // 杀棋步数 (正数 = 当前方可将杀)
}
```

#### 5. SearchResultAsync - 搜索结果聚合

```rust
pub struct SearchResultAsync {
    pub bestmove: Option<String>,      // 最佳走法
    pub ponder: Option<String>,        // 预期应对
    pub info_events: Vec<SearchInfo>,  // 所有搜索信息行
    pub final_info: Option<SearchInfo>,// 最终(最深)搜索信息
    pub raw_events: Vec<EngineEvent>,  // 原始事件列表
}
```

### 事件循环 (Event Loop)

事件循环在独立的 `tokio::task` 中运行，使用 `tokio::select!` 实现多路复用 I/O:

```rust
async fn event_loop(stdin, stdout, stderr, cmd_rx, event_tx) {
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            biased;  // 优先处理 I/O

            // 1. 读取引擎 stdout → 解析为事件 → 推入事件队列
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        let event = Self::parse_line(&line);
                        event_tx.send(event);
                    }
                    Ok(None) => { /* EOF - 引擎退出 */ break; }
                    Err(e) => { /* 错误 */ break; }
                }
            }

            // 2. 读取引擎 stderr → 记录错误事件
            result = stderr_reader.next_line() => { ... }

            // 3. 接收前端命令 → 写入引擎 stdin
            cmd = cmd_rx.recv() => {
                if let Some(cmd) = cmd {
                    stdin.write_all(format!("{}\n", cmd).as_bytes()).await;
                } else { break; }
            }
        }
    }
}
```

### 数据流

```
  前端命令                          引擎响应
  ┌─────────┐                    ┌─────────┐
  │ send()  │                    │ stdout  │
  │   "uci" │──► cmd_tx ──► cmd_rx│  "id name Pikafish" ──► parse_line()
  │   "isready"                  │  "uciok"                  │
  │   "position fen ..."         │  "info depth 1 cp 100"   │
  │   "go movetime 2000"         │  "bestmove h2e2"         │
  └─────────┘                    └─────────┘
                                      │
                                      ▼
                              ┌───────────────┐
                              │   event_tx     │
                              └───────┬───────┘
                                      │
                                      ▼
                              ┌───────────────┐
                              │   event_rx     │
                              └───────┬───────┘
                                      │
                                      ▼
                              ┌───────────────┐
                              │  recv() /      │
                              │  collect_until()│
                              └───────────────┘
```

## 通信协议

### UCI 协议 (Pikafish)

```
客户端                    引擎
  │                        │
  ├──── "uci" ────────────►│
  │                        ├── "id name Pikafish ..."
  │                        ├── "option name Hash ..."
  │                        └── "uciok"
  ├──── "isready" ────────►│
  │                        └── "readyok"
  ├──── "position fen ..."─►│
  ├──── "go movetime 2000"─►│
  │                        ├── "info depth 1 cp 100 nodes 500 ..."
  │                        ├── "info depth 2 cp 150 nodes 2000 ..."
  │                        └── "bestmove h2e2 ponder b0c2"
  └──── "quit" ───────────►│
                           │ (进程退出)
```

### UCCI 协议 (EleEye)

```
客户端                    引擎
  │                        │
  ├──── "ucci" ───────────►│
  │                        ├── "id name ElephantEye"
  │                        ├── "option hashsize type spin ..."
  │                        └── "ucciok"
  ├──── "isready" ────────►│
  │                        └── "readyok"
  ├──── "position fen ..."─►│
  ├──── "go time 200" ────►│  (UCCI 时间单位为厘秒)
  │                        ├── "info depth 0 score 1473 pv b2e2"
  │                        └── "bestmove h2e2 ponder h9g7"
  └──── "quit" ────────────►│
                           └── "bye"
```

### 协议差异对比

| 特性 | UCI (Pikafish) | UCCI (EleEye) |
|------|---------------|---------------|
| 初始化命令 | `uci` | `ucci` |
| 就绪响应 | `uciok` | `ucciok` |
| 设置参数 | `setoption name X value Y` | `setoption X Y` |
| 搜索时间单位 | 毫秒 (`go movetime 2000`) | 厘秒 (`go time 200`) |
| 无走法响应 | `bestmove 0000` | `nobestmove` |
| 退出响应 | (无) | `bye` |
| 评估格式 | `score cp 100` / `score mate 3` | `score 1473` (直接数字) |

## 使用示例

### 基础用法 - 单引擎搜索

```rust
#[tokio::main]
async fn main() {
    // 1. 创建并初始化引擎
    let mut engine = EngineDriver::spawn(
        "engine/pikafish/pikafish.exe".into(),
        Protocol::Uci
    ).await.unwrap();

    engine.init().await.unwrap();
    println!("Connected to: {}", engine.engine_name().unwrap());

    // 2. 设置局面并搜索
    engine.position_fen("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w - - 0 1").await.unwrap();

    // 3. 搜索并获取结构化结果
    let result = engine.search_movetime(3000).await.unwrap();

    println!("Best move: {}", result.bestmove.unwrap());
    println!("Depth: {}", result.depth().unwrap());
    println!("Score: {:?}", result.score());
    println!("Nodes: {:?}", result.nodes());
    println!("NPS: {:?}", result.nps());
    println!("PV: {}", result.pv_string());

    // 4. 退出
    engine.quit().await;
}
```

### 异步并发 - 双引擎对比

```rust
#[tokio::test]
async fn test_concurrent_engines() {
    // 两个引擎同时搜索
    let pikafish = async {
        let mut e = create_uci_engine().await.unwrap();
        e.position_fen(INITIAL_FEN).await.unwrap();
        let r = e.search_movetime(2000).await.unwrap();
        e.quit().await;
        ("Pikafish", r)
    };

    let eleeye = async {
        let mut e = create_ucci_engine().await.unwrap();
        e.position_fen(INITIAL_FEN).await.unwrap();
        let r = e.search_movetime(200).await.unwrap();
        e.quit().await;
        ("EleEye", r)
    };

    // 并发执行
    let (r1, r2) = tokio::join!(pikafish, eleeye);

    println!("Pikafish: {} (depth={}, score={:?})",
        r1.1.bestmove.unwrap(), r1.1.depth().unwrap(), r1.1.score());
    println!("EleEye:   {} (depth={})",
        r2.1.bestmove.unwrap_or_default(), r2.1.depth().unwrap_or(0));
}
```

### 事件流处理

```rust
#[tokio::test]
async fn test_info_stream() {
    let mut engine = create_uci_engine().await.unwrap();
    engine.position_fen(INITIAL_FEN).await.unwrap();
    engine.go_movetime(2000).await.unwrap();

    // 逐事件处理搜索过程
    let events = engine.wait_bestmove(Duration::from_secs(30)).await.unwrap();

    for event in &events {
        match event {
            EngineEvent::Info(info) => {
                println!("depth={} score={:?} nodes={:?}",
                    info.depth, info.score, info.nodes);
            }
            EngineEvent::BestMove { bestmove, ponder } => {
                println!("bestmove={} ponder={:?}", bestmove, ponder);
            }
            _ => {}
        }
    }

    engine.quit().await;
}
```

### 选项发现与配置

```rust
#[tokio::test]
async fn test_option_management() {
    let exe = "engine/pikafish/pikafish.exe".into();
    let mut engine = EngineDriver::spawn(exe, Protocol::Uci).await.unwrap();

    // 发送 uci 获取选项列表
    engine.send("uci").await.unwrap();
    let events = engine.collect_until(|e| e.is_ready(), Duration::from_secs(10)).await.unwrap();

    // 解析所有选项
    for event in &events {
        if let EngineEvent::Option(opt) = event {
            println!("{}: type={} default={:?} min={:?} max={:?}",
                opt.name, opt.r#type, opt.default, opt.min, opt.max);
        }
    }

    // 设置参数
    engine.setoption("Hash", "64").await.unwrap();
    engine.setoption("Threads", "4").await.unwrap();
    engine.send("isready").await.unwrap();
    engine.collect_until(|e| e.is_ready(), Duration::from_secs(10)).await.unwrap();

    engine.quit().await;
}
```

## 文件结构

```
cchess-rs/
├── tests/
│   ├── engine_async.rs        # 异步引擎驱动 + 异步测试 (12 tests)
│   └── engine_integration.rs  # 同步引擎测试 + 信息解析测试 (31 tests)
├── docs/
│   └── engine_architecture.md # 本文档
├── engine/
│   ├── pikafish/
│   │   ├── pikafish.exe       # UCI 引擎 (Pikafish)
│   │   └── pikafish.nnue      # 神经网络评估文件
│   └── eleeye/
│       ├── ELEEYE.EXE         # UCCI 引擎 (EleEye)
│       ├── BOOK.DAT           # 开局库
│       └── evaluate.dll       # 评估模块
└── Cargo.toml
    └── [dev-dependencies]
        tokio = { version = "1", features = ["full"] }
```

## 测试覆盖

### 异步测试 (engine_async.rs) - 12 项

| 测试 | 覆盖内容 |
|------|---------|
| `test_async_uci_handshake` | UCI 协议握手初始化 |
| `test_async_ucci_handshake` | UCCI 协议握手初始化 |
| `test_async_uci_search_with_info` | 搜索 + 结构化信息解析验证 |
| `test_async_ucci_search_with_info` | EleEye 搜索 + 信息解析 |
| `test_async_uci_options_discovery` | 引擎选项自动发现与解析 |
| `test_async_uci_setoption_and_search` | 参数设置后搜索验证 |
| `test_async_uci_info_events_stream` | 搜索过程中 info 事件流验证 |
| `test_async_uci_mate_detection` | 杀棋分数检测 (mate vs cp) |
| `test_async_uci_position_with_moves` | 走法序列局面搜索 |
| `test_async_concurrent_engines` | 双引擎并发搜索 (`tokio::join!`) |
| `test_async_uci_rapid_searches` | 连续多次搜索稳定性 |
| `test_async_event_line_parsing` | 行解析单元测试 (bestmove/info/option/id) |

### 同步测试 (engine_integration.rs) - 31 项

包含同步版本的所有上述测试，加上更详细的信息解析单元测试和引擎性能测试。

## 关键设计决策

### 1. 为什么不使用 `kill_on_drop`

`tokio::process::Command::kill_on_drop(true)` 会在 `Child` 结构体被 drop 时杀死进程。
但在我们的架构中，`Child` 在 `spawn()` 函数中创建后立即 drop（因为我们只取走了 stdin/stdout/stderr），
这会导致引擎进程被意外终止。

**解决方案:** 不使用 `kill_on_drop`，引擎进程通过 stdin/stdout 管道保持活跃，
在 `quit()` 方法中发送 "quit" 命令优雅退出。

### 2. 为什么使用 `mpsc::unbounded_channel`

- `unbounded` 确保引擎输出不会被背压阻塞
- 引擎搜索时可能短时间内输出大量 info 行，bounded channel 可能导致管道满和死锁
- 内存消耗可控，因为事件会被消费者及时取走

### 3. 为什么使用 `Arc<Mutex<Receiver>>`

`mpsc::Receiver` 不是 `Clone` 的，为了支持多个消费者（虽然当前是单消费者），
我们使用 `Arc<Mutex<>>` 包装。这也允许 `EngineDriver` 在 `&self` 引用下调用 `recv()`。

### 4. 为什么使用 `biased` select

`biased` 确保 `tokio::select!` 总是按声明顺序检查分支，优先处理 I/O 事件
(stdout/stderr)，再处理命令发送。这保证了引擎输出不会被命令发送延迟处理。

## 扩展方向

### 1. EnginePool 多引擎管理

```rust
pub struct EnginePool {
    engines: HashMap<EngineId, EngineDriver>,
    event_rx: mpsc::Receiver<PoolEvent>,  // 合并事件流
}

impl EnginePool {
    pub async fn add_engine(&mut self, config: EngineConfig) -> EngineId;
    pub async fn broadcast_command(&self, cmd: &str);
    pub async fn search_all(&self, position: &str, time_ms: u64) -> HashMap<EngineId, SearchResult>;
}
```

### 2. 实时事件订阅

```rust
// 支持多个订阅者接收引擎事件
pub struct EngineDriver {
    event_tx: broadcast::Sender<EngineEvent>,
}

// 订阅者
let mut sub = engine.subscribe();
while let Ok(event) = sub.recv().await {
    // 处理事件 (UI 更新、日志记录等)
}
```

### 3. 走法验证引擎

```rust
// 使用引擎验证走法合法性
pub async fn validate_move(engine: &mut EngineDriver, fen: &str, mov: &str) -> bool {
    engine.position_fen(fen).await?;
    let result = engine.search_movetime(100).await?;
    result.info_events.iter()
        .flat_map(|i| &i.pv)
        .any(|m| m == mov)
}
```
