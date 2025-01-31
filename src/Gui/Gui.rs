use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, List, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::Frame;

use crate::GuiState;

// 渲染通用布局和个人信息
pub fn render_common_layout(frame: &mut Frame, state: &GuiState) -> (Rect, Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(10), // 顶部标题区域
            Constraint::Min(10),        // 中间内容区域
            Constraint::Length(10),     // 底部控制台区域
        ])
        .split(frame.area());

    // 软件信息
    let personal_info = Paragraph::new("https://Inteside.github.io")
        .wrap(Wrap { trim: true })
        .style(Style::new().yellow())
        .block(
            Block::new()
                .title("UTermux")
                .title_style(Style::new().white().bold())
                .borders(Borders::ALL)
                .border_style(Style::new().red()),
        );
    frame.render_widget(personal_info, layout[0]);

    // 渲染控制台
    render_console(frame, state, layout[2]);

    (layout[1], layout[2]) // 返回中间内容区域和底部控制台区域的 Rect
}

// 渲染主界面
pub fn render_gui(frame: &mut Frame, state: &mut GuiState) {
    let content_area = render_common_layout(frame, state);

    // 功能列表
    let items = vec![
        "1.登录🔑️",
        "2.定时🕒️",
        "3.开始抢票🎫️",
        "4.设置🛠️",
        "5.退出🚪️",
    ]
    .iter()
    .enumerate()
    .map(|(i, item)| {
        if i == state.selected_index {
            Line::from(*item).style(Style::default().fg(Color::Black).bg(Color::White))
        } else {
            Line::from(*item)
        }
    })
    .collect::<Vec<Line>>();

    let function_list = List::new(items).block(
        Block::new()
            .title("功能列表")
            .title_style(Style::new().white().bold())
            .borders(Borders::ALL)
            .border_style(Style::new().red()),
    );
    frame.render_widget(function_list, content_area.0);
    render_console(frame, state, content_area.1);
}

// 主界面键盘输入
pub fn handle_key_input_main(state: &mut GuiState, key: KeyCode) {
    match key {
        KeyCode::Up => {
            if state.selected_index > 0 {
                state.selected_index -= 1;
            }
        }
        KeyCode::Down => {
            if state.selected_index < 4 {
                // 最多有5个选项，最大索引为4
                state.selected_index += 1;
            }
        }
        _ => {}
    }
}

// 渲染控制台（统一处理所有页面的控制台显示）
pub fn render_console(frame: &mut Frame, state: &GuiState, area: Rect) {
    // 计算可显示的最大行数（减去边框和标题占用的2行）
    let max_visible_lines = (area.height as usize).saturating_sub(2);

    // 将控制台信息按行分割,只过滤中间的空行
    let console_lines: Vec<&str> = state
        .console_info
        .lines()
        .enumerate()
        .filter(|(i, line)| {
            // 保留最后一行,过滤中间的空行
            !line.is_empty() || *i == state.console_info.lines().count() - 1
        })
        .map(|(_, line)| line)
        .collect();

    let total_lines = console_lines.len();

    // 根据滚动位置计算显示范围
    let start_index = state.console_scroll;
    let end_index = (start_index + max_visible_lines).min(total_lines);

    // 只显示当前滚动位置对应的行
    let visible_text = console_lines[start_index..end_index].join("\n");

    // 渲染控制台文本
    let console = Paragraph::new(visible_text)
        .block(Block::default().title("Info").borders(Borders::ALL))
        .style(Style::new().fg(Color::Yellow));

    frame.render_widget(console, area);

    // 只有当总行数超过可视区域时才显示滚动条
    if total_lines > max_visible_lines {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let scrollbar_height = ((max_visible_lines as f64) / (total_lines as f64) * 100.0) as u16;
        let mut scrollbar_state = ScrollbarState::new(total_lines)
            .position(state.console_scroll)
            .viewport_content_length(scrollbar_height as usize);

        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}
