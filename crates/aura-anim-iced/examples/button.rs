//! Interactive button with tweened hover and spring press feedback.

use std::time::Instant;

use aura_anim_core::{
    Animatable, Motion, MotionRuntime, Spring, SpringConfig,
    timing::{Easing, Timing},
};
use iced::{
    Background, Border, Color, Element, Fill, Shadow, Subscription, Theme, Vector,
    widget::{column, container, mouse_area, row, text},
};

#[derive(Clone, Debug, Animatable)]
struct ButtonMotion {
    width: f32,
    height: f32,
    glow: f32,
    lift: f32,
    color: Color,
}

struct ButtonExample {
    runtime: MotionRuntime,
    button: Motion<ButtonMotion>,
    hovered: bool,
    clicks: u32,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Frame(Instant),
    Enter,
    Exit,
    Press,
    Release,
}

fn main() -> iced::Result {
    iced::application(
        ButtonExample::new,
        ButtonExample::update,
        ButtonExample::view,
    )
    .title("Aura Anim - Interactive Button")
    .theme(theme)
    .subscription(ButtonExample::subscription)
    .window_size((720.0, 440.0))
    .run()
}

impl ButtonExample {
    fn new() -> Self {
        let mut runtime = MotionRuntime::new();
        let button =
            runtime.motion_with(resting(), Timing::new(170.0).with_easing(Easing::EaseOut));

        Self {
            runtime,
            button,
            hovered: false,
            clicks: 0,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => aura_anim_iced::frame(&mut self.runtime, now),
            Message::Enter => {
                self.hovered = true;
                self.button.transition_to(hovered(), &mut self.runtime);
            }
            Message::Exit => {
                self.hovered = false;
                self.button.transition_to(resting(), &mut self.runtime);
            }
            Message::Press => {
                let current = self.button.value(&self.runtime);
                self.button.play(
                    Spring::new(
                        current,
                        pressed(),
                        SpringConfig {
                            stiffness: 380.0,
                            damping: 22.0,
                            ..SpringConfig::default()
                        },
                    ),
                    &mut self.runtime,
                );
            }
            Message::Release => {
                self.clicks += 1;
                let target = if self.hovered { hovered() } else { resting() };
                self.button.transition_to(target, &mut self.runtime);
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        aura_anim_iced::subscription(&self.runtime).map(Message::Frame)
    }

    fn view(&self) -> Element<'_, Message> {
        let motion = self.button.value_ref(&self.runtime);
        let color = motion.color;
        let surface = container(
            row![
                text("Run animation").size(16).color(Color::WHITE),
                text("->").size(18).color(Color::WHITE)
            ]
            .spacing(12),
        )
        .center_x(motion.width)
        .center_y(motion.height)
        .style(move |_| container::Style {
            background: Some(Background::Color(color)),
            border: Border::default()
                .rounded(8)
                .width(1)
                .color(Color::from_rgba(1.0, 1.0, 1.0, 0.22)),
            shadow: Shadow {
                color: Color::from_rgba(color.r, color.g, color.b, motion.glow),
                offset: Vector::new(0.0, 8.0 + motion.lift),
                blur_radius: 18.0 + motion.glow * 28.0,
            },
            ..container::Style::default()
        });

        let interactive = mouse_area(surface)
            .on_enter(Message::Enter)
            .on_exit(Message::Exit)
            .on_press(Message::Press)
            .on_release(Message::Release);

        container(
            column![
                text("Interactive button").size(32).color(Color::WHITE),
                text("Hover uses transition_to. Press replaces it with a spring from the current value.")
                    .size(14)
                    .color(Color::from_rgb8(169, 178, 207)),
                container(interactive)
                    .width(Fill)
                    .height(180)
                    .center_x(Fill)
                    .center_y(Fill),
                text(format!(
                    "{} click(s)  |  {:?}",
                    self.clicks,
                    self.button.state(&self.runtime)
                ))
                .size(13)
                .color(Color::from_rgb8(111, 226, 198))
            ]
            .spacing(18),
        )
        .width(Fill)
        .height(Fill)
        .padding(32)
        .style(|_| page_style())
        .into()
    }
}

fn resting() -> ButtonMotion {
    ButtonMotion {
        width: 190.0,
        height: 56.0,
        glow: 0.18,
        lift: 0.0,
        color: Color::from_rgb8(83, 100, 224),
    }
}

fn hovered() -> ButtonMotion {
    ButtonMotion {
        width: 206.0,
        height: 60.0,
        glow: 0.55,
        lift: 4.0,
        color: Color::from_rgb8(107, 91, 238),
    }
}

fn pressed() -> ButtonMotion {
    ButtonMotion {
        width: 182.0,
        height: 52.0,
        glow: 0.72,
        lift: 0.0,
        color: Color::from_rgb8(131, 78, 223),
    }
}

fn theme(_: &ButtonExample) -> Theme {
    Theme::TokyoNight
}

fn page_style() -> container::Style {
    container::Style::default()
        .background(Color::from_rgb8(18, 21, 34))
        .color(Color::WHITE)
}
