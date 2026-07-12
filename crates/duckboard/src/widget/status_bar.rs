//! Thin status bar strip showing the user's current selection path.
//!
//! Each area produces its own `Vec<String>` of segments via its own
//! `breadcrumbs(...)` function; this widget renders them uniformly. An
//! optional `trailing` label is right-aligned and used by the shell to show
//! the current project folder. An optional Update chip sits just left of the
//! trailing label when the running binary is behind a self project's disk
//! version.

use iced::widget::text::Wrapping;
use iced::widget::{Space, button, container, row, text};
use iced::{Element, Length};

use crate::theme;

pub fn view<'a, Msg: Clone + 'a>(
    segments: Vec<String>,
    trailing: Option<String>,
    hint: Option<String>,
    on_update: Option<Msg>,
) -> Element<'a, Msg> {
    let mut bar = row![].spacing(theme::SPACING_XS);
    let last = segments.len().saturating_sub(1);
    for (i, seg) in segments.into_iter().enumerate() {
        if i > 0 {
            bar = bar.push(
                text("\u{203a}")
                    .size(theme::font_sm())
                    .color(theme::text_muted()),
            );
        }
        let color = if i == last {
            theme::text_primary()
        } else {
            theme::text_muted()
        };
        bar = bar.push(
            text(seg)
                .size(theme::font_sm())
                .wrapping(Wrapping::None)
                .color(color),
        );
    }
    let has_trailing = trailing.is_some() || hint.is_some() || on_update.is_some();
    if has_trailing {
        bar = bar.push(Space::new().width(Length::Fill));
    }
    if let Some(h) = hint {
        bar = bar.push(
            text(h)
                .size(theme::font_sm())
                .wrapping(Wrapping::None)
                .color(theme::text_muted()),
        );
    }
    if let Some(msg) = on_update {
        bar = bar.push(
            button(
                text("Update")
                    .size(theme::font_sm())
                    .wrapping(Wrapping::None)
                    .color(theme::warning()),
            )
            .padding([2.0, theme::SPACING_SM])
            .style(update_chip_style)
            .on_press(msg),
        );
    }
    if let Some(label) = trailing {
        bar = bar.push(
            text(label)
                .size(theme::font_sm())
                .wrapping(Wrapping::None)
                .color(theme::text_secondary()),
        );
    }
    container(bar)
        .padding([2.0, theme::SPACING_SM])
        .width(Length::Fill)
        .style(theme::surface)
        .into()
}

fn update_chip_style(theme: &iced::Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(iced::Background::Color(theme::warning().scale_alpha(0.12))),
        text_color: theme::warning(),
        border: iced::Border {
            color: theme::warning().scale_alpha(0.35),
            width: 1.0,
            radius: 3.0.into(),
        },
        ..button::Style::default()
    };
    match status {
        button::Status::Hovered | button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(theme::warning().scale_alpha(0.22))),
            ..base
        },
        _ => {
            let _ = theme;
            base
        }
    }
}
