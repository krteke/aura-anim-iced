//! Visual Tween example with interruption-safe retargeting.

use std::time::Instant;

use aura_anim_core::{
    Animatable, Motion, MotionRuntime, Tween,
    timing::{Easing, Timing},
};
use iced::{
    Background, Border, Color, Element, Fill, Padding, Shadow, Subscription, Theme, Vector,
    widget::{button, column, container, row, text},
};

#[derive(Clone, Debug, Animatable)]
struct Dot {
    x: f32,
    size: f32,
    glow: f32,
    color: Color,
}

struct TweenExample {
    runtime: MotionRuntime,
    dot: Motion<Dot>,
    at_end: bool,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Frame(Instant),
    Toggle,
    Replay,
}

fn main() -> iced::Result {
    iced::application(TweenExample::new, TweenExample::update, TweenExample::view)
        .title("Aura Anim - Tween")
        .theme(theme)
        .subscription(TweenExample::subscription)
        .window_size((820.0, 460.0))
        .run()
}

impl TweenExample {
    fn new() -> Self {
        let mut runtime = MotionRuntime::new();
        let dot = runtime.motion(start());

        let mut example = Self {
            runtime,
            dot,
            at_end: false,
        };
        example.animate_to(true);
        example
    }

    fn animate_to(&mut self, at_end: bool) {
        let current = self.dot.value(&self.runtime).unwrap();
        let target = if at_end { end() } else { start() };
        self.dot
            .play(
                Tween::between(
                    current,
                    target,
                    Timing::new(900.0).with_easing(Easing::EaseInOut),
                ),
                &mut self.runtime,
            )
            .unwrap();
        self.at_end = at_end;
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => aura_anim_iced::frame(&mut self.runtime, now),
            Message::Toggle => self.animate_to(!self.at_end),
            Message::Replay => {
                self.dot
                    .play(
                        Tween::between(
                            start(),
                            end(),
                            Timing::new(900.0).with_easing(Easing::EaseInOut),
                        ),
                        &mut self.runtime,
                    )
                    .unwrap();
                self.at_end = true;
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        aura_anim_iced::subscription(&self.runtime).map(Message::Frame)
    }

    fn view(&self) -> Element<'_, Message> {
        let dot = self.dot.value_ref(&self.runtime).unwrap();
        let color = dot.color;
        let marker = container("")
            .width(dot.size)
            .height(dot.size)
            .style(move |_| container::Style {
                background: Some(Background::Color(color)),
                border: Border::default().rounded(dot.size / 2.0),
                shadow: Shadow {
                    color: Color::from_rgba(color.r, color.g, color.b, dot.glow * 0.65),
                    offset: Vector::ZERO,
                    blur_radius: 18.0 + dot.glow * 30.0,
                },
                ..container::Style::default()
            });

        let track = container(
            row![
                container(marker).padding(Padding {
                    top: 18.0,
                    right: 0.0,
                    bottom: 18.0,
                    left: dot.x,
                }),
                container("").width(Fill)
            ]
            .width(Fill),
        )
        .width(Fill)
        .height(126)
        .style(|_| track_style());

        let state = format!(
            "{:?}  x: {:.0}  size: {:.0}",
            self.dot.state(&self.runtime),
            dot.x,
            dot.size
        );

        container(
            column![
                text("Tween").size(34).color(Color::WHITE),
                text("Interrupt the animation repeatedly: each tween starts from the current sampled value.")
                    .size(14)
                    .color(Color::from_rgb8(174, 182, 211)),
                track,
                row![
                    button("Retarget").on_press(Message::Toggle),
                    button("Replay").on_press(Message::Replay),
                    text(state).size(13).color(Color::from_rgb8(134, 225, 203))
                ]
                .spacing(12)
            ]
            .spacing(20),
        )
        .width(Fill)
        .height(Fill)
        .padding(34)
        .style(|_| page_style())
        .into()
    }
}

fn start() -> Dot {
    Dot {
        x: 0.0,
        size: 54.0,
        glow: 0.2,
        color: Color::from_rgb8(83, 216, 190),
    }
}

fn end() -> Dot {
    Dot {
        x: 560.0,
        size: 82.0,
        glow: 0.9,
        color: Color::from_rgb8(255, 113, 151),
    }
}

fn theme(_: &TweenExample) -> Theme {
    Theme::TokyoNight
}

fn page_style() -> container::Style {
    container::Style::default()
        .background(Color::from_rgb8(18, 21, 34))
        .color(Color::WHITE)
}

fn track_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(27, 32, 50))),
        border: Border::default()
            .rounded(8)
            .width(1)
            .color(Color::from_rgb8(61, 70, 99)),
        ..container::Style::default()
    }
}
