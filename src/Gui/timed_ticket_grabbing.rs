use crate::Gui::Gui::render_common_layout;
use crate::GuiState;
use chrono::{Datelike, NaiveDate};
use crossterm::event::KeyCode;
use dirs;
use ratatui::prelude::*;
use ratatui::widgets::{
    calendar::{CalendarEventStore, Monthly},
    Block, Borders, Padding, Paragraph,
};
use ratatui::Frame;
use std::fs::{self, OpenOptions};
use std::io::Write;
use time::{Date, Month, OffsetDateTime};

// 添加日期相关的状态
pub struct TimedGrabbingState {
    pub selected_date: Option<NaiveDate>,
    pub input_buffer: String,
    pub is_editing: bool,
    pub current_year: i32,
    pub current_month: Month,
    pub cursor_date: Date,
    pub selected_time: Option<(u8, u8, u8)>, // 保存选中的时分秒
    pub time_cursor: usize,                  // 0=时, 1=分, 2=秒
    pub is_time_focused: bool,               // 添加此字段表示当前是否在时间选择器上
}

impl Default for TimedGrabbingState {
    fn default() -> Self {
        let now = OffsetDateTime::now_local().unwrap();
        let saved_date = read_saved_date();
        let saved_time = read_saved_time().or_else(|| {
            // 如果没有保存的时间，使用当前系统时间
            Some((now.hour() as u8, now.minute() as u8, now.second() as u8))
        });

        Self {
            selected_date: saved_date,
            input_buffer: String::new(),
            is_editing: false,
            current_year: now.year(),
            current_month: now.month(),
            cursor_date: now.date(),
            selected_time: saved_time,
            time_cursor: 0,
            is_time_focused: false,
        }
    }
}

// 保存日期和时间到文件
pub fn save_date(date: &NaiveDate, time: Option<(u8, u8, u8)>) -> std::io::Result<()> {
    // 确保目录存在
    let config_dir = dirs::config_dir();
    let app_config_dir = config_dir.unwrap().join("UTermux");
    if let Some(parent) = app_config_dir.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(app_config_dir.join("saved_date"))?;

    let datetime_str = match time {
        Some((h, m, s)) => format!("{} {:02}:{:02}:{:02}", date.format("%Y-%m-%d"), h, m, s),
        None => date.format("%Y-%m-%d").to_string(),
    };
    file.write_all(datetime_str.as_bytes())?;
    Ok(())
}

// 从文件读取保存的日期
pub fn read_saved_date() -> Option<NaiveDate> {
    let date_path = dirs::config_dir()
        .unwrap()
        .join("UTermux")
        .join("saved_date");
    let contents = fs::read_to_string(date_path).unwrap_or_default();
    // 分割字符串，只取日期部分
    let date_str = contents.split_whitespace().next()?;
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()
}

// 从文件读取保存的时间
pub fn read_saved_time() -> Option<(u8, u8, u8)> {
    let date_path = dirs::config_dir()
        .unwrap()
        .join("UTermux")
        .join("saved_date");
    fs::read_to_string(date_path).ok().and_then(|contents| {
        let parts: Vec<&str> = contents.trim().split(' ').collect();
        if parts.len() > 1 {
            let time_parts: Vec<&str> = parts[1].split(':').collect();
            if time_parts.len() == 3 {
                Some((
                    time_parts[0].parse().unwrap_or(0),
                    time_parts[1].parse().unwrap_or(0),
                    time_parts[2].parse().unwrap_or(0),
                ))
            } else {
                None
            }
        } else {
            None
        }
    })
}

pub fn timed_ticket_grabbing_render(
    frame: &mut Frame,
    state: &mut GuiState,
    timed_state: &mut TimedGrabbingState,
) {
    let (content_area, _) = render_common_layout(frame, state);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(4), // 日期输入/显示区域
            Constraint::Min(10),   // 日历区域
            Constraint::Min(1),    // 其他内容区域
        ])
        .split(content_area);

    // 显示日期输入框或已选择的日期
    let date_text = if timed_state.is_editing {
        format!("{}_", timed_state.input_buffer)
    } else {
        match (timed_state.selected_date, timed_state.selected_time) {
            (Some(date), Some((h, m, s))) => {
                format!(
                    "预设日期时间: {} {:02}:{:02}:{:02}",
                    date.format("%Y-%m-%d"),
                    h,
                    m,
                    s
                )
            }
            (Some(date), None) => date.format("预设日期: %Y-%m-%d").to_string(),
            _ => "按Enter设置日期时间 (格式: YYYY-MM-DD) 或使用方向键选择".to_string(),
        }
    };

    let date_widget =
        Paragraph::new(date_text).block(Block::default().title("定时日期").borders(Borders::ALL));

    frame.render_widget(date_widget, layout[0]);

    // 当不在编辑模式时显示日历
    if !timed_state.is_editing {
        let calendar_area = layout[1];
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Ratio(1, 2), // 日历占用一半宽度
                Constraint::Ratio(1, 2), // 时间选择器占用一半宽度
            ])
            .split(calendar_area);

        draw_calendar(frame, split[0], timed_state);
        render_time_selector(frame, split[1], timed_state);
    }
}

// 日历渲染函数
fn draw_calendar(frame: &mut Frame, area: Rect, timed_state: &TimedGrabbingState) {
    let calendar_area = area;
    let mut list = CalendarEventStore::default();

    // 显示光标位置
    list.add(
        timed_state.cursor_date,
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Blue),
    );

    // 显示已选择的日期（如果有）
    if let Some(selected_date) = timed_state.selected_date {
        let selected_time = Date::from_calendar_date(
            selected_date.year_ce().1 as i32,
            Month::try_from(selected_date.month() as u8).unwrap(),
            selected_date.day() as u8,
        )
        .unwrap();

        list.add(
            selected_time,
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Green),
        );
    }

    let calendar = Monthly::new(
        Date::from_calendar_date(timed_state.current_year, timed_state.current_month, 1).unwrap(),
        &list,
    )
    .show_month_header(Style::default())
    .show_surrounding(Style::default().bg(Color::DarkGray))
    .show_weekdays_header(Style::default());

    let calendar_block = Block::default()
        .title("日历选择")
        .borders(Borders::ALL)
        .border_style(if !timed_state.is_time_focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        })
        .padding(Padding::new(1, 1, 0, 0));

    let inner_area = calendar_block.inner(calendar_area);
    frame.render_widget(calendar_block, calendar_area);

    // 计算水平居中的位置
    let calendar_width = 21; // 日历的标准宽度
    let x_offset = (inner_area.width.saturating_sub(calendar_width)) / 2;
    let centered_area = Rect::new(
        inner_area.x + x_offset,
        inner_area.y,
        calendar_width, // 限制宽度为日历标准宽度
        inner_area.height,
    );

    frame.render_widget(calendar, centered_area);
}

// 时分秒选择器渲染
fn render_time_selector(frame: &mut Frame, area: Rect, timed_state: &TimedGrabbingState) {
    let time_block = Block::default()
        .title("时间选择")
        .borders(Borders::ALL)
        .border_style(if timed_state.is_time_focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        })
        .padding(Padding::new(1, 1, 1, 1));

    let inner_area = time_block.inner(area);
    frame.render_widget(time_block, area);

    let (h, m, s) = timed_state.selected_time.unwrap_or((0, 0, 0));
    let time_text = vec![
        Line::from(vec![
            Span::styled(
                format!("{:02}", h),
                if timed_state.time_cursor == 0 {
                    Style::default().bg(Color::Blue)
                } else {
                    Style::default()
                },
            ),
            Span::raw(":"),
            Span::styled(
                format!("{:02}", m),
                if timed_state.time_cursor == 1 {
                    Style::default().bg(Color::Blue)
                } else {
                    Style::default()
                },
            ),
            Span::raw(":"),
            Span::styled(
                format!("{:02}", s),
                if timed_state.time_cursor == 2 {
                    Style::default().bg(Color::Blue)
                } else {
                    Style::default()
                },
            ),
        ]),
        Line::from(""),
        Line::from("↑/↓: 调整数值"),
        Line::from("←/→: 切换时分秒"),
        Line::from("Enter: 确认"),
    ];

    let time_widget = Paragraph::new(time_text).alignment(Alignment::Center);
    frame.render_widget(time_widget, inner_area);
}

pub fn handle_timed_input(
    state: &mut GuiState,
    timed_state: &mut TimedGrabbingState,
    key: KeyCode,
) {
    match key {
        KeyCode::Enter => {
            if !timed_state.is_editing {
                let selected_date = NaiveDate::from_ymd_opt(
                    timed_state.cursor_date.year(),
                    timed_state.cursor_date.month() as u32,
                    timed_state.cursor_date.day() as u32,
                )
                .unwrap();

                timed_state.selected_date = Some(selected_date);
                if let Err(e) = save_date(&selected_date, timed_state.selected_time) {
                    state.add_console_message(format!("保存日期时间失败: {}", e));
                } else {
                    let time_str = timed_state
                        .selected_time
                        .map(|(h, m, s)| format!(" {:02}:{:02}:{:02}", h, m, s))
                        .unwrap_or_default();
                    state.add_console_message(format!(
                        "已保存定时日期时间: {}{}",
                        selected_date.format("%Y-%m-%d"),
                        time_str
                    ));
                }
            }
        }
        KeyCode::Tab => {
            if !timed_state.is_editing {
                timed_state.is_time_focused = !timed_state.is_time_focused;
            }
        }
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
            if !timed_state.is_editing {
                if timed_state.is_time_focused {
                    // 时间选择模式
                    match key {
                        KeyCode::Up => {
                            if let Some((mut h, mut m, mut s)) = timed_state.selected_time {
                                match timed_state.time_cursor {
                                    0 => h = (h + 1) % 24,
                                    1 => m = (m + 1) % 60,
                                    2 => s = (s + 1) % 60,
                                    _ => {}
                                }
                                timed_state.selected_time = Some((h, m, s));
                            }
                        }
                        KeyCode::Down => {
                            if let Some((mut h, mut m, mut s)) = timed_state.selected_time {
                                match timed_state.time_cursor {
                                    0 => h = if h == 0 { 23 } else { h - 1 },
                                    1 => m = if m == 0 { 59 } else { m - 1 },
                                    2 => s = if s == 0 { 59 } else { s - 1 },
                                    _ => {}
                                }
                                timed_state.selected_time = Some((h, m, s));
                            }
                        }
                        KeyCode::Left => {
                            timed_state.time_cursor = (timed_state.time_cursor + 2) % 3
                        }
                        KeyCode::Right => {
                            timed_state.time_cursor = (timed_state.time_cursor + 1) % 3
                        }
                        _ => {}
                    }
                } else {
                    // 日历选择模式
                    match key {
                        KeyCode::Up => {
                            if timed_state.cursor_date.day() > 1 {
                                timed_state.cursor_date = timed_state
                                    .cursor_date
                                    .previous_day()
                                    .unwrap_or(timed_state.cursor_date);
                            }
                        }
                        KeyCode::Down => {
                            let last_day = get_last_day_of_month(
                                timed_state.current_year,
                                timed_state.current_month,
                            );
                            if timed_state.cursor_date.day() < last_day {
                                timed_state.cursor_date = timed_state
                                    .cursor_date
                                    .next_day()
                                    .unwrap_or(timed_state.cursor_date);
                            }
                        }
                        KeyCode::Left => {
                            if timed_state.cursor_date.day() > 1 {
                                timed_state.cursor_date = timed_state
                                    .cursor_date
                                    .previous_day()
                                    .unwrap_or(timed_state.cursor_date);
                            }
                        }
                        KeyCode::Right => {
                            let last_day = get_last_day_of_month(
                                timed_state.current_year,
                                timed_state.current_month,
                            );
                            if timed_state.cursor_date.day() < last_day {
                                timed_state.cursor_date = timed_state
                                    .cursor_date
                                    .next_day()
                                    .unwrap_or(timed_state.cursor_date);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        KeyCode::PageUp => {
            if !timed_state.is_editing {
                // 上一个月
                let new_month = timed_state.current_month.previous();
                let new_year = if new_month == Month::December {
                    timed_state.current_year - 1
                } else {
                    timed_state.current_year
                };
                timed_state.current_month = new_month;
                timed_state.current_year = new_year;

                // 更新光标日期
                timed_state.cursor_date =
                    Date::from_calendar_date(new_year, new_month, timed_state.cursor_date.day())
                        .unwrap_or(timed_state.cursor_date);
            }
        }
        KeyCode::PageDown => {
            if !timed_state.is_editing {
                // 下一个月
                let new_month = timed_state.current_month.next();
                let new_year = if new_month == Month::January {
                    timed_state.current_year + 1
                } else {
                    timed_state.current_year
                };
                timed_state.current_month = new_month;
                timed_state.current_year = new_year;

                // 更新光标日期
                timed_state.cursor_date =
                    Date::from_calendar_date(new_year, new_month, timed_state.cursor_date.day())
                        .unwrap_or(timed_state.cursor_date);
            }
        }
        KeyCode::Esc => {
            if timed_state.is_editing {
                timed_state.is_editing = false;
                timed_state.input_buffer.clear();
            } else {
                state.current_page = crate::function_list::Main;
            }
        }
        KeyCode::Char(c) => {
            if timed_state.is_editing {
                timed_state.input_buffer.push(c);
            }
        }
        KeyCode::Backspace => {
            if timed_state.is_editing {
                timed_state.input_buffer.pop();
            }
        }
        _ => {}
    }
}

// 添加一个辅助函数来获取月份的最后一天
fn get_last_day_of_month(year: i32, month: Month) -> u8 {
    (if month == Month::December {
        Date::from_calendar_date(year + 1, Month::January, 1)
    } else {
        Date::from_calendar_date(year, month.next(), 1)
    }
    .unwrap()
    .previous_day()
    .unwrap()
    .day()) as u8
}

#[tokio::test]
async fn test_timed_ticket_grabbing() {
    let date = read_saved_date();
    println!("这是日期: {:?}", date);
    let time = read_saved_time();
    println!("这是时间: {:?}", time);
}
