/// 走法中间表示模块
/// 实现中国象棋走法的中间表示，支持简体中文、繁体中文和紧凑格式
use crate::board::Board;
use crate::pieces::{Color, PieceType};

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
    Compact, // 紧凑格式
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
    pub piece_color: Color,           // 棋子颜色
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

        let color = piece_color.unwrap_or(Color::Red);

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
        // 关键：翻转后统一使用红方视角处理方向
        let (src_col_for_dir, src_row_for_dir, dst_col_for_dir, dst_row_for_dir) =
            if color == Color::Black {
                // 黑方走子：反转棋盘一次，然后用红方逻辑处理方向
                (
                    flip_coordinate(from_col, from_row).0,
                    flip_coordinate(from_col, from_row).1,
                    flip_coordinate(to_col, to_row).0,
                    flip_coordinate(to_col, to_row).1,
                )
            } else {
                // 红方走子：不反转
                (from_col, from_row, to_col, to_row)
            };

        // 计算方向：翻转后统一使用红方规则
        let direction = calculate_direction(
            Color::Red,
            src_col_for_dir,
            src_row_for_dir,
            dst_col_for_dir,
            dst_row_for_dir,
        );

        // 计算距离：使用翻转后的坐标
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

    /// 转换为紧凑格式
    pub fn to_compact(&self) -> String {
        let mut result = String::new();

        // 添加限定词
        if let Some(qualifier) = self.qualifier {
            match qualifier {
                Qualifier::Front => result.push('f'),
                Qualifier::Middle => result.push('m'),
                Qualifier::Back => result.push('b'),
                Qualifier::Number(n) => result.push_str(&n.to_string()),
            };
        }

        // 添加棋子字母
        result.push(piece_to_compact_letter(self.piece_type, self.piece_color));

        // 添加列
        result.push_str(&self.column.to_string());

        // 添加方向符号
        result.push(match self.direction {
            Direction::Forward => '+',
            Direction::Backward => '-',
            Direction::Horizontal => '=',
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

    /// 转换为ICCS格式
    pub fn to_iccs(&self, from: (usize, usize), to: (usize, usize)) -> String {
        let (from_col, from_row) = from;
        let (to_col, to_row) = to;

        // ICCS格式：字母表示列(a-i)，数字表示行(0-9)
        format!(
            "{}{}{}{}",
            (b'a' + from_col as u8) as char,
            from_row,
            (b'a' + to_col as u8) as char,
            to_row
        )
    }
}

/// 翻转坐标（棋盘翻转）
fn flip_coordinate(col: usize, row: usize) -> (usize, usize) {
    (8 - col, 9 - row)
}

/// 计算方向
fn calculate_direction(
    color: Color,
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
            Color::Red => dst_row > src_row,   // 红方：向上为进
            Color::Black => dst_row < src_row, // 黑方：向下为进
            Color::Any => dst_row > src_row,   // 默认红方
        };
        if is_forward {
            Direction::Forward
        } else {
            Direction::Backward
        }
    } else {
        // 列和行都不同：斜线移动（马、士、象）
        let is_forward = match color {
            Color::Red => dst_row > src_row,   // 红方：向上为进
            Color::Black => dst_row < src_row, // 黑方：向下为进
            Color::Any => dst_row > src_row,   // 默认红方
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
    color: Color,
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
    // 红方：row 从大到小（靠近对方底线=前，靠近己方底线=后）
    // 黑方：row 从小到大（靠近对方底线=前，靠近己方底线=后）
    same_pieces.sort_by(|r1, r2| {
        if color == Color::Red {
            // 红方：从大到小（前->后）
            r2.cmp(r1)
        } else {
            // 黑方：从小到大（前->后）
            r1.cmp(r2)
        }
    });

    // 添加当前棋子
    let mut all_pieces = same_pieces;
    all_pieces.push(row);

    // 重新排序
    all_pieces.sort_by(|r1, r2| {
        if color == Color::Red {
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
fn get_piece_name(piece_type: PieceType, color: Color, locale: ChineseLocale) -> String {
    match locale {
        ChineseLocale::Simplified => simplified_piece_name(piece_type, color),
        ChineseLocale::Traditional => traditional_piece_name(piece_type, color),
    }
}

/// 获取简体中文棋子名称
fn simplified_piece_name(piece_type: PieceType, color: Color) -> String {
    match (piece_type, color) {
        // 红方：帅、仕、相、马、车、炮、兵
        (PieceType::King, Color::Red) => "帅".to_string(),
        (PieceType::Advisor, Color::Red) => "仕".to_string(),
        (PieceType::Elephant, Color::Red) => "相".to_string(),
        (PieceType::Knight, Color::Red) => "马".to_string(),
        (PieceType::Rook, Color::Red) => "车".to_string(),
        (PieceType::Cannon, Color::Red) => "炮".to_string(),
        (PieceType::Pawn, Color::Red) => "兵".to_string(),

        // 黑方：将、士、象、马、车、炮、卒
        (PieceType::King, Color::Black) => "将".to_string(),
        (PieceType::Advisor, Color::Black) => "士".to_string(),
        (PieceType::Elephant, Color::Black) => "象".to_string(),
        (PieceType::Knight, Color::Black) => "马".to_string(),
        (PieceType::Rook, Color::Black) => "车".to_string(),
        (PieceType::Cannon, Color::Black) => "炮".to_string(),
        (PieceType::Pawn, Color::Black) => "卒".to_string(),

        _ => "?".to_string(),
    }
}

/// 获取繁体中文棋子名称
fn traditional_piece_name(piece_type: PieceType, color: Color) -> String {
    match (piece_type, color) {
        // 红方：帥、仕、相、馬、車、砲、兵
        (PieceType::King, Color::Red) => "帥".to_string(),
        (PieceType::Advisor, Color::Red) => "仕".to_string(),
        (PieceType::Elephant, Color::Red) => "相".to_string(),
        (PieceType::Knight, Color::Red) => "馬".to_string(),
        (PieceType::Rook, Color::Red) => "車".to_string(),
        (PieceType::Cannon, Color::Red) => "砲".to_string(),
        (PieceType::Pawn, Color::Red) => "兵".to_string(),

        // 黑方：將、士、象、傌、俥、砲、卒
        (PieceType::King, Color::Black) => "將".to_string(),
        (PieceType::Advisor, Color::Black) => "士".to_string(),
        (PieceType::Elephant, Color::Black) => "象".to_string(),
        (PieceType::Knight, Color::Black) => "傌".to_string(),
        (PieceType::Rook, Color::Black) => "俥".to_string(),
        (PieceType::Cannon, Color::Black) => "砲".to_string(),
        (PieceType::Pawn, Color::Black) => "卒".to_string(),

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
fn format_number(number: u8, color: Color, locale: ChineseLocale) -> String {
    if color == Color::Red {
        // 红方：中文数字
        match locale {
            ChineseLocale::Simplified => simplified_number(number),
            ChineseLocale::Traditional => traditional_number(number),
        }
    } else {
        // 黑方：全角数字
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

/// 棋子转换为紧凑格式字母
fn piece_to_compact_letter(piece_type: PieceType, color: Color) -> char {
    let base_char = match piece_type {
        PieceType::King => 'K',
        PieceType::Advisor => 'A',
        PieceType::Elephant => 'B',
        PieceType::Knight => 'N',
        PieceType::Rook => 'R',
        PieceType::Cannon => 'C',
        PieceType::Pawn => 'P',
    };

    match color {
        Color::Red => base_char,                        // 红方：大写
        Color::Black => base_char.to_ascii_lowercase(), // 黑方：小写
        Color::Any => base_char,
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
        // 红方方向
        assert_eq!(
            calculate_direction(Color::Red, 0, 0, 0, 1),
            Direction::Forward
        );
        assert_eq!(
            calculate_direction(Color::Red, 0, 1, 0, 0),
            Direction::Backward
        );
        assert_eq!(
            calculate_direction(Color::Red, 0, 0, 1, 0),
            Direction::Horizontal
        );

        // 黑方方向
        assert_eq!(
            calculate_direction(Color::Black, 0, 9, 0, 8),
            Direction::Forward
        );
        assert_eq!(
            calculate_direction(Color::Black, 0, 8, 0, 9),
            Direction::Backward
        );
        assert_eq!(
            calculate_direction(Color::Black, 0, 9, 1, 9),
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
        assert_eq!(simplified_piece_name(PieceType::King, Color::Red), "帅");
        assert_eq!(simplified_piece_name(PieceType::King, Color::Black), "将");
        assert_eq!(simplified_piece_name(PieceType::Rook, Color::Red), "车");
        assert_eq!(simplified_piece_name(PieceType::Rook, Color::Black), "车");
        assert_eq!(simplified_piece_name(PieceType::Cannon, Color::Red), "炮");
        assert_eq!(simplified_piece_name(PieceType::Cannon, Color::Black), "炮");
        assert_eq!(simplified_piece_name(PieceType::Pawn, Color::Red), "兵");
        assert_eq!(simplified_piece_name(PieceType::Pawn, Color::Black), "卒");

        // 测试繁体中文棋子名称
        assert_eq!(traditional_piece_name(PieceType::King, Color::Red), "帥");
        assert_eq!(traditional_piece_name(PieceType::King, Color::Black), "將");
        assert_eq!(traditional_piece_name(PieceType::Rook, Color::Red), "車");
        assert_eq!(traditional_piece_name(PieceType::Rook, Color::Black), "俥");
        assert_eq!(traditional_piece_name(PieceType::Cannon, Color::Red), "砲");
        assert_eq!(
            traditional_piece_name(PieceType::Cannon, Color::Black),
            "砲"
        );
    }

    #[test]
    fn test_compact_piece_letter() {
        assert_eq!(piece_to_compact_letter(PieceType::King, Color::Red), 'K');
        assert_eq!(piece_to_compact_letter(PieceType::King, Color::Black), 'k');
        assert_eq!(piece_to_compact_letter(PieceType::Rook, Color::Red), 'R');
        assert_eq!(piece_to_compact_letter(PieceType::Rook, Color::Black), 'r');
        assert_eq!(piece_to_compact_letter(PieceType::Cannon, Color::Red), 'C');
        assert_eq!(
            piece_to_compact_letter(PieceType::Cannon, Color::Black),
            'c'
        );
    }

    #[test]
    fn test_move_notation_creation() {
        let board = Board::new();

        // 测试红方车九进一
        let result = MoveNotation::from_board_move(&board, (0, 0), (0, 1));
        assert!(result.is_ok());
        let notation = result.unwrap();

        assert_eq!(notation.piece_type, PieceType::Rook);
        assert_eq!(notation.piece_color, Color::Red);
        assert_eq!(notation.column, 9); // 九路（最左边）
        assert_eq!(notation.direction, Direction::Forward);
        assert_eq!(notation.distance, 1);

        // 转换为中文
        let chinese = notation.to_chinese(ChineseLocale::Simplified);
        assert_eq!(chinese, "车九进一");

        // 转换为繁体中文
        let traditional = notation.to_chinese(ChineseLocale::Traditional);
        assert_eq!(traditional, "車九進一");

        // 转换为紧凑格式
        let compact = notation.to_compact();
        assert_eq!(compact, "R9+1");
    }

    #[test]
    fn test_black_move_notation() {
        let board = Board::new();

        // 测试黑方车9进1（从(0,9)到(0,8)）
        let result = MoveNotation::from_board_move(&board, (0, 9), (0, 8));
        assert!(result.is_ok());
        let notation = result.unwrap();

        assert_eq!(notation.piece_type, PieceType::Rook);
        assert_eq!(notation.piece_color, Color::Black);
        // 注意：经过翻转后，黑方车在红方视角下是第九路
        assert_eq!(notation.column, 9);
        assert_eq!(notation.direction, Direction::Forward);
        assert_eq!(notation.distance, 1);

        // 转换为中文（黑方使用全角数字）
        let chinese = notation.to_chinese(ChineseLocale::Simplified);
        assert_eq!(chinese, "车９进１");

        // 转换为紧凑格式（小写表示黑方）
        let compact = notation.to_compact();
        assert_eq!(compact, "r9+1");
    }

    #[test]
    fn test_cannon_move() {
        let board = Board::new();

        // 测试红方炮二平五（从(7,2)到(4,2)）
        // 右边的炮在(7,2)，这是二路（从右向左数：9-7=2）
        let result = MoveNotation::from_board_move(&board, (7, 2), (4, 2));
        assert!(result.is_ok());
        let notation = result.unwrap();

        assert_eq!(notation.piece_type, PieceType::Cannon);
        assert_eq!(notation.piece_color, Color::Red);
        assert_eq!(notation.column, 2); // 二路（从右向左数）
        assert_eq!(notation.direction, Direction::Horizontal);
        assert_eq!(notation.distance, 5); // 平到五路

        // 转换为中文
        let chinese = notation.to_chinese(ChineseLocale::Simplified);
        assert_eq!(chinese, "炮二平五");

        // 转换为繁体中文
        let traditional = notation.to_chinese(ChineseLocale::Traditional);
        assert_eq!(traditional, "砲二平五");

        // 转换为紧凑格式
        let compact = notation.to_compact();
        assert_eq!(compact, "C2=5");
    }

    #[test]
    fn test_knight_move() {
        let board = Board::new();

        // 测试红方马八进七（从(1,0)到(2,2)）
        // 左边的马在(1,0)，这是八路（从右向左数：9-1=8）
        // 移动到(2,2)，这是七路（9-2=7）
        let result = MoveNotation::from_board_move(&board, (1, 0), (2, 2));
        assert!(result.is_ok());
        let notation = result.unwrap();

        assert_eq!(notation.piece_type, PieceType::Knight);
        assert_eq!(notation.piece_color, Color::Red);
        assert_eq!(notation.column, 8); // 八路（从右向左数）
        assert_eq!(notation.direction, Direction::Forward);
        // 马进到七路，距离应该是目标路数
        assert_eq!(notation.distance, 7);

        // 转换为中文（马显示目标路数）
        let chinese = notation.to_chinese(ChineseLocale::Simplified);
        assert_eq!(chinese, "马八进七");
    }

    #[test]
    fn test_invalid_moves() {
        let board = Board::new();

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
        board.set_fen(4, 1, 'r'); // 后车
        board.set_fen(4, 3, 'r'); // 前车
        board.set_fen(4, 9, 'K'); // 黑将
        board.set_fen(4, 0, 'k'); // 红帅

        // 前车进一
        let result = MoveNotation::from_board_move(&board, (4, 3), (4, 4));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前车进一");
        assert_eq!(notation.to_compact(), "fR5+1");

        // 后车进一
        let result = MoveNotation::from_board_move(&board, (4, 1), (4, 2));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后车进一");
        assert_eq!(notation.to_compact(), "bR5+1");
    }

    #[test]
    fn test_qualifier_three_red_pawns() {
        // 创建三个红兵同列的局面（每个兵之间有空位）
        let mut board = Board::new();
        board.clear();
        board.set_fen(4, 1, 'p'); // 后兵
        board.set_fen(4, 3, 'p'); // 中兵
        board.set_fen(4, 5, 'p'); // 前兵
        board.set_fen(4, 9, 'K'); // 黑将
        board.set_fen(4, 0, 'k'); // 红帅

        // 前兵进一
        let notation = MoveNotation::from_board_move(&board, (4, 5), (4, 6)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前兵进一");
        assert_eq!(notation.to_compact(), "fP5+1");

        // 中兵进一
        let notation = MoveNotation::from_board_move(&board, (4, 3), (4, 4)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Middle));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "中兵进一");
        assert_eq!(notation.to_compact(), "mP5+1");

        // 后兵进一
        let notation = MoveNotation::from_board_move(&board, (4, 1), (4, 2)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后兵进一");
        assert_eq!(notation.to_compact(), "bP5+1");
    }

    #[test]
    fn test_qualifier_five_red_pawns() {
        // 创建五个红兵同列的极端局面（全部过河，可以横移）
        let mut board = Board::new();
        board.clear();
        board.set_fen(3, 0, 'k'); // 红帅（移到旁边）
        board.set_fen(5, 9, 'K'); // 黑将（移到旁边）
        board.set_fen(4, 1, 'p'); // 一兵（最后面）
        board.set_fen(4, 2, 'p'); // 二兵
        board.set_fen(4, 3, 'p'); // 三兵
        board.set_fen(4, 4, 'p'); // 四兵
        board.set_fen(4, 5, 'p'); // 五兵（最前面）

        // 最前面的兵（五兵）= 一兵（数字1），横移测试
        let notation = MoveNotation::from_board_move(&board, (4, 5), (3, 5)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Number(1)));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "一兵平六");
        assert_eq!(notation.to_compact(), "1P5=6");

        // 第二个兵 = 二兵，横移测试
        let notation = MoveNotation::from_board_move(&board, (4, 4), (5, 4)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Number(2)));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "二兵平四");
        assert_eq!(notation.to_compact(), "2P5=4");

        // 中间的兵 = 三兵，横移测试
        let notation = MoveNotation::from_board_move(&board, (4, 3), (3, 3)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Number(3)));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "三兵平六");
        assert_eq!(notation.to_compact(), "3P5=6");

        // 第四个兵 = 四兵，横移测试
        let notation = MoveNotation::from_board_move(&board, (4, 2), (5, 2)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Number(4)));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "四兵平四");
        assert_eq!(notation.to_compact(), "4P5=4");

        // 最后面的兵 = 五兵，横移测试（避免吃到自己的兵）
        let notation = MoveNotation::from_board_move(&board, (4, 1), (3, 1)).unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Number(5)));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "五兵平六");
        assert_eq!(notation.to_compact(), "5P5=6");
    }

    #[test]
    fn test_qualifier_two_black_rooks() {
        // 创建两辆黑车同列的局面
        let mut board = Board::new();
        board.clear();
        // 黑方底线是row=9，row小的靠近红方（前），row大的靠近己方（后）
        board.set_fen(4, 6, 'R'); // 黑方前车（row=6 靠近红方）
        board.set_fen(4, 8, 'R'); // 黑方后车（row=8 靠近己方）
        board.set_fen(4, 0, 'K'); // 红帅
        board.set_fen(4, 9, 'k'); // 黑将

        // 黑方前车进一（row减少=前进）
        let result = MoveNotation::from_board_move(&board, (4, 6), (4, 5));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        // 黑方使用全角数字，紧凑格式使用小写字母
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前车进１");
        assert_eq!(notation.to_compact(), "fr5+1");

        // 黑方后车进一
        let result = MoveNotation::from_board_move(&board, (4, 8), (4, 7));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后车进１");
        assert_eq!(notation.to_compact(), "br5+1");
    }

    #[test]
    fn test_qualifier_three_black_pawns() {
        // 创建三个黑卒同列的局面（每个卒之间有空位）
        // 黑方底线是row=9，row小=靠近红方=前，row大=靠近己方=后
        let mut board = Board::new();
        board.clear();
        board.set_fen(4, 0, 'K'); // 红帅
        board.set_fen(4, 9, 'k'); // 黑将
        board.set_fen(4, 3, 'P'); // 黑方前卒（row=3 靠近红方）
        board.set_fen(4, 5, 'P'); // 黑方中卒
        board.set_fen(4, 7, 'P'); // 黑方后卒（row=7 靠近己方）

        // 黑方前卒进一（row减少=前进）
        let result = MoveNotation::from_board_move(&board, (4, 3), (4, 2));
        assert!(result.is_ok(), "前卒进一失败: {:?}", result);
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前卒进１");
        assert_eq!(notation.to_compact(), "fp5+1");

        // 黑方中卒进一
        let result = MoveNotation::from_board_move(&board, (4, 5), (4, 4));
        assert!(result.is_ok(), "中卒进一失败: {:?}", result);
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Middle));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "中卒进１");
        assert_eq!(notation.to_compact(), "mp5+1");

        // 黑方后卒进一
        let result = MoveNotation::from_board_move(&board, (4, 7), (4, 6));
        assert!(result.is_ok(), "后卒进一失败: {:?}", result);
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后卒进１");
        assert_eq!(notation.to_compact(), "bp5+1");
    }

    #[test]
    fn test_qualifier_two_red_cannons() {
        // 创建两个红炮同列的局面
        let mut board = Board::new();
        board.clear();
        board.set_fen(4, 1, 'c'); // 后炮
        board.set_fen(4, 3, 'c'); // 前炮
        board.set_fen(4, 9, 'K'); // 黑将
        board.set_fen(4, 0, 'k'); // 红帅

        // 前炮平六
        let result = MoveNotation::from_board_move(&board, (4, 3), (3, 3));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前炮平六");
        assert_eq!(notation.to_compact(), "fC5=6");

        // 后炮平四
        let result = MoveNotation::from_board_move(&board, (4, 1), (5, 1));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后炮平四");
        assert_eq!(notation.to_compact(), "bC5=4");
    }

    #[test]
    fn test_qualifier_two_red_knights() {
        // 创建两个红马同列的局面
        let mut board = Board::new();
        board.clear();
        board.set_fen(3, 1, 'n'); // 后马
        board.set_fen(3, 3, 'n'); // 前马
        board.set_fen(4, 9, 'K'); // 黑将
        board.set_fen(4, 0, 'k'); // 红帅

        // 前马进七（马走日）
        let result = MoveNotation::from_board_move(&board, (3, 3), (2, 5));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Front));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "前马进七");
        assert_eq!(notation.to_compact(), "fN6+7");

        // 后马进七
        let result = MoveNotation::from_board_move(&board, (3, 1), (2, 3));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.qualifier, Some(Qualifier::Back));
        assert_eq!(notation.to_chinese(ChineseLocale::Simplified), "后马进七");
        assert_eq!(notation.to_compact(), "bN6+7");
    }

    #[test]
    fn test_no_qualifier_for_king_advisor_elephant() {
        // 将/帅、士/仕、象/相即使同列也没有限定词
        // 测试红方帅移动 - 单个帅，没有限定词
        let mut board = Board::new();
        board.clear();
        board.set_fen(4, 0, 'k'); // 红帅
        board.set_fen(3, 9, 'K'); // 黑将（移到旁边避免飞将）

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
        // 测试繁体中文限定词
        let mut board = Board::new();
        board.clear();
        board.set_fen(4, 1, 'r'); // 后车
        board.set_fen(4, 3, 'r'); // 前车
        board.set_fen(4, 9, 'K'); // 黑将
        board.set_fen(4, 0, 'k'); // 红帅

        // 前车进一（繁体）
        let result = MoveNotation::from_board_move(&board, (4, 3), (4, 4));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.to_chinese(ChineseLocale::Traditional), "前車進一");

        // 后车进一（繁体）
        let result = MoveNotation::from_board_move(&board, (4, 1), (4, 2));
        assert!(result.is_ok());
        let notation = result.unwrap();
        assert_eq!(notation.to_chinese(ChineseLocale::Traditional), "後車進一");
    }
}
