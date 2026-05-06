//! GUI installer — iced 0.12 Application.

use iced::executor;
use iced::widget::{
    button, column, container, progress_bar, radio, row, scrollable,
    text, text_input, Column, Space,
};
use iced::{
    Alignment, Application, Color, Command, Element, Length,
    Padding, Settings, Subscription, Theme,
};

use crate::hardware::HardwareInfo;
use crate::installer::{run_install, Filesystem, InstallConfig, InstallStep};

pub fn run() -> anyhow::Result<()> {
    InstallerApp::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(900.0, 640.0),
            min_size: Some(iced::Size::new(800.0, 560.0)),
            ..Default::default()
        },
        ..Default::default()
    })
    .map_err(|e| anyhow::anyhow!("{e}"))
}

// ── State ────────────────────────────────────────────────────────────────────

#[derive(Default)]
struct InstallerApp {
    step: InstallStep,
    hardware: Option<HardwareInfo>,
    progress: u8,
    error: Option<String>,

    // DiskSetup
    disk_idx: Option<usize>,
    filesystem: Filesystem,

    // Location
    locale: String,
    timezone: String,

    // Account
    username: String,
    password: String,
    hostname: String,

    // Install state
    install_tick: u8,
    install_done: bool,
}

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Message {
    Next,
    Back,
    HardwareProbed(Result<HardwareInfo, String>),
    DiskSelected(usize),
    FilesystemChanged(Filesystem),
    LocaleChanged(String),
    TimezoneChanged(String),
    UsernameChanged(String),
    PasswordChanged(String),
    HostnameChanged(String),
    Tick,
    InstallDone(Result<(), String>),
    Reboot,
}

// ── iced Application ─────────────────────────────────────────────────────────

impl Application for InstallerApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut app = Self {
            locale: "en_US".into(),
            timezone: "America/New_York".into(),
            hostname: "cobalt".into(),
            ..Default::default()
        };
        // Start hardware probe immediately
        let cmd = Command::perform(
            async { crate::hardware::probe().await.map_err(|e| e.to_string()) },
            Message::HardwareProbed,
        );
        (app, cmd)
    }

    fn title(&self) -> String {
        format!("CobaltOS Installer — {}", self.step.title())
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::HardwareProbed(result) => {
                match result {
                    Ok(hw) => self.hardware = Some(hw),
                    Err(e) => self.error = Some(format!("Hardware detection failed: {e}")),
                }
                Command::none()
            }

            Message::Next => {
                if let Some(next) = self.step.next() {
                    // Validate before advancing
                    if let Some(err) = self.validate_current() {
                        self.error = Some(err);
                        return Command::none();
                    }
                    self.error = None;

                    // Apply step-specific transitions
                    if self.step == InstallStep::DiskSetup {
                        if let Some(idx) = self.disk_idx {
                            if let Some(hw) = &self.hardware {
                                if let Some(disk) = hw.disks.get(idx) {
                                    // will be set on Confirm step via config
                                    let _ = disk;
                                }
                            }
                        }
                    }

                    if self.step == InstallStep::Confirm {
                        // Build config and start install
                        let disk = self
                            .disk_idx
                            .and_then(|i| self.hardware.as_ref()?.disks.get(i))
                            .map(|d| d.path.clone())
                            .unwrap_or_default();

                        let config = InstallConfig {
                            locale: self.locale.clone(),
                            timezone: self.timezone.clone(),
                            keyboard_layout: "us".into(),
                            disk,
                            username: self.username.clone(),
                            hostname: self.hostname.clone(),
                            use_full_disk: true,
                            password: self.password.clone(),
                            filesystem: self.filesystem.clone(),
                        };

                        self.step = InstallStep::Installing;
                        return Command::perform(
                            async move {
                                let (tx, _rx) = tokio::sync::mpsc::channel(100);
                                run_install(&config, tx)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            Message::InstallDone,
                        );
                    }

                    self.step = next;
                }
                Command::none()
            }

            Message::Back => {
                self.error = None;
                let steps = InstallStep::all();
                let idx = self.step.index();
                if idx > 0 {
                    self.step = steps[idx - 1].clone();
                }
                Command::none()
            }

            Message::DiskSelected(idx) => {
                self.disk_idx = Some(idx);
                Command::none()
            }
            Message::FilesystemChanged(fs) => { self.filesystem = fs; Command::none() }
            Message::LocaleChanged(v) => { self.locale = v; Command::none() }
            Message::TimezoneChanged(v) => { self.timezone = v; Command::none() }
            Message::UsernameChanged(v) => { self.username = v; Command::none() }
            Message::PasswordChanged(v) => { self.password = v; Command::none() }
            Message::HostnameChanged(v) => { self.hostname = v; Command::none() }

            Message::Tick => {
                if self.step == InstallStep::Installing && !self.install_done {
                    self.install_tick = self.install_tick.saturating_add(1);
                    self.progress = (self.install_tick.min(95)) as u8;
                }
                Command::none()
            }

            Message::InstallDone(result) => {
                self.install_done = true;
                self.progress = 100;
                match result {
                    Ok(()) => self.step = InstallStep::Done,
                    Err(e) => self.error = Some(format!("Installation failed: {e}")),
                }
                Command::none()
            }

            Message::Reboot => {
                let _ = std::process::Command::new("reboot").spawn();
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.step == InstallStep::Installing && !self.install_done {
            iced::time::every(std::time::Duration::from_millis(300))
                .map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn theme(&self) -> Theme {
        Theme::custom(
            "CobaltOS".into(),
            iced::theme::Palette {
                background: Color::from_rgb(0.067, 0.075, 0.094),
                text: Color::WHITE,
                primary: Color::from_rgb(0.0, 0.278, 0.671),
                success: Color::from_rgb(0.18, 0.8, 0.44),
                danger: Color::from_rgb(0.9, 0.2, 0.2),
            },
        )
    }

    fn view(&self) -> Element<Message> {
        let sidebar = self.sidebar();
        let content = self.step_view();

        let layout = row![sidebar, content]
            .width(Length::Fill)
            .height(Length::Fill);

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(DarkBg)))
            .into()
    }
}

// ── Validation ────────────────────────────────────────────────────────────────

impl InstallerApp {
    fn validate_current(&self) -> Option<String> {
        match self.step {
            InstallStep::DiskSetup => {
                if self.disk_idx.is_none() {
                    return Some("Please select an installation disk.".into());
                }
            }
            InstallStep::Account => {
                if self.username.trim().is_empty() {
                    return Some("Username cannot be empty.".into());
                }
                if self.password.len() < 6 {
                    return Some("Password must be at least 6 characters.".into());
                }
                if self.hostname.trim().is_empty() {
                    return Some("Hostname cannot be empty.".into());
                }
            }
            _ => {}
        }
        None
    }
}

// ── Sidebar ───────────────────────────────────────────────────────────────────

impl InstallerApp {
    fn sidebar(&self) -> Element<Message> {
        let current_idx = self.step.index();

        let steps: Vec<Element<Message>> = InstallStep::all()
            .iter()
            .filter(|s| **s != InstallStep::Installing)
            .map(|s| {
                let idx = s.index();
                let is_current = *s == self.step;
                let is_done = idx < current_idx;

                let indicator = if is_done {
                    text("✓").size(14).style(iced::theme::Text::Color(
                        Color::from_rgb(0.18, 0.8, 0.44),
                    ))
                } else if is_current {
                    text("●").size(14).style(iced::theme::Text::Color(
                        Color::from_rgb(0.0, 0.278, 0.671),
                    ))
                } else {
                    text("○").size(14).style(iced::theme::Text::Color(
                        Color::from_rgba(1.0, 1.0, 1.0, 0.3),
                    ))
                };

                let label = text(s.title())
                    .size(13)
                    .style(if is_current {
                        iced::theme::Text::Color(Color::WHITE)
                    } else if is_done {
                        iced::theme::Text::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.7))
                    } else {
                        iced::theme::Text::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.35))
                    });

                row![indicator, Space::with_width(10), label]
                    .align_items(Alignment::Center)
                    .into()
            })
            .collect();

        let col = Column::with_children(steps).spacing(16).padding(Padding::from([32, 24]));

        let brand = column![
            text("CobaltOS").size(20).style(iced::theme::Text::Color(
                Color::from_rgb(0.0, 0.278, 0.671)
            )),
            text("Installer").size(13).style(iced::theme::Text::Color(
                Color::from_rgba(1.0, 1.0, 1.0, 0.5)
            )),
        ]
        .padding(Padding::from([32, 24, 16, 24]));

        container(column![brand, col])
            .width(200)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(SidebarBg)))
            .into()
    }
}

// ── Step views ────────────────────────────────────────────────────────────────

impl InstallerApp {
    fn step_view(&self) -> Element<Message> {
        let content: Element<Message> = match &self.step {
            InstallStep::Welcome => self.view_welcome(),
            InstallStep::DeviceCheck => self.view_device_check(),
            InstallStep::DiskSetup => self.view_disk_setup(),
            InstallStep::Location => self.view_location(),
            InstallStep::Account => self.view_account(),
            InstallStep::Confirm => self.view_confirm(),
            InstallStep::Installing => self.view_installing(),
            InstallStep::Done => self.view_done(),
        };

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(Padding::from([40, 48]))
            .into()
    }

    fn view_welcome(&self) -> Element<Message> {
        let hw_status = if self.hardware.is_some() {
            text("✓ Hardware detected").size(13).style(iced::theme::Text::Color(
                Color::from_rgb(0.18, 0.8, 0.44),
            ))
        } else {
            text("Detecting hardware…").size(13).style(iced::theme::Text::Color(
                Color::from_rgba(1.0, 1.0, 1.0, 0.5),
            ))
        };

        column![
            Space::with_height(40),
            text("Welcome to CobaltOS").size(32),
            Space::with_height(12),
            text("A modern Linux distribution built for Chromebooks.")
                .size(16)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.65))),
            Space::with_height(32),
            hw_status,
            Space::with_height(48),
            self.nav_buttons(false, self.hardware.is_some(), "Next →"),
        ]
        .spacing(0)
        .into()
    }

    fn view_device_check(&self) -> Element<Message> {
        let hw = match &self.hardware {
            Some(h) => h,
            None => {
                return column![text("Detecting hardware…").size(16)].into();
            }
        };

        let board_row = info_row("Board", if hw.board_name.is_empty() { "Unknown".into() } else { hw.board_name.clone() });
        let ram_row = info_row("RAM", format!("{} MB", hw.ram_mb));
        let firmware_row = info_row(
            "Firmware",
            if hw.has_uefi_firmware { "MrChromebox UEFI ✓" } else { "⚠ UEFI firmware not detected" },
        );

        let warnings: Vec<Element<Message>> = hw.warnings.iter().map(|w| {
            container(
                text(format!("⚠ {w}")).size(13),
            )
            .padding(12)
            .style(iced::theme::Container::Custom(Box::new(WarnBox)))
            .width(Length::Fill)
            .into()
        }).collect();

        let mut col = column![
            text("Device Check").size(28),
            Space::with_height(24),
            board_row,
            Space::with_height(8),
            ram_row,
            Space::with_height(8),
            firmware_row,
            Space::with_height(16),
        ]
        .spacing(0);

        for w in warnings {
            col = col.push(w);
            col = col.push(Space::with_height(8));
        }

        col = col.push(Space::with_height(32));
        col = col.push(self.nav_buttons(true, true, "Next →"));

        col.into()
    }

    fn view_disk_setup(&self) -> Element<Message> {
        let disks = self
            .hardware
            .as_ref()
            .map(|h| h.disks.as_slice())
            .unwrap_or(&[]);

        let disk_list: Vec<Element<Message>> = disks
            .iter()
            .enumerate()
            .map(|(i, disk)| {
                let label = format!(
                    "{}  ({:.1} GB)  {}{}",
                    disk.path,
                    disk.size_gb,
                    disk.model,
                    if disk.removable { "  [removable]" } else { "" }
                );
                radio(label, i, self.disk_idx, Message::DiskSelected)
                    .size(16)
                    .into()
            })
            .collect();

        let disk_col = if disk_list.is_empty() {
            column![text("No suitable disks found. Check your drive connections.").size(14)]
        } else {
            Column::with_children(disk_list).spacing(12)
        };

        column![
            text("Choose Installation Disk").size(28),
            Space::with_height(8),
            text("⚠ ALL data on the selected disk will be erased.")
                .size(13)
                .style(iced::theme::Text::Color(Color::from_rgb(0.9, 0.55, 0.1))),
            Space::with_height(24),
            disk_col,
            Space::with_height(24),
            text("Filesystem").size(16),
            Space::with_height(8),
            radio("ext4  (recommended — fast, reliable)", Filesystem::Ext4, Some(self.filesystem.clone()), Message::FilesystemChanged)
                .size(16),
            Space::with_height(8),
            radio("btrfs  (snapshots + zstd compression)", Filesystem::Btrfs, Some(self.filesystem.clone()), Message::FilesystemChanged)
                .size(16),
            Space::with_height(16),
            self.error_text(),
            Space::with_height(32),
            self.nav_buttons(true, !disks.is_empty(), "Next →"),
        ]
        .spacing(0)
        .into()
    }

    fn view_location(&self) -> Element<Message> {
        column![
            text("Location & Language").size(28),
            Space::with_height(24),
            text("Locale").size(13).style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.6))),
            Space::with_height(6),
            text_input("e.g. en_US", &self.locale)
                .on_input(Message::LocaleChanged)
                .padding(10)
                .size(15),
            Space::with_height(16),
            text("Timezone").size(13).style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.6))),
            Space::with_height(6),
            text_input("e.g. America/New_York", &self.timezone)
                .on_input(Message::TimezoneChanged)
                .padding(10)
                .size(15),
            Space::with_height(32),
            self.nav_buttons(true, true, "Next →"),
        ]
        .spacing(0)
        .into()
    }

    fn view_account(&self) -> Element<Message> {
        column![
            text("Create Your Account").size(28),
            Space::with_height(24),
            text("Username").size(13).style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.6))),
            Space::with_height(6),
            text_input("lowercase, no spaces", &self.username)
                .on_input(Message::UsernameChanged)
                .padding(10)
                .size(15),
            Space::with_height(16),
            text("Password").size(13).style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.6))),
            Space::with_height(6),
            text_input("min. 6 characters", &self.password)
                .on_input(Message::PasswordChanged)
                .secure(true)
                .padding(10)
                .size(15),
            Space::with_height(16),
            text("Hostname").size(13).style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.6))),
            Space::with_height(6),
            text_input("your computer's name", &self.hostname)
                .on_input(Message::HostnameChanged)
                .padding(10)
                .size(15),
            Space::with_height(16),
            self.error_text(),
            Space::with_height(32),
            self.nav_buttons(true, true, "Next →"),
        ]
        .spacing(0)
        .into()
    }

    fn view_confirm(&self) -> Element<Message> {
        let disk_label = self
            .disk_idx
            .and_then(|i| self.hardware.as_ref()?.disks.get(i))
            .map(|d| format!("{} ({:.1} GB)", d.path, d.size_gb))
            .unwrap_or_else(|| "—".into());

        column![
            text("Ready to Install").size(28),
            Space::with_height(24),
            info_row("Disk", disk_label),
            Space::with_height(8),
            info_row("Locale", self.locale.clone()),
            Space::with_height(8),
            info_row("Timezone", self.timezone.clone()),
            Space::with_height(8),
            info_row("Username", self.username.clone()),
            Space::with_height(8),
            info_row("Hostname", self.hostname.clone()),
            Space::with_height(24),
            container(
                text("This will erase all data on the selected disk and install CobaltOS.")
                    .size(13)
            )
            .padding(12)
            .style(iced::theme::Container::Custom(Box::new(WarnBox)))
            .width(Length::Fill),
            Space::with_height(32),
            self.nav_buttons(true, true, "Install Now"),
        ]
        .spacing(0)
        .into()
    }

    fn view_installing(&self) -> Element<Message> {
        let pct = self.progress as f32 / 100.0;
        let label = if self.install_done {
            "Installation complete!".into()
        } else {
            format!("Installing… {}%", self.progress)
        };

        column![
            Space::with_height(60),
            text("Installing CobaltOS").size(28),
            Space::with_height(8),
            text("Please do not power off your computer.")
                .size(14)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.55))),
            Space::with_height(40),
            progress_bar(0.0..=1.0, pct).height(12),
            Space::with_height(12),
            text(label).size(14),
        ]
        .spacing(0)
        .align_items(Alignment::Center)
        .into()
    }

    fn view_done(&self) -> Element<Message> {
        column![
            Space::with_height(60),
            text("Installation Complete!").size(32),
            Space::with_height(16),
            text("CobaltOS has been installed successfully.")
                .size(15)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.65))),
            Space::with_height(8),
            text("Remove your USB drive and click Reboot.")
                .size(14)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.5))),
            Space::with_height(48),
            button(text("Reboot Now").size(15))
                .on_press(Message::Reboot)
                .padding(Padding::from([12, 32]))
                .style(iced::theme::Button::Primary),
        ]
        .spacing(0)
        .align_items(Alignment::Center)
        .into()
    }

    fn nav_buttons(&self, show_back: bool, next_enabled: bool, next_label: &str) -> Element<Message> {
        let next_btn = if next_enabled {
            button(text(next_label).size(14))
                .on_press(Message::Next)
                .padding(Padding::from([10, 28]))
                .style(iced::theme::Button::Primary)
        } else {
            button(text(next_label).size(14))
                .padding(Padding::from([10, 28]))
                .style(iced::theme::Button::Secondary)
        };

        if show_back {
            row![
                button(text("← Back").size(14))
                    .on_press(Message::Back)
                    .padding(Padding::from([10, 20]))
                    .style(iced::theme::Button::Text),
                Space::with_width(Length::Fill),
                next_btn,
            ]
            .into()
        } else {
            row![Space::with_width(Length::Fill), next_btn].into()
        }
    }

    fn error_text(&self) -> Element<Message> {
        if let Some(err) = &self.error {
            text(err)
                .size(13)
                .style(iced::theme::Text::Color(Color::from_rgb(0.9, 0.3, 0.3)))
                .into()
        } else {
            Space::with_height(0).into()
        }
    }
}

// ── Helper functions ──────────────────────────────────────────────────────────

fn info_row(label: &str, value: impl ToString) -> Element<'static, Message> {
    row![
        text(format!("{label}:"))
            .size(13)
            .width(120)
            .style(iced::theme::Text::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.55))),
        text(value.to_string()).size(13),
    ]
    .align_items(Alignment::Center)
    .into()
}

// ── Custom container styles ───────────────────────────────────────────────────

struct DarkBg;
impl iced::widget::container::StyleSheet for DarkBg {
    type Style = Theme;
    fn appearance(&self, _theme: &Theme) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.067, 0.075, 0.094))),
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}

struct SidebarBg;
impl iced::widget::container::StyleSheet for SidebarBg {
    type Style = Theme;
    fn appearance(&self, _theme: &Theme) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgb(0.05, 0.056, 0.071))),
            text_color: Some(Color::WHITE),
            ..Default::default()
        }
    }
}

struct WarnBox;
impl iced::widget::container::StyleSheet for WarnBox {
    type Style = Theme;
    fn appearance(&self, _theme: &Theme) -> iced::widget::container::Appearance {
        iced::widget::container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.9, 0.55, 0.1, 0.12))),
            text_color: Some(Color::from_rgb(0.9, 0.75, 0.4)),
            border: iced::Border {
                color: Color::from_rgba(0.9, 0.55, 0.1, 0.3),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        }
    }
}
