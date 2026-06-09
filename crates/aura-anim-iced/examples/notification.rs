//! Popup notification with animated enter, replacement, and exit.

use std::time::Instant;

use aura_anim_core::{
    Animatable, MotionRuntime, Presence, Spring, SpringConfig, Tween,
    timing::{Easing, Timing},
};
use iced::{
    Background, Border, Color, Element, Fill, Shadow, Subscription, Theme, Vector,
    widget::{button, column, container, row, text},
};

#[derive(Clone, Debug, Animatable)]
struct ToastMotion {
    width: f32,
    opacity: f32,
    lift: f32,
    glow: f32,
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Success,
    Warning,
    Error,
}

impl Kind {
    const fn title(self) -> &'static str {
        match self {
            Self::Success => "Changes saved",
            Self::Warning => "Connection unstable",
            Self::Error => "Upload failed",
        }
    }

    const fn detail(self) -> &'static str {
        match self {
            Self::Success => "Your workspace is up to date.",
            Self::Warning => "Working offline until the network returns.",
            Self::Error => "Check the file and try again.",
        }
    }

    const fn color(self) -> Color {
        match self {
            Self::Success => Color::from_rgb8(55, 201, 151),
            Self::Warning => Color::from_rgb8(241, 178, 74),
            Self::Error => Color::from_rgb8(239, 91, 122),
        }
    }
}

struct NotificationExample {
    runtime: MotionRuntime,
    toast: Presence<ToastMotion>,
    kind: Kind,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Frame(Instant),
    Show(Kind),
    Dismiss,
}

fn main() -> iced::Result {
    iced::application(
        NotificationExample::new,
        NotificationExample::update,
        NotificationExample::view,
    )
    .title("Aura Anim - Popup Notification")
    .theme(theme)
    .subscription(NotificationExample::subscription)
    .window_size((820.0, 520.0))
    .run()
}

impl NotificationExample {
    fn new() -> Self {
        let mut runtime = MotionRuntime::new();
        let toast = Presence::new(
            &mut runtime,
            hidden(),
            visible(),
            Timing::new(180.0).with_easing(Easing::EaseOut),
        );

        Self {
            runtime,
            toast,
            kind: Kind::Success,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => {
                aura_anim_iced::frame(&mut self.runtime, now);
                self.toast.sync(&self.runtime).unwrap();
            }
            Message::Show(kind) => {
                self.kind = kind;
                let current = self.toast.value(&self.runtime).unwrap().clone();
                self.toast
                    .show_with(
                        Spring::new(
                            current,
                            visible(),
                            SpringConfig {
                                stiffness: 330.0,
                                damping: 24.0,
                                ..SpringConfig::default()
                            },
                        ),
                        &mut self.runtime,
                    )
                    .unwrap();
            }
            Message::Dismiss => {
                let current = self.toast.value(&self.runtime).unwrap().clone();
                self.toast
                    .hide_with(
                        Tween::between(
                            current,
                            hidden(),
                            Timing::new(160.0).with_easing(Easing::EaseIn),
                        ),
                        &mut self.runtime,
                    )
                    .unwrap();
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        aura_anim_iced::subscription(&self.runtime).map(Message::Frame)
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            button("Success").on_press(Message::Show(Kind::Success)),
            button("Warning").on_press(Message::Show(Kind::Warning)),
            button("Error").on_press(Message::Show(Kind::Error))
        ]
        .spacing(10);

        let toast: Element<'_, Message> = if self.toast.is_mounted() {
            let motion = self.toast.value(&self.runtime).unwrap();
            let accent = self.kind.color();
            container(
                row![
                    container("").width(4).height(58).style(move |_| {
                        container::Style::default()
                            .background(accent)
                            .border(Border::default().rounded(2))
                    }),
                    column![
                        text(self.kind.title()).size(17).color(Color::from_rgba(
                            1.0,
                            1.0,
                            1.0,
                            motion.opacity
                        )),
                        text(self.kind.detail()).size(13).color(Color::from_rgba(
                            0.72,
                            0.75,
                            0.86,
                            motion.opacity
                        ))
                    ]
                    .spacing(7)
                    .width(Fill),
                    button("Dismiss").on_press(Message::Dismiss)
                ]
                .spacing(14),
            )
            .width(motion.width.max(1.0))
            .padding(16.0 + motion.lift)
            .style(move |_| toast_style(accent, motion.glow, motion.opacity))
            .into()
        } else {
            container("").width(0).into()
        };

        container(
            column![
                text("Popup notification").size(32).color(Color::WHITE),
                text("Showing a new kind retargets the mounted notification instead of rebuilding its animation state.")
                    .size(14)
                    .color(Color::from_rgb8(169, 178, 207)),
                controls,
                container(toast)
                    .width(Fill)
                    .height(230)
                    .center_x(Fill)
                    .center_y(Fill),
                text(format!(
                    "mounted: {}  visible target: {}",
                    self.toast.is_mounted(),
                    self.toast.is_visible()
                ))
                .size(13)
                .color(Color::from_rgb8(130, 139, 170))
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

fn hidden() -> ToastMotion {
    ToastMotion {
        width: 280.0,
        opacity: 0.0,
        lift: 0.0,
        glow: 0.0,
    }
}

fn visible() -> ToastMotion {
    ToastMotion {
        width: 520.0,
        opacity: 1.0,
        lift: 3.0,
        glow: 0.45,
    }
}

fn theme(_: &NotificationExample) -> Theme {
    Theme::TokyoNight
}

fn page_style() -> container::Style {
    container::Style::default()
        .background(Color::from_rgb8(18, 21, 34))
        .color(Color::WHITE)
}

fn toast_style(accent: Color, glow: f32, opacity: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            0.12, 0.14, 0.22, opacity,
        ))),
        border: Border::default()
            .rounded(8)
            .width(1)
            .color(Color::from_rgba(
                accent.r,
                accent.g,
                accent.b,
                opacity * 0.55,
            )),
        shadow: Shadow {
            color: Color::from_rgba(accent.r, accent.g, accent.b, glow * 0.3),
            offset: Vector::new(0.0, 10.0),
            blur_radius: 24.0,
        },
        ..container::Style::default()
    }
}
