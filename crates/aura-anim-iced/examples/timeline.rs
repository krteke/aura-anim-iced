//! Visual Sequence, Parallel, and Hold composition example.

use std::time::{Duration, Instant};

use aura_anim_core::{
    Animatable, AnimationExt, Hold, Motion, MotionRuntime, Parallel, Tween,
    timing::{Easing, Timing},
};
use iced::{
    Background, Border, Color, Element, Fill, Subscription, Theme,
    widget::{button, column, container, row, text},
};

#[derive(Clone, Debug, Animatable)]
struct Card {
    x: f32,
    y: f32,
    width: f32,
    opacity: f32,
}

struct TimelineExample {
    runtime: MotionRuntime,
    card: Motion<Card>,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Frame(Instant),
    Replay,
}

fn main() -> iced::Result {
    iced::application(
        TimelineExample::new,
        TimelineExample::update,
        TimelineExample::view,
    )
    .title("Aura Anim - Timeline")
    .theme(theme)
    .subscription(TimelineExample::subscription)
    .window_size((820.0, 520.0))
    .run()
}

impl TimelineExample {
    fn new() -> Self {
        let mut runtime = MotionRuntime::new();
        let card = runtime.motion(start());
        let mut example = Self { runtime, card };
        example.replay();
        example
    }

    fn replay(&mut self) {
        let start = start();
        let enter_x = Tween::between(
            start.clone(),
            Card {
                x: 390.0,
                ..start.clone()
            },
            Timing::new(520.0).with_easing(Easing::EaseOut),
        )
        .delay(Duration::from_millis(180));
        let enter_y = Tween::between(
            start.clone(),
            Card {
                y: 78.0,
                opacity: 1.0,
                ..start.clone()
            },
            Timing::new(700.0).with_easing(Easing::EaseOut),
        );
        let enter = Parallel::new(start.clone(), |outputs: &[Card]| Card {
            x: outputs[0].x,
            y: outputs[1].y,
            width: outputs[1].width,
            opacity: outputs[1].opacity,
        })
        .with(enter_x)
        .with(enter_y);
        let settled = Card {
            x: 390.0,
            y: 78.0,
            width: 230.0,
            opacity: 1.0,
        };
        let timeline = enter
            .then(Hold::new(settled.clone(), Duration::from_millis(420)))
            .then(Tween::between(
                settled,
                Card {
                    x: 250.0,
                    y: 96.0,
                    width: 360.0,
                    opacity: 0.35,
                },
                Timing::new(480.0).with_easing(Easing::EaseInOut),
            ));
        self.card.play(timeline, &mut self.runtime).unwrap();
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => aura_anim_iced::frame(&mut self.runtime, now),
            Message::Replay => self.replay(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        aura_anim_iced::subscription(&self.runtime).map(Message::Frame)
    }

    fn view(&self) -> Element<'_, Message> {
        let card = self.card.value_ref(&self.runtime).unwrap();
        let opacity = card.opacity;
        let visual = container(
            column![
                text("Parallel")
                    .size(13)
                    .color(Color::from_rgba8(126, 230, 205, opacity)),
                text("x + y + opacity")
                    .size(21)
                    .color(Color::from_rgba(1.0, 1.0, 1.0, opacity)),
                text("then Hold, then Tween")
                    .size(13)
                    .color(Color::from_rgba8(174, 182, 211, opacity))
            ]
            .spacing(8),
        )
        .width(card.width)
        .padding(20)
        .style(move |_| card_style(opacity));

        let stage = container(
            column![
                container("").height(card.y),
                row![
                    container("").width(card.x),
                    visual,
                    container("").width(Fill)
                ]
            ]
            .width(Fill),
        )
        .width(Fill)
        .height(260)
        .style(|_| stage_style());

        container(
            column![
                text("Timeline composition").size(34).color(Color::WHITE),
                text("Sequence controls phases. Parallel composes independent fields. Hold creates a pause.")
                    .size(14)
                    .color(Color::from_rgb8(174, 182, 211)),
                stage,
                row![
                    button("Replay timeline").on_press(Message::Replay),
                    text(format!(
                        "{:?}  x: {:.0}  y: {:.0}  width: {:.0}",
                        self.card.state(&self.runtime),
                        card.x,
                        card.y,
                        card.width
                    ))
                    .size(13)
                    .color(Color::from_rgb8(126, 230, 205))
                ]
                .spacing(14)
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

fn theme(_: &TimelineExample) -> Theme {
    Theme::TokyoNight
}

fn start() -> Card {
    Card {
        x: 24.0,
        y: 170.0,
        width: 150.0,
        opacity: 0.25,
    }
}

fn page_style() -> container::Style {
    container::Style::default()
        .background(Color::from_rgb8(18, 21, 34))
        .color(Color::WHITE)
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

fn card_style(opacity: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba8(95, 82, 210, opacity))),
        border: Border::default()
            .rounded(8)
            .width(1)
            .color(Color::from_rgba8(151, 139, 255, opacity)),
        ..container::Style::default()
    }
}
