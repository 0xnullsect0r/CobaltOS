use iced::theme::Palette;
use iced::{Color, Theme};

pub fn cobalt_theme() -> Theme {
    Theme::custom(
        "CobaltOS".to_string(),
        Palette {
            background: Color::from_rgb(0.067, 0.075, 0.094),
            text: Color::WHITE,
            primary: Color::from_rgb(0.0, 0.278, 0.671),
            success: Color::from_rgb(0.18, 0.8, 0.44),
            danger: Color::from_rgb(0.9, 0.2, 0.2),
        },
    )
}
