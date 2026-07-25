//! Visual Keyframes example with per-segment easing.

use std::time::Instant;

use aura_anim_core::{
    keyframes::Keyframes,
    macros::Animatable,
    runtime::{Motion, MotionRuntime},
    timing::{Easing, IterationCount},
};
use aura_anim_iced::Subscribe;
use iced::{
    Background, Border, Color, Element, Fill, Shadow, Subscription, Theme, Vector,
    widget::{button, column, container, row, text},
};

#[derive(Clone, Debug, Animatable)]
struct Ball {
    x: f32,
    y: f32,
    scale: f32,
    hue: Color,
}

struct KeyframesExample {
    runtime: MotionRuntime,
    ball: Motion<Ball>,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Frame(Instant),
    Replay,
}

fn main() -> iced::Result {
    iced::application(
        KeyframesExample::new,
        KeyframesExample::update,
        KeyframesExample::view,
    )
    .title("Aura Anim - Keyframes")
    .theme(theme)
    .subscription(KeyframesExample::subscription)
    .window_size((820.0, 520.0))
    .run()
}

impl KeyframesExample {
    fn new() -> Self {
        let mut runtime = MotionRuntime::new();
        let ball = runtime.motion(start());
        let mut example = Self { runtime, ball };
        example.replay();
        example
    }

    fn replay(&mut self) {
        self.ball.play(animation(), &mut self.runtime).unwrap();
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => self.runtime.frame(now),
            Message::Replay => self.replay(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        self.runtime.subscription().map(Message::Frame)
    }

    fn view(&self) -> Element<'_, Message> {
        let ball = self.ball.value_ref(&self.runtime).unwrap();
        let size = 46.0 * ball.scale;
        let hue = ball.hue;
        let marker = container("")
            .width(size)
            .height(size)
            .style(move |_| container::Style {
                background: Some(Background::Color(hue)),
                border: Border::default().rounded(size / 2.0),
                shadow: Shadow {
                    color: Color::from_rgba(hue.r, hue.g, hue.b, 0.55),
                    offset: Vector::ZERO,
                    blur_radius: 24.0,
                },
                ..container::Style::default()
            });

        let stage = container(
            column![
                container("").height(ball.y),
                row![
                    container("").width(ball.x),
                    marker,
                    container("").width(Fill)
                ]
            ]
            .width(Fill),
        )
        .width(Fill)
        .height(250)
        .style(|_| stage_style());

        container(
            column![
                text("Keyframes").size(34).color(Color::WHITE),
                text("Four time-positioned poses create anticipation, overshoot, and recovery.")
                    .size(14)
                    .color(Color::from_rgb8(174, 182, 211)),
                stage,
                row![
                    button("Replay").on_press(Message::Replay),
                    text("0 ms").size(12),
                    text("350 ms: anticipate").size(12),
                    text("760 ms: overshoot").size(12),
                    text("1200 ms: settle").size(12)
                ]
                .spacing(16),
                text(format!(
                    "{:?}  position: ({:.0}, {:.0})  scale: {:.2}",
                    self.ball.state(&self.runtime),
                    ball.x,
                    ball.y,
                    ball.scale
                ))
                .size(13)
                .color(Color::from_rgb8(255, 189, 116))
            ]
            .spacing(18),
        )
        .width(Fill)
        .height(Fill)
        .padding(34)
        .style(|_| page_style())
        .into()
    }
}

fn start() -> Ball {
    Ball {
        x: 30.0,
        y: 140.0,
        scale: 1.0,
        hue: Color::from_rgb8(93, 214, 190),
    }
}

fn animation() -> Keyframes<Ball> {
    Keyframes::new(start())
        .push_eased(
            350.0,
            Ball {
                x: 8.0,
                y: 154.0,
                scale: 0.82,
                hue: Color::from_rgb8(93, 214, 190),
            },
            Easing::EaseIn,
        )
        .push_eased(
            760.0,
            Ball {
                x: 650.0,
                y: 30.0,
                scale: 1.35,
                hue: Color::from_rgb8(255, 118, 157),
            },
            Easing::EaseOut,
        )
        .push_eased(
            1200.0,
            Ball {
                x: 570.0,
                y: 116.0,
                scale: 1.0,
                hue: Color::from_rgb8(119, 105, 255),
            },
            Easing::EaseInOut,
        )
        .with_iterations(IterationCount::ONCE)
}

fn theme(_: &KeyframesExample) -> Theme {
    Theme::TokyoNight
}

fn page_style() -> container::Style {
    container::Style::default()
        .background(Color::from_rgb8(18, 21, 34))
        .color(Color::from_rgb8(174, 182, 211))
}

fn stage_style() -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(27, 32, 50))),
        border: Border::default()
            .rounded(8)
            .width(1)
            .color(Color::from_rgb8(61, 70, 99)),
        ..container::Style::default()
    }
}
