# 棋盘内部结构文档 / Internal Board Structure

## 坐标系概述

`cchess-rs` 使用 **ICCS/UCI 对齐的内部坐标系**，与外部记谱法标准保持一致，消除了不必要的坐标转换。

### 核心约定

| 维度 | 范围 | 说明 |
|------|------|------|
| 列 (col) | 0-8 | 从左到右 (a-i) |
| 行 (row) | 0-9 | **row 0 = 红方底线**, **row 9 = 黑方底线** |

### 棋盘布局

```
row 9  r  n  b  a  k  a  b  n  r    ← 黑方底线 (Black back rank)
row 8  .  .  .  .  .  .  .  .  .
row 7  .  c  .  .  .  .  .  c  .    ← 黑方砲
row 6  p  .  p  .  p  .  p  .  p    ← 黑方卒
row 5  .  .  .  .  .  .  .  .  .    ← 楚河
row 4  .  .  .  .  .  .  .  .  .    ← 汉界
row 3  P  .  P  .  P  .  P  .  P    ← 红方兵
row 2  .  C  .  .  .  .  .  C  .    ← 红方炮
row 1  .  .  .  .  .  .  .  .  .
row 0  R  N  B  A  K  A  B  N  R    ← 红方底线 (Red back rank)
         0  1  2  3  4  5  6  7  8  ← col
```

### FEN 字符约定

| 棋子 | 红方 (Red) | 黑方 (Black) |
|------|-----------|-------------|
| 帅/将 | `K` | `k` |
| 仕/士 | `A` | `a` |
| 相/象 | `B` | `b` |
| 马/傌 | `N` | `n` |
| 车/俥 | `R` | `r` |
| 炮/砲 | `C` | `c` |
| 兵/卒 | `P` | `p` |

- **红方 = 大写 FEN 字符** (uppercase)
- **黑方 = 小写 FEN 字符** (lowercase)

---

## 关键区域定义

### 九宫 (Palace)

| 方 | 列范围 | 行范围 | 中心点 |
|----|--------|--------|--------|
| 红方 | 3-5 | **0-2** | (4, 1) |
| 黑方 | 3-5 | **7-9** | (4, 8) |

### 河界 (River)

- **楚河汉界** 位于 row 4 和 row 5 之间
- 红方过河: `row >= 5`
- 黑方过河: `row <= 4`

### 前进方向 (Forward)

| 方 | 行变化 | 说明 |
|----|--------|------|
| 红方 | `row` **增大** | 从 row 0 向 row 9 移动 |
| 黑方 | `row` **减小** | 从 row 9 向 row 0 移动 |

---

## 模块坐标一致性

整个代码库使用统一的内部坐标系，**无需**在模块间转换坐标：

```
pieces.rs (FEN 字符定义)
    ↓
board.rs (squares[row][col] 存储)
    ↓
move_gen.rs (使用相同 row/col 生成走法)
    ↓
game.rs (走法树、UCI 记谱)
    ↓
move_notation.rs (中文/WXF/ICCS 输出)
```

### ICCS 坐标映射

ICCS 坐标与内部坐标**完全一致**，无需转换：

| ICCS | 内部 (col, row) | 说明 |
|------|----------------|------|
| `a0` | (0, 0) | 红方左下车 |
| `i0` | (8, 0) | 红方右下车 |
| `a9` | (0, 9) | 黑方左下俥 |
| `i9` | (8, 9) | 黑方右下俥 |
| `h2` | (7, 2) | 红方右炮 |

### UCI 记谱格式

UCI 格式直接使用内部坐标: `{col_letter}{row}{col_letter}{row}`

- 列: `a`(0) 到 `i`(8)
- 行: `0`(红方底线) 到 `9`(黑方底线)

示例:
- `a0a1` → 红方左车前进一格
- `h2e2` → 红方炮二平五
- `a9a8` → 黑方左俥前进一格

---

## 走法记谱法 (Move Notation)

### 中文传统记谱

中文记谱从**己方视角**从右到左数路数 (1-9)：

| 内部列 | 红方路数 | 黑方路数 |
|--------|---------|---------|
| 0 | 九路 | 1路 |
| 1 | 八路 | 2路 |
| 2 | 七路 | 3路 |
| 3 | 六路 | 4路 |
| 4 | 五路 | 5路 |
| 5 | 四路 | 6路 |
| 6 | 三路 | 7路 |
| 7 | 二路 | 8路 |
| 8 | 一路 | 9路 |

计算公式:
- 红方路数: `9 - col`
- 黑方路数: `col + 1`

### 方向判断

```rust
// Red: increasing row = forward
// Black: decreasing row = forward
let is_forward = match color {
    Side::Red => dst_row > src_row,
    Side::Black => dst_row < src_row,
};
```

### 数字格式

| 方 | 路数数字 | 示例 |
|----|---------|------|
| 红方 | 中文数字 | 一、二、三...九 |
| 黑方 | 全角阿拉伯数字 | １、２、３...９ |

---

## 数据结构

### Board 结构

```rust
pub struct Board {
    pub squares: [[char; 9]; 10],  // squares[row][col]
}
```

- `squares[row][col]` — 先行后列的二维数组
- 空位用 `'.'` 表示

### Move 结构

```rust
pub struct Move {
    pub from_col: usize,
    pub from_row: usize,
    pub to_col: usize,
    pub to_row: usize,
    pub captured: Option<char>,
}
```

所有坐标均使用内部坐标系，无需转换。

---

## 已修复的坐标相关 Bug

以下 Bug 已在坐标系统统一过程中修复：

| 文件 | 函数 | 问题 | 修复 |
|------|------|------|------|
| `board.rs` | `validate_advisor_move` | 红方九宫行范围写反 (7-9 应为 0-2) | ✅ 已修复 |
| `move_gen.rs` | `generate_king_moves` | 红方九宫行范围写反 (7-9 应为 0-2) | ✅ 已修复 |
| `move_gen.rs` | `generate_advisor_moves` | 红方九宫行范围写反 (7-9 应为 0-2) | ✅ 已修复 |
| `move_notation.rs` | `calculate_direction` | 前进/后退逻辑颠倒 | ✅ 已修复 |
| `move_notation.rs` | `from_board_move` | 不必要的黑方坐标翻转 | ✅ 已移除 |
| `game.rs` | `make_move` | UCI 记谱使用了 `9-row` 翻转 | ✅ 已修复 |
| `game.rs` | `make_variation` | UCI 记谱使用了 `9-row` 翻转 | ✅ 已修复 |

---

## 历史背景

### 旧坐标系 (已废弃)

早期实现中存在坐标系不一致的问题：
- `initial_position()` 使用 row 0 = 红方
- 部分验证函数期望 row 0 = 黑方
- 导致需要大量的 `9 - row` 坐标翻转

### 新坐标系 (当前)

统一为 ICCS/UCI 标准：
- 内部坐标 = ICCS 坐标
- 消除了所有不必要的坐标翻转
- 前进方向逻辑直接使用 `>`/`<` 比较行号

---

## 测试要点

编写测试时注意：

1. **红方棋子** 位于 row 0-3 区域
2. **黑方棋子** 位于 row 6-9 区域
3. 红方**前进** = row 增大，黑方**前进** = row 减小
4. 九宫: 红方 rows 0-2，黑方 rows 7-9
5. ICCS/UCI 坐标直接使用内部 `(col, row)` 值
