//! Width value-change demo driven by [`BehaviorRule`] and [`PropertyTransition`].

mod shared;

use aura_anim_iced::{
    AnimationRuntime, AnimationTargetId, BehaviorRule, Easing, EffectSnapshot, PropertyTransition,
    Timing, WIDTH, iced_ext,
};
use iced::{
    Background, Border, Color, Element, Length, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    widget::{column, container, text},
};
use std::time::Instant;

const INITIAL_WIDTH: f32 = 90.0;
const TARGET_WIDTH: f32 = 420.0;
const TRANSITION_MS: f64 = 1_800.0;

fn main() -> iced::Result {
    iced::application(Demo::default, Demo::update, Demo::view)
        .title(title)
        .subscription(Demo::subscription)
        .run()
}

fn title(_: &Demo) -> String {
    String::from("aura-anim-iced behavior width")
}

#[derive(Debug, Clone, Copy)]
enum Message {
    AnimationTick(Instant),
}

#[derive(Debug)]
struct Demo {
    runtime: AnimationRuntime,
    width_target: AnimationTargetId,
    width_transition: PropertyTransition<aura_anim_iced::property::Scalar>,
    effects: EffectSnapshot,
}

impl Default for Demo {
    fn default() -> Self {
        let mut runtime = AnimationRuntime::new();
        let width_target = AnimationTargetId::new();
        let rule = BehaviorRule::new(WIDTH)
            .with_timing(Timing::new(TRANSITION_MS).with_easing(Easing::EaseOut));
        let mut width_transition = rule.bind(width_target);

        width_transition.transition_to(&mut runtime, INITIAL_WIDTH);
        width_transition.transition_to(&mut runtime, TARGET_WIDTH);

        Self {
            runtime,
            width_target,
            width_transition,
            effects: width_effects(INITIAL_WIDTH),
        }
    }
}

impl Demo {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AnimationTick(tick_instant) => {
                let tick = iced_ext::update_tick(&mut self.runtime, tick_instant);
                let effects = aura_anim_iced::tick_effect_snapshot_for(&tick, self.width_target);

                if !effects.is_empty() {
                    self.effects = shared::merge_effects(&self.effects, &effects);
                }

                self.width_transition.handle_completion(&self.runtime);
            }
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_ext::subscription(&self.runtime, Message::AnimationTick)
    }

    fn view(&self) -> Element<'_, Message> {
        let width = self.effects.width.unwrap_or(INITIAL_WIDTH);

        container(
            column![
                container(text("Width").size(18).color(Color::WHITE))
                    .width(Length::Fixed(width))
                    .height(Length::Fixed(72.0))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .style(bar_style),
                text(format!("{width:.0}px"))
                    .size(16)
                    .color(Color::from_rgb(0.62, 0.72, 0.78)),
            ]
            .spacing(16)
            .align_x(Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(48)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

fn width_effects(width: f32) -> EffectSnapshot {
    EffectSnapshot {
        width: Some(width),
        ..EffectSnapshot::default()
    }
}

fn bar_style(_theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(Color::WHITE),
        background: Some(Background::Color(Color::from_rgb(0.18, 0.35, 0.42))),
        border: Border {
            color: Color::from_rgb(0.50, 0.78, 0.82),
            width: 1.0,
            radius: 10.0.into(),
        },
        ..container::Style::default()
    }
}
