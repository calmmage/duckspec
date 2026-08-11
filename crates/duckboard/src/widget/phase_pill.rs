//! Short lifecycle / VCS phase pills for the change list and chat composer.

use iced::widget::text::Wrapping;
use iced::widget::{button, container, row, text, tooltip};
use iced::{Element, Length};

use crate::area::change::{PhaseDisplay, PhaseShort, VcsPill};
use crate::theme;

/// Soft face tint for a lifecycle short stage.
pub fn lifecycle_tint(short: PhaseShort) -> iced::Color {
    match short {
        PhaseShort::Explore | PhaseShort::Empty | PhaseShort::Archived => theme::text_muted(),
        PhaseShort::Proposal | PhaseShort::Design | PhaseShort::Specs => theme::accent_dim(),
        PhaseShort::Steps => theme::warning(),
        PhaseShort::Review => theme::accent(),
        PhaseShort::Ready => theme::success(),
    }
}

fn vcs_tint(vcs: VcsPill) -> iced::Color {
    match vcs {
        VcsPill::Uncommitted => theme::warning(),
        VcsPill::Committed => theme::success(),
    }
}

fn pill_button_style(
    tint: iced::Color,
) -> impl Fn(&iced::Theme, button::Status) -> button::Style + Copy {
    move |_theme: &iced::Theme, status: button::Status| {
        let base = button::Style {
            background: Some(iced::Background::Color(tint.scale_alpha(0.12))),
            text_color: tint,
            border: iced::Border {
                color: tint.scale_alpha(0.35),
                width: 1.0,
                radius: 3.0.into(),
            },
            ..button::Style::default()
        };
        match status {
            button::Status::Hovered | button::Status::Pressed => button::Style {
                background: Some(iced::Background::Color(tint.scale_alpha(0.22))),
                ..base
            },
            _ => base,
        }
    }
}

fn inert_pill_style(tint: iced::Color) -> impl Fn(&iced::Theme) -> container::Style + Copy {
    move |_theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(tint.scale_alpha(0.12))),
        border: iced::Border {
            color: tint.scale_alpha(0.35),
            width: 1.0,
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

fn hover_tip<'a, Msg: 'a>(
    content: Element<'a, Msg>,
    hover: String,
) -> Element<'a, Msg> {
    if hover.is_empty() {
        return content;
    }
    tooltip(
        content,
        container(text(hover).size(theme::font_sm()).color(theme::text_primary()))
            .padding([theme::SPACING_XS, theme::SPACING_SM])
            .style(|_t: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(theme::bg_elevated())),
                border: iced::Border {
                    color: theme::border_color(),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }),
        tooltip::Position::Top,
    )
    .into()
}

/// One pill (lifecycle or VCS). Clickable when `on_press` is `Some`.
pub fn view_pill<'a, Msg: Clone + 'a>(
    label: &'static str,
    tint: iced::Color,
    hover: String,
    on_press: Option<Msg>,
) -> Element<'a, Msg> {
    let label_el = text(label)
        .size(theme::font_sm())
        .wrapping(Wrapping::None)
        .color(tint);

    let face: Element<'a, Msg> = if let Some(msg) = on_press {
        button(label_el)
            .padding([2.0, theme::SPACING_SM])
            .style(pill_button_style(tint))
            .on_press(msg)
            .into()
    } else {
        container(label_el)
            .padding([2.0, theme::SPACING_SM])
            .style(inert_pill_style(tint))
            .into()
    };
    hover_tip(face, hover)
}

/// Lifecycle pill plus optional late-stage VCS pill.
///
/// Clones hover strings so callers can pass a temporary `PhaseDisplay`.
pub fn view_pair<'a, Msg: Clone + 'a>(
    display: &PhaseDisplay,
    on_lifecycle: Option<Msg>,
    on_vcs: Option<Msg>,
) -> Element<'a, Msg> {
    let mut r = row![].spacing(theme::SPACING_XS).align_y(iced::Center);
    r = r.push(view_pill(
        display.short.label(),
        lifecycle_tint(display.short),
        display.lifecycle_hover.clone(),
        on_lifecycle,
    ));
    if let Some(vcs) = display.vcs {
        r = r.push(view_pill(
            vcs.label(),
            vcs_tint(vcs),
            display.vcs_hover.unwrap_or("").to_string(),
            on_vcs,
        ));
    }
    container(r).width(Length::Shrink).into()
}

/// Alias for list/chat call sites that own a temporary display.
pub fn view_pair_owned<'a, Msg: Clone + 'a>(
    display: &PhaseDisplay,
    on_lifecycle: Option<Msg>,
    on_vcs: Option<Msg>,
) -> Element<'a, Msg> {
    view_pair(display, on_lifecycle, on_vcs)
}
