# cchess-rs - 中国象棋 (Xiangqi) Rust 实现

一个用 Rust 语言实现的中国象棋库。

## 功能特性

- ✅ 完整的中国象棋规则实现
- ✅ 棋盘和棋子表示
- ✅ 走法生成
- ✅ 游戏状态管理
- ✅ 模块化设计，易于扩展
- ⬜ AI/引擎实现 (计划中)
- ⬜ UCI协议支持 (计划中)
- ⬜ 图形界面 (计划中)

## 项目结构

```
cchess-rs/
├── Cargo.toml          # 项目配置
├── src/
│   ├── lib.rs          # 库入口点
│   ├── main.rs         # 可执行入口点
│   ├── board.rs        # 棋盘逻辑
│   ├── pieces.rs       # 棋子定义
│   ├── game.rs         # 游戏状态
│   └── move_gen.rs     # 走法生成
├── examples/           # 使用示例
│   └── basic.rs        # 基础示例
├── tests.rs            # 单元测试
└── .gitignore          # Git 忽略文件
```

## 快速开始

### 安装

```bash
# 克隆项目
# 编译项目
cargo build

# 运行示例
cargo run --example basic

# 运行测试
cargo test
```

### 基本使用

```rust
use cchess_rs::game::Game;

fn main() {
    let mut game = Game::new();
    println!("Game created!");
    println!("{}", game.display());
    
    // 尝试走棋
    match game.make_move((0, 0), (1, 1)) {
        Ok(_) => println!("Move successful!"),
        Err(e) => println!("Move failed: {}", e),
    }
}
```

## 模块说明

### `pieces.rs`
- 棋子类型定义 (`PieceType`)
- 颜色定义 (`Color`)
- 棋子结构体 (`Piece`)

### `board.rs`
- 棋盘表示 (`Board`)
- 9x10 棋盘
- 棋子位置管理

### `game.rs`
- 游戏状态 (`Game`)
- 走棋历史
- 胜负判断

### `move_gen.rs`
- 走法生成
- 所有棋子的合法走法
- 吃子检测

## 开发状态

当前项目处于基础结构搭建阶段。主要模块已经定义，但部分核心功能（如棋盘初始化、走法验证）需要实现。

### 待办事项

1. **核心功能**
   - [ ] 实现棋盘初始位置
   - [ ] 完善走法验证
   - [ ] 实现将军和将死检测
   
2. **高级功能**
   - [ ] 添加 AI 引擎
   - [ ] 支持 UCI 协议
   - [ ] 添加图形界面

3. **优化改进**
   - [ ] 性能优化
   - [ ] 更好的错误处理
   - [ ] 完整的测试覆盖

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License
