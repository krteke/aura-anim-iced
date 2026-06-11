//! Visual Spring example with live retargeting.

use std::time::Instant;

use aura_anim_core::{Animatable, Motion, MotionRuntime, Spring, SpringConfig};
use iced::{
    Background, Border, Color, Element, Fill, Shadow, Subscription, Theme, Vector,
    widget::{button, column, container, row, text},
};

#[derive(Clone, Debug, Animatable)]
struct SpringValue {
    x: f32,
    size: f32,
}

struct SpringExample {
    runtime: MotionRuntime,
    value: Motion<SpringValue>,
    target_right: bool,
    bouncy: bool,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Frame(Instant),
    Retarget,
    ToggleDamping,
}

fn main() -> iced::Result {
    iced::application(
        SpringExample::new,
        SpringExample::update,
        SpringExample::view,
    )
    .title("Aura Anim - Spring")
    .theme(theme)
    .subscription(SpringExample::subscription)
    .window_size((820.0, 470.0))
    .run()
}

impl SpringExample {
    fn new() -> Self {
        let mut runtime = MotionRuntime::new();
        let value = runtime.motion(SpringValue {
            x: 28.0,
            size: 58.0,
        });
        let mut example = Self {
            runtime,
            value,
            target_right: false,
            bouncy: true,
        };
        example.retarget();
        example
    }

    fn retarget(&mut self) {
        self.target_right = !self.target_right;
        let current = self.value.value(&self.runtime).unwrap();
        let target = if self.target_right {
            SpringValue {
                x: 620.0,
                size: 84.0,
            }
        } else {
            SpringValue {
                x: 28.0,
                size: 58.0,
            }
        };
        let movement = SpringConfig::new(210.0, if self.bouncy { 11.0 } else { 30.0 });
        let size = SpringConfig::new(420.0, if self.bouncy { 24.0 } else { 42.0 });
        self.value
            .play(
                Spring::with_channels(current, target, [movement, size], |outputs| SpringValue {
                    x: outputs[0].x,
                    size: outputs[1].size,
                }),
                &mut self.runtime,
            )
            .unwrap();
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => aura_anim_iced::frame(&mut self.runtime, now),
            Message::Retarget => self.retarget(),
            Message::ToggleDamping => {
                self.bouncy = !self.bouncy;
                self.retarget();
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        aura_anim_iced::subscription(&self.runtime).map(Message::Frame)
    }

    fn view(&self) -> Element<'_, Message> {
        let value = self.value.value_ref(&self.runtime).unwrap();
        let color = if self.bouncy {
            Color::from_rgb8(255, 124, 159)
        } else {
            Color::from_rgb8(83, 216, 190)
        };
        let body = container("")
            .width(value.size)
            .height(value.size)
            .style(move |_| container::Style {
                background: Some(Background::Color(color)),
                border: Border::default().rounded(value.size / 2.0),
                shadow: Shadow {
                    color: Color::from_rgba(color.r, color.g, color.b, 0.55),
                    offset: Vector::ZERO,
                    blur_radius: 28.0,
                },
                ..container::Style::default()
            });
        let stage = container(row![
            container("").width(value.x),
            body,
            container("").width(Fill)
        ])
        .width(Fill)
        .height(142)
        .padding([28, 0])
        .style(|_| stage_style());

        container(
            column![
                text("Spring").size(34).color(Color::WHITE),
                text("Position and size use independent physical channels and can be retargeted while moving.")
                    .size(14)
                    .color(Color::from_rgb8(174, 182, 211)),
                stage,
                row![
                    button("Retarget").on_press(Message::Retarget),
                    button(if self.bouncy {
                        "Use critical damping"
                    } else {
                        "Use bouncy damping"
                    })
                    .on_press(Message::ToggleDamping)
                ]
                .spacing(12),
                text(format!(
                    "{:?}  x: {:.1}  size: {:.1}  movement damping: {}",
                    self.value.state(&self.runtime),
                    value.x,
                    value.size,
                    if self.bouncy { 11 } else { 30 }
                ))
                .size(13)
                .color(color)
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

fn theme(_: &SpringExample) -> Theme {
    Theme::TokyoNight
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
