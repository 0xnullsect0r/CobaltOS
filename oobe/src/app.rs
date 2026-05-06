use anyhow::Result;
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{executor, Alignment, Application, Command, Element, Length, Padding, Settings, Subscription, Theme};

use crate::pages::OobePage;
use crate::theme::cobalt_theme;

pub struct OobeApp {
    page: OobePage,
    name_input: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Next,
    NameChanged(String),
    Finish,
}

pub fn run() -> Result<()> {
    OobeApp::run(Settings::default()).map_err(|e| anyhow::anyhow!("iced: {e}"))
}

impl Application for OobeApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self { page: OobePage::Welcome, name_input: String::new() }, Command::none())
    }

    fn title(&self) -> String {
        "CobaltOS Setup".to_string()
    }

    fn update(&mut self, msg: Message) -> Command<Message> {
        match msg {
            Message::Next => {
                self.page = match self.page {
                    OobePage::Welcome => OobePage::Connect,
                    OobePage::Connect => OobePage::Ready,
                    OobePage::Ready => OobePage::Ready,
                };
            }
            Message::NameChanged(s) => self.name_input = s,
            Message::Finish => {
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
            OobePage::Welcome => self.view_welcome(),
            OobePage::Connect => self.view_connect(),
            OobePage::Ready => self.view_ready(),
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

impl OobeApp {
    fn view_welcome(&self) -> Element<'_, Message> {
        column![
            text("Welcome to CobaltOS").size(40.0),
            Space::with_height(Length::Fixed(12.0)),
            text("Let's get your Chromebook set up.").size(18.0),
            Space::with_height(Length::Fixed(40.0)),
            button(text("Get Started  →"))
                .style(iced::theme::Button::Primary)
                .on_press(Message::Next),
        ]
        .align_items(Alignment::Center)
        .spacing(8.0)
        .into()
    }

    fn view_connect(&self) -> Element<'_, Message> {
        column![
            text("WiFi Setup").size(32.0),
            Space::with_height(Length::Fixed(16.0)),
            text("Connect to WiFi to continue setup.").size(16.0),
            Space::with_height(Length::Fill),
            row![
                button(text("Skip for now"))
                    .style(iced::theme::Button::Text)
                    .on_press(Message::Next),
                Space::with_width(Length::Fill),
                button(text("Continue  →"))
                    .style(iced::theme::Button::Primary)
                    .on_press(Message::Next),
            ]
            .width(Length::Fill),
        ]
        .spacing(8.0)
        .height(Length::Fixed(360.0))
        .into()
    }

    fn view_ready(&self) -> Element<'_, Message> {
        column![
            text("You're all set!").size(40.0),
            Space::with_height(Length::Fixed(16.0)),
            text("What should we call you?").size(16.0),
            text_input("Enter your name", &self.name_input)
                .on_input(Message::NameChanged)
                .padding(12.0),
            Space::with_height(Length::Fixed(32.0)),
            button(text("Start using CobaltOS"))
                .style(iced::theme::Button::Primary)
                .on_press(Message::Finish),
        ]
        .align_items(Alignment::Center)
        .spacing(12.0)
        .into()
    }
}
