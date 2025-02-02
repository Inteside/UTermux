use crate::api::info::get_info;
use crate::api::{queryMobilePhone::read_saved_token, receive::fetch_receive};
use crate::timed_ticket_grabbing_state;
use crate::Gui::Gui::render_common_layout;
use crate::GuiState;
use crossterm::event::KeyCode;
use once_cell::sync::Lazy;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List};
use ratatui::Frame;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Barrier;
use std::sync::Mutex;

// 修改数据结构以包含专区名称和对应的任务ID
static TICKET_IDS: Lazy<Mutex<Vec<(&'static str, Vec<i64>)>>> = Lazy::new(|| {
    Mutex::new(vec![
        ("置顶专区", vec![]),
        ("新人专区", vec![]),
        ("每周专区", vec![]),
        ("每日专区", vec![]),
    ])
});

static INFO_REQUESTED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct Button<'a> {
    label: Line<'a>,
    theme: Theme,
    state: State,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Normal,
    Selected,
    Active,
}

#[derive(Debug, Clone, Copy)]
struct Theme {
    text: Color,
    background: Color,
    highlight: Color,
    shadow: Color,
}

const BLUE: Theme = Theme {
    text: Color::Rgb(16, 24, 48),
    background: Color::Rgb(48, 72, 144),
    highlight: Color::Rgb(64, 96, 192),
    shadow: Color::Rgb(32, 48, 96),
};

const RED: Theme = Theme {
    text: Color::Rgb(48, 16, 16),
    background: Color::Rgb(144, 48, 48),
    highlight: Color::Rgb(192, 64, 64),
    shadow: Color::Rgb(96, 32, 32),
};

const GREEN: Theme = Theme {
    text: Color::Rgb(16, 48, 16),
    background: Color::Rgb(48, 144, 48),
    highlight: Color::Rgb(64, 192, 64),
    shadow: Color::Rgb(32, 96, 32),
};

impl<'a> Button<'a> {
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Button {
            label: label.into(),
            theme: BLUE,
            state: State::Normal,
        }
    }

    pub const fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub const fn state(mut self, state: State) -> Self {
        self.state = state;
        self
    }
}

impl<'a> Widget for Button<'a> {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (background, text, shadow, highlight) = self.colors();
        buf.set_style(area, Style::new().bg(background).fg(text));

        // render top line if there's enough space
        if area.height > 2 {
            buf.set_string(
                area.x,
                area.y,
                "▔".repeat(area.width as usize),
                Style::new().fg(highlight).bg(background),
            );
        }
        // render bottom line if there's enough space
        if area.height > 1 {
            buf.set_string(
                area.x,
                area.y + area.height - 1,
                "▁".repeat(area.width as usize),
                Style::new().fg(shadow).bg(background),
            );
        }
        // render label centered
        buf.set_line(
            area.x + (area.width.saturating_sub(self.label.width() as u16)) / 2,
            area.y + (area.height.saturating_sub(1)) / 2,
            &self.label,
            area.width,
        );
    }
}

impl Button<'_> {
    const fn colors(&self) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        match self.state {
            State::Normal => (theme.background, theme.text, theme.shadow, theme.highlight),
            State::Selected => (theme.highlight, theme.text, theme.shadow, theme.highlight),
            State::Active => (theme.background, theme.text, theme.highlight, theme.shadow),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameConfig {
    active: bool,
    communityId: String,
    red_pack_tasks: HashMap<String, Vec<String>>,
}

// 渲染抢票页面
pub fn start_ticket_grabbing_render(
    f: &mut Frame,
    gui_state: &mut GuiState,
    timed_state: &mut timed_ticket_grabbing_state,
) {
    // 只在第一次渲染时发送请求
    if !INFO_REQUESTED.load(Ordering::SeqCst) {
        INFO_REQUESTED.store(true, Ordering::SeqCst);
        let auth_token = gui_state.auth_token.clone();
        tokio::spawn(async move {
            if let Ok(info) = get_info(auth_token, "14".to_string()).await {
                // 解析返回的分类和ID信息
                let categories: Vec<(&str, Vec<i64>)> = info
                    .split(';')
                    .filter_map(|cat_info| {
                        let parts: Vec<&str> = cat_info.split(':').collect();
                        if parts.len() == 2 {
                            let ids: Vec<i64> = parts[1]
                                .split(',')
                                .filter_map(|id| id.parse().ok())
                                .collect();
                            Some((parts[0], ids))
                        } else {
                            None
                        }
                    })
                    .collect();

                let mut tickets = TICKET_IDS.lock().unwrap();
                // 更新专区对应的任务ID列表
                for (category, ids) in &categories {
                    if let Some(idx) = tickets.iter().position(|(t, _)| t == category) {
                        // 找到匹配的专区，更新其任务ID列表
                        if !ids.is_empty() {
                            tickets[idx].1 = ids.clone(); // 更新对应专区的任务ID列表
                        }
                    }
                }
            }
        });
    }

    let (content_area, _) = render_common_layout(f, gui_state);

    // 将内容区域分为上下两部分，上面是列表，下面是按钮
    let [main_area, button_area] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(content_area)
    else {
        return;
    };

    // 原有的左右列表布局现在放在 main_area 中
    let [left_area, right_area] = *Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_area)
    else {
        return;
    };

    // 左侧可选票种列表
    let available_tickets: Vec<&str> = TICKET_IDS
        .lock()
        .unwrap()
        .iter()
        .map(|(name, _)| *name)
        .collect();

    let left_items: Vec<Line> = available_tickets
        .iter()
        .enumerate()
        .filter(|(i, _)| !timed_state.selected_tickets.contains(i))
        .enumerate()
        .map(|(filtered_idx, (original_idx, item))| {
            if !timed_state.is_right_panel && filtered_idx == timed_state.selected_index {
                Line::from(*item).style(Style::default().fg(Color::Black).bg(Color::White))
            } else {
                Line::from(*item)
            }
        })
        .collect();

    // 右侧已选票种
    let right_items: Vec<Line> = timed_state
        .selected_tickets
        .iter()
        .enumerate()
        .map(|(i, &ticket_idx)| {
            let item = available_tickets[ticket_idx];
            if timed_state.is_right_panel && i == timed_state.selected_index {
                Line::from(item).style(Style::default().fg(Color::Black).bg(Color::White))
            } else {
                Line::from(item)
            }
        })
        .collect();

    let left_widget =
        List::new(left_items).block(Block::new().title("选择票种").borders(Borders::ALL));

    let right_widget =
        List::new(right_items).block(Block::default().title("已选票种").borders(Borders::ALL));

    f.render_widget(left_widget, left_area);
    f.render_widget(right_widget, right_area);

    // 渲染底部按钮
    let [start_area, clear_area] = *Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(button_area)
    else {
        return;
    };

    let start_button = Button::new("开始抢票🚀").theme(GREEN).state(
        if timed_state.is_button_mode && timed_state.button_focus == 0 {
            State::Selected
        } else {
            State::Normal
        },
    );

    let clear_button = Button::new("清空选择🗑️").theme(RED).state(
        if timed_state.is_button_mode && timed_state.button_focus == 1 {
            State::Selected
        } else {
            State::Normal
        },
    );

    f.render_widget(start_button, start_area);
    f.render_widget(clear_button, clear_area);
}

// 处理抢票页面的输入
pub fn handle_start_ticket_grabbing_input(
    gui_state: &mut GuiState,
    timed_state: &mut timed_ticket_grabbing_state,
    key: KeyCode,
) {
    match key {
        KeyCode::Esc => {
            if timed_state.is_button_mode {
                // 如果在按钮模式下，返回到票种选择模式
                timed_state.is_button_mode = false;
                timed_state.button_focus = 0; // 重置按钮焦点
            } else {
                // 只有在票种选择模式下，才返回主页面
                gui_state.current_page = crate::function_list::Main;
            }
        }
        KeyCode::Tab => {
            if !timed_state.is_button_mode {
                // 首次进入按钮模式时，选择"开始抢票"按钮
                timed_state.is_button_mode = true;
                timed_state.button_focus = 0;
            } else {
                // 已在按钮模式时，在两个按钮之间切换
                timed_state.button_focus = (timed_state.button_focus + 1) % 2;
            }
        }
        KeyCode::Up | KeyCode::Down => {
            if !timed_state.is_button_mode {
                // 只在非按钮模式下处理上下移动
                let max_index = if timed_state.is_right_panel {
                    timed_state.selected_tickets.len().saturating_sub(1)
                } else {
                    // 计算实际可选的票种数量
                    (0..8)
                        .filter(|i| !timed_state.selected_tickets.contains(i))
                        .count()
                        .saturating_sub(1)
                };

                match key {
                    KeyCode::Up if timed_state.selected_index > 0 => {
                        timed_state.selected_index -= 1;
                    }
                    KeyCode::Down if timed_state.selected_index < max_index => {
                        timed_state.selected_index += 1;
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Left | KeyCode::Right => {
            if !timed_state.is_button_mode {
                // 只在非按钮模式下处理左右切换
                // 当已选票种为空时，禁止切换到右侧面板
                if timed_state.selected_tickets.is_empty() {
                    if timed_state.selected_index == timed_state.selected_tickets.len() {
                        timed_state.is_right_panel = false;
                        timed_state.selected_index = 0;
                    }
                    return;
                }
                timed_state.is_right_panel = !timed_state.is_right_panel;
                timed_state.selected_index = 0;
            }
        }
        KeyCode::Enter => {
            if timed_state.is_button_mode {
                match timed_state.button_focus {
                    0 => {
                        // 开始抢票逻辑
                        if !timed_state.selected_tickets.is_empty() {
                            gui_state.add_console_message("开始抢票...".to_string());

                            let selected_tickets = timed_state.selected_tickets.clone();
                            let console_sender = gui_state.console_sender.clone();
                            tokio::spawn(async move {
                                start_ticket_grabbing_logic(
                                    read_saved_token().unwrap(),
                                    selected_tickets,
                                    console_sender,
                                )
                                .await;
                            });
                        } else {
                            gui_state.add_console_message("请先选择要抢的票种！".to_string());
                        }
                    }
                    1 => {
                        // 清空选择
                        timed_state.selected_tickets.clear();
                        timed_state.selected_index = 0;
                        gui_state.add_console_message("已清空所有选择".to_string());
                    }
                    _ => {}
                }
            } else {
                if !timed_state.is_right_panel {
                    // 获取实际可选的票种（排除已选择的）
                    let available_tickets: Vec<usize> = (0..8)
                        .filter(|i| !timed_state.selected_tickets.contains(i))
                        .collect();

                    if available_tickets.is_empty() {
                        gui_state.console_info = "已经没有可选的票种了".to_string();
                        return;
                    }

                    // 使用 available_tickets 中的索引
                    if let Some(&ticket_idx) = available_tickets.get(timed_state.selected_index) {
                        timed_state.selected_tickets.push(ticket_idx);

                        // 如果这是最后一个可选票种，将索引重置为0
                        if available_tickets.len() == 1 {
                            timed_state.selected_index = 0;
                            timed_state.is_right_panel = true; // 自动切换到右侧面板
                        } else if timed_state.selected_index >= available_tickets.len() - 1 {
                            // 否则调整选择索引
                            timed_state.selected_index = available_tickets.len() - 2.max(0);
                        }
                    }
                } else {
                    // 在右侧面板时，从已选列表中移除选中的票种
                    if !timed_state.selected_tickets.is_empty()
                        && timed_state.selected_index < timed_state.selected_tickets.len()
                    {
                        timed_state
                            .selected_tickets
                            .remove(timed_state.selected_index);

                        // 如果移除后已选票种为空，自动切换到左侧面板
                        if timed_state.selected_tickets.is_empty() {
                            timed_state.selected_index = 0;
                            timed_state.is_right_panel = false;
                        } else if timed_state.selected_index >= timed_state.selected_tickets.len() {
                            // 如果删除的是最后一个，将选择移到上一个
                            timed_state.selected_index = timed_state.selected_tickets.len() - 1;
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

// 抢票页面逻辑
pub async fn start_ticket_grabbing_logic(
    auth_token: String,
    selected_tickets: Vec<usize>,
    console_sender: tokio::sync::mpsc::Sender<String>,
) {
    use crate::Gui::timed_ticket_grabbing::{read_saved_date, read_saved_time};
    use chrono::{Local, NaiveDateTime};
    use tokio::time::{sleep, Duration};

    // 获取保存的日期和时间
    let target_date = match read_saved_date() {
        Some(date) => date,
        None => {
            let _ = console_sender.send("未设置抢票日期！".to_string()).await;
            return;
        }
    };

    let target_time = match read_saved_time() {
        Some((h, m, s)) => (h, m, s),
        None => {
            let _ = console_sender.send("未设置抢票时间！".to_string()).await;
            return;
        }
    };

    // 构建目标时间
    let target_datetime = NaiveDateTime::new(
        target_date,
        chrono::NaiveTime::from_hms_opt(
            target_time.0 as u32,
            target_time.1 as u32,
            target_time.2 as u32,
        )
        .unwrap(),
    );

    let now = Local::now().naive_local();
    if target_datetime <= now {
        let _ = console_sender
            .send("错误：设置的时间已经过期，请重新设置！".to_string())
            .await;
        return;
    }

    let _ = console_sender
        .send(format!("定时抢票时间设置为: {}", target_datetime))
        .await;

    // 等待直到目标时间前1秒
    loop {
        let now = Local::now().naive_local();
        if now >= target_datetime {
            break;
        }

        let duration = target_datetime - now;
        let remaining_secs = duration.num_seconds();

        if remaining_secs > 60 {
            // 大于1分钟时显示分钟
            if remaining_secs % 60 == 0 {
                let _ = console_sender
                    .send(format!("距离抢票还有 {} 分钟", remaining_secs / 60))
                    .await;
            }
        } else {
            // 最后一分钟显示秒数
            let _ = console_sender
                .send(format!("距离抢票还有 {} 秒", remaining_secs))
                .await;
        }

        // 根据剩余时间调整睡眠间隔
        let sleep_duration = if remaining_secs > 60 {
            Duration::from_secs(1)
        } else {
            Duration::from_millis(50) // 最后一分钟更频繁地检查
        };

        sleep(sleep_duration).await;
    }

    let _ = console_sender.send("开始发送抢票请求...".to_string()).await;

    // 读取游戏配置
    let config_path = crate::Gui::Setting::SettingState::get_config_path();
    let game_configs = match fs::read_to_string(config_path) {
        Ok(content) => {
            serde_json::from_str::<HashMap<String, GameConfig>>(&content).unwrap_or_default()
        }
        Err(_) => {
            let _ = console_sender.send("无法读取游戏配置".to_string()).await;
            return;
        }
    };

    // 收集所有激活的游戏的communityId
    let active_community_ids: Vec<String> = game_configs
        .iter()
        .filter(|(_, config)| config.active)
        .map(|(_, config)| config.communityId.clone())
        .collect();

    if active_community_ids.is_empty() {
        let _ = console_sender.send("没有选择任何游戏".to_string()).await;
        return;
    }

    // 创建游戏ID到名称的映射
    let game_names: HashMap<String, String> = game_configs
        .iter()
        .map(|(name, config)| (config.communityId.clone(), name.clone()))
        .collect();

    let mut handles = vec![];
    let mut total_tasks = 0;

    // 首先计算总任务数
    for community_id in &active_community_ids {
        let game_name = game_names
            .get(community_id)
            .unwrap_or(&"未知游戏".to_string())
            .clone();
        let game_config = game_configs.get(&game_name).unwrap().clone();

        for ticket_idx in &selected_tickets {
            let ticket_name = TICKET_IDS.lock().unwrap()[*ticket_idx].0;
            if let Some(task_ids) = game_config.red_pack_tasks.get(ticket_name) {
                total_tasks += task_ids.len();
            }
        }
    }

    // 创建barrier等待所有任务就绪
    let barrier = Arc::new(Barrier::new(total_tasks));

    // 设置每个任务的并发数
    const CONCURRENT_REQUESTS: usize = 5; // 每个任务发送5个并发请求

    // 为每个任务创建独立线程
    for community_id in active_community_ids {
        let game_name = game_names
            .get(&community_id)
            .unwrap_or(&"未知游戏".to_string())
            .clone();
        let game_config = game_configs.get(&game_name).unwrap().clone();

        for ticket_idx in &selected_tickets {
            let ticket_name = TICKET_IDS.lock().unwrap()[*ticket_idx].0;

            if let Some(task_ids) = game_config.red_pack_tasks.get(ticket_name) {
                let task_ids = task_ids.clone();
                for task_id in task_ids {
                    // 为每个任务创建多个并发请求
                    for _ in 0..CONCURRENT_REQUESTS {
                        let auth_token = auth_token.clone();
                        let console_sender = console_sender.clone();
                        let community_id = community_id.clone();
                        let game_name = game_name.clone();
                        let ticket_name = ticket_name.to_string();
                        let task_id = task_id.clone();
                        let barrier = barrier.clone();

                        let handle = std::thread::spawn(move || {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(async {
                                barrier.wait();

                                match fetch_receive(auth_token, task_id.clone(), community_id).await {
                                    Ok(msg) => {
                                        let _ = console_sender
                                            .send(format!("{}({}): {}", game_name, ticket_name, msg))
                                            .await;
                                    }
                                    Err(e) => {
                                        let _ = console_sender
                                            .send(format!(
                                                "{}({})抢票失败：{}",
                                                game_name, ticket_name, e
                                            ))
                                            .await;
                                    }
                                }
                            });
                        });
                        handles.push(handle);
                    }
                }
            }
        }
    }

    // 等待所有线程完成
    for handle in handles {
        let _ = handle.join();
    }

    let _ = console_sender.send("抢票任务已完成".to_string()).await;
}
