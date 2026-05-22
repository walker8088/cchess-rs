# XQF 1.1 文件格式规范

## 概述

XQF（Xiangqi File）是中国象棋棋谱的二进制存储格式。版本 1.1（版本号 > 0x0A）引入了加密支持和更完善的走子数据结构，包括多分支（变着）支持。

---

## 1. 文件整体结构

```
┌─────────────────────────────────────────────────────┐
│                    XQF File                          │
├─────────────────────────────────────────────────────┤
│  Header (1024 bytes = 0x400)                        │
│  ├── Magic (2 bytes): "XQ"                          │
│  ├── Version (1 byte)                               │
│  ├── Encryption Keys (13 bytes)                     │
│  ├── Board Position (32 bytes)                      │
│  └── Metadata Strings (variable)                    │
├─────────────────────────────────────────────────────┤
│  Move Data (variable length, encrypted)             │
│  ├── Step 1 (4 bytes header + optional annotation) │
│  ├── Step 2                                         │
│  ├── ...                                            │
│  └── Variations (recursive structure)               │
└─────────────────────────────────────────────────────┘
```

---

## 2. 文件头详解（1024 字节）

### 2.1 基础头部（偏移 0-15）

| 偏移 | 大小 | 字段 | 说明 |
|------|------|------|------|
| 0x00 | 2 | Magic | 固定为 `58 51` ("XQ") |
| 0x02 | 1 | Version | 版本号，> 0x0A 表示加密版本 |
| 0x03 | 1 | KeyMask | 加密掩码 |
| 0x04 | 4 | ProductId | 产品号 |
| 0x08 | 1 | KeyOrA | 加密因子 A |
| 0x09 | 1 | KeyOrB | 加密因子 B |
| 0x0A | 1 | KeyOrC | 加密因子 C |
| 0x0B | 1 | KeyOrD | 加密因子 D |
| 0x0C | 1 | KeysSum | 加密钥匙和 |
| 0x0D | 1 | KeyXY | 棋子位置加密因子 |
| 0x0E | 1 | KeyXYf | 走子起点加密因子 |
| 0x0F | 1 | KeyXYt | 走子终点加密因子 |

### 2.2 棋盘布局数据（偏移 0x10-0x2F，32 字节）

存储 32 个棋子的位置信息，每字节表示一个棋子的位置编码：

```
位置编码 = 行号 × 10 + 列号
```

- `0xFF` 表示该位置无棋子
- 棋子顺序固定：`RNBAKABNRCCPPPPP`（每方 16 个）
  - R = 车 (Rook)
  - N = 马 (Knight)
  - B = 士/仕 (Advisor)
  - A = 士/仕 (Advisor)
  - K = 将/帅 (King)
  - C = 炮/砲 (Cannon)
  - P = 兵/卒 (Pawn)

### 2.3 元数据字符串区

元数据采用 **长度前缀 + 字符串** 的格式存储：

| 字段 | 偏移 | 说明 |
|------|------|------|
| Title Length | 可变 | 标题长度（1 字节） |
| Title | 可变 | 棋谱标题（GB18030 编码） |
| Match Name Length | 可变 | 赛事名称长度 |
| Match Name | 可变 | 赛事名称 |
| Red Player Length | 可变 | 红方名称长度 |
| Red Player | 可变 | 红方棋手名 |
| Black Player Length | 可变 | 黑方名称长度 |
| Black Player | 可变 | 黑方棋手名 |
| Result | 可变 | 结果代码（0=未知, 1=红胜, 2=黑胜, 3=和） |

---

## 3. 加密机制

### 3.1 版本判断

- **版本 ≤ 0x0A**：不加密，直接解析
- **版本 > 0x0A**：需要解密走子数据

### 3.2 密钥计算

```rust
// 棋子位置加密因子
KeyXY = (((((((KeyXY² × 3 + 9) × 3 + 8) × 2 + 1) × 3 + 8) × KeyXY) & 0xFF)

// 走子起点加密因子
KeyXYf = (((((((KeyXYf² × 3 + 9) × 3 + 8) × 2 + 1) × 3 + 8) × KeyXY) & 0xFF)

// 走子终点加密因子
KeyXYt = (((((((KeyXYt² × 3 + 9) × 3 + 8) × 2 + 1) × 3 + 8) × KeyXYf) & 0xFF)

// 注释大小加密因子
KeyRMKSize = ((KeysSum × 256 + KeyXY) % 32000 + 767) & 0xFFFF
```

### 3.3 F32Keys 生成

```
Base = "[(C) Copyright Mr. Dong Shiwei.]" (32 字节)

FKeyBytes = [
    (KeysSum & KeyMask) | KeyOrA,
    (KeyXY & KeyMask) | KeyOrB,
    (KeyXYf & KeyMask) | KeyOrC,
    (KeyXYt & KeyMask) | KeyOrD,
]

for i in 0..32:
    F32Keys[i] = Base[i] & FKeyBytes[i % 4]
```

### 3.4 走子数据解密

```rust
for i in 0..data.len():
    KeyByte = F32Keys[(0x400 + i) % 32]
    Decrypted[i] = (Encrypted[i] - KeyByte) & 0xFF
```

---

## 4. 走子数据结构

### 4.1 走子头部（4 字节）

```
┌──────────┬──────────┬──────────┬──────────┐
│  From    │   To     │  Flags   │ Reserved │
│ (1 byte) │ (1 byte) │ (1 byte) │ (1 byte) │
└──────────┴──────────┴──────────┴──────────┘
```

### 4.2 标志位定义

| 位 | 掩码 | 含义 |
|----|------|------|
| 7 | 0x80 | `HAS_NEXT` - 有后续主走法 |
| 6 | 0x40 | `HAS_VAR` - 有变着分支 |
| 5 | 0x20 | `HAS_ANNO` - 有注释 |

### 4.3 位置解码

```rust
// XQF 位置编码 → (列, 行)
fn decode_pos(man_pos: u8) -> (u8, u8) {
    (man_pos / 10, man_pos % 10)
}

// 高版本需要减去加密因子
from_pos = (raw_from - 0x18 - KeyXYf) & 0xFF
to_pos = (raw_to - 0x20 - KeyXYt) & 0xFF

// 低版本直接减偏移
from_pos = (raw_from - 0x18) & 0xFF
to_pos = (raw_to - 0x20) & 0xFF
```

### 4.4 注释数据

如果 `Flags & 0x20 != 0`，则走子头部后跟随注释：

```
┌──────────────────────┬─────────────────────┐
│ Annotation Length    │ Annotation Text     │
│ (4 bytes, little-endian) │ (variable, GB18030) │
└──────────────────────┴─────────────────────┘
```

高版本注释长度需要解密：
```rust
real_length = stored_length - KeyRMKSize
```

---

## 5. 多分支（变着）结构

### 5.1 树形解析流程

```
read_steps():
    1. 读取 4 字节走子头部
    2. 解析标志位
    3. 读取注释（如果有）
    4. 创建当前走子节点
    5. 如果 HAS_NEXT:
        递归读取主走法 → 设为 main_line
    6. 如果 HAS_VAR:
        递归读取变着（使用棋盘备份） → 加入 variations
        branches += 1
```

### 5.2 示例结构

```
Move 1 (a)
    ├── main_line → Move 2 (b)
    │       ├── main_line → Move 3 (c)
    │       └── variations → [Var 1 (x)]
    │
    └── variations → [Var 1 (d)]
            └── main_line → Move 2' (e)
```

### 5.3 递归解析示意图

```
          Root
         /    \
       (a)    (d) ← variation
       /  \      \
     (b)  (x)←var (e)
     / \
   (c)  ...
```

---

## 6. 棋盘初始化

### 6.1 高版本（version >= 12）

```rust
// 位置重排
for i in 0..32:
    idx = (KeyXY + i + 1) & 0x1F
    tmpMan[idx] = manBuff[i]

// 位置解密
for i in 0..32:
    tmpMan[i] = (tmpMan[i] - KeyXY) & 0xFF
    if tmpMan[i] > 89:
        tmpMan[i] = 0xFF
```

### 6.2 低版本（version < 12）

```rust
for i in 0..32:
    tmpMan[i] = manBuff[i]
```

---

## 7. 文件读写流程

### 7.1 读取流程

```
1. 读取文件头 1024 字节
2. 检查 Magic ("XQ")
3. 读取版本号
4. 如果是高版本 (> 0x0A):
   a. 计算解密密钥
   b. 解密棋盘数据
   c. 解密走子数据
5. 解析元数据字符串
6. 初始化棋盘
7. 递归解析走子树
```

### 7.2 写入流程

```
1. 写入文件头
2. 写入加密密钥
3. 加密并写入棋盘数据
4. 写入元数据
5. 递归序列化走子树
6. 加密走子数据（高版本）
```

---

## 8. 数据编码参考

### 8.1 位置编码表

| 位置 | 编码 | 示例 |
|------|------|------|
| (0, 0) | 0 | 左上角 |
| (8, 0) | 8 | 右上角 |
| (0, 9) | 90 | 左下角 |
| (8, 9) | 98 | 右下角 |

### 8.2 结果代码

| 代码 | 含义 |
|------|------|
| 0 | 未知 (*) |
| 1 | 红胜 (1-0) |
| 2 | 黑胜 (0-1) |
| 3 | 和棋 (1/2-1/2) |
| 4 | 和棋 (*) |

---

## 9. Rust 实现 API

```rust
// 读取带变着的 XQF 文件
pub fn read_xqf_with_variations(path: &str) -> Result<XqfFileWithVariations, XqfError>;

// 从字节数组读取
pub fn read_xqf_from_bytes(contents: &[u8]) -> Result<XqfFileWithVariations, XqfError>;

// 转换为 Game 对象
pub fn xqf_file_to_game(xqf_file: &XqfFileWithVariations) -> Result<Game, XqfError>;
```

---

## 10. 注意事项

1. **编码**: 字符串使用 GB18030 编码（简体中文）
2. **字节序**: 整数使用小端序（Little-Endian）
3. **位置验证**: 解码后位置 > 89 视为无效（0xFF）
4. **递归深度**: 变着可以嵌套，需要控制递归深度
5. **棋盘备份**: 解析变着时需要保存父节点棋盘状态

---

*文档版本: 1.0*
*基于 Python io_xqf.py 和 Rust xqf.rs 实现总结*
