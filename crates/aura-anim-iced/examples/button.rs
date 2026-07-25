//! Interactive button with tweened hover and spring press feedback.

use std::time::Instant;

use aura_anim_core::{
    binding::{MotionBinding, MotionBindingState},
    macros::Animatable,
    runtime::{Motion, MotionRuntime},
    spring::SpringConfig,
    timing::Timing,
};
use aura_anim_iced::Subscribe;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ButtonState {
    Idle,
    Hovered,
    Pressed,
}

struct ButtonExample {
    runtime: MotionRuntime,
    binding: MotionBinding<ButtonState, ButtonMotion>,
    binding_state: MotionBindingState<ButtonState>,
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
        let binding = button_binding();
        let (button, binding_state) = binding.create_motion(&mut runtime);

        Self {
            runtime,
            binding,
            binding_state,
            button,
            hovered: false,
            clicks: 0,
        }
    }

    fn set_button_state(&mut self, state: ButtonState) {
        if let Err(error) = self.binding.set_state(
            &mut self.binding_state,
            state,
            self.button,
            &mut self.runtime,
        ) {
            eprintln!("button motion binding failed: {error}");
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => self.runtime.frame(now),
            Message::Enter => {
                self.hovered = true;
                self.set_button_state(ButtonState::Hovered);
            }
            Message::Exit => {
                self.hovered = false;
                self.set_button_state(ButtonState::Idle);
            }
            Message::Press => self.set_button_state(ButtonState::Pressed),
            Message::Release => {
                self.clicks += 1;
                let state = if self.hovered {
                    ButtonState::Hovered
                } else {
                    ButtonState::Idle
                };
                self.set_button_state(state);
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        self.runtime.subscription().map(Message::Frame)
    }

    fn view(&self) -> Element<'_, Message> {
        let motion = self.button.value_ref(&self.runtime).unwrap();
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
                text("MotionBinding maps button states to targets and chooses Tween or Spring automatically.")
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

fn button_binding() -> MotionBinding<ButtonState, ButtonMotion> {
    MotionBinding::new(ButtonState::Idle, resting())
        .when(ButtonState::Hovered, hovered())
        .when(ButtonState::Pressed, pressed())
        .transition(ButtonState::Idle, ButtonState::Hovered, |context| {
            context.tween(Timing::new(170.0))
        })
        .transition(ButtonState::Hovered, ButtonState::Pressed, |context| {
            context.spring(SpringConfig::snappy())
        })
        .fallback(|context| context.tween(Timing::new(120.0)))
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
