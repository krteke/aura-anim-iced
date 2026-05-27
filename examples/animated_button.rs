//! Animated button demo for hover, press, focus, color, shadow, and scale.

use std::time::Instant;

use aura_anim_iced::{iced_ext, prelude::*};
use iced::{
    Background, Border, Color, Element, Length, Shadow, Subscription, Task, Theme, Vector,
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, mouse_area, row, text},
};

fn main() -> iced::Result {
    iced::application(Demo::default, update, view)
        .title(title)
        .subscription(subscription)
        .run()
}

fn title(_: &Demo) -> String {
    String::from("aura-anim-iced animated button")
}

#[derive(Debug, Clone)]
enum Message {
    HoverChanged(bool),
    PressChanged(bool),
    FocusToggled,
    AnimationTick(Instant),
}

#[derive(Debug)]
struct Demo {
    runtime: AnimationRuntime,
    effects: EffectSnapshot,
    hovered: bool,
    pressed: bool,
    focused: bool,
}

impl Default for Demo {
    fn default() -> Self {
        let mut runtime = AnimationRuntime::new();
        let effects = target_effects(ButtonVisualState::Rest);

        runtime.register_timeline(button_timeline(effects.clone(), effects.clone()));

        Self {
            runtime,
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

fn update(demo: &mut Demo, message: Message) -> Task<Message> {
    match message {
        Message::HoverChanged(hovered) => {
            demo.hovered = hovered;
            register_transition(demo);
        }
        Message::PressChanged(pressed) => {
            demo.pressed = pressed;
            register_transition(demo);
        }
        Message::FocusToggled => {
            demo.focused = !demo.focused;
            register_transition(demo);
        }
        Message::AnimationTick(tick_instant) => {
            let tick = iced_ext::update_tick(&mut demo.runtime, tick_instant);
            let effects = tick_effect_snapshot(&tick);

            if !effects.is_empty() {
                demo.effects = merge_effects(&demo.effects, &effects);
            }
        }
    }

    Task::none()
}

fn subscription(demo: &Demo) -> Subscription<Message> {
    iced_ext::subscription(&demo.runtime, Message::AnimationTick)
}

fn view(demo: &Demo) -> Element<'_, Message> {
    let animated = animated_button(demo);
    let focus_toggle = button(text(if demo.focused {
        "Clear focus"
    } else {
        "Toggle focus"
    }))
    .width(150.0)
    .on_press(Message::FocusToggled);
    let state = row![
        text(if demo.hovered { "hover" } else { "rest" }).width(60.0),
        text(if demo.pressed { "pressed" } else { "released" }).width(60.0),
        text(if demo.focused { "focused" } else { "unfocused" }).width(60.0),
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

fn animated_button(demo: &Demo) -> Element<'_, Message> {
    let scale = demo.effects.scale.unwrap_or(1.0);
    let radius = demo.effects.radius.unwrap_or(14.0);
    let background = demo
        .effects
        .background
        .unwrap_or(Color::from_rgb(0.16, 0.24, 0.34));
    let border_color = demo
        .effects
        .border_color
        .unwrap_or(Color::from_rgb(0.45, 0.61, 0.78));
    let text_color = demo.effects.text_color.unwrap_or(Color::WHITE);
    let shadow = demo.effects.shadow.unwrap_or(Shadow {
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
                    width: if demo.focused { 3.0 } else { 1.5 },
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

fn register_transition(demo: &mut Demo) {
    let target = target_effects(visual_state(demo));
    let timeline = button_timeline(demo.effects.clone(), target);

    demo.runtime.register_timeline(timeline);
}

fn visual_state(demo: &Demo) -> ButtonVisualState {
    if demo.pressed {
        ButtonVisualState::Pressed
    } else if demo.focused {
        ButtonVisualState::Focused
    } else if demo.hovered {
        ButtonVisualState::Hovered
    } else {
        ButtonVisualState::Rest
    }
}

fn button_timeline(from: EffectSnapshot, to: EffectSnapshot) -> Timeline {
    let timing = Timing::new(140.0).with_easing(Easing::EaseOut);

    Timeline::parallel([
        color_track(
            UiProperty::Background,
            from.background.unwrap_or(rest_background()),
            to.background.unwrap_or(rest_background()),
            timing,
        )
        .into(),
        color_track(
            UiProperty::BorderColor,
            from.border_color.unwrap_or(rest_border()),
            to.border_color.unwrap_or(rest_border()),
            timing,
        )
        .into(),
        color_track(
            UiProperty::TextColor,
            from.text_color.unwrap_or(Color::WHITE),
            to.text_color.unwrap_or(Color::WHITE),
            timing,
        )
        .into(),
        scalar_track(
            UiProperty::Scale,
            from.scale.unwrap_or(1.0),
            to.scale.unwrap_or(1.0),
            timing,
        )
        .into(),
        scalar_track(
            UiProperty::Radius,
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

fn scalar_track(property: UiProperty, from: f32, to: f32, timing: Timing) -> Track {
    Track::new(
        Keyframes::new()
            .with_timing(timing)
            .at(0.0, [(property, PropertyValue::Scalar(from))])
            .at(1.0, [(property, PropertyValue::Scalar(to))]),
    )
}

fn color_track(property: UiProperty, from: Color, to: Color, timing: Timing) -> Track {
    Track::new(
        Keyframes::new()
            .with_timing(timing)
            .at(0.0, [(property, PropertyValue::Color(from))])
            .at(1.0, [(property, PropertyValue::Color(to))]),
    )
}

fn shadow_track(from: Shadow, to: Shadow, timing: Timing) -> Track {
    Track::new(
        Keyframes::new()
            .with_timing(timing)
            .at(0.0, [(UiProperty::Shadow, PropertyValue::Shadow(from))])
            .at(1.0, [(UiProperty::Shadow, PropertyValue::Shadow(to))]),
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
