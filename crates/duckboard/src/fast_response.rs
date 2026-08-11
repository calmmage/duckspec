//! Re-export shared fast-response helpers from duckcore.
//!
//! Iced theme scenarios for awaiting-composer chrome stay in this crate's tests
//! below so duckcore remains UI-free.

pub use duckcore::fast_response::*;

#[cfg(test)]
mod theme_tests {
    use super::*;

    // @spec chat/fast-response Awaiting composer chrome: Awaiting user applies quiet accent tint to the composer section
    #[test]
    fn awaiting_user_applies_quiet_accent_tint_to_the_composer_section() {
        assert!(awaiting_composer_chrome(true));
        let theme = iced::Theme::Dark;
        let awaiting = crate::theme::chat_composer_awaiting(&theme);
        let normal = crate::theme::chat_input(&theme);
        assert_ne!(
            awaiting.background, normal.background,
            "awaiting composer must differ from normal paper input"
        );
        assert_eq!(
            awaiting.background,
            Some(iced::Background::Color(crate::theme::quiet_accent_surface()))
        );
    }

    // @spec chat/fast-response Awaiting composer chrome: Not awaiting leaves the composer section untinted
    #[test]
    fn not_awaiting_leaves_the_composer_section_untinted() {
        assert!(!awaiting_composer_chrome(false));
        let theme = iced::Theme::Dark;
        let normal = crate::theme::chat_input(&theme);
        assert_eq!(
            normal.background,
            Some(iced::Background::Color(crate::theme::bg_base()))
        );
    }

    // @spec chat/fast-response Awaiting composer chrome: Model selector matches the composer section tint while awaiting
    #[test]
    fn model_selector_matches_the_composer_section_tint_while_awaiting() {
        assert!(
            awaiting_composer_chrome(true),
            "model selector shares the awaiting chrome gate"
        );
        let theme = iced::Theme::Dark;
        let composer = crate::theme::chat_composer_awaiting(&theme);
        let pick = crate::theme::pick_list_ghost_awaiting_style(
            &theme,
            iced::widget::pick_list::Status::Active,
        );
        let Some(iced::Background::Color(composer_bg)) = composer.background else {
            panic!("composer awaiting must paint a color");
        };
        assert_eq!(
            pick.background,
            iced::Background::Color(composer_bg),
            "model selector active fill must match composer awaiting tint"
        );
    }
}
