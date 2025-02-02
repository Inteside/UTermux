use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::api::queryMobilePhone::query_mobile_phone;
use crate::Gui::Gui::render_common_layout;
use crate::GuiState;

pub fn login_render(frame: &mut Frame, state: &mut GuiState) {
    let (content_area, _) = render_common_layout(frame, state);

    // 创建主布局，包含所有元素
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(6), // AuthToken
            Constraint::Length(6), // UserAgent
            Constraint::Min(0),    // 剩余空间
        ])
        .split(content_area);

    // authToken输入框
    let input = Paragraph::new(if state.active_input == 0 {
        state.input_buffer.as_str()
    } else {
        &state.auth_token
    })
    .wrap(Wrap::default())
    .block(
        Block::default()
            .title("AuthToken")
            .borders(Borders::ALL)
            .border_style(if state.active_input == 0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            }),
    );

    // UserAgent输入框
    let user_agent_input = Paragraph::new(if state.active_input == 1 {
        state.input_buffer.as_str()
    } else {
        &state.user_agent
    })
    .wrap(Wrap::default())
    .block(
        Block::default()
            .title("UserAgent")
            .borders(Borders::ALL)
            .border_style(if state.active_input == 1 {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            }),
    );

    frame.render_widget(input, main_layout[0]);
    frame.render_widget(user_agent_input, main_layout[1]);
}

pub async fn handle_login_input(state: &mut GuiState) {
    match state.last_key {
        KeyCode::Tab => {
            // 切换输入框
            state.active_input = (state.active_input + 1) % 2;
            // 保存当前输入框的内容
            if state.active_input == 0 {
                state.user_agent = state.input_buffer.clone();
            } else {
                state.auth_token = state.input_buffer.clone();
            }
            state.input_buffer.clear();
        }
        KeyCode::Enter => {
            if !state.input_buffer.is_empty() {
                if state.active_input == 0 {
                    state.auth_token = state.input_buffer.clone();
                } else {
                    state.user_agent = state.input_buffer.clone();
                }
                // 无论在哪个输入框，都发送请求
                match query_mobile_phone(&state.auth_token, &state.user_agent).await {
                    Ok(response) => state.add_console_message(response),
                    Err(e) => state.add_console_message(format!("错误: {}", e)),
                }
                state.input_buffer.clear();
            }
        }
        _ => {}
    }
}
