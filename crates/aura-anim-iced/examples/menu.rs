//! Animated side menu using Presence and Spring.

use std::time::Instant;

use aura_anim_core::{
    macros::Animatable, presence::Presence, runtime::MotionRuntime, spring::SpringConfig,
    target::spring_to, timing::Timing,
};
use aura_anim_iced::Subscribe;
use iced::{
    Background, Border, Color, Element, Fill, Shadow, Subscription, Theme, Vector,
    widget::{button, column, container, row, text},
};

#[derive(Clone, Debug, Animatable)]
struct MenuMotion {
    width: f32,
    opacity: f32,
    glow: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Inbox,
    Projects,
    Archive,
}

impl Section {
    const ALL: [Self; 3] = [Self::Inbox, Self::Projects, Self::Archive];

    const fn label(self) -> &'static str {
        match self {
            Self::Inbox => "Inbox",
            Self::Projects => "Projects",
            Self::Archive => "Archive",
        }
    }
}

struct MenuExample {
    runtime: MotionRuntime,
    menu: Presence<MenuMotion>,
    selected: Section,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Frame(Instant),
    Toggle,
    Select(Section),
}

fn main() -> iced::Result {
    iced::application(MenuExample::new, MenuExample::update, MenuExample::view)
        .title("Aura Anim - Animated Menu")
        .theme(theme)
        .subscription(MenuExample::subscription)
        .window_size((860.0, 520.0))
        .run()
}

impl MenuExample {
    fn new() -> Self {
        let mut runtime = MotionRuntime::new();
        let menu = Presence::new(&mut runtime, hidden(), visible(), Timing::ease_out(190.0));

        Self {
            runtime,
            menu,
            selected: Section::Inbox,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => {
                self.runtime.frame(now);
                for event in self.runtime.take_events() {
                    self.menu.handle_event(&event);
                }
            }
            Message::Toggle => {
                let target = if self.menu.is_visible() {
                    hidden()
                } else {
                    visible()
                };
                let spring = spring_to(target, SpringConfig::new(300.0, 29.0));

                if self.menu.is_visible() {
                    self.menu.hide_with(spring, &mut self.runtime).unwrap();
                } else {
                    self.menu.show_with(spring, &mut self.runtime).unwrap();
                }
            }
            Message::Select(section) => self.selected = section,
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        self.runtime.subscription().map(Message::Frame)
    }

    fn view(&self) -> Element<'_, Message> {
        let main = container(
            column![
                row![
                    button(if self.menu.is_visible() {
                        "Close menu"
                    } else {
                        "Open menu"
                    })
                    .on_press(Message::Toggle),
                    text("Workspace").size(14).color(Color::from_rgb8(160, 169, 200))
                ]
                .spacing(14),
                text(self.selected.label()).size(32).color(Color::WHITE),
                text("The main content stays interactive while the menu mounts, animates, and unmounts.")
                    .size(14)
                    .color(Color::from_rgb8(169, 178, 207)),
                container(
                    column![
                        text("Selected view").size(13).color(Color::from_rgb8(111, 226, 198)),
                        text(format!("{} content", self.selected.label()))
                            .size(22)
                            .color(Color::WHITE)
                    ]
                    .spacing(10)
                )
                .width(Fill)
                .height(220)
                .padding(24)
                .style(|_| panel_style(Color::from_rgb8(55, 65, 96), 0.12))
            ]
            .spacing(20),
        )
        .width(Fill)
        .height(Fill);

        let menu: Element<'_, Message> = if self.menu.is_mounted() {
            let motion = self.menu.value(&self.runtime).unwrap();
            let entries = column(Section::ALL.map(|section| {
                let selected = section == self.selected;
                button(text(section.label()).size(15))
                    .width(Fill)
                    .on_press(Message::Select(section))
                    .style(move |theme, status| {
                        if selected {
                            button::primary(theme, status)
                        } else {
                            button::text(theme, status)
                        }
                    })
                    .into()
            }))
            .spacing(8);

            container(
                column![
                    text("Navigation").size(20).color(Color::from_rgba(
                        1.0,
                        1.0,
                        1.0,
                        motion.opacity
                    )),
                    entries,
                    text("Presence keeps this mounted until exit completes.")
                        .size(12)
                        .color(Color::from_rgba(0.66, 0.7, 0.84, motion.opacity))
                ]
                .spacing(18),
            )
            .width(motion.width.max(1.0))
            .height(Fill)
            .padding(20)
            .style(move |_| panel_style(Color::from_rgb8(95, 82, 210), motion.glow))
            .into()
        } else {
            container("").width(0).into()
        };

        container(row![menu, main].spacing(20))
            .width(Fill)
            .height(Fill)
            .padding(28)
            .style(|_| page_style())
            .into()
    }
}

fn hidden() -> MenuMotion {
    MenuMotion {
        width: 0.0,
        opacity: 0.0,
        glow: 0.0,
    }
}

fn visible() -> MenuMotion {
    MenuMotion {
        width: 240.0,
        opacity: 1.0,
        glow: 0.42,
    }
}

fn theme(_: &MenuExample) -> Theme {
    Theme::TokyoNight
}

fn page_style() -> container::Style {
    container::Style::default()
        .background(Color::from_rgb8(18, 21, 34))
        .color(Color::WHITE)
}

fn panel_style(accent: Color, glow: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            accent.r * 0.22,
            accent.g * 0.22,
            accent.b * 0.22,
            0.96,
        ))),
        border: Border::default()
            .rounded(8)
            .width(1)
            .color(Color::from_rgba(
                accent.r,
                accent.g,
                accent.b,
                0.28 + glow * 0.4,
            )),
        shadow: Shadow {
            color: Color::from_rgba(accent.r, accent.g, accent.b, glow * 0.3),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 18.0 + glow * 24.0,
        },
        ..container::Style::default()
    }
}
