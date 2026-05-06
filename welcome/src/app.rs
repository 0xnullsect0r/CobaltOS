use anyhow::Result;
use iced::widget::{button, column, container, radio, row, scrollable, text, Column, Space};
use iced::{executor, Alignment, Application, Color, Command, Element, Length, Padding, Settings, Subscription, Theme};

use crate::pages::WelcomePage;
use crate::theme::cobalt_theme;

// ── App catalogue ─────────────────────────────────────────────────────────────

struct AppEntry {
    name: &'static str,
    flatpak_id: &'static str,
    default: bool,
    description: &'static str,
}

const ALL_APPS: &[AppEntry] = &[
    AppEntry { name: "Firefox ESR",       flatpak_id: "org.mozilla.firefox",       default: true,  description: "Web browser" },
    AppEntry { name: "GNOME Software",    flatpak_id: "org.gnome.Software",        default: true,  description: "App store (Flatpak)" },
    AppEntry { name: "LibreOffice",       flatpak_id: "org.libreoffice.LibreOffice", default: false, description: "Office suite" },
    AppEntry { name: "VLC",              flatpak_id: "org.videolan.VLC",          default: false, description: "Media player" },
    AppEntry { name: "GIMP",             flatpak_id: "org.gimp.GIMP",             default: false, description: "Image editor" },
    AppEntry { name: "Inkscape",         flatpak_id: "org.inkscape.Inkscape",     default: false, description: "Vector graphics" },
    AppEntry { name: "Thunderbird",      flatpak_id: "org.mozilla.Thunderbird",   default: false, description: "Email client" },
    AppEntry { name: "Obsidian",         flatpak_id: "md.obsidian.Obsidian",      default: false, description: "Note-taking" },
    AppEntry { name: "Spotify",          flatpak_id: "com.spotify.Client",        default: false, description: "Music streaming" },
    AppEntry { name: "Discord",          flatpak_id: "com.discordapp.Discord",    default: false, description: "Chat" },
];

// ── Browser catalogue ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserChoice {
    Firefox,
    Chromium,
    Brave,
    None,
}

impl BrowserChoice {
    fn label(self) -> &'static str {
        match self {
            BrowserChoice::Firefox  => "Firefox ESR  (recommended, already included)",
            BrowserChoice::Chromium => "Chromium  (open-source Chrome)",
            BrowserChoice::Brave    => "Brave  (privacy-focused, Chromium-based)",
            BrowserChoice::None     => "None — I'll install my own",
        }
    }

    fn flatpak_id(self) -> Option<&'static str> {
        match self {
            BrowserChoice::Firefox  => None, // pre-installed
            BrowserChoice::Chromium => Some("org.chromium.Chromium"),
            BrowserChoice::Brave    => Some("com.brave.Browser"),
            BrowserChoice::None     => None,
        }
    }
}

// ── Locale helpers ───────────────────────────────────────────────────────────

const LOCALES: &[&str] = &[
    "en_US", "en_GB", "en_CA", "en_AU",
    "de_DE", "fr_FR", "es_ES", "es_MX",
    "it_IT", "pt_BR", "nl_NL", "pl_PL",
    "ru_RU", "ja_JP", "ko_KR", "zh_CN", "zh_TW",
    "ar_SA", "tr_TR", "sv_SE", "da_DK", "fi_FI",
];

// ── State ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice { Dark, Light }

pub struct WelcomeApp {
    current_page: WelcomePage,
    theme_choice: ThemeChoice,
    locale: String,
    browser: BrowserChoice,
    selected_apps: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Next,
    Back,
    ThemeChanged(ThemeChoice),
    LocaleSelected(String),
    BrowserChanged(BrowserChoice),
    AppToggled(String),
    Exit,
}

pub fn run() -> Result<()> {
    WelcomeApp::run(Settings::default()).map_err(|e| anyhow::anyhow!("iced: {e}"))
}

// ── Application ──────────────────────────────────────────────────────────────

impl Application for WelcomeApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let selected_apps = ALL_APPS.iter()
            .filter(|a| a.default)
            .map(|a| a.name.to_string())
            .collect();
        (
            Self {
                current_page: WelcomePage::Welcome,
                theme_choice: ThemeChoice::Dark,
                locale: "en_US".to_string(),
                browser: BrowserChoice::Firefox,
                selected_apps,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String { "Welcome to CobaltOS".to_string() }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::Next => {
                self.current_page = match self.current_page {
                    WelcomePage::Welcome    => WelcomePage::Appearance,
                    WelcomePage::Appearance => WelcomePage::Locale,
                    WelcomePage::Locale     => WelcomePage::Browser,
                    WelcomePage::Browser    => WelcomePage::Apps,
                    WelcomePage::Apps       => {
                        self.apply_settings();
                        WelcomePage::Finish
                    }
                    WelcomePage::Finish     => WelcomePage::Finish,
                };
            }
            Message::Back => {
                self.current_page = match self.current_page {
                    WelcomePage::Welcome    => WelcomePage::Welcome,
                    WelcomePage::Appearance => WelcomePage::Welcome,
                    WelcomePage::Locale     => WelcomePage::Appearance,
                    WelcomePage::Browser    => WelcomePage::Locale,
                    WelcomePage::Apps       => WelcomePage::Browser,
                    WelcomePage::Finish     => WelcomePage::Apps,
                };
            }
            Message::ThemeChanged(c)    => self.theme_choice = c,
            Message::LocaleSelected(l)  => self.locale = l,
            Message::BrowserChanged(b)  => self.browser = b,
            Message::AppToggled(name)   => {
                if let Some(pos) = self.selected_apps.iter().position(|a| *a == name) {
                    self.selected_apps.remove(pos);
                } else {
                    self.selected_apps.push(name);
                }
            }
            Message::Exit => std::process::exit(0),
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let page = match self.current_page {
            WelcomePage::Welcome    => self.view_welcome(),
            WelcomePage::Appearance => self.view_appearance(),
            WelcomePage::Locale     => self.view_locale(),
            WelcomePage::Browser    => self.view_browser(),
            WelcomePage::Apps       => self.view_apps(),
            WelcomePage::Finish     => self.view_finish(),
        };

        let progress = self.progress_bar();

        container(
            column![progress, container(page).max_width(700.0).padding(Padding::from([32.0, 60.0]))]
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

// ── On-finish side effects ───────────────────────────────────────────────────

impl WelcomeApp {
    fn apply_settings(&mut self) {
        // Write COSMIC theme preference
        self.write_cosmic_theme();
        // Write locale choice
        self.write_locale();
        // Queue flatpak installs
        self.queue_flatpak_installs();
    }

    fn write_cosmic_theme(&self) {
        let config_dir = dirs_or_home().join(".config/cobaltos");
        let _ = std::fs::create_dir_all(&config_dir);
        let mode = match self.theme_choice {
            ThemeChoice::Dark  => "dark",
            ThemeChoice::Light => "light",
        };
        let content = format!("theme = \"{mode}\"\n");
        let _ = std::fs::write(config_dir.join("theme.toml"), content);
    }

    fn write_locale(&self) {
        let config_dir = dirs_or_home().join(".config/cobaltos");
        let _ = std::fs::create_dir_all(&config_dir);
        let content = format!("locale = \"{}\"\n", self.locale);
        let _ = std::fs::write(config_dir.join("locale.toml"), content);
    }

    fn queue_flatpak_installs(&self) {
        // Write a list of flatpak IDs to install to a queue file.
        // The system service cobalt-welcome.service can run the install.
        let config_dir = dirs_or_home().join(".config/cobaltos");
        let _ = std::fs::create_dir_all(&config_dir);

        let mut ids: Vec<&str> = Vec::new();

        if let Some(id) = self.browser.flatpak_id() {
            ids.push(id);
        }

        for app in &self.selected_apps {
            if let Some(entry) = ALL_APPS.iter().find(|a| a.name == app.as_str()) {
                ids.push(entry.flatpak_id);
            }
        }

        let content = ids.join("\n");
        let _ = std::fs::write(config_dir.join("flatpak-queue.txt"), content);
    }
}

fn dirs_or_home() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/root"))
}

// ── Views ────────────────────────────────────────────────────────────────────

impl WelcomeApp {
    fn progress_bar(&self) -> Element<'_, Message> {
        let pages = [
            WelcomePage::Welcome,
            WelcomePage::Appearance,
            WelcomePage::Locale,
            WelcomePage::Browser,
            WelcomePage::Apps,
            WelcomePage::Finish,
        ];
        let total = pages.len();
        let current_idx = pages.iter().position(|p| *p == self.current_page).unwrap_or(0);

        let dots: Vec<Element<'_, Message>> = pages.iter().enumerate().map(|(i, _)| {
            let (symbol, color) = if i < current_idx {
                ("●", Color::from_rgb(0.0, 0.278, 0.671))
            } else if i == current_idx {
                ("●", Color::WHITE)
            } else {
                ("○", Color::from_rgba(1.0, 1.0, 1.0, 0.3))
            };
            text(symbol)
                .size(14.0)
                .style(iced::theme::Text::Color(color))
                .into()
        }).collect();
        let _ = total;

        container(
            row(dots).spacing(8.0).align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .center_x()
        .padding(Padding::from([12.0, 0.0]))
        .into()
    }

    fn nav_buttons(&self, show_back: bool, next_label: &'static str) -> Element<'_, Message> {
        let next_btn = button(text(next_label))
            .style(iced::theme::Button::Primary)
            .on_press(Message::Next);

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

    fn view_welcome(&self) -> Element<'_, Message> {
        column![
            Space::with_height(Length::Fixed(40.0)),
            text("◈ CobaltOS").size(56.0),
            text("Your Chromebook, reimagined.").size(20.0),
            Space::with_height(Length::Fixed(16.0)),
            text("Let's get everything set up — it only takes a minute.")
                .size(15.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.7))),
            Space::with_height(Length::Fixed(48.0)),
            self.nav_buttons(false, "Get Started  →"),
        ]
        .align_items(Alignment::Center)
        .spacing(8.0)
        .into()
    }

    fn view_appearance(&self) -> Element<'_, Message> {
        column![
            text("Appearance").size(36.0),
            text("How should CobaltOS look?").size(15.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.65))),
            Space::with_height(Length::Fixed(32.0)),
            radio("Dark  (default)", ThemeChoice::Dark, Some(self.theme_choice), Message::ThemeChanged).size(18),
            Space::with_height(Length::Fixed(12.0)),
            radio("Light", ThemeChoice::Light, Some(self.theme_choice), Message::ThemeChanged).size(18),
            Space::with_height(Length::Fill),
            self.nav_buttons(true, "Next  →"),
        ]
        .height(Length::Fixed(440.0))
        .spacing(4.0)
        .into()
    }

    fn view_locale(&self) -> Element<'_, Message> {
        let locale_btns: Vec<Element<'_, Message>> = LOCALES.iter().map(|&loc| {
            let is_sel = self.locale == loc;
            button(text(loc).size(14))
                .style(if is_sel { iced::theme::Button::Primary } else { iced::theme::Button::Secondary })
                .on_press(Message::LocaleSelected(loc.to_string()))
                .into()
        }).collect();

        column![
            text("Language & Region").size(36.0),
            text("Choose your preferred locale.").size(15.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.65))),
            Space::with_height(Length::Fixed(24.0)),
            scrollable(
                Column::with_children(locale_btns)
                    .spacing(6.0)
                    .width(Length::Fill)
            ).height(Length::Fixed(280.0)),
            Space::with_height(Length::Fixed(16.0)),
            self.nav_buttons(true, "Next  →"),
        ]
        .spacing(4.0)
        .into()
    }

    fn view_browser(&self) -> Element<'_, Message> {
        let options = [
            BrowserChoice::Firefox,
            BrowserChoice::Chromium,
            BrowserChoice::Brave,
            BrowserChoice::None,
        ];
        let mut col = Column::new().spacing(14.0);
        for opt in &options {
            col = col.push(
                radio(opt.label(), *opt, Some(self.browser), Message::BrowserChanged).size(17),
            );
        }

        column![
            text("Default Browser").size(36.0),
            text("Which browser would you like to use?").size(15.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.65))),
            Space::with_height(Length::Fixed(28.0)),
            col,
            Space::with_height(Length::Fill),
            self.nav_buttons(true, "Next  →"),
        ]
        .height(Length::Fixed(460.0))
        .spacing(4.0)
        .into()
    }

    fn view_apps(&self) -> Element<'_, Message> {
        let app_btns: Vec<Element<'_, Message>> = ALL_APPS.iter().map(|a| {
            let is_sel = self.selected_apps.iter().any(|n| n == a.name);
            let check = if is_sel { "✓  " } else { "   " };
            let label = format!("{}{} — {}", check, a.name, a.description);
            button(text(label).size(14))
                .style(if is_sel { iced::theme::Button::Primary } else { iced::theme::Button::Secondary })
                .width(Length::Fill)
                .on_press(Message::AppToggled(a.name.to_string()))
                .into()
        }).collect();

        column![
            text("Recommended Apps").size(36.0),
            text("These will be installed from Flathub. You can add or remove apps later.")
                .size(14.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.65))),
            Space::with_height(Length::Fixed(16.0)),
            scrollable(
                Column::with_children(app_btns).spacing(6.0).width(Length::Fill)
            ).height(Length::Fixed(300.0)),
            Space::with_height(Length::Fixed(16.0)),
            self.nav_buttons(true, "Finish Setup  →"),
        ]
        .spacing(4.0)
        .into()
    }

    fn view_finish(&self) -> Element<'_, Message> {
        let pending = self.selected_apps.len()
            + self.browser.flatpak_id().map(|_| 1).unwrap_or(0);

        let subtitle = if pending > 0 {
            format!("{pending} app(s) will be installed in the background.")
        } else {
            "Everything is set up and ready to go.".to_string()
        };

        column![
            Space::with_height(Length::Fixed(40.0)),
            text("You're all set!").size(48.0),
            Space::with_height(Length::Fixed(12.0)),
            text(subtitle).size(16.0)
                .style(iced::theme::Text::Color(Color::from_rgba(1.0,1.0,1.0,0.7))),
            Space::with_height(Length::Fixed(48.0)),
            button(text("Start using CobaltOS  →"))
                .style(iced::theme::Button::Primary)
                .on_press(Message::Exit),
        ]
        .align_items(Alignment::Center)
        .spacing(8.0)
        .into()
    }
}
