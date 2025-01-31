use color_eyre::Result;
use crossterm::event::KeyEventKind;
use crossterm::event::{self, Event, KeyCode};
use ratatui::DefaultTerminal;

mod Gui;
use Gui::timed_ticket_grabbing::TimedGrabbingState;
use Gui::Gui::handle_key_input_main;
use Gui::Gui::render_gui;
use Gui::Login::handle_login_input;
use Gui::Setting::SettingState;
use UTermux::api;
// 主界面状态
pub struct GuiState {
    pub selected_index: usize,       // 当前选中的功能列表项
    pub current_page: function_list, // 添加当前页面状态
    pub input_buffer: String,        // 输入缓冲区
    pub auth_token: String,          // auth_token
    pub console_info: String,        // 控制台输出信息
    pub console_sender: tokio::sync::mpsc::Sender<String>, // 明确使用完整路径
    pub console_receiver: tokio::sync::mpsc::Receiver<String>, // 明确使用完整路径
    pub console_scroll: usize,       // 控制台滚动条位置
    pub max_console_lines: usize,    // 控制台最大行数
    pub auto_scroll: bool,           // 自动滚动
    pub setting_index: usize,        // 设置页面索引
    pub show_prop: bool,             // 是否显示弹窗
    pub setting_state: SettingState, // 设置页面状态
}

// 定时抢票界面状态
pub struct timed_ticket_grabbing_state {
    pub selected_index: usize,
    pub current_page: function_list,
    pub selected_tickets: Vec<usize>, // 存储已选择的票种索引
    pub is_right_panel: bool,         // 当前是否在右侧面板
    pub is_button_mode: bool,         // 当前是否在按钮模式
    pub button_focus: usize,          // 按钮焦点
}

// 功能列表
pub enum function_list {
    Main,
    Login,
    TimedTicketGrabbing,
    StartGrabbingTickets,
    Setting,
}

pub enum ticket_list {
    None,
    Ticket1,
    Ticket2,
    Ticket3,
    Ticket4,
    Ticket5,
    Ticket6,
    Ticket7,
    Ticket8,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal).await;
    ratatui::restore();
    result
}

async fn run(mut terminal: DefaultTerminal) -> Result<()> {
    // 使用 tokio 的通道
    let (sender, receiver) = tokio::sync::mpsc::channel(100); // 设置缓冲区大小为100

    // 主界面状态
    let mut state = GuiState {
        selected_index: 0,                 // 当前选中的功能列表项
        current_page: function_list::Main, // 当前页面
        input_buffer: String::new(),       // 输入缓冲区
        auth_token: String::new(),         // 用户认证token
        console_info: String::new(),       // 控制台输出信息
        console_sender: sender,            // 明确使用完整路径
        console_receiver: receiver,        // 明确使用完整路径
        console_scroll: 0,                 // 控制台滚动条位置
        max_console_lines: 100,            // 控制台最大行数
        auto_scroll: true,                 // 自动滚动
        setting_index: 0,                  // 设置页面索引
        show_prop: false,                  // 是否显示弹窗
        setting_state: SettingState::default(),
    };

    // 将TimedGrabbingState移到这里，作为持久化状态
    let mut timed_grabbing_state = TimedGrabbingState::default();

    // 定时抢票界面状态
    let mut timed_ticket_grabbing_state = timed_ticket_grabbing_state {
        selected_index: 0,
        current_page: function_list::TimedTicketGrabbing,
        selected_tickets: Vec::new(), // 初始化为空向量
        is_right_panel: false,        // 初始化为左侧面板
        is_button_mode: false,        // 初始化为按钮模式
        button_focus: 0,              // 初始化按钮焦点为0
    };

    const MAX_INPUT_LENGTH: usize = 1000; // 设置输入缓冲区的最大长度
    const POLL_INTERVAL: u64 = 50; // 设置事件轮询间隔时间 (毫秒)

    loop {
        // 检查是否有事件发生
        if event::poll(std::time::Duration::from_millis(POLL_INTERVAL))? {
            if let Event::Key(key) = event::read()? {
                // 只处理按键按下的事件
                if key.kind == KeyEventKind::Press {
                    // 首先处理全局键盘事件
                    match key.code {
                        KeyCode::PageUp => {
                            if state.console_scroll > 0 {
                                state.console_scroll = state.console_scroll.saturating_sub(1);
                            }
                        }
                        KeyCode::PageDown => {
                            let max_scroll = state.console_info.lines().count().saturating_sub(1);
                            if state.console_scroll < max_scroll {
                                state.console_scroll = state.console_scroll.saturating_add(1);
                            }
                        }
                        KeyCode::F(3) => {
                            state.auto_scroll = !state.auto_scroll;
                            state.add_console_message(format!(
                                "自动滚动已{}",
                                if state.auto_scroll {
                                    "开启"
                                } else {
                                    "关闭"
                                }
                            ));
                            // 向下滚动一行
                            state.console_scroll = state.console_scroll.saturating_add(1);
                        }
                        KeyCode::Delete => {
                            state.console_info.clear();
                            state.console_scroll = 0; // 重置滚动位置
                        }
                        _ => {
                            // 处理其他页面特定的键盘事件
                            match state.current_page {
                                function_list::Main => {
                                    handle_key_input_main(&mut state, key.code);

                                    if key.code == KeyCode::Enter {
                                        // 主页面enter键处理
                                        match state.selected_index {
                                            0 => state.current_page = function_list::Login,
                                            1 => {
                                                state.current_page =
                                                    function_list::TimedTicketGrabbing;
                                                timed_ticket_grabbing_state.selected_index = 0;
                                            }
                                            2 => {
                                                state.current_page =
                                                    function_list::StartGrabbingTickets;
                                            }
                                            3 => {
                                                state.current_page = function_list::Setting;
                                            }
                                            4 => return Ok(()),
                                            _ => {}
                                        }
                                    }
                                }
                                function_list::Login => {
                                    // 登录页面的输入处理逻辑
                                    match key.code {
                                        KeyCode::Char(c) => {
                                            if state.input_buffer.len() < MAX_INPUT_LENGTH {
                                                state.input_buffer.push(c);
                                            } else {
                                                state.console_info =
                                                    "输入过长，已截断！".to_string();
                                            }
                                        }
                                        KeyCode::Backspace => {
                                            state.input_buffer.pop();
                                        }
                                        KeyCode::Enter => {
                                            handle_login_input(&mut state).await;
                                        }
                                        KeyCode::Esc => {
                                            state.current_page = function_list::Main;
                                        }
                                        _ => {}
                                    }
                                }
                                function_list::StartGrabbingTickets => {
                                    Gui::start_grabbing_tickets::handle_start_ticket_grabbing_input(
                                        &mut state,
                                        &mut timed_ticket_grabbing_state,
                                        key.code,
                                    );
                                }
                                function_list::TimedTicketGrabbing => {
                                    Gui::timed_ticket_grabbing::handle_timed_input(
                                        &mut state,
                                        &mut timed_grabbing_state,
                                        key.code,
                                    );

                                    if key.code == KeyCode::Esc {
                                        state.current_page = function_list::Main;
                                    }
                                }
                                function_list::Setting => {
                                    SettingState::setting_handle_key(&mut state, key.code);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 检查是否有新的控制台消息
        if let Ok(msg) = state.console_receiver.try_recv() {
            state.add_console_message(msg);
        }

        // 渲染界面
        terminal.draw(|f| match state.current_page {
            function_list::Main => render_gui(f, &mut state),
            function_list::Login => Gui::Login::login_render(f, &mut state),
            function_list::TimedTicketGrabbing => {
                Gui::timed_ticket_grabbing::timed_ticket_grabbing_render(
                    f,
                    &mut state,
                    &mut timed_grabbing_state,
                )
            }
            function_list::StartGrabbingTickets => {
                Gui::start_grabbing_tickets::start_ticket_grabbing_render(
                    f,
                    &mut state,
                    &mut timed_ticket_grabbing_state,
                )
            }
            function_list::Setting => {
                state.setting_state.render(f, &state);
            }
        })?;
    }
}

impl GuiState {
    // 添加控制台消息
    pub fn add_console_message(&mut self, message: String) {
        // 获取当前时间并格式化
        let now = chrono::Local::now();
        let timestamp = now.format("[%m-%d %H:%M]").to_string();

        // 添加带时间戳的新消息
        if !self.console_info.is_empty() {
            self.console_info.push_str("\n");
        }
        self.console_info
            .push_str(&format!("{} {}", timestamp, message));

        // 限制控制台消息的总行数
        let lines: Vec<&str> = self.console_info.lines().collect();
        if lines.len() > self.max_console_lines {
            // 只保留最后 max_console_lines 行
            self.console_info = lines[lines.len() - self.max_console_lines..].join("\n");
        }

        // 如果启用了自动滚动，则始终滚动到最新消息
        if self.auto_scroll {
            let total_lines = self.console_info.lines().count();
            let max_visible_lines = 8; // 控制台可见行数（减去边框和标题）
            if total_lines > max_visible_lines {
                self.console_scroll = total_lines.saturating_sub(max_visible_lines);
            }
        }

        // 确保滚动位置不会超出总行数
        let total_lines = self.console_info.lines().count();
        if total_lines > 0 {
            self.console_scroll = self.console_scroll.min(total_lines.saturating_sub(1));
        } else {
            self.console_scroll = 0;
        }
    }
}
