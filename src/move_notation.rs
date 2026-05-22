/// 走法中间表示模块
/// 实现中国象棋走法的中间表示，支持简体中文、繁体中文和紧凑格式
use crate::board::Board;
use crate::pieces::{PieceType, Side};

/// 走法方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,    // 进
    Backward,   // 退
    Horizontal, // 平
}

/// 限定词类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Qualifier {
    Front,      // 前
    Middle,     // 中
    Back,       // 后
    Number(u8), // 数字限定词 (1-9)
}

/// 输出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveFormat {
    Chinese, // 中文传统记谱法
    WXF,     // WXF (World XiangQi Federation) 格式
    ICCS,    // ICCS坐标格式
}

/// 中文本地化设置
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChineseLocale {
    Simplified,  // 简体中文
    Traditional, // 繁体中文
}

/// 走法中间表示
#[derive(Debug, Clone)]
pub struct MoveNotation {
    pub piece_type: PieceType,        // 棋子类型
    pub piece_color: Side,            // 棋子颜色
    pub column: u8,                   // 列 (0-8，红方视角)
    pub direction: Direction,         // 方向
    pub distance: u8,                 // 距离/目标列
    pub qualifier: Option<Qualifier>, // 限定词
    pub is_capture: bool,             // 是否吃子
    pub is_check: bool,               // 是否将军
    pub is_checkmate: bool,           // 是否将死
}

impl MoveNotation {
    /// 从棋盘走法创建中间表示
    pub fn from_board_move(
        board: &Board,
        from: (usize, usize),
        to: (usize, usize),
    ) -> Result<Self, String> {
        let (from_col, from_row) = from;
        let (to_col, to_row) = to;

        // 验证坐标合法性
        if !Board::is_within_bounds(from_col, from_row) {
            return Err(format!(
                "起始坐标 ({}, {}) 超出棋盘范围",
                from_col, from_row
            ));
        }
        if !Board::is_within_bounds(to_col, to_row) {
            return Err(format!("目标坐标 ({}, {}) 超出棋盘范围", to_col, to_row));
        }

        // 检查起始位置是否有棋子
        if board.is_empty_at(from_col, from_row) {
            return Err(format!("起始位置 ({}, {}) 没有棋子", from_col, from_row));
        }

        // 获取棋子信息
        let (piece_fen, piece_color) = board.get_fen_and_color(from_col, from_row);
        let piece_type = match PieceType::from_fen(piece_fen) {
            Some(pt) => pt,
            None => return Err(format!("无效的棋子字符: {}", piece_fen)),
        };

        let color = piece_color.unwrap_or(Side::Black);

        // 检查是否为吃子
        let is_capture = !board.is_empty_at(to_col, to_row);
        if is_capture {
            let (_target_fen, target_color) = board.get_fen_and_color(to_col, to_row);
            // 检查是否吃自己的棋子
            if let (Some(pc), Some(tc)) = (piece_color, target_color) {
                if pc == tc {
                    return Err("不能吃自己的棋子".to_string());
                }
            }
        }

        // 根据颜色应用棋盘翻转（仅用于方向计算）
        // 内部坐标系已与 ICCS 对齐 (row 0 = 红方底线)，无需翻转
        let (src_col_for_dir, src_row_for_dir, dst_col_for_dir, dst_row_for_dir) =
            (from_col, from_row, to_col, to_row);

        // 计算方向：直接使用原始坐标
        let direction = calculate_direction(
            color,
            src_col_for_dir,
            src_row_for_dir,
            dst_col_for_dir,
            dst_row_for_dir,
        );

        // 计算距离：使用原始坐标
        let distance = calculate_distance(
            piece_type,
            src_col_for_dir,
            src_row_for_dir,
            dst_col_for_dir,
            dst_row_for_dir,
        );

        // 计算列（红方视角，从右到左1-9）
        // 注意：列号使用原始坐标，不翻转
        let column = (9 - from_col) as u8;

        // 计算限定词
        let qualifier = calculate_qualifier(board, piece_type, color, from_col, from_row);

        Ok(MoveNotation {
            piece_type,
            piece_color: color,
            column,
            direction,
            distance,
            qualifier,
            is_capture,
            is_check: false,     // 需要额外检查将军
            is_checkmate: false, // 需要额外检查将死
        })
    }

    /// 转换为中文
    pub fn to_chinese(&self, locale: ChineseLocale) -> String {
        let mut result = String::new();

        // 判断限定词类型
        let use_qualifier_prefix = matches!(
            self.qualifier,
            Some(Qualifier::Front | Qualifier::Middle | Qualifier::Back)
        );
        let use_number_qualifier = matches!(self.qualifier, Some(Qualifier::Number(_)));

        if use_qualifier_prefix {
            // 前/中/后限定词：限定词 + 棋子名 + 方向 + 距离（省略路数）
            if let Some(qualifier) = self.qualifier {
                result.push_str(&qualifier_to_string(qualifier, locale));
            }
            result.push_str(&get_piece_name(self.piece_type, self.piece_color, locale));
        } else if use_number_qualifier {
            // 数字限定词（4+个棋子）：数字 + 棋子名 + 方向 + 距离（省略路数）
            if let Some(qualifier) = self.qualifier {
                result.push_str(&qualifier_to_string(qualifier, locale));
            }
            result.push_str(&get_piece_name(self.piece_type, self.piece_color, locale));
        } else {
            // 无限定词：棋子名 + 路数 + 方向 + 距离
            result.push_str(&get_piece_name(self.piece_type, self.piece_color, locale));
            result.push_str(&format_number(self.column, self.piece_color, locale));
        }

        // 添加方向
        result.push_str(&direction_to_string(self.direction, locale));

        // 添加距离/目标列
        result.push_str(&format_number(self.distance, self.piece_color, locale));

        // 添加吃子标记（可选）
        if self.is_capture {
            result.push('吃');
        }

        // 添加将军标记（可选）
        if self.is_check {
            result.push('将');
        }

        // 添加将死标记（可选）
        if self.is_checkmate {
            result.push('杀');
        }

        result
    }

    /// 转换为 WXF (World XiangQi Federation) 格式
    /// e.g., 炮二平五 -> C2.5, 马8进7 -> N8+7
    pub fn to_wxf(&self) -> String {
        let mut result = String::new();

        // 添加限定词（前/中/后 → +/-/.)
        if let Some(qualifier) = self.qualifier {
            match qualifier {
                Qualifier::Front => result.push('+'),
                Qualifier::Middle => result.push('-'),
                Qualifier::Back => result.push('.'),
                Qualifier::Number(n) => result.push_str(&n.to_string()),
            };
        }

        // 添加棋子字母
        result.push(piece_to_wxf_letter(self.piece_type));

        // 添加路数
        result.push_str(&self.column.to_string());

        // 方向符号（WXF用 . 表示平）
        result.push(match self.direction {
            Direction::Forward => '+',
            Direction::Backward => '-',
            Direction::Horizontal => '.',
        });

        // 添加距离
        result.push_str(&self.distance.to_string());

        // 添加吃子标记
        if self.is_capture {
            result.push('x');
        }

        // 添加将军标记
        if self.is_check {
            result.push('+');
        }

        // 添加将死标记
        if self.is_checkmate {
            result.push('#');
        }

        result
    }

    /// 将 Move 格式化为 ICCS 字符串 (如 "h0e0")
    /// ICCS 坐标与内部坐标一致：row 0 = 红方底线
    pub fn to_iccs(&self, from: (usize, usize), to: (usize, usize)) -> String {
        let (from_col, from_row) = from;
        let (to_col, to_row) = to;

        format!(
            "{}{}{}{}",
            (b'a' + from_col as u8) as char,
            from_row,
            (b'a' + to_col as u8) as char,
            to_row
        )
    }
}

/// 翻转坐标（棋盘视角转换，用于可视化/调试）
/// 注意：走法记谱已不再使用此函数，内部坐标已与 ICCS 对齐
#[allow(dead_code)]
fn flip_coordinate(col: usize, row: usize) -> (usize, usize) {
    (8 - col, 9 - row)
}

/// 计算方向
fn calculate_direction(
    color: Side,
    src_col: usize,
    src_row: usize,
    dst_col: usize,
    dst_row: usize,
) -> Direction {
    // 行相同：水平移动（平）
    if src_row == dst_row {
        return Direction::Horizontal;
    }

    // 列相同：垂直移动
    if src_col == dst_col {
        let is_forward = match color {
            Side::Black => dst_row < src_row, // Black: decreasing row = forward (toward Red)
            Side::Red => dst_row > src_row,   // Red: increasing row = forward (toward Black)
            Side::Any => dst_row < src_row,   // default: Black-like
        };
        if is_forward {
            Direction::Forward
        } else {
            Direction::Backward
        }
    } else {
        // 列和行都不同：斜线移动（马、士、象）
        let is_forward = match color {
            Side::Black => dst_row < src_row, // Black: decreasing row = forward
            Side::Red => dst_row > src_row,   // Red: increasing row = forward
            Side::Any => dst_row < src_row,   // default: Black-like
        };
        if is_forward {
            Direction::Forward
        } else {
            Direction::Backward
        }
    }
}

/// 计算距离
fn calculate_distance(
    piece_type: PieceType,
    src_col: usize,
    src_row: usize,
    dst_col: usize,
    dst_row: usize,
) -> u8 {
    // 对于所有棋子的平移，显示目标路数
    if src_col != dst_col {
        // 水平移动：显示目标路数
        return (9 - dst_col) as u8;
    }

    // 垂直移动
    match piece_type {
        PieceType::King | PieceType::Rook | PieceType::Cannon | PieceType::Pawn => {
            // 王、车、炮、兵：显示步数
            dst_row.abs_diff(src_row) as u8
        }
        PieceType::Knight | PieceType::Advisor | PieceType::Elephant => {
            // 马、士、象：显示目标路数
            (9 - dst_col) as u8
        }
    }
}

/// 计算限定词
fn calculate_qualifier(
    board: &Board,
    piece_type: PieceType,
    color: Side,
    col: usize,
    row: usize,
) -> Option<Qualifier> {
    // 将/帅、士/仕、象/相没有限定词
    match piece_type {
        PieceType::King | PieceType::Advisor | PieceType::Elephant => return None,
        _ => {}
    }

    // 查找同列所有相同棋子
    let mut same_pieces = Vec::new();

    for r in 0..10 {
        if r == row {
            continue;
        }

        if let Some(pt) = board.get_piece_type(col, r) {
            if pt == piece_type {
                if let Some(c) = board.get_color_at(col, r) {
                    if c == color {
                        same_pieces.push(r);
                    }
                }
            }
        }
    }

    // 如果没有相同棋子，不需要限定词
    if same_pieces.is_empty() {
        return None;
    }

    // 按"从前到后"排序，position 0 = 前，position last = 后
    // 新坐标系 (row 0 = 红方底线):
    // 红方：row 从小到大（小=靠近己方=后，大=靠近对方=前）
    // 黑方：row 从大到小（大=靠近己方=后，小=靠近对方=前）
    same_pieces.sort_by(|r1, r2| {
        if color == Side::Red {
            // Red: smaller row = rear, larger row = front
            r2.cmp(r1) // descending: front first
        } else {
            // Black: larger row = rear, smaller row = front
            r1.cmp(r2) // ascending: front first
        }
    });

    // 添加当前棋子
    let mut all_pieces = same_pieces;
    all_pieces.push(row);

    // 重新排序
    all_pieces.sort_by(|r1, r2| {
        if color == Side::Red {
            r2.cmp(r1)
        } else {
            r1.cmp(r2)
        }
    });

    // 分配限定词
    let position = all_pieces.iter().position(|r| *r == row).unwrap();

    match all_pieces.len() {
        2 => {
            // 两个相同棋子：前/后
            match position {
                0 => Some(Qualifier::Front), // 最前面 = 前
                1 => Some(Qualifier::Back),  // 最后面 = 后
                _ => None,
            }
        }
        3 => {
            // 三个相同棋子：前/中/后
            match position {
                0 => Some(Qualifier::Front),
                1 => Some(Qualifier::Middle),
                2 => Some(Qualifier::Back),
                _ => None,
            }
        }
        4..=9 => {
            // 四个或更多：使用数字
            Some(Qualifier::Number(position as u8 + 1))
        }
        _ => None,
    }
}

/// 获取棋子名称
fn get_piece_name(piece_type: PieceType, color: Side, locale: ChineseLocale) -> String {
    match locale {
        ChineseLocale::Simplified => simplified_piece_name(piece_type, color),
        ChineseLocale::Traditional => traditional_piece_name(piece_type, color),
    }
}

/// 获取简体中文棋子名称
fn simplified_piece_name(piece_type: PieceType, color: Side) -> String {
    match (piece_type, color) {
        // Red (红方): 帅、仕、相、马、车、炮、兵
        (PieceType::King, Side::Red) => "帅".to_string(),
        (PieceType::Advisor, Side::Red) => "仕".to_string(),
        (PieceType::Elephant, Side::Red) => "相".to_string(),
        (PieceType::Knight, Side::Red) => "马".to_string(),
        (PieceType::Rook, Side::Red) => "车".to_string(),
        (PieceType::Cannon, Side::Red) => "炮".to_string(),
        (PieceType::Pawn, Side::Red) => "兵".to_string(),

        // Black (黑方): 将、士、象、马、车、炮、卒
        (PieceType::King, Side::Black) => "将".to_string(),
        (PieceType::Advisor, Side::Black) => "士".to_string(),
        (PieceType::Elephant, Side::Black) => "象".to_string(),
        (PieceType::Knight, Side::Black) => "马".to_string(),
        (PieceType::Rook, Side::Black) => "车".to_string(),
        (PieceType::Cannon, Side::Black) => "炮".to_string(),
        (PieceType::Pawn, Side::Black) => "卒".to_string(),

        _ => "?".to_string(),
    }
}

/// 获取繁体中文棋子名称
fn traditional_piece_name(piece_type: PieceType, color: Side) -> String {
    match (piece_type, color) {
        // Red (红方): 帥、仕、相、馬、車、砲、兵
        (PieceType::King, Side::Red) => "帥".to_string(),
        (PieceType::Advisor, Side::Red) => "仕".to_string(),
        (PieceType::Elephant, Side::Red) => "相".to_string(),
        (PieceType::Knight, Side::Red) => "馬".to_string(),
        (PieceType::Rook, Side::Red) => "車".to_string(),
        (PieceType::Cannon, Side::Red) => "砲".to_string(),
        (PieceType::Pawn, Side::Red) => "兵".to_string(),

        // Black (黑方): 將、士、象、傌、俥、砲、卒
        (PieceType::King, Side::Black) => "將".to_string(),
        (PieceType::Advisor, Side::Black) => "士".to_string(),
        (PieceType::Elephant, Side::Black) => "象".to_string(),
        (PieceType::Knight, Side::Black) => "傌".to_string(),
        (PieceType::Rook, Side::Black) => "俥".to_string(),
        (PieceType::Cannon, Side::Black) => "砲".to_string(),
        (PieceType::Pawn, Side::Black) => "卒".to_string(),

        _ => "?".to_string(),
    }
}

/// 限定词转换为字符串
fn qualifier_to_string(qualifier: Qualifier, locale: ChineseLocale) -> String {
    match locale {
        ChineseLocale::Simplified => match qualifier {
            Qualifier::Front => "前".to_string(),
            Qualifier::Middle => "中".to_string(),
            Qualifier::Back => "后".to_string(),
            Qualifier::Number(n) => simplified_number(n),
        },
        ChineseLocale::Traditional => match qualifier {
            Qualifier::Front => "前".to_string(),
            Qualifier::Middle => "中".to_string(),
            Qualifier::Back => "後".to_string(),
            Qualifier::Number(n) => traditional_number(n),
        },
    }
}

/// 方向转换为字符串
fn direction_to_string(direction: Direction, locale: ChineseLocale) -> String {
    match locale {
        ChineseLocale::Simplified => match direction {
            Direction::Forward => "进".to_string(),
            Direction::Backward => "退".to_string(),
            Direction::Horizontal => "平".to_string(),
        },
        ChineseLocale::Traditional => match direction {
            Direction::Forward => "進".to_string(),
            Direction::Backward => "退".to_string(),
            Direction::Horizontal => "平".to_string(),
        },
    }
}

/// 格式化数字（根据颜色）
/// Red (红方) 使用中文数字 (一, 二, 三)
/// Black (黑方) 使用阿拉伯数字全角形式 (１, ２, ３)
fn format_number(number: u8, color: Side, locale: ChineseLocale) -> String {
    if color == Side::Red {
        // Red: Chinese numerals
        match locale {
            ChineseLocale::Simplified => simplified_number(number),
            ChineseLocale::Traditional => traditional_number(number),
        }
    } else {
        // Black: fullwidth Arabic numerals
        fullwidth_number(number)
    }
}

/// 简体中文数字
fn simplified_number(number: u8) -> String {
    match number {
        1 => "一".to_string(),
        2 => "二".to_string(),
        3 => "三".to_string(),
        4 => "四".to_string(),
        5 => "五".to_string(),
        6 => "六".to_string(),
        7 => "七".to_string(),
        8 => "八".to_string(),
        9 => "九".to_string(),
        _ => format!("{}", number),
    }
}

/// 繁体中文数字
fn traditional_number(number: u8) -> String {
    // 繁体中文数字与简体相同
    simplified_number(number)
}

/// 全角数字（黑方使用）
fn fullwidth_number(number: u8) -> String {
    match number {
        1 => "１".to_string(),
        2 => "２".to_string(),
        3 => "３".to_string(),
        4 => "４".to_string(),
        5 => "５".to_string(),
        6 => "６".to_string(),
        7 => "７".to_string(),
        8 => "８".to_string(),
        9 => "９".to_string(),
        _ => format!("{}", number),
    }
}

/// ICCS 列字符: a=0, b=1, ..., i=8
const ICCS_COL_CHARS: [char; 9] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'];

// ===========================================================================
// ICCS 坐标走法解析与格式化
// ===========================================================================
//
// ICCS (International Computer Chess Society) 记谱法用于中国象棋:
// - 列: a-i (从左到右)
// - 行: 0-9 (row 0 = 红方底线, row 9 = 黑方底线)
// - 格式: "h0e0" (4字符紧凑格式) 或 "h0-e0" (带连字符)
//
// Rust 内部坐标与 ICCS 一致:
// - 列: 0-8
// - 行: 0-9 (row 0 = 红方底线, row 9 = 黑方底线)

/// 解析 ICCS 格式走法字符串 (如 "h0e2") 为 Move 结构
///
/// 支持两种格式:
/// - 紧凑格式: "h0e2" (4字符)
/// - 连字符格式: "h0-e2" (5字符)
pub fn parse_iccs_move(s: &str) -> crate::move_gen::Move {
    let chars: Vec<char> = s.chars().collect();

    // 跳过连字符 (h0-e2 → h0e2)
    let compact: String = chars.iter().filter(|&&c| c != '-').collect();
    let c: Vec<char> = compact.chars().collect();

    if c.len() != 4 {
        panic!("Invalid ICCS notation '{}': expected 4 characters", s);
    }

    let from_col = iccs_col_to_index(c[0]).expect("Invalid ICCS column");
    let from_row = c[1].to_digit(10).expect("Invalid ICCS row") as usize;
    let to_col = iccs_col_to_index(c[2]).expect("Invalid ICCS column");
    let to_row = c[3].to_digit(10).expect("Invalid ICCS row") as usize;

    // ICCS 坐标与内部坐标一致，无需转换
    crate::move_gen::Move::new(from_col, from_row, to_col, to_row)
}

/// 解析 ICCS 格式走法字符串，返回 Result
pub fn try_parse_iccs_move(s: &str) -> Result<crate::move_gen::Move, String> {
    let chars: Vec<char> = s.chars().collect();
    let compact: String = chars.iter().filter(|&&c| c != '-').collect();
    let c: Vec<char> = compact.chars().collect();

    if c.len() != 4 {
        return Err(format!(
            "Invalid ICCS notation '{}': expected 4 characters",
            s
        ));
    }

    let from_col = iccs_col_to_index(c[0])?;
    let from_row = c[1].to_digit(10).ok_or("Invalid ICCS row")? as usize;
    let to_col = iccs_col_to_index(c[2])?;
    let to_row = c[3].to_digit(10).ok_or("Invalid ICCS row")? as usize;

    // ICCS 坐标与内部坐标一致，无需转换
    Ok(crate::move_gen::Move::new(
        from_col, from_row, to_col, to_row,
    ))
}

/// 将 Move 格式化为 ICCS 字符串 (如 "h0e0")
pub fn format_iccs_move(mv: &crate::move_gen::Move) -> String {
    // ICCS 坐标与内部坐标一致，直接使用
    let from_row_iccs = mv.from_row.min(9);
    let to_row_iccs = mv.to_row.min(9);
    let from_col = ICCS_COL_CHARS[mv.from_col.min(8)];
    let to_col = ICCS_COL_CHARS[mv.to_col.min(8)];
    format!("{}{}{}{}", from_col, from_row_iccs, to_col, to_row_iccs)
}

/// 将 ICCS 列字符转换为索引 (a=0, b=1, ..., i=8)
fn iccs_col_to_index(c: char) -> Result<usize, String> {
    match c {
        'a'..='i' => Ok(c as usize - 'a' as usize),
        _ => Err(format!("Invalid ICCS column '{}': expected a-i", c)),
    }
}

/// 将索引转换为 ICCS 列字符 (0=a, 1=b, ..., 8=i)
pub fn index_to_iccs_col(col: usize) -> Result<char, String> {
    if col > 8 {
        Err(format!("Invalid column index {}: expected 0-8", col))
    } else {
        Ok(ICCS_COL_CHARS[col])
    }
}

/// 棋子转换为 WXF 格式字母 (统一大写)
fn piece_to_wxf_letter(piece_type: PieceType) -> char {
    match piece_type {
        PieceType::King => 'K',
        PieceType::Advisor => 'A',
        PieceType::Elephant => 'B',
        PieceType::Knight => 'N',
        PieceType::Rook => 'R',
        PieceType::Cannon => 'C',
        PieceType::Pawn => 'P',
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    fn test_flip_coordinate() {
        assert_eq!(flip_coordinate(0, 0), (8, 9));
        assert_eq!(flip_coordinate(8, 9), (0, 0));
        assert_eq!(flip_coordinate(4, 5), (4, 4));
    }

    #[test]
    fn test_calculate_direction() {
        // Black direction: forward = decreasing row (toward Red's side)
        assert_eq!(
            calculate_direction(Side::Black, 0, 1, 0, 0),
            Direction::Forward
        );
        assert_eq!(
            calculate_direction(Side::Black, 0, 0, 0, 1),
            Direction::Backward
        );
        assert_eq!(
            calculate_direction(Side::Black, 0, 0, 1, 0),
            Direction::Horizontal
        );

        // Red direction: forward = increasing row (toward Black's side)
        assert_eq!(
            calculate_direction(Side::Red, 0, 8, 0, 9),
            Direction::Forward
        );
        assert_eq!(
            calculate_direction(Side::Red, 0, 9, 0, 8),
            Direction::Backward
        );
        assert_eq!(
            calculate_direction(Side::Red, 0, 9, 1, 9),
            Direction::Horizontal
        );
    }

    #[test]
    fn test_simplified_number() {
        assert_eq!(simplified_number(1), "一");
        assert_eq!(simplified_number(5), "五");
        assert_eq!(simplified_number(9), "九");
    }

    #[test]
    fn test_fullwidth_number() {
        assert_eq!(fullwidth_number(1), "１");
        assert_eq!(fullwidth_number(5), "５");
        assert_eq!(fullwidth_number(9), "９");
    }

    #[test]
    fn test_piece_names() {
        // 测试简体中文棋子名称
        // Red (红方): 帅、仕、相、马、车、炮、兵
        assert_eq!(simplified_piece_name(PieceType::King, Side::Red), "帅");
        // Black (黑方): 将、士、象、马、车、炮、卒
        assert_eq!(simplified_piece_name(PieceType::King, Side::Black), "将");
        // Both use 车 for rook in simplified
        assert_eq!(simplified_piece_name(PieceType::Rook, Side::Black), "车");
        assert_eq!(simplified_piece_name(PieceType::Rook, Side::Red), "车");
        // Both use 炮 for cannon in simplified
        assert_eq!(simplified_piece_name(PieceType::Cannon, Side::Black), "炮");
        assert_eq!(simplified_piece_name(PieceType::Cannon, Side::Red), "炮");
        // Red=兵, Black=卒
        assert_eq!(simplified_piece_name(PieceType::Pawn, Side::Red), "兵");
        assert_eq!(simplified_piece_name(PieceType::Pawn, Side::Black), "卒");

        // 测试繁体中文棋子名称
        // Red (红方): 帥、仕、相、馬、車、砲、兵
        assert_eq!(traditional_piece_name(PieceType::King, Side::Red), "帥");
        // Black (黑方): 將、士、象、傌、俥、砲、卒
        assert_eq!(traditional_piece_name(PieceType::King, Side::Black), "將");
        // Red=車, Black=俥
        assert_eq!(traditional_piece_name(PieceType::Rook, Side::Red), "車");
        assert_eq!(traditional_piece_name(PieceType::Rook, Side::Black), "俥");
        // Both use 砲 for cannon in traditional
        assert_eq!(traditional_piece_name(PieceType::Cannon, Side::Black), "砲");
        assert_eq!(traditional_piece_name(PieceType::Cannon, Side::Red), "砲");
    }

    #[test]
    fn test_wxf_piece_letter() {
        // WXF 统一使用大写字母
        assert_eq!(piece_to_wxf_letter(PieceType::King), 'K');
        assert_eq!(piece_to_wxf_letter(PieceType::Advisor), 'A');
        assert_eq!(piece_to_wxf_letter(PieceType::Elephant), 'B');
        assert_eq!(piece_to_wxf_letter(PieceType::Knight), 'N');
        assert_eq!(piece_to_wxf_letter(PieceType::Rook), 'R');
        assert_eq!(piece_to_wxf_letter(PieceType::Cannon), 'C');
        assert_eq!(piece_to_wxf_letter(PieceType::Pawn), 'P');
    }

    #[test]
    fn test_move_notation_creation() {
        let mut board = Board::new();
        board.initial_position();

        // Test Red rook at (0,0) moving forward to (0,1)
        // Red rooks are at row 0 in the new coordinate system
        // Column 0 = 9路 (from Red's perspective, right-to-left)
        let result = MoveNotation::from_board_move(&board, (0, 0), (0, 1));
        assert!(result.is_ok());
        let notation = result.unwrap();

        assert_eq!(notation.piece_type, PieceType::Rook);
        assert_eq!(notation.piece_color, Side::Red);
        assert_eq!(notation.column, 9); // 九路
        assert_eq!(notation.direction, Direction::Forward);
        assert_eq!(notation.distance, 1);

        // Red uses Chinese numerals
        let chinese = notation.to_chinese(ChineseLocale::Simplified);
        assert_eq!(chinese, "车九进一");

        let traditional = notation.to_chinese(ChineseLocale::Traditional);
        assert_eq!(traditional, "車九進一");

        let wxf = notation.to_wxf();
        assert_eq!(wxf, "R9+1");
    }

    #[test]
    fn test_black_move_notation() {
        let mut board = Board::new();
        board.initial_position();

        // Test Black rook at (0,9) moving forward to (0,8)
        // Black rooks are at row 9 in the new coordinate system
        let result = MoveNotation::from_board_move(&board, (0, 9), (0, 8));
        assert!(result.is_ok());
        let notation = result.unwrap();

        assert_eq!(notation.piece_type, PieceType::Rook);
        assert_eq!(notation.piece_color, Side::Black);
        // Column calculation uses original coords (not flipped): 9 - 0 = 9
        assert_eq!(notation.column, 9);
        assert_eq!(notation.direction, Direction::Forward);
        assert_eq!(notation.distance, 1);

        // Black uses full-width Arabic numerals, rook = 车
        let chinese = notation.to_chinese(ChineseLocale::Simplified);
        assert_eq!(chinese, "车９进１");

        // WXF uses uppercase letters for all pieces
        let wxf = notation.to_wxf();
        assert_eq!(wxf, "R9+1");
    }

    #[test]
    fn test_cannon_move() {
        let mut board = Board::new();
        board.initial_position();

        // Test Red cannon at (7,2) moving horizontally to (4,2)
        // Red cannons are at row 2, column 2 and 7
        // Column 7 = 二路 (from Red's perspective: 9-7=2)
        let result = MoveNotation::from_board_move(&board, (7, 2), (4, 2));
        assert!(result.is_ok());
        let notation = result.unwrap();

        assert_eq!(notation.piece_type, PieceType::Cannon);
        assert_eq!(notation.piece_color, Side::Red);
        assert_eq!(notation.column, 2); // 二路
        assert_eq!(notation.direction, Direction::Horizontal);
        assert_eq!(notation.distance, 5); // 平到五路

        // Red uses Chinese numerals
        let chinese = notation.to_chinese(ChineseLocale::Simplified);
        assert_eq!(chinese, "炮二平五");

        let traditional = notation.to_chinese(ChineseLocale::Traditional);
        assert_eq!(traditional, "砲二平五");

        let wxf = notation.to_wxf();
        assert_eq!(wxf, "C2.5");
    }

    #[test]
    fn test_knight_move() {
        let mut board = Board::new();
        board.initial_position();

        // Test Red knight at (1,0) moving to (2,2)
        // Red knights are at row 0
        // Column 1 = 8路 (from Red's perspective: 9-1=8)
        let result = MoveNotation::from_board_move(&board, (1, 0), (2, 2));
        assert!(result.is_ok());
        let notation = result.unwrap();

        assert_eq!(notation.piece_type, PieceType::Knight);
        assert_eq!(notation.piece_color, Side::Red);
        assert_eq!(notation.column, 8); // 八路
        assert_eq!(notation.direction, Direction::Forward);
        assert_eq!(notation.distance, 7); // 进到七路

        // Red uses Chinese numerals
        let chinese = notation.to_chinese(ChineseLocale::Simplified);
        assert_eq!(chinese, "马八进七");
    }

    #[test]
    fn test_invalid_moves() {
        let mut board = Board::new();
        board.initial_position();

        // 测试无效坐标
        let result = MoveNotation::from_board_move(&board, (10, 10), (0, 0));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("超出棋盘范围"));

        // 测试空位置
        let result = MoveNotation::from_board_move(&board, (4, 4), (4, 5));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("没有棋子"));

        // 测试吃自己的棋子（红方吃红方）
        // 创建一个红方车吃红方马的局面
        let mut board = Board::new();
        // 在(1,0)放一个红方马，在(0,0)放一个红方车
        board.set_fen(0, 0, 'r');
        board.set_fen(1, 0, 'n');
        let result = MoveNotation::from_board_move(&board, (0, 0), (1, 0));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不能吃自己的棋子"));
    }

    // ========== 限定词测试 ==========

    #[test]
    fn test_qualifier_two_red_rooks() {
        // 创建两辆红车同列的局面
        let mut board = Board::new();
        board.clear();
        // 红方两车同列（列4），一上一下
        // New coords: row 0 = Red bottom, higher rows = towards Black
        board.set_fen(4, 1, 'R'); // 后车 (closer to Red's bottom)
        board.set_fen(4, 3, 'R'); // 前车 (closer to Black's side)
        board.set_fen(4, 9, 'k'); // 黑将
        board.set_fen(4, 0, 'K'); // 红帅

        // 前车进一 (forward = increasing row for Red)
        let result = MoveNotation::from_board_move(&board, (4, 3), (4, 4));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前车进一");
        assert_eq!(notation.to_wxf(), "+R5+1");

        // 后车进一
        let result = MoveNotation::from_board_move(&board, (4, 1), (4, 2));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后车进一");
        assert_eq!(notation.to_wxf(), ".R5+1");
    }

    #[test]
    fn test_qualifier_three_red_pawns() {
        // 创建三个红兵同列的局面（每个兵之间有空位）
        let mut board = Board::new();
        board.clear();
        // New coords: row 0 = Red bottom, higher rows = towards Black
        board.set_fen(4, 1, 'P'); // 后兵
        board.set_fen(4, 3, 'P'); // 中兵
        board.set_fen(4, 5, 'P'); // 前兵
        board.set_fen(4, 9, 'k'); // 黑将
        board.set_fen(4, 0, 'K'); // 红帅

        // 前兵进一（从原始位置）- Red forward = increasing row
        let notation = MoveNotation::from_board_move(&board, (4, 5), (4, 6)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前兵进一");
        assert_eq!(notation.to_wxf(), "+P5+1");

        // 中兵进一（从原始位置）
        let notation = MoveNotation::from_board_move(&board, (4, 3), (4, 4)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Middle));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "中兵进一");
        assert_eq!(notation.to_wxf(), "-P5+1");

        // 后兵进一（从原始位置）
        let notation = MoveNotation::from_board_move(&board, (4, 1), (4, 2)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后兵进一");
        assert_eq!(notation.to_wxf(), ".P5+1");
    }

    #[test]
    fn test_qualifier_five_red_pawns() {
        // 创建五个红兵同列的极端局面（全部过河，可以横移）
        let mut board = Board::new();
        board.clear();
        // New coords: row 0 = Red bottom
        board.set_fen(3, 0, 'K'); // 红帅（移到旁边）
        board.set_fen(5, 9, 'k'); // 黑将（移到旁边）
        board.set_fen(4, 1, 'P'); // 一兵（最后面）
        board.set_fen(4, 2, 'P'); // 二兵
        board.set_fen(4, 3, 'P'); // 三兵
        board.set_fen(4, 4, 'P'); // 四兵
        board.set_fen(4, 5, 'P'); // 五兵（最前面）

        // 最前面的兵（五兵）= 一兵（数字1），横移测试
        let notation = MoveNotation::from_board_move(&board, (4, 5), (3, 5)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Number(1)));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "一兵平六");
        assert_eq!(notation.to_wxf(), "1P5.6");

        // 第二个兵 = 二兵，横移测试
        let notation = MoveNotation::from_board_move(&board, (4, 4), (5, 4)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Number(2)));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "二兵平四");
        assert_eq!(notation.to_wxf(), "2P5.4");

        // 中间的兵 = 三兵，横移测试
        let notation = MoveNotation::from_board_move(&board, (4, 3), (3, 3)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Number(3)));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "三兵平六");
        assert_eq!(notation.to_wxf(), "3P5.6");

        // 第四个兵 = 四兵，横移测试
        let notation = MoveNotation::from_board_move(&board, (4, 2), (5, 2)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Number(4)));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "四兵平四");
        assert_eq!(notation.to_wxf(), "4P5.4");

        // 最后面的兵 = 五兵，横移测试（避免吃到自己的兵）
        let notation = MoveNotation::from_board_move(&board, (4, 1), (3, 1)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Number(5)));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "五兵平六");
        assert_eq!(notation.to_wxf(), "5P5.6");
    }

    #[test]
    fn test_qualifier_two_black_rooks() {
        // 创建两辆黑车同列的局面
        let mut board = Board::new();
        board.clear();
        // New coords: row 9 = Black top (Black's back rank)
        // Black forward = decreasing row (towards Red's side)
        // Lower row = front (closer to Red), higher row = back (closer to Black's back)
        board.set_fen(4, 1, 'r'); // 黑方前车（row=1 靠近红方 = front）
        board.set_fen(4, 3, 'r'); // 黑方后车（row=3 靠近己方 = back）
        board.set_fen(3, 0, 'K'); // 红帅 (moved to col 3 to avoid capture)
        board.set_fen(5, 9, 'k'); // 黑将 (moved to col 5)

        // 黑方前车进一（row减少=前进 for Black）
        let result = MoveNotation::from_board_move(&board, (4, 1), (4, 0));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        // Black uses full-width Arabic numerals, rook = 车
        // With qualifier: 前/后 + piece name + direction + distance (no file number)
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前车进１");
        // WXF uses uppercase letters
        assert_eq!(notation.to_wxf(), "+R5+1");

        // 黑方后车进一
        let result = MoveNotation::from_board_move(&board, (4, 3), (4, 2));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后车进１");
        assert_eq!(notation.to_wxf(), ".R5+1");
    }

    #[test]
    fn test_qualifier_three_black_pawns() {
        // 创建三个黑卒同列的局面（每个卒之间有空位）
        // New coords: row 9 = Black top (back rank), Black forward = decreasing row
        // Lower row = front (closer to Red), higher row = back (closer to Black's back)
        let mut board = Board::new();
        board.clear();
        board.set_fen(4, 0, 'K'); // 红帅
        board.set_fen(4, 9, 'k'); // 黑将
        board.set_fen(4, 2, 'p'); // 黑方前卒（row=2 靠近红方 = front）
        board.set_fen(4, 4, 'p'); // 黑方中卒
        board.set_fen(4, 6, 'p'); // 黑方后卒（row=6 靠近己方 = back）

        // 黑方前卒进一（row减少=前进 for Black）
        let result = MoveNotation::from_board_move(&board, (4, 2), (4, 1));
        assert!(result.is_ok(), "前卒进一失败: {:?}", result);
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前卒进１");
        assert_eq!(notation.to_wxf(), "+P5+1");

        // 黑方中卒进一
        let result = MoveNotation::from_board_move(&board, (4, 4), (4, 3));
        assert!(result.is_ok(), "中卒进一失败: {:?}", result);
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Middle));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "中卒进１");
        assert_eq!(notation.to_wxf(), "-P5+1");

        // 黑方后卒进一
        let result = MoveNotation::from_board_move(&board, (4, 6), (4, 5));
        assert!(result.is_ok(), "后卒进一失败: {:?}", result);
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后卒进１");
        assert_eq!(notation.to_wxf(), ".P5+1");
    }

    #[test]
    fn test_qualifier_two_red_cannons() {
        // 创建两个红炮同列的局面
        let mut board = Board::new();
        board.clear();
        // New coords: Red forward = increasing row
        board.set_fen(4, 1, 'C'); // 后炮
        board.set_fen(4, 3, 'C'); // 前炮
        board.set_fen(4, 9, 'k'); // 黑将
        board.set_fen(4, 0, 'K'); // 红帅

        // 前炮平六
        let result = MoveNotation::from_board_move(&board, (4, 3), (3, 3));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前炮平六");
        assert_eq!(notation.to_wxf(), "+C5.6");

        // 后炮平四
        let result = MoveNotation::from_board_move(&board, (4, 1), (5, 1));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后炮平四");
        assert_eq!(notation.to_wxf(), ".C5.4");
    }

    #[test]
    fn test_qualifier_two_red_knights() {
        // 创建两个红马同列的局面
        let mut board = Board::new();
        board.clear();
        // New coords: Red forward = increasing row
        board.set_fen(3, 1, 'N'); // 后马
        board.set_fen(3, 3, 'N'); // 前马
        board.set_fen(4, 9, 'k'); // 黑将
        board.set_fen(4, 0, 'K'); // 红帅

        // 前马进七（马走日）
        let result = MoveNotation::from_board_move(&board, (3, 3), (2, 5));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前马进七");
        assert_eq!(notation.to_wxf(), "+N6+7");

        // 后马进七
        let result = MoveNotation::from_board_move(&board, (3, 1), (2, 3));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后马进七");
        assert_eq!(notation.to_wxf(), ".N6+7");
    }

    #[test]
    fn test_no_qualifier_for_king_advisor_elephant() {
        // 将/帅、士/仕、象/相即使同列也没有限定词
        // 测试红方帅移动 - 单个帅，没有限定词
        let mut board = Board::new();
        board.clear();
        board.set_fen(4, 0, 'K'); // 红帅
        board.set_fen(3, 9, 'k'); // 黑将（移到旁边避免飞将）

        // 红帅移动
        let notation = MoveNotation::from_board_move(&board, (4, 0), (4, 1)).unwrap();
        assert_eq!(notation.qualifier, None);
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "帅五进一");

        // 测试双士（士也没有限定词）
        board.set_fen(3, 0, 'a'); // 红方仕
        board.set_fen(5, 2, 'a'); // 红方另一个仕（不同列）
        board.set_fen(4, 9, 'K'); // 黑将

        // 左仕移动 - 仕没有限定词
        let notation = MoveNotation::from_board_move(&board, (3, 0), (4, 1)).unwrap();
        assert_eq!(notation.qualifier, None);

        // 测试双象（象也没有限定词）
        board.clear();
        board.set_fen(4, 0, 'k'); // 红帅
        board.set_fen(4, 9, 'K'); // 黑将
        board.set_fen(2, 0, 'b'); // 红方相
        board.set_fen(6, 2, 'b'); // 红方另一个相

        // 左相移动 - 象没有限定词
        let notation = MoveNotation::from_board_move(&board, (2, 0), (4, 2)).unwrap();
        assert_eq!(notation.qualifier, None);
    }

    #[test]
    fn test_qualifier_traditional_chinese() {
        // 测试繁体中文限定词 - Red rooks
        let mut board = Board::new();
        board.clear();
        // New coords: row 0 = Red bottom, Red forward = increasing row
        // Lower row = back (closer to Red's bottom), higher row = front (closer to Black)
        board.set_fen(4, 1, 'R'); // 后车 (back rook)
        board.set_fen(4, 3, 'R'); // 前车 (front rook)
        board.set_fen(4, 9, 'k'); // 黑将
        board.set_fen(4, 0, 'K'); // 红帅

        // 前车进一（繁体）- front rook moves forward (increasing row)
        let result = MoveNotation::from_board_move(&board, (4, 3), (4, 4));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        // Red rook in traditional = 車
        assert_eq!(notation.to_chinese(ChineseLocale::Traditional), "前車進一");

        // 后车进一（繁体）- back rook moves forward
        let result = MoveNotation::from_board_move(&board, (4, 1), (4, 2));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Traditional), "後車進一");
    }

    // =========================================================================
    // ICCS 坐标走法测试
    // =========================================================================

    #[test]
    fn test_parse_iccs_move_basic() {
        // ICCS "h2e2" → internal row 2 (Red cannon position)
        let mv = parse_iccs_move("h2e2");
        assert_eq!(mv.from_col, 7);
        assert_eq!(mv.from_row, 2);
        assert_eq!(mv.to_col, 4);
        assert_eq!(mv.to_row, 2);
    }

    #[test]
    fn test_parse_iccs_move_with_hyphen() {
        // ICCS "h2-e2" → internal row 2 (Red cannon position)
        let mv = parse_iccs_move("h2-e2");
        assert_eq!(mv.from_col, 7);
        assert_eq!(mv.from_row, 2);
        assert_eq!(mv.to_col, 4);
        assert_eq!(mv.to_row, 2);
    }

    #[test]
    fn test_parse_iccs_move_edges() {
        // a0 = ICCS col 0, row 0 (Red bottom-left) → internal row 0
        let mv = parse_iccs_move("a0a1");
        assert_eq!(mv.from_col, 0);
        assert_eq!(mv.from_row, 0);
        assert_eq!(mv.to_col, 0);
        assert_eq!(mv.to_row, 1);
    }

    #[test]
    fn test_parse_iccs_move_invalid() {
        assert!(try_parse_iccs_move("abc").is_err()); // Too short
        assert!(try_parse_iccs_move("j0e2").is_err()); // Invalid column
        assert!(try_parse_iccs_move("hxe2").is_err()); // Invalid row
    }

    #[test]
    fn test_format_iccs_move() {
        // Internal (7,2) → (4,2) → ICCS "h2e2" (Red cannon position)
        let mv = crate::move_gen::Move::new(7, 2, 4, 2);
        assert_eq!(format_iccs_move(&mv), "h2e2");
    }

    #[test]
    fn test_format_iccs_move_edges() {
        // Internal (0,0) → (0,1) → ICCS "a0a1" (Red bottom-left)
        let mv = crate::move_gen::Move::new(0, 0, 0, 1);
        assert_eq!(format_iccs_move(&mv), "a0a1");
    }

    #[test]
    fn test_iccs_col_conversion() {
        assert_eq!(index_to_iccs_col(0).unwrap(), 'a');
        assert_eq!(index_to_iccs_col(4).unwrap(), 'e');
        assert_eq!(index_to_iccs_col(8).unwrap(), 'i');
        assert!(index_to_iccs_col(9).is_err());
    }

    #[test]
    fn test_iccs_roundtrip() {
        // Verify parse and format are inverse operations
        let original = "h2e2";
        let mv = parse_iccs_move(original);
        assert_eq!(format_iccs_move(&mv), original);

        let original2 = "a0i9";
        let mv2 = parse_iccs_move(original2);
        assert_eq!(format_iccs_move(&mv2), original2);
    }

    #[test]
    fn test_move_notation_to_iccs() {
        let mut board = Board::new();
        board.initial_position();

        // Red cannon move: 炮二平五 from (7,2) to (4,2)
        let notation = MoveNotation::from_board_move(&board, (7, 2), (4, 2)).unwrap();
        let iccs = notation.to_iccs((7, 2), (4, 2));
        assert_eq!(iccs, "h2e2");
    }
}
