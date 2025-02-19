use crate::api::queryMobilePhone::get_config_path;
use crate::api::queryMobilePhone::TokenStorage;
use crate::Gui::Gui::render_common_layout;
use crate::{function_list, GuiState};
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List};
use ratatui::Frame;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tokio;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SettingState {
    pub setting_index: usize,
    pub show_prop: bool,
    pub game_selections: Vec<bool>, // 跟踪每个游戏是否被选中
    pub popup_index: usize,         // 弹窗中的当前选择位置
    #[serde(skip)] // 不序列化这个字段
    pub games: Vec<String>,
    // 添加账号管理相关字段
    #[serde(skip)]
    pub accounts: Vec<String>, // 用于存储账号列表
}

impl Default for SettingState {
    fn default() -> Self {
        let config_path = get_config_path();
        // 读取并解析账号配置
        let accounts = if let Some(path) = config_path {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(storage) = serde_json::from_str::<TokenStorage>(&content) {
                    storage
                        .records
                        .into_iter()
                        .map(|record| {
                            if record.active {
                                format!("[*] {}", record.mobile_phone)
                            } else {
                                format!("[ ] {}", record.mobile_phone)
                            }
                        })
                        .collect()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        Self {
            setting_index: 0,
            show_prop: false,
            game_selections: vec![false; 3],
            popup_index: 0,
            games: vec![
                "三国杀".to_string(),
                "王者荣耀".to_string(),
                "火影忍者".to_string(),
            ],
            accounts,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct GameConfig {
    active: bool,
    communityId: String,
    #[serde(flatten)]
    amounts: std::collections::HashMap<String, u32>,
    red_pack_tasks: HashMap<String, Vec<String>>,
}

impl SettingState {
    pub fn new(gui_state: &mut GuiState) -> Self {
        let games = vec![
            "三国杀".to_string(),
            "王者荣耀".to_string(),
            "火影忍者".to_string(),
        ];
        gui_state.add_console_message(format!("初始化设置状态，游戏列表：{:?}", games));

        let mut state = Self {
            setting_index: 0,
            show_prop: false,
            game_selections: vec![false; games.len()], // 初始化时所有游戏都未选中
            popup_index: 0,
            games: games.clone(),
            accounts: vec!["账号1".to_string(), "账号2".to_string()], // 添加默认账号
        };

        // 先加载设置，这样可以恢复保存的选择状态
        state.load_settings(gui_state);

        // 确保游戏列表不为空
        if state.games.is_empty() {
            state.games = games;
            state.game_selections = vec![false; state.games.len()]; // 重置为未选中状态
            gui_state.add_console_message(format!("重新初始化空游戏列表：{:?}", state.games));
        }

        // 确保配置文件存在，如果不存在则创建默认配置
        if !Self::get_config_path().exists() {
            Self::save_settings_static(&state.games, &state.game_selections, gui_state);
        }

        gui_state.add_console_message(format!("最终游戏列表：{:?}", state.games));
        state
    }

    pub fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().expect("无法获取配置目录");
        path.push("UTermux");
        path.push("AppGame.json");
        path
    }

    pub fn save_settings(&self, gui_state: &mut GuiState) {
        let config_path = Self::get_config_path();
        if let Some(parent) = config_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                gui_state.add_console_message(format!("创建配置目录失败: {}", e));
                return;
            }
        }

        // 创建游戏名称到选择状态的映射
        let mut game_map = HashMap::new();
        for (i, game) in self.games.iter().enumerate() {
            game_map.insert(game, self.game_selections[i]);
        }

        match serde_json::to_string_pretty(&game_map) {
            Ok(json) => {
                if let Err(e) = fs::write(&config_path, json) {
                    gui_state.add_console_message(format!("保存设置失败: {}", e));
                } else {
                    gui_state.add_console_message("设置保存成功".to_string());
                }
            }
            Err(e) => gui_state.add_console_message(format!("序列化设置失败: {}", e)),
        }
    }

    fn load_settings(&mut self, gui_state: &mut GuiState) {
        match fs::read_to_string(Self::get_config_path()) {
            Ok(content) => {
                match serde_json::from_str::<HashMap<String, GameConfig>>(&content) {
                    Ok(game_map) => {
                        // 重置所有选择状态
                        self.game_selections.fill(false);

                        // 根据保存的配置更新选择状态
                        for (i, game) in self.games.iter().enumerate() {
                            if let Some(config) = game_map.get(game) {
                                self.game_selections[i] = config.active;
                            }
                        }
                        gui_state.add_console_message("设置加载成功".to_string());
                    }
                    Err(e) => {
                        gui_state.add_console_message(format!("解析设置文件失败: {}", e));
                        self.game_selections = vec![false; self.games.len()];
                    }
                }
            }
            Err(e) => {
                gui_state.add_console_message(format!("读取设置文件失败: {}", e));
                self.game_selections = vec![false; self.games.len()];
            }
        }
    }

    // 渲染设置页面
    pub fn render(&self, frame: &mut Frame, state: &GuiState) {
        let (content_area, _) = render_common_layout(frame, state);
        let items = vec!["1.选择游戏🎮", "2.账号管理📒", "3.关于作者🧑"]
            .iter()
            .enumerate()
            .map(|(i, item)| {
                if i == self.setting_index {
                    Line::from(*item).style(Style::default().fg(Color::Black).bg(Color::White))
                } else {
                    Line::from(*item)
                }
            })
            .collect::<Vec<Line>>();

        let layout = List::new(items).block(
            Block::new()
                .title("设置")
                .title_style(Style::new().not_bold())
                .borders(Borders::ALL)
                .border_style(Style::new().white()),
        );

        frame.render_widget(layout, content_area);

        // 如果show_prop为true，显示弹窗
        if self.show_prop {
            let title = match self.setting_index {
                0 => "选择游戏(可多选)",
                1 => "账号管理(可多选)",
                2 => "关于作者",
                _ => "",
            };
            self.render_popup(frame, content_area, title);
        }
    }

    // 添加渲染弹窗的辅助函数
    fn render_popup(&self, frame: &mut Frame, area: Rect, title: &str) {
        let popup_area = self.centered_rect(60, 50, area);
        let popup = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));

        frame.render_widget(popup.clone(), popup_area);

        let inner_area = popup_area.inner(Default::default());
        let inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(3)])
            .split(inner_area);

        match self.setting_index {
            0 => {
                // 游戏列表渲染逻辑
                let items: Vec<Line> = self
                    .games
                    .iter()
                    .enumerate()
                    .map(|(i, game)| {
                        let prefix = if self.game_selections[i] {
                            Span::styled("[*] ", Style::default().fg(Color::Red))
                        } else {
                            Span::raw("[ ] ")
                        };
                        let game_span = Span::raw(game);
                        let spans = vec![prefix, game_span];

                        if i == self.popup_index {
                            Line::from(spans)
                                .style(Style::default().fg(Color::Black).bg(Color::White))
                        } else {
                            Line::from(spans)
                        }
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::NONE))
                    .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

                frame.render_widget(list, inner_layout[1]);
            }
            1 => {
                // 账号管理列表渲染逻辑
                let items: Vec<Line> = self
                    .accounts
                    .iter()
                    .enumerate()
                    .map(|(i, account)| {
                        let is_active = account.starts_with("[*]");
                        let account_text = account
                            .trim_start_matches("[*] ")
                            .trim_start_matches("[ ] ");

                        let prefix = if is_active {
                            Span::styled("[*] ", Style::default().fg(Color::Red))
                        } else {
                            Span::raw("[ ] ")
                        };

                        let account_span = Span::raw(account_text);
                        let spans = vec![prefix, account_span];

                        if i == self.popup_index {
                            Line::from(spans)
                                .style(Style::default().fg(Color::Black).bg(Color::White))
                        } else {
                            Line::from(spans)
                        }
                    })
                    .collect();

                let list = List::new(items)
                    .block(Block::default().borders(Borders::NONE))
                    .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

                frame.render_widget(list, inner_layout[1]);
            }
            _ => {}
        }
    }

    pub fn setting_handle_key(gui_state: &mut GuiState, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                if gui_state.setting_state.show_prop {
                    gui_state.setting_state.show_prop = false;
                    return;
                }
                gui_state.current_page = function_list::Main;
            }
            KeyCode::Up => {
                if gui_state.setting_state.show_prop {
                    match gui_state.setting_state.setting_index {
                        0 => {
                            if !gui_state.setting_state.games.is_empty()
                                && gui_state.setting_state.popup_index > 0
                            {
                                gui_state.setting_state.popup_index =
                                    gui_state.setting_state.popup_index.saturating_sub(1);
                            }
                        }
                        1 => {
                            if !gui_state.setting_state.accounts.is_empty()
                                && gui_state.setting_state.popup_index > 0
                            {
                                gui_state.setting_state.popup_index =
                                    gui_state.setting_state.popup_index.saturating_sub(1);
                            }
                        }
                        _ => {}
                    }
                    return;
                }

                if gui_state.setting_state.setting_index > 0 {
                    gui_state.setting_state.setting_index =
                        gui_state.setting_state.setting_index.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                if gui_state.setting_state.show_prop {
                    match gui_state.setting_state.setting_index {
                        0 => {
                            if !gui_state.setting_state.games.is_empty()
                                && gui_state.setting_state.popup_index
                                    < gui_state.setting_state.games.len().saturating_sub(1)
                            {
                                gui_state.setting_state.popup_index =
                                    gui_state.setting_state.popup_index.saturating_add(1);
                            }
                        }
                        1 => {
                            if !gui_state.setting_state.accounts.is_empty()
                                && gui_state.setting_state.popup_index
                                    < gui_state.setting_state.accounts.len().saturating_sub(1)
                            {
                                gui_state.setting_state.popup_index =
                                    gui_state.setting_state.popup_index.saturating_add(1);
                            }
                        }
                        _ => {}
                    }
                    return;
                }

                if gui_state.setting_state.setting_index < 2 {
                    gui_state.setting_state.setting_index =
                        gui_state.setting_state.setting_index.saturating_add(1);
                }
            }
            KeyCode::Enter => {
                if gui_state.setting_state.show_prop {
                    match gui_state.setting_state.setting_index {
                        0 => {
                            if !gui_state.setting_state.games.is_empty() {
                                // 切换游戏选中状态
                                gui_state.setting_state.game_selections
                                    [gui_state.setting_state.popup_index] = !gui_state
                                    .setting_state
                                    .game_selections[gui_state.setting_state.popup_index];

                                // 保存设置
                                let games = gui_state.setting_state.games.clone();
                                let selections = gui_state.setting_state.game_selections.clone();
                                let console_sender = gui_state.console_sender.clone();

                                tokio::spawn(async move {
                                    let mut gui_state_clone = GuiState {
                                        console_sender,
                                        current_page: crate::function_list::Main,
                                        selected_index: 0,
                                        input_buffer: String::new(),
                                        auth_token: String::new(),
                                        console_info: String::new(),
                                        console_receiver: tokio::sync::mpsc::channel::<String>(100)
                                            .1,
                                        console_scroll: 0,
                                        max_console_lines: 100,
                                        auto_scroll: true,
                                        setting_state: SettingState::default(),
                                        setting_index: 0,
                                        show_prop: false,
                                        active_input: 0,
                                        last_key: KeyCode::Enter,
                                        user_agent: String::new(),
                                    };

                                    SettingState::save_settings_static(
                                        &games,
                                        &selections,
                                        &mut gui_state_clone,
                                    )
                                    .await;
                                });

                                gui_state.add_console_message(
                                    format!(
                                        "已保存-{}",
                                        gui_state.setting_state.games
                                            [gui_state.setting_state.popup_index]
                                    )
                                    .to_string(),
                                );
                            }
                        }
                        1 => {
                            if !gui_state.setting_state.accounts.is_empty() {
                                let account_index = gui_state.setting_state.popup_index;

                                // 读取当前的token存储
                                if let Some(path) = get_config_path() {
                                    if let Ok(content) = fs::read_to_string(&path) {
                                        if let Ok(mut storage) =
                                            serde_json::from_str::<TokenStorage>(&content)
                                        {
                                            // 切换选中状态
                                            if let Some(record) =
                                                storage.records.get_mut(account_index)
                                            {
                                                record.active = !record.active;
                                            }

                                            // 保存更新后的配置
                                            if let Ok(json_content) =
                                                serde_json::to_string_pretty(&storage)
                                            {
                                                if let Err(e) = fs::write(&path, json_content) {
                                                    gui_state.add_console_message(format!(
                                                        "保存账号配置失败: {}",
                                                        e
                                                    ));
                                                } else {
                                                    // 更新UI显示
                                                    gui_state.setting_state.accounts = storage
                                                        .records
                                                        .into_iter()
                                                        .map(|record| {
                                                            if record.active {
                                                                format!(
                                                                    "[*] {}",
                                                                    record.mobile_phone
                                                                )
                                                            } else {
                                                                format!(
                                                                    "[ ] {}",
                                                                    record.mobile_phone
                                                                )
                                                            }
                                                        })
                                                        .collect();

                                                    gui_state.add_console_message(
                                                        "账号状态切换成功".to_string(),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    return;
                }
                gui_state.setting_state.show_prop = true;
                gui_state.setting_state.popup_index = 0;
                if gui_state.setting_state.setting_index == 0 {
                    // 加载游戏设置
                    let mut setting_state = gui_state.setting_state.clone();
                    setting_state.load_settings(gui_state);
                    gui_state.setting_state = setting_state;
                }
            }
            _ => {}
        }
    }

    // 添加一个辅助函数来计算弹窗的位置和大小
    pub fn centered_rect(&self, percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    pub async fn save_settings_static(
        games: &[String],
        selections: &[bool],
        gui_state: &mut GuiState,
    ) {
        let config_path = Self::get_config_path();
        gui_state.add_console_message(format!("配置文件路径: {:?}", config_path));

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                gui_state.add_console_message(format!("创建配置目录失败: {}", e));
                return;
            }
            gui_state.add_console_message("配置目录创建成功".to_string());
        }

        // 如果文件不存在，创建一个空的配置文件
        if !config_path.exists() {
            let default_config: HashMap<String, GameConfig> = HashMap::new();
            let json = serde_json::to_string_pretty(&default_config).unwrap_or_default();
            match fs::write(&config_path, json) {
                Ok(_) => gui_state.add_console_message("配置文件创建成功".to_string()),
                Err(e) => {
                    gui_state.add_console_message(format!("创建配置文件失败: {}", e));
                    return;
                }
            }
        }

        // 读取现有配置
        let mut game_map = match fs::read_to_string(&config_path) {
            Ok(content) => {
                serde_json::from_str::<std::collections::HashMap<String, GameConfig>>(&content)
                    .unwrap_or_default()
            }
            Err(_) => std::collections::HashMap::new(),
        };

        for (i, game) in games.iter().enumerate() {
            let (community_id, mut red_pack_tasks) = match game.as_str() {
                "三国杀" => ("14", HashMap::new()),
                "王者荣耀" => ("7", HashMap::new()),
                "火影忍者" => ("10", HashMap::new()),
                _ => ("14", HashMap::new()),
            };

            if selections[i] {
                if let Some(auth_token) = crate::api::queryMobilePhone::read_saved_token() {
                    match crate::api::info::get_info(auth_token, community_id.to_string()).await {
                        Ok(info) => {
                            // 解析返回的信息
                            for zone_info in info.split(';') {
                                if let Some((zone_name, ids)) = zone_info.split_once(':') {
                                    red_pack_tasks.insert(
                                        zone_name.to_string(),
                                        ids.split(',').map(String::from).collect(),
                                    );
                                }
                            }
                            gui_state
                                .add_console_message(format!("成功获取 {} 的红包任务ID", game));
                        }
                        Err(e) => {
                            gui_state.add_console_message(format!(
                                "获取 {} 的红包任务ID失败: {}",
                                game, e
                            ));
                        }
                    }
                }
            }

            game_map
                .entry(game.clone())
                .and_modify(|config| {
                    config.active = selections[i];
                    if !red_pack_tasks.is_empty() {
                        config.red_pack_tasks = red_pack_tasks.clone();
                    }
                })
                .or_insert(GameConfig {
                    active: selections[i],
                    communityId: community_id.to_string(),
                    amounts: std::collections::HashMap::new(),
                    red_pack_tasks,
                });
        }

        // 保存更新后的配置
        if let Some(parent) = config_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                gui_state.add_console_message(format!("创建配置目录失败: {}", e));
                return;
            }
        }

        match serde_json::to_string_pretty(&game_map) {
            Ok(json) => {
                if let Err(e) = fs::write(&config_path, json) {
                    gui_state.add_console_message(format!("保存设置失败: {}", e));
                } else {
                    gui_state.add_console_message("设置保存成功".to_string());
                }
            }
            Err(e) => gui_state.add_console_message(format!("序列化设置失败: {}", e)),
        }
    }

    // 确保games字段可以被访问
    pub fn get_games(&self) -> &Vec<String> {
        &self.games
    }
}
