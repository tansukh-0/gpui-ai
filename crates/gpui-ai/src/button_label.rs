//! Theme-aware text labels for composed gpui-component buttons.

use gpui::{
    App, InteractiveElement as _, IntoElement, ParentElement as _, RenderOnce, SharedString,
    Styled as _, Window, div, relative,
};
use gpui_component::{ActiveTheme as _, button::Button};

/// Adds a text label with the theme's leading rather than a one-em line box.
///
/// This composes the upstream button's child slot: sizes, colors, focus,
/// activation, icons, and disabled states remain owned by gpui-component.
/// The label inherits the button's font size and the theme's body-text leading.
pub trait ButtonLabelExt {
    /// Sets the visible text and accessible name. Use instead of `Button::label`.
    ///
    /// A later `accessibility_label` call can supply a different spoken name.
    /// A long label still ends in an ellipsis; descenders and accents keep the
    /// theme's line-height clearance. Call once per button.
    ///
    /// ```
    /// use gpui_ai::ButtonLabelExt as _;
    /// use gpui_component::button::Button;
    ///
    /// let button = Button::new("apply").text_label("Apply changes");
    /// ```
    fn text_label(self, label: impl Into<SharedString>) -> Self;
}

impl ButtonLabelExt for Button {
    fn text_label(self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        self.accessibility_label(label.clone())
            .child(ButtonText { label })
    }
}

#[derive(IntoElement)]
struct ButtonText {
    label: SharedString,
}

impl RenderOnce for ButtonText {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let body = cx.theme().typography_tokens().sm;
        // A relative leading follows every upstream Button size and rem zoom.
        // `Button::label` sets a one-em line box, which is the whole reason to
        // compose this slot instead: at leading 1.0 there is no room under the
        // baseline, and on DirectWrite the bottom of y/g/p and tall accented
        // glyphs goes with it.
        //
        // No mask. `truncate()` is `overflow_hidden` plus nowrap plus ellipsis,
        // and the hidden overflow was doing the same clipping this label exists
        // to avoid - hidden only because a taller line box left the descenders
        // room inside it. Upstream removed the same mask from its own label in
        // #2921 for the same reason; the ellipsis needs the other two.
        //
        // And nothing here offsets the label optically. It is tempting: GPUI
        // centres a line's ascent-to-descent box, so a word with no tall
        // accents hangs low inside it, and half the ascent's excess over the
        // cap height would centre what the eye actually sees. Measured, it is
        // 2.29px at this size - and a 24px control has only 2.9px above that
        // box, so spending it puts the ring of an Å on the border. The label
        // cannot know the control's height, so it cannot know what is spare.
        // Room is the control's to give: see `ControlMetricsExt`.
        // Named after what it says, so a geometry test can find it. Dropped
        // unread outside cfg(test), so the name costs nothing to carry.
        let named = self.label.clone();
        div()
            .debug_selector(move || format!("button-label-{named}"))
            .min_w_0()
            .whitespace_nowrap()
            .text_ellipsis()
            .line_height(relative(f32::from(body.line_height) / f32::from(body.size)))
            .child(self.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        Context, Element as _, Render, Role, TestAppContext, accesskit, canvas,
        prelude::FluentBuilder as _,
    };
    use gpui_component::Disableable as _;
    use std::sync::{Arc, Mutex};

    type Captured = Arc<Mutex<Vec<(Role, accesskit::Node)>>>;

    struct Probe {
        captured: Captured,
    }

    impl Render for Probe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let captured = self.captured.clone();
            canvas(
                move |_, window, cx| {
                    for disabled in [false, true] {
                        let label = if disabled {
                            "Unavailable"
                        } else {
                            "Apply gyjpq ÅÉ ﬁ ﬂ"
                        };

                        let mut erased = Button::new("label-probe")
                            .text_label("Apply gyjpq ÅÉ ﬁ ﬂ")
                            .disabled(disabled)
                            .when(disabled, |button| button.accessibility_label("Unavailable"))
                            .on_click(|_, _, _| {})
                            .render(window, cx)
                            .into_any_element();

                        assert!(
                            erased
                                .downcast_mut::<gpui::ViewElement<gpui_base::Button>>()
                                .is_some(),
                            "the styled button composes gpui-base's Button"
                        );

                        let element = crate::control::composed_button("label-probe", label)
                            .disabled(disabled)
                            .on_click(|_, _, _| {})
                            .render(window, cx)
                            .into_element();
                        let role = element.a11y_role().expect("the base button carries a role");
                        let mut node = accesskit::Node::new(Role::Button);
                        element.write_a11y_info(&mut node);
                        captured.lock().expect("capture mutex").push((role, node));
                    }
                },
                |_, _, _, _| {},
            )
        }
    }

    #[gpui::test]
    fn composed_label_keeps_upstream_name_and_activation(cx: &mut TestAppContext) {
        cx.update(crate::init);
        let captured: Captured = Arc::new(Mutex::new(Vec::new()));
        let result = captured.clone();
        let (_, cx) = cx.add_window_view(move |_, _| Probe { captured });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        let nodes = result.lock().expect("capture mutex");
        assert_eq!(nodes[0].0, Role::Button);
        assert_eq!(nodes[0].1.label(), Some("Apply gyjpq ÅÉ ﬁ ﬂ"));
        assert!(nodes[0].1.supports_action(accesskit::Action::Click)); // enabled → activation live
        assert_eq!(nodes[1].1.label(), Some("Unavailable"));
        assert!(!nodes[1].1.supports_action(accesskit::Action::Click)); // disabled → activation barred
    }
}
