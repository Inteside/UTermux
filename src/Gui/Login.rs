use chrono::format;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::api::queryMobilePhone::query_mobile_phone;
use crate::Gui::Gui::render_common_layout;
use crate::GuiState;

pub fn login_render(frame: &mut Frame, state: &mut GuiState) {
    let (content_area, _) = render_common_layout(frame, state);

    // 在content_area内创建登录页面布局
    let login_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(10),     // auth_token输入框
            Constraint::Percentage(80), // 剩余空间
        ])
        .split(content_area);

    // authToken输入框
    let input = Paragraph::new(state.input_buffer.as_str())
        .block(Block::default().title("AuthToken").borders(Borders::ALL))
        .wrap(Wrap::default());

    frame.render_widget(input, login_layout[0]);
}

pub async fn handle_login_input(state: &mut GuiState) {
    if !state.input_buffer.is_empty() {
        state.auth_token = state.input_buffer.clone();
        state.input_buffer.clear();
        match query_mobile_phone(&state.auth_token).await {
            Ok(response) => state.add_console_message(response),
            Err(e) => state.add_console_message(format!("错误: {}", e)),
        }
    }
}
