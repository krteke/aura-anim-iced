//! Toast demo driven by a timeline sequence.

mod shared;

use aura_anim_iced::{
    iced_ext::{self, EffectSnapshot},
    property::{self, PropertyKey, PropertySpec},
    runtime::{AnimationHandle, AnimationRuntime, AnimationTargetId},
    timeline::{Hold, Timeline},
    timing::{Duration, Easing, Timing},
};
use iced::{
    Background, Border, Color, Element, Length, Shadow, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    widget::{Space, button, column, container, row, text},
};
use std::time::Instant;

const TOAST_Y: PropertySpec<property::Scalar> =
    PropertySpec::new(PropertyKey::new("example", "toast-y"), 21);

fn main() -> iced::Result {
    iced::application(Demo::default, Demo::update, Demo::view)
        .title(title)
        .subscription(Demo::subscription)
        .run()
}

fn title(_: &Demo) -> String {
    String::from("aura-anim-iced timeline toast")
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ShowToast,
    DismissToast,
    AnimationTick(Instant),
}

#[derive(Debug)]
struct Demo {
    runtime: AnimationRuntime,
    toast_target: AnimationTargetId,
    effects: EffectSnapshot,
    y_offset: f32,
    toast: Option<Toast>,
}

#[derive(Debug)]
struct Toast {
    text: String,
    handle: AnimationHandle,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            runtime: AnimationRuntime::new(),
            toast_target: AnimationTargetId::new(),
            effects: hidden_effects(),
            y_offset: 28.0,
            toast: None,
        }
    }
}

impl Demo {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowToast => self.show_toast(),
            Message::DismissToast => self.dismiss_toast(),
            Message::AnimationTick(tick_instant) => {
                let tick = iced_ext::update_tick(&mut self.runtime, tick_instant);
                let effects = iced_ext::tick_effect_snapshot_for(&tick, self.toast_target);

                if !effects.is_empty() {
                    self.effects = shared::merge_effects(&self.effects, &effects);
                }

                if let Some(y) = shared::tick_scalar(&tick, self.toast_target, TOAST_Y) {
                    self.y_offset = y;
                }

                if let Some(toast) = &self.toast
                    && tick.completed().contains(&toast.handle)
                {
                    self.toast = None;
                    self.effects = hidden_effects();
                    self.y_offset = 28.0;
                }
            }
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_ext::subscription(&self.runtime, Message::AnimationTick)
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            button(text("Show toast")).on_press(Message::ShowToast),
            button(text("Dismiss"))
                .on_press_maybe(self.toast.as_ref().map(|_| Message::DismissToast)),
        ]
        .spacing(12);

        let toast_area: Element<'_, Message> = if let Some(toast) = &self.toast {
            column![
                Space::new().height(Length::Fixed(self.y_offset.max(0.0))),
                toast_card(toast.text.clone(), &self.effects),
            ]
            .align_x(Horizontal::Center)
            .into()
        } else {
            container(text("No active toast").size(16))
                .width(Length::Fill)
                .height(Length::Fixed(120.0))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into()
        };

        container(
            column![controls, toast_area]
                .spacing(28)
                .align_x(Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(48)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    fn show_toast(&mut self) {
        let timeline = toast_lifecycle_timeline();
        let registration = self.runtime.register_timeline(self.toast_target, timeline);

        self.toast = Some(Toast {
            text: String::from("Enter, hold, exit, then cleanup."),
            handle: registration.handle(),
        });
        self.effects = hidden_effects();
        self.y_offset = 28.0;
    }

    fn dismiss_toast(&mut self) {
        if self.toast.is_none() {
            return;
        }

        let timeline = toast_exit_timeline(&self.effects, self.y_offset);
        let registration = self.runtime.register_timeline(self.toast_target, timeline);

        if let Some(toast) = &mut self.toast {
            toast.handle = registration.handle();
        }
    }
}

fn toast_card(label: String, effects: &EffectSnapshot) -> Element<'_, Message> {
    let opacity = effects.opacity.unwrap_or(1.0).clamp(0.0, 1.0);
    let scale = effects.scale.unwrap_or(1.0);
    let background = effects
        .background
        .unwrap_or(Color::from_rgb(0.12, 0.14, 0.17));
    let border_color = effects
        .border_color
        .unwrap_or(Color::from_rgb(0.30, 0.45, 0.55));
    let shadow = effects
        .shadow
        .unwrap_or_else(|| shared::card_shadow(0.22, 10.0, 24.0));

    container(
        row![
            text("Timeline Toast")
                .size(16.0 * scale)
                .color(Color::from_rgba(1.0, 1.0, 1.0, opacity)),
            text(label)
                .size(14.0)
                .color(Color::from_rgba(0.78, 0.86, 0.92, opacity)),
        ]
        .spacing(16)
        .align_y(Vertical::Center),
    )
    .width(Length::Fixed(420.0 * scale))
    .height(Length::Fixed(66.0 * scale))
    .padding([0, 20])
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
            radius: 14.0.into(),
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

fn toast_lifecycle_timeline() -> Timeline {
    Timeline::sequence([
        toast_enter_timeline().root().clone().into(),
        Hold::new(Duration::from_millis(1_200.0)).into(),
        toast_exit_timeline(&visible_effects(), 0.0)
            .root()
            .clone()
            .into(),
    ])
}

fn toast_enter_timeline() -> Timeline {
    let timing = Timing::new(220.0).with_easing(Easing::EaseOut);

    Timeline::parallel([
        shared::scalar_track(property::OPACITY, 0.0, 1.0, timing).into(),
        shared::scalar_track(property::SCALE, 0.98, 1.0, timing).into(),
        shared::scalar_track(TOAST_Y, 28.0, 0.0, timing).into(),
        shared::color_track(
            property::BACKGROUND,
            Color::from_rgb(0.12, 0.14, 0.17),
            Color::from_rgb(0.12, 0.14, 0.17),
            timing,
        )
        .into(),
        shared::color_track(
            property::BORDER_COLOR,
            Color::from_rgb(0.30, 0.45, 0.55),
            Color::from_rgb(0.30, 0.45, 0.55),
            timing,
        )
        .into(),
        shared::shadow_track(
            shared::card_shadow(0.12, 6.0, 16.0),
            shared::card_shadow(0.22, 10.0, 24.0),
            timing,
        )
        .into(),
    ])
}

fn toast_exit_timeline(from: &EffectSnapshot, from_y: f32) -> Timeline {
    let timing = Timing::new(180.0).with_easing(Easing::EaseIn);

    Timeline::parallel([
        shared::scalar_track(property::OPACITY, from.opacity.unwrap_or(1.0), 0.0, timing).into(),
        shared::scalar_track(property::SCALE, from.scale.unwrap_or(1.0), 0.98, timing).into(),
        shared::scalar_track(TOAST_Y, from_y, 18.0, timing).into(),
        shared::shadow_track(
            from.shadow
                .unwrap_or_else(|| shared::card_shadow(0.22, 10.0, 24.0)),
            shared::card_shadow(0.0, 6.0, 16.0),
            timing,
        )
        .into(),
    ])
}

fn visible_effects() -> EffectSnapshot {
    EffectSnapshot {
        opacity: Some(1.0),
        scale: Some(1.0),
        background: Some(Color::from_rgb(0.12, 0.14, 0.17)),
        border_color: Some(Color::from_rgb(0.30, 0.45, 0.55)),
        shadow: Some(shared::card_shadow(0.22, 10.0, 24.0)),
        ..EffectSnapshot::default()
    }
}

fn hidden_effects() -> EffectSnapshot {
    EffectSnapshot {
        opacity: Some(0.0),
        scale: Some(0.98),
        background: Some(Color::from_rgb(0.12, 0.14, 0.17)),
        border_color: Some(Color::from_rgb(0.30, 0.45, 0.55)),
        shadow: Some(shared::card_shadow(0.0, 6.0, 16.0)),
        ..EffectSnapshot::default()
    }
}
