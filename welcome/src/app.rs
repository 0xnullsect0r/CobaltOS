use anyhow::Result;
use iced::widget::{button, column, container, radio, row, text, Column, Space};
use iced::{executor, Alignment, Application, Command, Element, Length, Padding, Settings, Subscription, Theme};

use crate::pages::WelcomePage;
use crate::theme::cobalt_theme;

const ALL_APPS: &[(&str, bool)] = &[
    ("Firefox ESR", true),
    ("GNOME Software", true),
    ("LibreOffice", false),
    ("VLC", false),
    ("GIMP", false),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    Dark,
    Light,
}

pub struct WelcomeApp {
    current_page: WelcomePage,
    theme_choice: ThemeChoice,
    selected_apps: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Next,
    Back,
    ThemeChanged(ThemeChoice),
    AppToggled(String),
    Exit,
}

pub fn run() -> Result<()> {
    WelcomeApp::run(Settings::default()).map_err(|e| anyhow::anyhow!("iced: {e}"))
}

impl Application for WelcomeApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let selected_apps = ALL_APPS
            .iter()
            .filter(|(_, default)| *default)
            .map(|(name, _)| name.to_string())
            .collect();
        (
            Self { current_page: WelcomePage::Welcome, theme_choice: ThemeChoice::Dark, selected_apps },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Welcome to CobaltOS".to_string()
    }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::Next => {
                self.current_page = match self.current_page {
                    WelcomePage::Welcome => WelcomePage::Appearance,
                    WelcomePage::Appearance => WelcomePage::Apps,
                    WelcomePage::Apps => WelcomePage::Finish,
                    WelcomePage::Finish => WelcomePage::Finish,
                };
            }
            Message::Back => {
                self.current_page = match self.current_page {
                    WelcomePage::Welcome => WelcomePage::Welcome,
                    WelcomePage::Appearance => WelcomePage::Welcome,
                    WelcomePage::Apps => WelcomePage::Appearance,
                    WelcomePage::Finish => WelcomePage::Apps,
                };
            }
            Message::ThemeChanged(choice) => self.theme_choice = choice,
            Message::AppToggled(name) => {
                if let Some(pos) = self.selected_apps.iter().position(|a| a == &name) {
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
            WelcomePage::Welcome => self.view_welcome(),
            WelcomePage::Appearance => self.view_appearance(),
            WelcomePage::Apps => self.view_apps(),
            WelcomePage::Finish => self.view_finish(),
        };

        container(
            container(page)
                .max_width(700.0)
                .padding(Padding::from([40.0, 60.0])),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }

    fn theme(&self) -> Theme {
        cobalt_theme()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

impl WelcomeApp {
    fn view_welcome(&self) -> Element<'_, Message> {
        column![
            text("◈ CobaltOS").size(52.0),
            text("Your Chromebook, reimagined.").size(20.0),
            Space::with_height(Length::Fixed(32.0)),
            button(text("Get Started  →"))
                .style(iced::theme::Button::Primary)
                .on_press(Message::Next),
        ]
        .align_items(Alignment::Center)
        .spacing(12.0)
        .into()
    }

    fn view_appearance(&self) -> Element<'_, Message> {
        column![
            text("Appearance").size(32.0),
            text("Choose how CobaltOS looks.").size(16.0),
            Space::with_height(Length::Fixed(20.0)),
            radio("Dark theme", ThemeChoice::Dark, Some(self.theme_choice), Message::ThemeChanged),
            radio("Light theme", ThemeChoice::Light, Some(self.theme_choice), Message::ThemeChanged),
            Space::with_height(Length::Fill),
            row![
                button(text("←  Back"))
                    .style(iced::theme::Button::Text)
                    .on_press(Message::Back),
                Space::with_width(Length::Fill),
                button(text("Next  →"))
                    .style(iced::theme::Button::Primary)
                    .on_press(Message::Next),
            ]
            .width(Length::Fill),
        ]
        .spacing(12.0)
        .height(Length::Fixed(400.0))
        .into()
    }

    fn view_apps(&self) -> Element<'_, Message> {
        let mut apps_col = Column::new().spacing(8.0);
        for (name, _) in ALL_APPS {
            let name_str = name.to_string();
            let is_selected = self.selected_apps.contains(&name_str);
            let style = if is_selected {
                iced::theme::Button::Primary
            } else {
                iced::theme::Button::Secondary
            };
            apps_col = apps_col.push(
                button(text(*name))
                    .style(style)
                    .width(Length::Fill)
                    .on_press(Message::AppToggled(name_str)),
            );
        }

        let nav = row![
            button(text("←  Back"))
                .style(iced::theme::Button::Text)
                .on_press(Message::Back),
            Space::with_width(Length::Fill),
            button(text("Next  →"))
                .style(iced::theme::Button::Primary)
                .on_press(Message::Next),
        ]
        .width(Length::Fill);

        let mut col = Column::new().spacing(12.0);
        col = col.push(text("Software").size(32.0));
        col = col.push(text("Select apps to install.").size(16.0));
        col = col.push(Space::with_height(Length::Fixed(8.0)));
        col = col.push(apps_col);
        col = col.push(Space::with_height(Length::Fill));
        col = col.push(nav);
        col.height(Length::Fixed(500.0)).into()
    }

    fn view_finish(&self) -> Element<'_, Message> {
        column![
            text("Setup Complete!").size(40.0),
            text("CobaltOS is ready to use.").size(18.0),
            Space::with_height(Length::Fixed(32.0)),
            button(text("Start exploring CobaltOS"))
                .style(iced::theme::Button::Primary)
                .on_press(Message::Exit),
        ]
        .align_items(Alignment::Center)
        .spacing(12.0)
        .into()
    }
}
