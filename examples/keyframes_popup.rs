//! Popup demo driven directly by keyframes.

mod shared;

use aura_anim_iced::{AnimationHandle, AnimationRuntime, AnimationTargetId, EffectSnapshot};
use aura_anim_iced::{Easing, Keyframes, Timing, iced_ext, property};
use iced::{
    Background, Border, Color, Element, Length, Shadow, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, row, text},
};
use std::time::Instant;

fn main() -> iced::Result {
    iced::application(Demo::default, update, view)
        .title(title)
        .subscription(subscription)
        .run()
}

fn title(_: &Demo) -> String {
    String::from("aura-anim-iced keyframes popup")
}

#[derive(Debug, Clone)]
enum Message {
    TogglePopup,
    AnimationTick(Instant),
}

#[derive(Debug)]
struct Demo {
    runtime: AnimationRuntime,
    popup_target: AnimationTargetId,
    effects: EffectSnapshot,
    visible: bool,
    closing: Option<AnimationHandle>,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            runtime: AnimationRuntime::new(),
            popup_target: AnimationTargetId::new(),
            effects: hidden_effects(),
            visible: false,
            closing: None,
        }
    }
}

fn update(demo: &mut Demo, message: Message) -> Task<Message> {
    match message {
        Message::TogglePopup => {
            if demo.visible {
                close_popup(demo);
            } else {
                open_popup(demo);
            }
        }
        Message::AnimationTick(tick_instant) => {
            let tick = iced_ext::update_tick(&mut demo.runtime, tick_instant);
            let effects = aura_anim_iced::tick_effect_snapshot_for(&tick, demo.popup_target);

            if !effects.is_empty() {
                demo.effects = shared::merge_effects(&demo.effects, &effects);
            }

            if demo
                .closing
                .is_some_and(|handle| tick.completed().contains(&handle))
            {
                demo.visible = false;
                demo.closing = None;
                demo.effects = hidden_effects();
            }
        }
    }

    Task::none()
}

fn subscription(demo: &Demo) -> Subscription<Message> {
    iced_ext::subscription(&demo.runtime, Message::AnimationTick)
}

fn view(demo: &Demo) -> Element<'_, Message> {
    let toggle_label = if demo.visible {
        "Close popup"
    } else {
        "Open popup"
    };
    let controls = row![button(text(toggle_label)).on_press(Message::TogglePopup)].spacing(12);
    let mut content = column![controls].spacing(24).align_x(Horizontal::Center);

    if demo.visible {
        content = content.push(popup_card(&demo.effects));
    } else {
        content = content.push(
            container(text("Popup hidden").size(16))
                .width(Length::Fixed(300.0))
                .height(Length::Fixed(120.0))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
        );
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(48)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

fn popup_card(effects: &EffectSnapshot) -> Element<'_, Message> {
    let opacity = effects.opacity.unwrap_or(1.0).clamp(0.0, 1.0);
    let scale = effects.scale.unwrap_or(1.0);
    let radius = effects.radius.unwrap_or(18.0);
    let background = effects
        .background
        .unwrap_or(Color::from_rgb(0.09, 0.12, 0.16));
    let border_color = effects
        .border_color
        .unwrap_or(Color::from_rgb(0.35, 0.54, 0.74));
    let shadow = effects
        .shadow
        .unwrap_or_else(|| shared::card_shadow(0.28, 16.0, 32.0));

    container(
        column![
            text("Keyframes Popup")
                .size(22.0 * scale)
                .color(Color::from_rgba(1.0, 1.0, 1.0, opacity)),
            text("Opacity and scale overshoot come from one keyframe track.")
                .size(14.0)
                .color(Color::from_rgba(0.82, 0.88, 0.94, opacity)),
        ]
        .spacing(10)
        .align_x(Horizontal::Center),
    )
    .width(Length::Fixed(320.0 * scale))
    .height(Length::Fixed(150.0 * scale))
    .align_x(Horizontal::Center)
    .align_y(Vertical::Center)
    .style(move |_theme: &Theme| container::Style {
        text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, opacity)),
        background: Some(Background::Color(Color {
            a: opacity,
            ..background
        })),
        border: Border {
            color: Color {
                a: opacity,
                ..border_color
            },
            width: 1.0,
            radius: radius.into(),
        },
        shadow: Shadow {
            color: Color {
                a: shadow.color.a * opacity,
                ..shadow.color
            },
            ..shadow
        },
        ..container::Style::default()
    })
    .into()
}

fn open_popup(demo: &mut Demo) {
    demo.visible = true;
    demo.closing = None;

    let registration = demo
        .runtime
        .register_keyframes(demo.popup_target, popup_open_keyframes(&demo.effects));

    if let Some(properties) = registration.properties() {
        demo.effects =
            shared::merge_effects(&demo.effects, &EffectSnapshot::from_properties(properties));
    }
}

fn close_popup(demo: &mut Demo) {
    let registration = demo
        .runtime
        .register_keyframes(demo.popup_target, popup_close_keyframes(&demo.effects));

    demo.closing = Some(registration.handle());
}

fn popup_open_keyframes(from: &EffectSnapshot) -> Keyframes {
    Keyframes::new()
        .with_timing(Timing::new(280.0).with_easing(Easing::EaseOut))
        .at(0.0, (property::OPACITY, from.opacity.unwrap_or(0.0)))
        .at(0.0, (property::SCALE, from.scale.unwrap_or(0.92)))
        .at(0.0, (property::RADIUS, from.radius.unwrap_or(18.0)))
        .at(0.68, (property::OPACITY, 1.0))
        .at(0.68, (property::SCALE, 1.07))
        .at(1.0, (property::OPACITY, 1.0))
        .at(1.0, (property::SCALE, 1.0))
        .at(1.0, (property::RADIUS, 18.0))
        .at(
            1.0,
            (property::BACKGROUND, Color::from_rgb(0.09, 0.12, 0.16)),
        )
        .at(
            1.0,
            (property::BORDER_COLOR, Color::from_rgb(0.35, 0.54, 0.74)),
        )
        .at(
            1.0,
            (property::SHADOW, shared::card_shadow(0.28, 16.0, 32.0)),
        )
}

fn popup_close_keyframes(from: &EffectSnapshot) -> Keyframes {
    Keyframes::new()
        .with_timing(Timing::new(160.0).with_easing(Easing::EaseIn))
        .at(0.0, (property::OPACITY, from.opacity.unwrap_or(1.0)))
        .at(0.0, (property::SCALE, from.scale.unwrap_or(1.0)))
        .at(1.0, (property::OPACITY, 0.0))
        .at(1.0, (property::SCALE, 0.94))
}

fn hidden_effects() -> EffectSnapshot {
    EffectSnapshot {
        opacity: Some(0.0),
        scale: Some(0.92),
        radius: Some(18.0),
        background: Some(Color::from_rgb(0.09, 0.12, 0.16)),
        border_color: Some(Color::from_rgb(0.35, 0.54, 0.74)),
        shadow: Some(shared::card_shadow(0.0, 10.0, 20.0)),
        ..EffectSnapshot::default()
    }
}
