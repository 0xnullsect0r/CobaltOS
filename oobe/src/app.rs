use anyhow::Result;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Column, Space};
use iced::{executor, Alignment, Application, Color, Command, Element, Length, Padding, Settings, Subscription, Theme};

use crate::pages::OobePage;
use crate::theme::cobalt_theme;

// ── Timezone data ────────────────────────────────────────────────────────────

const TIMEZONES: &[(&str, &str)] = &[
    ("UTC",                    "UTC (Universal Time)"),
    ("America/New_York",       "Eastern Time (US & Canada)"),
    ("America/Chicago",        "Central Time (US & Canada)"),
    ("America/Denver",         "Mountain Time (US & Canada)"),
    ("America/Los_Angeles",    "Pacific Time (US & Canada)"),
    ("America/Anchorage",      "Alaska"),
    ("Pacific/Honolulu",       "Hawaii"),
    ("Europe/London",          "London, Dublin"),
    ("Europe/Paris",           "Paris, Berlin, Amsterdam"),
    ("Europe/Moscow",          "Moscow, St. Petersburg"),
    ("Asia/Kolkata",           "Mumbai, New Delhi"),
    ("Asia/Shanghai",          "Beijing, Chongqing"),
    ("Asia/Tokyo",             "Tokyo, Osaka"),
    ("Australia/Sydney",       "Sydney, Melbourne"),
    ("Pacific/Auckland",       "Auckland, Wellington"),
];

// ── State ────────────────────────────────────────────────────────────────────

pub struct OobeApp {
    page: OobePage,
    timezone: String,
    privacy_analytics: bool,
    privacy_location:  bool,
    display_name: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Next,
    Back,
    TimezoneSelected(String),
    AnalyticsToggled(bool),
    LocationToggled(bool),
    NameChanged(String),
    Finish,
}

pub fn run() -> Result<()> {
    OobeApp::run(Settings::default()).map_err(|e| anyhow::anyhow!("iced: {e}"))
}

// ── Application ──────────────────────────────────────────────────────────────

impl Application for OobeApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                page: OobePage::Welcome,
                timezone: "UTC".to_string(),
                privacy_analytics: false,
                privacy_location: false,
                display_name: String::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String { "CobaltOS Setup".to_string() }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::Next => {
                self.page = match self.page {
                    OobePage::Welcome  => OobePage::Connect,
                    OobePage::Connect  => OobePage::Timezone,
                    OobePage::Timezone => OobePage::Privacy,
                    OobePage::Privacy  => OobePage::Account,
                    OobePage::Account  => OobePage::Ready,
                    OobePage::Ready    => OobePage::Ready,
                };
            }
            Message::Back => {
                self.page = match self.page {
                    OobePage::Welcome  => OobePage::Welcome,
                    OobePage::Connect  => OobePage::Welcome,
                    OobePage::Timezone => OobePage::Connect,
                    OobePage::Privacy  => OobePage::Timezone,
                    OobePage::Account  => OobePage::Privacy,
                    OobePage::Ready    => OobePage::Account,
                };
            }
            Message::TimezoneSelected(tz)   => self.timezone = tz,
            Message::AnalyticsToggled(v)    => self.privacy_analytics = v,
            Message::LocationToggled(v)     => self.privacy_location = v,
            Message::NameChanged(s)         => self.display_name = s,
            Message::Finish => {
                self.apply_settings();
                let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
                let marker = std::path::Path::new(&home).join(".cobalt-oobe-done");
                let _ = std::fs::write(&marker, "");
                std::process::exit(0);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let page = match self.page {
            OobePage::Welcome  => self.view_welcome(),
            OobePage::Connect  => self.view_connect(),
            OobePage::Timezone => self.view_timezone(),
            OobePage::Privacy  => self.view_privacy(),
            OobePage::Account  => self.view_account(),
            OobePage::Ready    => self.view_ready(),
        };

        let progress = self.progress_bar();

        container(
            column![progress, container(page).max_width(680.0).padding(Padding::from([28.0, 56.0]))]
                .spacing(0.0),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .into()
    }

    fn theme(&self) -> Theme { cobalt_theme() }
    fn subscription(&self) -> Subscription<Message> { Subscription::none() }
}

// ── On-finish ────────────────────────────────────────────────────────────────

impl OobeApp {
    fn apply_settings(&self) {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        let cfg = std::path::PathBuf::from(&home).join(".config/cobaltos");
        let _ = std::fs::create_dir_all(&cfg);

        // Write timezone choice
        let tz_content = format!("timezone = \"{}\"\n", self.timezone);
        let _ = std::fs::write(cfg.join("timezone.toml"), tz_content);

        // Write privacy prefs
        let priv_content = format!(
            "analytics = {}\nlocation = {}\n",
            self.privacy_analytics, self.privacy_location
        );
        let _ = std::fs::write(cfg.join("privacy.toml"), priv_content);

        // Write display name
        if !self.display_name.is_empty() {
            let acct_content = format!("display_name = \"{}\"\n", self.display_name);
            let _ = std::fs::write(cfg.join("account.toml"), acct_content);
        }
    }
}

// ── Progress bar ─────────────────────────────────────────────────────────────

impl OobeApp {
    fn progress_bar(&self) -> Element<'_, Message> {
        let pages = [
            OobePage::Welcome, OobePage::Connect, OobePage::Timezone,
            OobePage::Privacy, OobePage::Account, OobePage::Ready,
        ];
        let current_idx = pages.iter().position(|p| *p == self.page).unwrap_or(0);

        let dots: Vec<Element<'_, Message>> = pages.iter().enumerate().map(|(i, _)| {
            let (sym, col) = if i < current_idx {
                ("●", Color::from_rgb(0.0, 0.278, 0.671))
            } else if i == current_idx {
                ("●", Color::WHITE)
            } else {
                ("○", Color::from_rgba(1.0, 1.0, 1.0, 0.3))
            };
            text(sym).size(14.0).style(iced::theme::Text::Color(col)).into()
        }).collect();

        container(
            row(dots).spacing(8.0).align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .center_x()
        .padding(Padding::from([12.0, 0.0]))
        .into()
    }

    fn nav_buttons(&self, show_back: bool, next_label: &'static str, msg: Message) -> Element<'_, Message> {
        let next_btn = button(text(next_label))
            .style(iced::theme::Button::Primary)
            .on_press(msg);

        if show_back {
            row![
                button(text("←  Back")).style(iced::theme::Button::Text).on_press(Message::Back),
                Space::with_width(Length::Fill),
                next_btn,
            ]
            .width(Length::Fill)
            .into()
        } else {
            container(next_btn).width(Length::Fill).center_x().into()
        }
    }
}

// ── Page views ───────────────────────────────────────────────────────────────

impl OobeApp {
    fn view_welcome(&self) -> Element<'_, Message> {
        column![
            Space::with_height(Length::Fixed(36.0)),
            text("◈ CobaltOS").size(56.0),
            text("First-run setup").size(20.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.65))),
            Space::with_height(Length::Fixed(48.0)),
            self.nav_buttons(false, "Begin  →", Message::Next),
        ]
        .align_items(Alignment::Center)
        .spacing(8.0)
        .into()
    }

    fn view_connect(&self) -> Element<'_, Message> {
        column![
            text("WiFi Setup").size(36.0),
            text("Connect to a network to enable updates and app downloads.")
                .size(14.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.65))),
            Space::with_height(Length::Fixed(20.0)),
            container(
                text("WiFi network management is provided by NetworkManager.\nUse the COSMIC desktop network applet after setup to connect.")
                    .size(14.0)
                    .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.55)))
            )
            .padding(Padding::from([12.0, 16.0])),
            Space::with_height(Length::Fill),
            self.nav_buttons(true, "Continue  →", Message::Next),
        ]
        .height(Length::Fixed(380.0))
        .spacing(4.0)
        .into()
    }

    fn view_timezone(&self) -> Element<'_, Message> {
        let tz_btns: Vec<Element<'_, Message>> = TIMEZONES.iter().map(|(tz, label)| {
            let is_sel = self.timezone == *tz;
            button(
                column![
                    text(*label).size(14),
                    text(*tz).size(11)
                        .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.5))),
                ]
                .spacing(2.0),
            )
            .style(if is_sel { iced::theme::Button::Primary } else { iced::theme::Button::Secondary })
            .width(Length::Fill)
            .on_press(Message::TimezoneSelected(tz.to_string()))
            .into()
        }).collect();

        column![
            text("Date & Time").size(36.0),
            text("Select your time zone.").size(14.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.65))),
            Space::with_height(Length::Fixed(16.0)),
            scrollable(
                Column::with_children(tz_btns).spacing(6.0).width(Length::Fill),
            ).height(Length::Fixed(280.0)),
            Space::with_height(Length::Fixed(16.0)),
            self.nav_buttons(true, "Next  →", Message::Next),
        ]
        .spacing(4.0)
        .into()
    }

    fn view_privacy(&self) -> Element<'_, Message> {
        column![
            text("Privacy").size(36.0),
            text("Control what data CobaltOS shares.").size(14.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.65))),
            Space::with_height(Length::Fixed(28.0)),
            checkbox(
                "Send anonymous usage analytics to help improve CobaltOS",
                self.privacy_analytics,
            ).on_toggle(Message::AnalyticsToggled).size(18),
            Space::with_height(Length::Fixed(12.0)),
            checkbox(
                "Allow location access for time zone detection",
                self.privacy_location,
            ).on_toggle(Message::LocationToggled).size(18),
            Space::with_height(Length::Fixed(16.0)),
            text("You can change these at any time in System Settings.")
                .size(12.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.45))),
            Space::with_height(Length::Fill),
            self.nav_buttons(true, "Next  →", Message::Next),
        ]
        .height(Length::Fixed(420.0))
        .spacing(4.0)
        .into()
    }

    fn view_account(&self) -> Element<'_, Message> {
        column![
            text("Your Account").size(36.0),
            text("What should we call you?").size(14.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.65))),
            Space::with_height(Length::Fixed(24.0)),
            text("Display Name").size(13.0),
            text_input("e.g. Alex Smith", &self.display_name)
                .on_input(Message::NameChanged)
                .padding(12.0),
            Space::with_height(Length::Fixed(12.0)),
            text("This name appears on the lock screen and in user menus. Your system username was set during installation.")
                .size(12.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.45))),
            Space::with_height(Length::Fill),
            self.nav_buttons(true, "Next  →", Message::Next),
        ]
        .height(Length::Fixed(420.0))
        .spacing(6.0)
        .into()
    }

    fn view_ready(&self) -> Element<'_, Message> {
        let greeting = if self.display_name.is_empty() {
            "You're all set!".to_string()
        } else {
            format!("Welcome, {}!", self.display_name)
        };

        column![
            Space::with_height(Length::Fixed(40.0)),
            text(greeting).size(48.0),
            Space::with_height(Length::Fixed(12.0)),
            text("Your Chromebook is ready to go.")
                .size(16.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.7))),
            Space::with_height(Length::Fixed(48.0)),
            button(text("Start using CobaltOS  →"))
                .style(iced::theme::Button::Primary)
                .on_press(Message::Finish),
        ]
        .align_items(Alignment::Center)
        .spacing(8.0)
        .into()
    }
}
