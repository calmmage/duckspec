//! Draggable vertical strip between the list column and content.
//!
//! Thin grip (no chevrons): drag to set absolute list-column width.

use iced::advanced::layout;
use iced::advanced::mouse as adv_mouse;
use iced::advanced::renderer;
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Layout, Shell};
use iced::mouse;
use iced::{Border, Color, Element, Event, Length, Rectangle, Size, Theme};

use crate::theme;

/// Width of the list↔content drag strip.
pub const HANDLE_WIDTH: f32 = 6.0;
/// Minimum list-column width.
pub const MIN_LIST_WIDTH: f32 = 180.0;
/// Soft maximum list-column width (also clamped by remaining window space).
pub const MAX_LIST_WIDTH: f32 = 480.0;

const DRAG_THRESHOLD: f32 = 4.0;
const GRIP_DOT: f32 = 2.5;
const GRIP_DOT_GAP: f32 = 3.0;
const GRIP_DOT_COUNT: usize = 3;

struct DragState {
    start_x: f32,
    base_width: f32,
    dragging: bool,
}

#[derive(Default)]
struct HandleState {
    drag: Option<DragState>,
    hovered: bool,
}

/// Messages produced by the list resize handle.
#[derive(Debug, Clone)]
pub enum ListResizeMsg {
    SetWidth(f32),
}

/// The list-column resize grip.
pub struct ListResizeHandle<'a, M> {
    current_width: f32,
    /// Upper clamp for drag (window-dependent).
    max_width: f32,
    on_event: Box<dyn Fn(ListResizeMsg) -> M + 'a>,
}

impl<'a, M> ListResizeHandle<'a, M> {
    pub fn new(
        current_width: f32,
        max_width: f32,
        on_event: impl Fn(ListResizeMsg) -> M + 'a,
    ) -> Self {
        Self {
            current_width,
            max_width,
            on_event: Box::new(on_event),
        }
    }
}

impl<'a, M: Clone> Widget<M, Theme, iced::Renderer> for ListResizeHandle<'a, M> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(HANDLE_WIDTH), Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(HANDLE_WIDTH).height(Length::Fill);
        layout::Node::new(limits.resolve(HANDLE_WIDTH, Length::Fill, Size::ZERO))
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<HandleState>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(HandleState::default())
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: adv_mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let widget_state = tree.state.downcast_mut::<HandleState>();

        if let Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorLeft) = event {
            let now_hovered = cursor.is_over(bounds);
            if widget_state.hovered != now_hovered {
                widget_state.hovered = now_hovered;
                shell.request_redraw();
            }
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(bounds) {
                    let pos = cursor.position().unwrap();
                    widget_state.drag = Some(DragState {
                        start_x: pos.x,
                        base_width: self.current_width,
                        dragging: false,
                    });
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                if let Some(state) = widget_state.drag.as_mut() {
                    let dx = position.x - state.start_x;
                    if !state.dragging && dx.abs() > DRAG_THRESHOLD {
                        state.dragging = true;
                    }
                    if state.dragging {
                        // Positive dx (drag right) = grow list.
                        let max = self.max_width.max(MIN_LIST_WIDTH);
                        let new_width = (state.base_width + dx).clamp(MIN_LIST_WIDTH, max);
                        shell.publish((self.on_event)(ListResizeMsg::SetWidth(new_width)));
                        shell.capture_event();
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if widget_state.drag.take().is_some() {
                    shell.capture_event();
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: adv_mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let widget_state = tree.state.downcast_ref::<HandleState>();
        if widget_state.drag.is_some() || cursor.is_over(layout.bounds()) {
            mouse::Interaction::ResizingHorizontally
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: adv_mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let hovered = tree.state.downcast_ref::<HandleState>().hovered
            || tree.state.downcast_ref::<HandleState>().drag.is_some();

        let bg = if hovered {
            theme::bg_surface()
        } else {
            theme::bg_base()
        };
        renderer::Renderer::fill_quad(
            renderer,
            renderer::Quad {
                bounds,
                border: Border::default(),
                ..renderer::Quad::default()
            },
            bg,
        );

        // Edge hairlines so the strip reads as a hit zone.
        let sep = theme::border_color();
        for x in [bounds.x, bounds.x + bounds.width - 1.0] {
            renderer::Renderer::fill_quad(
                renderer,
                renderer::Quad {
                    bounds: Rectangle {
                        x,
                        y: bounds.y,
                        width: 1.0,
                        height: bounds.height,
                    },
                    border: Border::default(),
                    ..renderer::Quad::default()
                },
                sep,
            );
        }

        // Middle grip dots.
        let icon_color: Color = theme::text_muted();
        let total_h =
            GRIP_DOT_COUNT as f32 * GRIP_DOT + (GRIP_DOT_COUNT as f32 - 1.0) * GRIP_DOT_GAP;
        let dot_x = bounds.x + (HANDLE_WIDTH - GRIP_DOT) / 2.0;
        let mut dot_y = bounds.y + (bounds.height - total_h) / 2.0;
        for _ in 0..GRIP_DOT_COUNT {
            renderer::Renderer::fill_quad(
                renderer,
                renderer::Quad {
                    bounds: Rectangle {
                        x: dot_x,
                        y: dot_y,
                        width: GRIP_DOT,
                        height: GRIP_DOT,
                    },
                    border: Border {
                        radius: (GRIP_DOT / 2.0).into(),
                        ..Border::default()
                    },
                    ..renderer::Quad::default()
                },
                icon_color,
            );
            dot_y += GRIP_DOT + GRIP_DOT_GAP;
        }
    }
}

impl<'a, M: Clone + 'a> From<ListResizeHandle<'a, M>> for Element<'a, M> {
    fn from(h: ListResizeHandle<'a, M>) -> Self {
        Element::new(h)
    }
}

/// Build the list resize handle.
pub fn view<'a, M: Clone + 'a>(
    current_width: f32,
    max_width: f32,
    map: impl Fn(ListResizeMsg) -> M + 'a,
) -> Element<'a, M> {
    ListResizeHandle::new(current_width, max_width, map).into()
}
