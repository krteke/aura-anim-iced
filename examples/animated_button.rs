//! Animated button demo for hover, press, focus, color, shadow, and scale.

use std::time::Instant;

use aura_anim_iced::{iced_ext, prelude::*, property};
use iced::{
    Background, Border, Color, Element, Length, Shadow, Subscription, Task, Theme, Vector,
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, mouse_area, row, text},
};

fn main() -> iced::Result {
    iced::application(Demo::default, Demo::update, Demo::view)
        .title(title)
        .subscription(Demo::subscription)
        .run()
}

fn title(_: &Demo) -> String {
    String::from("aura-anim-iced animated button")
}

#[derive(Debug, Clone, Copy)]
enum Message {
    HoverChanged(bool),
    PressChanged(bool),
    FocusToggled,
    AnimationTick(Instant),
}

#[derive(Debug)]
struct Demo {
    runtime: AnimationRuntime,
    button_target: AnimationTargetId,
    effects: EffectSnapshot,
    hovered: bool,
    pressed: bool,
    focused: bool,
}

impl Default for Demo {
    fn default() -> Self {
        let mut runtime = AnimationRuntime::new();
        let button_target = AnimationTargetId::new();
        let effects = target_effects(ButtonVisualState::Rest);

        runtime.register_timeline(button_target, button_timeline(&effects, &effects));

        Self {
            runtime,
            button_target,
            effects,
            hovered: false,
            pressed: false,
            focused: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ButtonVisualState {
    Rest,
    Hovered,
    Pressed,
    Focused,
}

impl Demo {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::HoverChanged(hovered) => {
                self.hovered = hovered;
                self.register_transition();
            }
            Message::PressChanged(pressed) => {
                self.pressed = pressed;
                self.register_transition();
            }
            Message::FocusToggled => {
                self.focused = !self.focused;
                self.register_transition();
            }
            Message::AnimationTick(tick_instant) => {
                let tick = iced_ext::update_tick(&mut self.runtime, tick_instant);
                let effects = tick_effect_snapshot_for(&tick, self.button_target);

                if !effects.is_empty() {
                    self.effects = merge_effects(&self.effects, &effects);
                }
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let animated = self.animated_button();
        let focus_toggle = button(text(if self.focused {
            "Clear focus"
        } else {
            "Toggle focus"
        }))
        .width(150.0)
        .on_press(Message::FocusToggled);
        let state = row![
            text(if self.hovered { "hover" } else { "rest" }).width(60.0),
            text(if self.pressed { "pressed" } else { "released" }).width(60.0),
            text(if self.focused { "focused" } else { "unfocused" }).width(60.0),
        ]
        .spacing(16);

        let button_container = container(animated)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(48)
            .center_x(Length::Fill)
            .center_y(Length::Fill);
        let content_container = container(column![focus_toggle, state].spacing(24))
            .width(Length::Fixed(180.0))
            .height(Length::Fill)
            .padding(48)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        container(column![button_container, content_container].spacing(24)).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_ext::subscription(&self.runtime, Message::AnimationTick)
    }

    fn visual_state(&self) -> ButtonVisualState {
        if self.pressed {
            ButtonVisualState::Pressed
        } else if self.focused {
            ButtonVisualState::Focused
        } else if self.hovered {
            ButtonVisualState::Hovered
        } else {
            ButtonVisualState::Rest
        }
    }

    fn register_transition(&mut self) {
        let target = target_effects(self.visual_state());
        let timeline = button_timeline(&self.effects, &target);

        self.runtime.register_timeline(self.button_target, timeline);
    }

    fn animated_button(&self) -> Element<'_, Message> {
        let scale = self.effects.scale.unwrap_or(1.0);
        let radius = self.effects.radius.unwrap_or(14.0);
        let background = self
            .effects
            .background
            .unwrap_or(Color::from_rgb(0.16, 0.24, 0.34));
        let border_color = self
            .effects
            .border_color
            .unwrap_or(Color::from_rgb(0.45, 0.61, 0.78));
        let text_color = self.effects.text_color.unwrap_or(Color::WHITE);
        let shadow = self.effects.shadow.unwrap_or(Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.24),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 18.0,
        });

        mouse_area(
            container(text("Animated Button").size(18.0 * scale))
                .width(Length::Fixed(190.0 * scale))
                .height(Length::Fixed(56.0 * scale))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .style(move |_theme: &Theme| container::Style {
                    text_color: Some(text_color),
                    background: Some(Background::Color(background)),
                    border: Border {
                        color: border_color,
                        width: if self.focused { 3.0 } else { 1.5 },
                        radius: radius.into(),
                    },
                    shadow,
                    ..container::Style::default()
                }),
        )
        .on_enter(Message::HoverChanged(true))
        .on_exit(Message::HoverChanged(false))
        .on_press(Message::PressChanged(true))
        .on_release(Message::PressChanged(false))
        .into()
    }
}

fn button_timeline(from: &EffectSnapshot, to: &EffectSnapshot) -> Timeline {
    let timing = Timing::new(140.0).with_easing(Easing::EaseOut);

    Timeline::parallel([
        color_track(
            property::BACKGROUND,
            from.background.unwrap_or(rest_background()),
            to.background.unwrap_or(rest_background()),
            timing,
        )
        .into(),
        color_track(
            property::BORDER_COLOR,
            from.border_color.unwrap_or(rest_border()),
            to.border_color.unwrap_or(rest_border()),
            timing,
        )
        .into(),
        color_track(
            property::TEXT_COLOR,
            from.text_color.unwrap_or(Color::WHITE),
            to.text_color.unwrap_or(Color::WHITE),
            timing,
        )
        .into(),
        scalar_track(
            property::SCALE,
            from.scale.unwrap_or(1.0),
            to.scale.unwrap_or(1.0),
            timing,
        )
        .into(),
        scalar_track(
            property::RADIUS,
            from.radius.unwrap_or(14.0),
            to.radius.unwrap_or(14.0),
            timing,
        )
        .into(),
        shadow_track(
            from.shadow.unwrap_or(rest_shadow()),
            to.shadow.unwrap_or(rest_shadow()),
            timing,
        )
        .into(),
    ])
}

fn scalar_track(
    property: PropertySpec<property::Scalar>,
    from: f32,
    to: f32,
    timing: Timing,
) -> Track {
    Track::new(
        KeyframesBuilder::new()
            .with_timing(timing)
            .at(0.0, (property, from))
            .at(1.0, (property, to))
            .finish(),
    )
}

fn color_track(
    property: PropertySpec<property::Color>,
    from: Color,
    to: Color,
    timing: Timing,
) -> Track {
    Track::new(
        KeyframesBuilder::new()
            .with_timing(timing)
            .at(0.0, (property, from))
            .at(1.0, (property, to))
            .finish(),
    )
}

fn shadow_track(from: Shadow, to: Shadow, timing: Timing) -> Track {
    Track::new(
        KeyframesBuilder::new()
            .with_timing(timing)
            .at(0.0, (property::SHADOW, from))
            .at(1.0, (property::SHADOW, to))
            .finish(),
    )
}

fn target_effects(state: ButtonVisualState) -> EffectSnapshot {
    match state {
        ButtonVisualState::Rest => EffectSnapshot {
            scale: Some(1.0),
            radius: Some(14.0),
            background: Some(rest_background()),
            border_color: Some(rest_border()),
            text_color: Some(Color::WHITE),
            shadow: Some(rest_shadow()),
            ..EffectSnapshot::default()
        },
        ButtonVisualState::Hovered => EffectSnapshot {
            scale: Some(1.05),
            radius: Some(16.0),
            background: Some(Color::from_rgb(0.19, 0.38, 0.58)),
            border_color: Some(Color::from_rgb(0.62, 0.84, 1.0)),
            text_color: Some(Color::WHITE),
            shadow: Some(Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.32),
                offset: Vector::new(0.0, 12.0),
                blur_radius: 26.0,
            }),
            ..EffectSnapshot::default()
        },
        ButtonVisualState::Pressed => EffectSnapshot {
            scale: Some(0.96),
            radius: Some(12.0),
            background: Some(Color::from_rgb(0.08, 0.20, 0.32)),
            border_color: Some(Color::from_rgb(0.86, 0.94, 1.0)),
            text_color: Some(Color::WHITE),
            shadow: Some(Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.18),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 10.0,
            }),
            ..EffectSnapshot::default()
        },
        ButtonVisualState::Focused => EffectSnapshot {
            scale: Some(1.03),
            radius: Some(18.0),
            background: Some(Color::from_rgb(0.18, 0.29, 0.46)),
            border_color: Some(Color::from_rgb(1.0, 0.76, 0.32)),
            text_color: Some(Color::WHITE),
            shadow: Some(Shadow {
                color: Color::from_rgba(1.0, 0.64, 0.16, 0.34),
                offset: Vector::new(0.0, 10.0),
                blur_radius: 24.0,
            }),
            ..EffectSnapshot::default()
        },
    }
}

fn merge_effects(current: &EffectSnapshot, update: &EffectSnapshot) -> EffectSnapshot {
    EffectSnapshot {
        opacity: update.opacity.or(current.opacity),
        width: update.width.or(current.width),
        height: update.height.or(current.height),
        padding: update.padding.or(current.padding),
        translation: update.translation.or(current.translation),
        scale: update.scale.or(current.scale),
        radius: update.radius.or(current.radius),
        background: update.background.or(current.background),
        border_color: update.border_color.or(current.border_color),
        text_color: update.text_color.or(current.text_color),
        shadow: update.shadow.or(current.shadow),
    }
}

fn rest_background() -> Color {
    Color::from_rgb(0.16, 0.24, 0.34)
}

fn rest_border() -> Color {
    Color::from_rgb(0.45, 0.61, 0.78)
}

fn rest_shadow() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.24),
        offset: Vector::new(0.0, 8.0),
        blur_radius: 18.0,
    }
}
