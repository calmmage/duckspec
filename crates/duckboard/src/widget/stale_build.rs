//! Recipe panel for a stale self-build: show versions + copyable `just install`.

use iced::widget::{Space, button, column, container, row, text};
use iced::{Center, Element, Length};

use crate::self_version::{self, StaleBuildInfo};
use crate::theme;

#[derive(Debug, Clone)]
pub enum Msg {
    Close,
    Copy,
}

pub fn view(info: &StaleBuildInfo) -> Element<'_, Msg> {
    let title = text("Update available")
        .size(theme::font_md())
        .color(theme::text_primary());

    let versions = text(format!("{} → {}", info.running, info.disk))
        .size(theme::font_md())
        .font(theme::content_font())
        .color(theme::warning());

    let steps = column![
        text("1. Quit duckboard")
            .size(theme::font_sm())
            .color(theme::text_secondary()),
        text("2. Run this in a terminal, then reopen the app")
            .size(theme::font_sm())
            .color(theme::text_secondary()),
    ]
    .spacing(theme::SPACING_XS);

    let command = self_version::install_command(&info.project_root);
    let command_box = container(
        text(command)
            .size(theme::font_sm())
            .font(theme::content_font())
            .color(theme::text_primary()),
    )
    .padding(theme::SPACING_MD)
    .width(Length::Fill)
    .style(command_box_style);

    let actions = row![
        button(text("Copy").size(theme::font_sm()))
            .padding([theme::SPACING_XS, theme::SPACING_MD])
            .style(theme::nav_button)
            .on_press(Msg::Copy),
        button(text("Close").size(theme::font_sm()))
            .padding([theme::SPACING_XS, theme::SPACING_MD])
            .style(theme::nav_button)
            .on_press(Msg::Close),
    ]
    .spacing(theme::SPACING_SM);

    let panel = container(
        column![title, versions, steps, command_box, actions]
            .spacing(theme::SPACING_MD)
            .max_width(520.0),
    )
    .padding(theme::SPACING_LG)
    .style(panel_style)
    .max_width(520.0);

    container(column![Space::new().height(100.0), panel].align_x(Center))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Center)
        .style(overlay_backdrop_style)
        .into()
}

fn overlay_backdrop_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(
            iced::Color {
                a: 0.5,
                ..theme::bg_base()
            }
            .into(),
        ),
        ..container::Style::default()
    }
}

fn panel_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(theme::bg_base().into()),
        border: iced::Border {
            color: theme::border_color(),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    }
}

fn command_box_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(theme::bg_surface().into()),
        border: iced::Border {
            color: theme::border_color(),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..container::Style::default()
    }
}
