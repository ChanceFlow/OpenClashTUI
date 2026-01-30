use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io, thread, time::Duration};
use tungstenite::client::IntoClientRequest;
use tungstenite::connect;

// ============================================================================
// CLI Arguments
// ============================================================================

#[derive(Parser)]
#[command(name = "clashtui")]
#[command(about = "A TUI tool for managing Clash proxies and rules", long_about = None)]
struct Cli {
    /// Clash external controller address
    #[arg(short, long, default_value = "127.0.0.1:9090")]
    controller: String,

    /// API secret (optional)
    #[arg(short, long)]
    secret: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List all proxies
    Proxies,
    /// List all rules
    Rules,
    /// List all proxy groups
    Groups,
    /// Show Clash version
    Version,
    /// Interactive TUI mode (default)
    Tui,
}

// ============================================================================
// Clash API Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct ProxiesResponse {
    proxies: HashMap<String, Proxy>,
}

#[derive(Debug, Deserialize, Clone)]
struct Proxy {
    name: String,
    #[serde(rename = "type")]
    proxy_type: String,
    #[serde(default)]
    all: Vec<String>,
    #[serde(default)]
    now: Option<String>,
    #[serde(default)]
    history: Vec<HistoryItem>,
}

#[derive(Debug, Deserialize, Clone)]
struct HistoryItem {
    delay: i64,
}

#[derive(Debug, Deserialize)]
struct RulesResponse {
    rules: Vec<Rule>,
}

#[derive(Debug, Deserialize, Clone)]
struct Rule {
    #[serde(rename = "type")]
    rule_type: String,
    payload: String,
    proxy: String,
}

#[derive(Debug, Deserialize)]
struct VersionResponse {
    version: String,
}

#[derive(Debug, Deserialize)]
struct DelayResponse {
    delay: i64,
}

#[derive(Debug, Serialize)]
struct SelectProxyRequest {
    name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
struct Traffic {
    up: u64,
    down: u64,
}

#[derive(Debug, Deserialize)]
struct ConnectionsResponse {
    downloadTotal: u64,
    uploadTotal: u64,
    connections: Vec<Connection>,
}

#[derive(Debug, Deserialize, Clone)]
struct Connection {
    id: String,
    metadata: ConnectionMetadata,
    upload: u64,
    download: u64,
    start: String,
    chains: Vec<String>,
    rule: String,
}

#[derive(Debug, Deserialize, Clone)]
struct ConnectionMetadata {
    network: String,
    #[serde(rename = "type")]
    conn_type: String,
    #[serde(rename = "sourceIP")]
    source_ip: String,
    #[serde(rename = "destinationIP")]
    destination_ip: String,
    #[serde(rename = "sourcePort")]
    source_port: String,
    #[serde(rename = "destinationPort")]
    destination_port: String,
    host: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConfigResponse {
    mode: String,
    #[serde(rename = "port")]
    http_port: u16,
    #[serde(rename = "socks-port")]
    socks_port: u16,
}

#[derive(Debug, Serialize)]
struct UpdateConfigRequest {
    mode: String,
}

// ============================================================================
// Clash API Client
// ============================================================================

struct ClashClient {
    base_url: String,
    secret: Option<String>,
    client: reqwest::blocking::Client,
}

impl ClashClient {
    fn new(controller: &str, secret: Option<String>) -> Self {
        let base_url = if controller.starts_with("http") {
            controller.to_string()
        } else {
            format!("http://{}", controller)
        };

        Self {
            base_url,
            secret,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn request(&self, method: reqwest::Method, endpoint: &str) -> reqwest::blocking::RequestBuilder {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut req = self.client.request(method, &url);
        if let Some(ref secret) = self.secret {
            req = req.header("Authorization", format!("Bearer {}", secret));
        }
        req
    }

    fn get_proxies(&self) -> Result<ProxiesResponse> {
        let resp = self
            .request(reqwest::Method::GET, "/proxies")
            .send()
            .context("Failed to fetch proxies")?
            .json::<ProxiesResponse>()
            .context("Failed to parse proxies response")?;
        Ok(resp)
    }

    fn get_rules(&self) -> Result<RulesResponse> {
        let resp = self
            .request(reqwest::Method::GET, "/rules")
            .send()
            .context("Failed to fetch rules")?
            .json::<RulesResponse>()
            .context("Failed to parse rules response")?;
        Ok(resp)
    }

    fn get_version(&self) -> Result<VersionResponse> {
        let resp = self
            .request(reqwest::Method::GET, "/version")
            .send()
            .context("Failed to fetch version")?
            .json::<VersionResponse>()
            .context("Failed to parse version response")?;
        Ok(resp)
    }

    fn select_proxy(&self, group: &str, proxy: &str) -> Result<()> {
        let endpoint = format!("/proxies/{}", urlencoding::encode(group));
        self.request(reqwest::Method::PUT, &endpoint)
            .json(&SelectProxyRequest {
                name: proxy.to_string(),
            })
            .send()
            .context("Failed to select proxy")?;
        Ok(())
    }

    fn test_delay(&self, proxy: &str, url: &str, timeout: u64) -> Result<i64> {
        let endpoint = format!(
            "/proxies/{}/delay?url={}&timeout={}",
            urlencoding::encode(proxy),
            urlencoding::encode(url),
            timeout
        );
        let resp = self
            .request(reqwest::Method::GET, &endpoint)
            .send()
            .context("Failed to test delay")?;

        if resp.status().is_success() {
            let delay_resp: DelayResponse = resp.json().context("Failed to parse delay response")?;
            Ok(delay_resp.delay)
        } else {
            Ok(-1) // Timeout or error
        }
    }

    fn get_config(&self) -> Result<ConfigResponse> {
        let resp = self
            .request(reqwest::Method::GET, "/configs")
            .send()
            .context("Failed to fetch config")?
            .json::<ConfigResponse>()
            .context("Failed to parse config response")?;
        Ok(resp)
    }

    fn get_connections(&self) -> Result<ConnectionsResponse> {
        let resp = self
            .request(reqwest::Method::GET, "/connections")
            .send()
            .context("Failed to fetch connections")?
            .json::<ConnectionsResponse>()
            .context("Failed to parse connections response")?;
        Ok(resp)
    }

    fn update_mode(&self, mode: &str) -> Result<()> {
        let req = UpdateConfigRequest {
            mode: mode.to_string(),
        };
        self.request(reqwest::Method::PATCH, "/configs")
            .json(&req)
            .send()
            .context("Failed to update config")?;
        Ok(())
    }
}

// ============================================================================
// TUI Application
// ============================================================================

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Proxies,
    Rules,
    Conns,
}

struct App {
    client: ClashClient,
    current_tab: Tab,
    // Proxy groups
    groups: Vec<String>,
    group_state: ListState,
    // Proxies in selected group
    proxies: Vec<(String, i64)>, // (name, delay)
    proxy_state: ListState,
    // Current group info
    current_group: Option<Proxy>,
    // Rules
    rules: Vec<Rule>,
    rule_state: ListState,
    // Connections
    conns: Vec<Connection>,
    conn_state: ListState,
    // Status message
    status: String,
    // Traffic
    traffic: Traffic,
    traffic_rx: crossbeam_channel::Receiver<Traffic>,
    // Current Clash Mode
    mode: String,
    // Show help popup
    show_help: bool,
    // Focus: 0 = groups, 1 = proxies
    focus: usize,
}

impl App {
    fn new(client: ClashClient, traffic_rx: crossbeam_channel::Receiver<Traffic>) -> Self {
        let mut app = Self {
            client,
            current_tab: Tab::Proxies,
            groups: Vec::new(),
            group_state: ListState::default(),
            proxies: Vec::new(),
            proxy_state: ListState::default(),
            current_group: None,
            rules: Vec::new(),
            rule_state: ListState::default(),
            conns: Vec::new(),
            conn_state: ListState::default(),
            status: String::from("Press ? for help"),
            traffic: Traffic::default(),
            traffic_rx,
            mode: String::from("Unknown"),
            show_help: false,
            focus: 0,
        };
        app.refresh_data();
        app
    }

    fn refresh_data(&mut self) {
        // Also refresh config/mode
        if let Ok(config) = self.client.get_config() {
            self.mode = config.mode;
        }

        // Process any traffic updates
        while let Ok(traffic) = self.traffic_rx.try_recv() {
            self.traffic = traffic;
        }

        match self.current_tab {
            Tab::Proxies => self.refresh_proxies(),
            Tab::Rules => self.refresh_rules(),
            Tab::Conns => self.refresh_conns(),
        }
    }

    fn refresh_proxies(&mut self) {
        match self.client.get_proxies() {
            Ok(resp) => {
                // Filter only Selector type groups
                self.groups = resp
                    .proxies
                    .iter()
                    .filter(|(_, p)| p.proxy_type == "Selector" || p.proxy_type == "URLTest" || p.proxy_type == "Fallback")
                    .map(|(name, _)| name.clone())
                    .collect();
                self.groups.sort();

                if !self.groups.is_empty() && self.group_state.selected().is_none() {
                    self.group_state.select(Some(0));
                }

                self.update_proxies_for_group(&resp.proxies);
                self.status = format!("Loaded {} groups", self.groups.len());
            }
            Err(e) => {
                self.status = format!("Error: {}", e);
            }
        }
    }

    fn update_proxies_for_group(&mut self, all_proxies: &HashMap<String, Proxy>) {
        if let Some(idx) = self.group_state.selected() {
            if let Some(group_name) = self.groups.get(idx) {
                if let Some(group) = all_proxies.get(group_name) {
                    self.current_group = Some(group.clone());
                    self.proxies = group
                        .all
                        .iter()
                        .map(|name| {
                            let delay = all_proxies
                                .get(name)
                                .and_then(|p| p.history.last())
                                .map(|h| h.delay)
                                .unwrap_or(-1);
                            (name.clone(), delay)
                        })
                        .collect();

                    if !self.proxies.is_empty() && self.proxy_state.selected().is_none() {
                        self.proxy_state.select(Some(0));
                    }
                }
            }
        }
    }

    fn refresh_rules(&mut self) {
        match self.client.get_rules() {
            Ok(resp) => {
                self.rules = resp.rules;
                if !self.rules.is_empty() && self.rule_state.selected().is_none() {
                    self.rule_state.select(Some(0));
                }
                self.status = format!("Loaded {} rules", self.rules.len());
            }
            Err(e) => {
                self.status = format!("Error: {}", e);
            }
        }
    }

    fn refresh_conns(&mut self) {
        match self.client.get_connections() {
            Ok(resp) => {
                self.conns = resp.connections;
                // Sort by start time desc
                self.conns.sort_by(|a, b| b.start.cmp(&a.start));
                
                if !self.conns.is_empty() && self.conn_state.selected().is_none() {
                    self.conn_state.select(Some(0));
                }
                self.status = format!("Loaded {} connections. Up: {}, Down: {}", 
                    self.conns.len(), 
                    format_bytes(resp.uploadTotal), 
                    format_bytes(resp.downloadTotal));
            }
            Err(e) => {
                self.status = format!("Error: {}", e);
            }
        }
    }

    fn select_proxy(&mut self) {
        if let (Some(group), Some(proxy_idx)) = (&self.current_group, self.proxy_state.selected()) {
            if let Some((proxy_name, _)) = self.proxies.get(proxy_idx) {
                match self.client.select_proxy(&group.name, proxy_name) {
                    Ok(_) => {
                        self.status = format!("Selected: {} -> {}", group.name, proxy_name);
                        self.refresh_proxies();
                    }
                    Err(e) => {
                        self.status = format!("Error selecting proxy: {}", e);
                    }
                }
            }
        }
    }

    fn test_selected_delay(&mut self) {
        if let Some(proxy_idx) = self.proxy_state.selected() {
            if let Some((proxy_name, _)) = self.proxies.get(proxy_idx).cloned() {
                self.status = format!("Testing {}...", proxy_name);
                match self.client.test_delay(&proxy_name, "http://www.gstatic.com/generate_204", 5000) {
                    Ok(delay) => {
                        if delay > 0 {
                            self.status = format!("{}: {}ms", proxy_name, delay);
                            // Update the delay in our list
                            if let Some(p) = self.proxies.get_mut(proxy_idx) {
                                p.1 = delay;
                            }
                        } else {
                            self.status = format!("{}: timeout", proxy_name);
                        }
                    }
                    Err(e) => {
                        self.status = format!("Error testing delay: {}", e);
                    }
                }
            }
        }
    }

    fn toggle_mode(&mut self) {
        let new_mode = match self.mode.as_str() {
            "Rule" => "Global",
            "Global" => "Direct",
            _ => "Rule",
        };
        
        match self.client.update_mode(new_mode) {
            Ok(_) => {
                self.mode = new_mode.to_string();
                self.status = format!("Switched to {} mode", new_mode);
            }
            Err(e) => {
                self.status = format!("Error switching mode: {}", e);
            }
        }
    }

    #[allow(dead_code)]
    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Proxies => Tab::Rules,
            Tab::Rules => Tab::Conns,
            Tab::Conns => Tab::Proxies,
        };
        self.refresh_data();
    }

    fn prev_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Proxies => Tab::Conns,
            Tab::Conns => Tab::Rules,
            Tab::Rules => Tab::Proxies,
        };
        self.refresh_data();
    }

    fn move_up(&mut self) {
        match self.current_tab {
            Tab::Proxies => {
                if self.focus == 0 {
                    let i = self.group_state.selected().unwrap_or(0);
                    if i > 0 {
                        self.group_state.select(Some(i - 1));
                        self.proxy_state.select(Some(0));
                        if let Ok(resp) = self.client.get_proxies() {
                            self.update_proxies_for_group(&resp.proxies);
                        }
                    }
                } else {
                    let i = self.proxy_state.selected().unwrap_or(0);
                    if i > 0 {
                        self.proxy_state.select(Some(i - 1));
                    }
                }
            }
            Tab::Rules => {
                let i = self.rule_state.selected().unwrap_or(0);
                if i > 0 {
                    self.rule_state.select(Some(i - 1));
                }
            }
            Tab::Conns => {
                let i = self.conn_state.selected().unwrap_or(0);
                if i > 0 {
                    self.conn_state.select(Some(i - 1));
                }
            }
        }
    }

    fn move_down(&mut self) {
        match self.current_tab {
            Tab::Proxies => {
                if self.focus == 0 {
                    let i = self.group_state.selected().unwrap_or(0);
                    if i < self.groups.len().saturating_sub(1) {
                        self.group_state.select(Some(i + 1));
                        self.proxy_state.select(Some(0));
                        if let Ok(resp) = self.client.get_proxies() {
                            self.update_proxies_for_group(&resp.proxies);
                        }
                    }
                } else {
                    let i = self.proxy_state.selected().unwrap_or(0);
                    if i < self.proxies.len().saturating_sub(1) {
                        self.proxy_state.select(Some(i + 1));
                    }
                }
            }
            Tab::Rules => {
                let i = self.rule_state.selected().unwrap_or(0);
                if i < self.rules.len().saturating_sub(1) {
                    self.rule_state.select(Some(i + 1));
                }
            }
            Tab::Conns => {
                let i = self.conn_state.selected().unwrap_or(0);
                if i < self.conns.len().saturating_sub(1) {
                    self.conn_state.select(Some(i + 1));
                }
            }
        }
    }

    fn toggle_focus(&mut self) {
        if self.current_tab == Tab::Proxies {
            self.focus = 1 - self.focus;
        }
    }
}

// ============================================================================
// UI Rendering
// ============================================================================

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Main content
            Constraint::Length(4), // Status bar + Hints
        ])
        .split(f.size());

    // Tabs
    let tab_titles = vec!["Proxies [1]", "Rules [2]", "Conns [3]"];
    let selected_tab = match app.current_tab {
        Tab::Proxies => 0,
        Tab::Rules => 1,
        Tab::Conns => 2,
    };
    
    // Format traffic
    let up_speed = format_speed(app.traffic.up);
    let down_speed = format_speed(app.traffic.down);
    
    let title = format!(" ClashTUI - Mode: {} | ↑ {} | ↓ {} ", app.mode, up_speed, down_speed);
    
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title(title))
        .select(selected_tab)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );
    f.render_widget(tabs, chunks[0]);

    // Main content
    match app.current_tab {
        Tab::Proxies => render_proxies_tab(f, app, chunks[1]),
        Tab::Rules => render_rules_tab(f, app, chunks[1]),
        Tab::Conns => render_conns_tab(f, app, chunks[1]),
    }

    // Status bar
    let status_text = vec![
        Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(&app.status, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled(" ? ", Style::default().fg(Color::Cyan)),
            Span::raw("Help "),
            Span::styled(" q ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit "),
            Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
            Span::raw("Select "),
            Span::styled(" t ", Style::default().fg(Color::Cyan)),
            Span::raw("Test "),
            Span::styled(" r ", Style::default().fg(Color::Cyan)),
            Span::raw("Refresh "),
            Span::styled(" m ", Style::default().fg(Color::Cyan)),
            Span::raw("Mode "),
            Span::styled(" Tab ", Style::default().fg(Color::Cyan)),
            Span::raw("Switch Focus "),
        ]),
    ];

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title(" Status "));
    f.render_widget(status, chunks[2]);

    // Help popup
    if app.show_help {
        render_help_popup(f);
    }
}

fn render_proxies_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(area);

    // Groups list
    let group_items: Vec<ListItem> = app
        .groups
        .iter()
        .map(|name| {
            // 在名称前添加空格，增加 padding
            let content = Line::from(Span::raw(format!(" {}", name)));
            ListItem::new(content)
        })
        .collect();

    let groups_block = Block::default()
        .borders(Borders::ALL)
        .title(" Groups ")
        .border_style(if app.focus == 0 {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });

    let groups = List::new(group_items)
        .block(groups_block)
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 50)) // 浅灰背景
                .fg(Color::Cyan)            // 亮青文字
                .add_modifier(Modifier::BOLD), // 仅在选中时加粗
        )
        .highlight_symbol("▎"); // 使用竖条作为指示符，更简洁

    f.render_stateful_widget(groups, chunks[0], &mut app.group_state);

    // Proxies list
    let current_proxy = app.current_group.as_ref().and_then(|g| g.now.clone());
    let proxy_items: Vec<ListItem> = app
        .proxies
        .iter()
        .map(|(name, delay)| {
            let is_selected = current_proxy.as_ref() == Some(name);
            let delay_str = if *delay > 0 {
                format!("{}ms", delay)
            } else {
                "---".to_string()
            };

            let delay_color = if *delay < 0 {
                Color::DarkGray
            } else if *delay < 200 {
                Color::Green
            } else if *delay < 500 {
                Color::Yellow
            } else {
                Color::Red
            };

            let marker = if is_selected { "●" } else { " " }; // 移除圆点后的空格，由格式化控制
            let marker_color = if is_selected { Color::Green } else { Color::White };

            // 增加 name 字段的 padding，移除不必要的加粗
            let name_style = if is_selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let content = Line::from(vec![
                Span::styled(format!(" {} ", marker), Style::default().fg(marker_color)),
                Span::styled(
                    format!("{:<30} ", name), // 增加右侧空格
                    name_style,
                ),
                Span::styled(format!("{:>8} ", delay_str), Style::default().fg(delay_color)), // 增加右侧空格
            ]);
            ListItem::new(content)
        })
        .collect();

    let group_title = app
        .current_group
        .as_ref()
        .map(|g| format!(" {} ({}) ", g.name, g.proxy_type))
        .unwrap_or_else(|| " Proxies ".to_string());

    let proxies_block = Block::default()
        .borders(Borders::ALL)
        .title(group_title)
        .border_style(if app.focus == 1 {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        });

    let proxies = List::new(proxy_items)
        .block(proxies_block)
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 50))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▎");

    f.render_stateful_widget(proxies, chunks[1], &mut app.proxy_state);
}

fn render_rules_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let rule_items: Vec<ListItem> = app
        .rules
        .iter()
        .enumerate()
        .map(|(idx, rule)| {
            let content = Line::from(vec![
                Span::styled(
                    format!("{:>4} ", idx + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<15}", rule.rule_type),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("{:<40}", truncate_str(&rule.payload, 38)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!(" → {}", rule.proxy),
                    Style::default().fg(Color::Yellow),
                ),
            ]);
            ListItem::new(content)
        })
        .collect();

    let rules = List::new(rule_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Rules ({}) ", app.rules.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 50))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▎");

    f.render_stateful_widget(rules, area, &mut app.rule_state);
}

fn render_conns_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let conn_items: Vec<ListItem> = app
        .conns
        .iter()
        .map(|c| {
            let host_or_ip = if c.metadata.host.is_empty() {
                &c.metadata.destination_ip
            } else {
                &c.metadata.host
            };

            let chains = if c.chains.is_empty() {
                "DIRECT".to_string()
            } else {
                c.chains.iter().rev().cloned().collect::<Vec<_>>().join(" ← ")
            };
            
            let content = Line::from(vec![
                Span::styled(
                    format!("{:<20} ", c.metadata.source_ip),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<30} ", truncate_str(host_or_ip, 28)),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<30} ", truncate_str(&chains, 28)),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("↓{:<10} ↑{:<10}", format_bytes(c.download), format_bytes(c.upload)),
                    Style::default().fg(Color::Yellow),
                ),
            ]);
            ListItem::new(content)
        })
        .collect();

    let conns = List::new(conn_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Connections ({}) ", app.conns.len())),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 50))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▎");

    f.render_stateful_widget(conns, area, &mut app.conn_state);
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_speed(bytes_per_sec: u64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec))
}

fn render_help_popup(f: &mut Frame) {
    let area = centered_rect(60, 70, f.size());

    let help_text = vec![
        "",
        "  Navigation",
        "  ──────────────────────────────",
        "  ↑/k       Move up",
        "  ↓/j       Move down",
        "  ←/h       Prev tab / Focus groups",
        "  →/l       Next tab / Focus proxies",
        "  Tab       Switch focus",
        "  1-3       Switch tabs (Proxies/Rules/Conns)",
        "",
        "  Actions",
        "  ──────────────────────────────",
        "  Enter     Select proxy",
        "  t         Test delay for selected proxy",
        "  m         Switch mode (Rule/Global/Direct)",
        "  r         Refresh data",
        "",
        "  General",
        "  ──────────────────────────────",
        "  ?         Toggle this help",
        "  q/Esc     Quit",
        "",
    ];

    let help = Paragraph::new(help_text.join("\n"))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(Clear, area);
    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}…", &s[..max_len - 1])
    } else {
        s.to_string()
    }
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = ClashClient::new(&cli.controller, cli.secret.clone());

    // Start traffic monitoring thread
    let (traffic_tx, traffic_rx) = crossbeam_channel::unbounded();
    let controller = cli.controller.clone();
    let secret = cli.secret.clone();
    
    thread::spawn(move || {
        let ws_url = format!("ws://{}/traffic", controller);
        let ws_url = if let Some(s) = secret {
            format!("{}?token={}", ws_url, urlencoding::encode(&s))
        } else {
            ws_url
        };

        loop {
            if let Ok((mut socket, _)) = connect(&ws_url) {
                while let Ok(msg) = socket.read() {
                    if msg.is_text() || msg.is_binary() {
                        let data = msg.into_data();
                        if let Ok(traffic) = serde_json::from_slice::<Traffic>(&data) {
                            let _ = traffic_tx.send(traffic);
                        }
                    }
                }
            }
            // Retry delay
            thread::sleep(Duration::from_secs(3));
        }
    });

    match cli.command {
        Some(Commands::Proxies) => {
            let resp = client.get_proxies()?;
            for (name, proxy) in resp.proxies.iter() {
                if proxy.proxy_type == "Selector" || proxy.proxy_type == "URLTest" {
                    println!("\n[{}] ({})", name, proxy.proxy_type);
                    if let Some(ref now) = proxy.now {
                        println!("  Current: {}", now);
                    }
                    println!("  Available:");
                    for p in &proxy.all {
                        println!("    - {}", p);
                    }
                }
            }
        }
        Some(Commands::Rules) => {
            let resp = client.get_rules()?;
            println!("Rules ({}):", resp.rules.len());
            for (idx, rule) in resp.rules.iter().enumerate() {
                println!(
                    "{:4}. [{:<12}] {:<40} → {}",
                    idx + 1,
                    rule.rule_type,
                    rule.payload,
                    rule.proxy
                );
            }
        }
        Some(Commands::Groups) => {
            let resp = client.get_proxies()?;
            let groups: Vec<_> = resp
                .proxies
                .iter()
                .filter(|(_, p)| {
                    p.proxy_type == "Selector"
                        || p.proxy_type == "URLTest"
                        || p.proxy_type == "Fallback"
                })
                .collect();

            println!("Proxy Groups ({}):", groups.len());
            for (name, proxy) in groups {
                let current = proxy.now.as_deref().unwrap_or("N/A");
                println!(
                    "  [{:<12}] {:<20} → {}",
                    proxy.proxy_type, name, current
                );
            }
        }
        Some(Commands::Version) => {
            let resp = client.get_version()?;
            println!("Clash version: {}", resp.version);
        }
        Some(Commands::Tui) | None => {
            run_tui(client, traffic_rx)?;
        }
    }

    Ok(())
}

fn run_tui(client: ClashClient, traffic_rx: crossbeam_channel::Receiver<Traffic>) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(client, traffic_rx);

    // Main loop
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if app.show_help {
                                app.show_help = false;
                            } else {
                                break;
                            }
                        }
                        KeyCode::Char('?') => {
                            app.show_help = !app.show_help;
                        }
                        KeyCode::Char('1') => {
                            app.current_tab = Tab::Proxies;
                            app.refresh_data();
                        }
                        KeyCode::Char('2') => {
                            app.current_tab = Tab::Rules;
                            app.refresh_data();
                        }
                        KeyCode::Char('3') => {
                            app.current_tab = Tab::Conns;
                            app.refresh_data();
                        }
                        KeyCode::Tab => {
                            if app.show_help {
                                continue;
                            }
                            app.toggle_focus();
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            if app.show_help {
                                continue;
                            }
                            // 如果在 Proxies Tab 且 Focus 在右侧（代理列表），左键回到组列表
                            // 否则切换到上一个 Tab
                            if app.current_tab == Tab::Proxies && app.focus == 1 {
                                app.focus = 0;
                            } else {
                                app.prev_tab();
                            }
                        }
                        KeyCode::Right | KeyCode::Char('l') => {
                            if app.show_help {
                                continue;
                            }
                            // 如果在 Proxies Tab 且 Focus 在左侧（组列表），右键进入代理列表
                            // 否则切换到下一个 Tab
                            if app.current_tab == Tab::Proxies && app.focus == 0 {
                                app.focus = 1;
                            } else {
                                app.next_tab();
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if app.show_help {
                                continue;
                            }
                            app.move_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if app.show_help {
                                continue;
                            }
                            app.move_down();
                        }
                        KeyCode::Enter => {
                            if app.show_help {
                                app.show_help = false;
                                continue;
                            }
                            if app.current_tab == Tab::Proxies && app.focus == 1 {
                                app.select_proxy();
                            }
                        }
                        KeyCode::Char('t') => {
                            if app.show_help {
                                continue;
                            }
                            if app.current_tab == Tab::Proxies && app.focus == 1 {
                                app.test_selected_delay();
                            }
                        }
                        KeyCode::Char('r') => {
                            if app.show_help {
                                continue;
                            }
                            app.refresh_data();
                        }
                        KeyCode::Char('m') => {
                            if app.show_help {
                                continue;
                            }
                            app.toggle_mode();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
