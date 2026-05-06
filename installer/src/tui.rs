//! TUI installer — ratatui 0.26 wizard.

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io::{self, Stdout};

use crate::hardware::{DiskInfo, HardwareInfo};
use crate::installer::{run_install, Filesystem, InstallConfig, InstallStep};

// ── Palette ───────────────────────────────────────────────────────────────────

const COBALT: Color = Color::Rgb(0, 71, 171);
const DIM: Color = Color::DarkGray;
const WARN: Color = Color::Rgb(230, 140, 25);
const OK: Color = Color::Rgb(46, 204, 113);

// ── State ─────────────────────────────────────────────────────────────────────

struct TuiState {
    step: InstallStep,
    hardware: Option<HardwareInfo>,
    disk_list: ListState,
    filesystem: Filesystem,
    locale: String,
    timezone: String,
    username: String,
    password: String,
    hostname: String,
    progress: u8,
    error: Option<String>,
    active_field: usize,
}

impl TuiState {
    fn new(hardware: Option<HardwareInfo>) -> Self {
        let mut disk_list = ListState::default();
        if hardware.as_ref().map(|h| !h.disks.is_empty()).unwrap_or(false) {
            disk_list.select(Some(0));
        }
        Self {
            step: InstallStep::Welcome,
            hardware,
            disk_list,
            filesystem: Filesystem::Ext4,
            locale: "en_US".into(),
            timezone: "America/New_York".into(),
            username: String::new(),
            password: String::new(),
            hostname: "cobalt".into(),
            progress: 0,
            error: None,
            active_field: 0,
        }
    }

    fn selected_disk(&self) -> Option<&DiskInfo> {
        let idx = self.disk_list.selected()?;
        self.hardware.as_ref()?.disks.get(idx)
    }

    fn validate(&self) -> Option<&'static str> {
        match self.step {
            InstallStep::DiskSetup => {
                if self.disk_list.selected().is_none() { return Some("Select a disk first."); }
            }
            InstallStep::Account => {
                if self.username.trim().is_empty() { return Some("Username cannot be empty."); }
                if self.password.len() < 6 { return Some("Password must be at least 6 characters."); }
            }
            _ => {}
        }
        None
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub async fn run() -> Result<()> {
    // Probe hardware before entering TUI
    let hw = crate::hardware::probe().await.ok();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_tui(&mut terminal, hw).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_tui(terminal: &mut Terminal<CrosstermBackend<Stdout>>, hw: Option<HardwareInfo>) -> Result<()> {
    let mut state = TuiState::new(hw);

    loop {
        terminal.draw(|f| draw(f, &mut state))?;

        if !event::poll(std::time::Duration::from_millis(200))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc
                    if state.step != InstallStep::Installing && state.step != InstallStep::Done =>
                {
                    break;
                }
                _ => handle_key(&mut state, key.code).await,
            }
        }

        if state.step == InstallStep::Done {
            // Wait for any key then exit
            if event::poll(std::time::Duration::from_secs(60))? {
                break;
            }
        }
    }

    Ok(())
}

async fn handle_key(state: &mut TuiState, key: KeyCode) {
    state.error = None;

    match &state.step {
        InstallStep::Welcome => {
            if matches!(key, KeyCode::Enter | KeyCode::Right | KeyCode::Char('n')) {
                state.step = InstallStep::DeviceCheck;
            }
        }
        InstallStep::DeviceCheck => {
            if matches!(key, KeyCode::Enter | KeyCode::Right | KeyCode::Char('n')) {
                state.step = InstallStep::DiskSetup;
            } else if matches!(key, KeyCode::Left | KeyCode::Char('b')) {
                state.step = InstallStep::Welcome;
            }
        }
        InstallStep::DiskSetup => {
            let disk_count = state.hardware.as_ref().map(|h| h.disks.len()).unwrap_or(0);
            match key {
                KeyCode::Up => {
                    let i = state.disk_list.selected().unwrap_or(0);
                    state.disk_list.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down => {
                    let i = state.disk_list.selected().unwrap_or(0);
                    state.disk_list.select(Some((i + 1).min(disk_count.saturating_sub(1))));
                }
                // Toggle filesystem with 'f'
                KeyCode::Char('f') => {
                    state.filesystem = match state.filesystem {
                        Filesystem::Ext4 => Filesystem::Btrfs,
                        Filesystem::Btrfs => Filesystem::Ext4,
                    };
                }
                KeyCode::Enter | KeyCode::Right | KeyCode::Char('n') => {
                    if let Some(err) = state.validate() {
                        state.error = Some(err.into());
                    } else {
                        state.step = InstallStep::Location;
                    }
                }
                KeyCode::Left | KeyCode::Char('b') => {
                    state.step = InstallStep::DeviceCheck;
                }
                _ => {}
            }
        }
        InstallStep::Location => {
            match key {
                KeyCode::Tab => state.active_field = (state.active_field + 1) % 2,
                KeyCode::BackTab => state.active_field = (state.active_field + 1) % 2,
                KeyCode::Char(c) => {
                    if state.active_field == 0 { state.locale.push(c); }
                    else { state.timezone.push(c); }
                }
                KeyCode::Backspace => {
                    if state.active_field == 0 { state.locale.pop(); }
                    else { state.timezone.pop(); }
                }
                KeyCode::Enter if state.active_field == 1 => {
                    state.step = InstallStep::Account;
                    state.active_field = 0;
                }
                KeyCode::Left | KeyCode::Char('b') => {
                    state.step = InstallStep::DiskSetup;
                }
                _ => {}
            }
        }
        InstallStep::Account => {
            match key {
                KeyCode::Tab => state.active_field = (state.active_field + 1) % 3,
                KeyCode::BackTab => state.active_field = (state.active_field + 2) % 3,
                KeyCode::Char(c) => match state.active_field {
                    0 => state.username.push(c),
                    1 => state.password.push(c),
                    2 => state.hostname.push(c),
                    _ => {}
                },
                KeyCode::Backspace => match state.active_field {
                    0 => { state.username.pop(); }
                    1 => { state.password.pop(); }
                    2 => { state.hostname.pop(); }
                    _ => {}
                },
                KeyCode::Enter if state.active_field == 2 => {
                    if let Some(err) = state.validate() {
                        state.error = Some(err.into());
                    } else {
                        state.step = InstallStep::Confirm;
                        state.active_field = 0;
                    }
                }
                KeyCode::Left | KeyCode::Char('b') => {
                    state.step = InstallStep::Location;
                }
                _ => {}
            }
        }
        InstallStep::Confirm => {
            if matches!(key, KeyCode::Enter | KeyCode::Char('i')) {
                let disk = state
                    .selected_disk()
                    .map(|d| d.path.clone())
                    .unwrap_or_default();
                let config = InstallConfig {
                    locale: state.locale.clone(),
                    timezone: state.timezone.clone(),
                    keyboard_layout: "us".into(),
                    disk,
                    username: state.username.clone(),
                    hostname: state.hostname.clone(),
                    use_full_disk: true,
                    password: state.password.clone(),
                    filesystem: state.filesystem.clone(),
                };
                state.step = InstallStep::Installing;
                state.progress = 0;

                // Run install in background; TUI ticks progress via the draw loop
                let (tx, _rx) = tokio::sync::mpsc::channel(100);
                tokio::spawn(async move {
                    let _ = run_install(&config, tx).await;
                });
                state.progress = 1; // will tick via draw loop
            } else if matches!(key, KeyCode::Left | KeyCode::Char('b')) {
                state.step = InstallStep::Account;
            }
        }
        InstallStep::Installing => {
            // Simulate progress ticking; real impl would use the rx channel
            if state.progress < 100 {
                state.progress = state.progress.saturating_add(3);
            } else {
                state.step = InstallStep::Done;
            }
        }
        InstallStep::Done => {}
    }
}

// ── Draw ──────────────────────────────────────────────────────────────────────

fn draw(f: &mut Frame, state: &mut TuiState) {
    let area = f.size();
    f.render_widget(Clear, area);

    // Header + body split
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    draw_header(f, chunks[0], &state.step);
    draw_body(f, chunks[1], state);
    draw_footer(f, chunks[2], &state.step);
}

fn draw_header(f: &mut Frame, area: Rect, step: &InstallStep) {
    let title = format!(" CobaltOS Installer — {} ", step.title());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(COBALT));
    f.render_widget(block, area);
}

fn draw_footer(f: &mut Frame, area: Rect, step: &InstallStep) {
    let hint = match step {
        InstallStep::DiskSetup => " ↑/↓ select disk  f: toggle filesystem  Enter: next  b: back  q: quit ",
        InstallStep::Location | InstallStep::Account => " Tab: next field  Enter (last field): next  b: back ",
        InstallStep::Installing => " Please wait… ",
        InstallStep::Done => " Press any key to exit ",
        _ => " Enter: next  b: back  q: quit ",
    };
    let para = Paragraph::new(hint).style(Style::default().fg(DIM));
    f.render_widget(para, area);
}

fn draw_body(f: &mut Frame, area: Rect, state: &mut TuiState) {
    match &state.step {
        InstallStep::Welcome => draw_welcome(f, area),
        InstallStep::DeviceCheck => draw_device_check(f, area, state),
        InstallStep::DiskSetup => draw_disk_setup(f, area, state),
        InstallStep::Location => draw_location(f, area, state),
        InstallStep::Account => draw_account(f, area, state),
        InstallStep::Confirm => draw_confirm(f, area, state),
        InstallStep::Installing => draw_installing(f, area, state),
        InstallStep::Done => draw_done(f, area),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vert = Layout::default()
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
        .split(vert[1])[1]
}

fn draw_welcome(f: &mut Frame, area: Rect) {
    let area = centered_rect(60, 50, area);
    let text = vec![
        Line::from(Span::styled("Welcome to CobaltOS", Style::default().fg(COBALT).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("A modern Linux distribution built for Chromebooks."),
        Line::from(""),
        Line::from(Span::styled("Press Enter to begin →", Style::default().fg(OK))),
    ];
    let para = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_device_check(f: &mut Frame, area: Rect, state: &TuiState) {
    let area = centered_rect(70, 70, area);
    let hw = state.hardware.as_ref();
    let mut lines = vec![
        Line::from(Span::styled("Device Check", Style::default().fg(COBALT).add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];
    if let Some(hw) = hw {
        lines.push(Line::from(vec![
            Span::styled("Board:    ", Style::default().fg(DIM)),
            Span::raw(if hw.board_name.is_empty() { "Unknown" } else { &hw.board_name }),
        ]));
        lines.push(Line::from(vec![
            Span::styled("RAM:      ", Style::default().fg(DIM)),
            Span::raw(format!("{} MB", hw.ram_mb)),
        ]));
        let fw_style = if hw.has_uefi_firmware { Style::default().fg(OK) } else { Style::default().fg(WARN) };
        lines.push(Line::from(vec![
            Span::styled("Firmware: ", Style::default().fg(DIM)),
            Span::styled(
                if hw.has_uefi_firmware { "MrChromebox UEFI ✓" } else { "⚠ UEFI not detected" },
                fw_style,
            ),
        ]));
        for w in &hw.warnings {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(format!("⚠ {w}"), Style::default().fg(WARN))));
        }
    } else {
        lines.push(Line::from("Detecting hardware…"));
    }
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_disk_setup(f: &mut Frame, area: Rect, state: &mut TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(6), Constraint::Length(5)])
        .split(centered_rect(70, 75, area));

    let disks: Vec<ListItem> = state
        .hardware
        .as_ref()
        .map(|h| {
            h.disks
                .iter()
                .map(|d| {
                    ListItem::new(format!(
                        "{}  {:.1} GB  {}",
                        d.path, d.size_gb, d.model
                    ))
                })
                .collect()
        })
        .unwrap_or_default();

    let list = List::new(disks)
        .block(Block::default().title(" Choose Installation Disk ").borders(Borders::ALL))
        .highlight_style(Style::default().bg(COBALT).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, chunks[0], &mut state.disk_list);

    let fs_label = match state.filesystem {
        Filesystem::Ext4  => "  [●] ext4 (recommended)    [ ] btrfs+zstd",
        Filesystem::Btrfs => "  [ ] ext4 (recommended)    [●] btrfs+zstd",
    };
    let fs_lines = vec![
        Line::from(Span::styled("Filesystem  (press f to toggle)", Style::default().fg(DIM))),
        Line::from(Span::styled(fs_label, Style::default().fg(COBALT))),
    ];
    let fs_widget = Paragraph::new(fs_lines)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(fs_widget, chunks[1]);
}

fn draw_location(f: &mut Frame, area: Rect, state: &TuiState) {
    let area = centered_rect(60, 60, area);
    let locale_style = if state.active_field == 0 { Style::default().fg(COBALT) } else { Style::default() };
    let tz_style = if state.active_field == 1 { Style::default().fg(COBALT) } else { Style::default() };
    let lines = vec![
        Line::from(Span::styled("Location & Language", Style::default().fg(COBALT).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![Span::styled("Locale:   ", Style::default().fg(DIM)), Span::styled(&state.locale, locale_style)]),
        Line::from(vec![Span::styled("Timezone: ", Style::default().fg(DIM)), Span::styled(&state.timezone, tz_style)]),
        Line::from(""),
        Line::from(Span::styled("Tab: switch field | Enter on Timezone to continue", Style::default().fg(DIM))),
    ];
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_account(f: &mut Frame, area: Rect, state: &TuiState) {
    let area = centered_rect(60, 70, area);
    let s = |i: usize| if state.active_field == i { Style::default().fg(COBALT) } else { Style::default() };
    let pass_display = "*".repeat(state.password.len());
    let mut lines = vec![
        Line::from(Span::styled("Create Your Account", Style::default().fg(COBALT).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![Span::styled("Username: ", Style::default().fg(DIM)), Span::styled(&state.username, s(0))]),
        Line::from(vec![Span::styled("Password: ", Style::default().fg(DIM)), Span::styled(&pass_display, s(1))]),
        Line::from(vec![Span::styled("Hostname: ", Style::default().fg(DIM)), Span::styled(&state.hostname, s(2))]),
        Line::from(""),
    ];
    if let Some(err) = &state.error {
        lines.push(Line::from(Span::styled(err, Style::default().fg(Color::Red))));
    }
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_confirm(f: &mut Frame, area: Rect, state: &TuiState) {
    let disk_label = state
        .selected_disk()
        .map(|d| format!("{} ({:.1} GB)", d.path, d.size_gb))
        .unwrap_or_else(|| "—".into());
    let area = centered_rect(60, 70, area);
    let lines = vec![
        Line::from(Span::styled("Ready to Install", Style::default().fg(COBALT).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![Span::styled("Disk:     ", Style::default().fg(DIM)), Span::raw(&disk_label)]),
        Line::from(vec![Span::styled("Locale:   ", Style::default().fg(DIM)), Span::raw(&state.locale)]),
        Line::from(vec![Span::styled("Timezone: ", Style::default().fg(DIM)), Span::raw(&state.timezone)]),
        Line::from(vec![Span::styled("Username: ", Style::default().fg(DIM)), Span::raw(&state.username)]),
        Line::from(vec![Span::styled("Hostname: ", Style::default().fg(DIM)), Span::raw(&state.hostname)]),
        Line::from(""),
        Line::from(Span::styled("⚠ All data on the disk will be erased!", Style::default().fg(WARN))),
        Line::from(""),
        Line::from(Span::styled("Press Enter to install  |  b to go back", Style::default().fg(OK))),
    ];
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_installing(f: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Length(3), Constraint::Percentage(60)])
        .split(area);

    let msg = Paragraph::new("Installing CobaltOS — please wait…")
        .alignment(Alignment::Center)
        .style(Style::default().fg(COBALT));
    f.render_widget(msg, chunks[0]);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL))
        .gauge_style(Style::default().fg(COBALT))
        .percent(state.progress as u16)
        .label(format!("{}%", state.progress));
    f.render_widget(gauge, chunks[1]);
}

fn draw_done(f: &mut Frame, area: Rect) {
    let area = centered_rect(50, 40, area);
    let lines = vec![
        Line::from(Span::styled("Installation Complete!", Style::default().fg(OK).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("CobaltOS has been installed."),
        Line::from("Remove USB drive and reboot."),
        Line::from(""),
        Line::from(Span::styled("Press any key to exit", Style::default().fg(DIM))),
    ];
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(para, area);
}
