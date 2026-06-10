//! Route transition that swaps content between exit and enter animations.

use std::time::Instant;

use aura_anim_core::{
    Animatable, Motion, MotionRuntime, PlaybackId, Tween, timing::Timing, tween_to,
};
use iced::{
    Background, Border, Color, Element, Fill, Subscription, Theme,
    widget::{button, column, container, row, text},
};

#[derive(Clone, Debug, Animatable)]
struct RouteMotion {
    opacity: f32,
    offset: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Route {
    Overview,
    Activity,
    Settings,
}

impl Route {
    const ALL: [Self; 3] = [Self::Overview, Self::Activity, Self::Settings];

    const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Activity => "Activity",
            Self::Settings => "Settings",
        }
    }
}

struct RouteExample {
    runtime: MotionRuntime,
    motion: Motion<RouteMotion>,
    route: Route,
    pending: Option<Route>,
    entering: bool,
    playback: Option<PlaybackId>,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Frame(Instant),
    Navigate(Route),
}

fn main() -> iced::Result {
    iced::application(RouteExample::new, RouteExample::update, RouteExample::view)
        .title("Aura Anim - Route Transition")
        .theme(theme)
        .subscription(RouteExample::subscription)
        .window_size((900.0, 560.0))
        .run()
}

impl RouteExample {
    fn new() -> Self {
        let mut runtime = MotionRuntime::new();
        let motion = runtime.motion(visible());
        Self {
            runtime,
            motion,
            route: Route::Overview,
            pending: None,
            entering: false,
            playback: None,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => {
                aura_anim_iced::frame(&mut self.runtime, now);

                for event in self.runtime.take_events() {
                    let Some(playback) = self.playback else {
                        continue;
                    };
                    if !event.is_completed_for(playback) {
                        continue;
                    }

                    if self.entering {
                        self.pending = None;
                        self.entering = false;
                        self.playback = None;
                    } else {
                        let Some(pending) = self.pending else {
                            continue;
                        };
                        self.route = pending;
                        self.entering = true;
                        self.playback = Some(
                            self.motion
                                .play_tracked(
                                    Tween::between(
                                        hidden_right(),
                                        visible(),
                                        Timing::ease_out(260.0),
                                    ),
                                    &mut self.runtime,
                                )
                                .unwrap(),
                        );
                    }
                }
            }
            Message::Navigate(route) => {
                if route == self.route || self.pending == Some(route) {
                    return;
                }

                self.playback = Some(
                    self.motion
                        .play_tracked(
                            tween_to(hidden_left(), Timing::ease_in(150.0)),
                            &mut self.runtime,
                        )
                        .unwrap(),
                );
                self.pending = Some(route);
                self.entering = false;
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        aura_anim_iced::subscription(&self.runtime).map(Message::Frame)
    }

    fn view(&self) -> Element<'_, Message> {
        let motion = self.motion.value_ref(&self.runtime).unwrap();
        let navigation = row(Route::ALL.map(|route| {
            let selected = route == self.route;
            button(text(route.label()).size(14))
                .on_press(Message::Navigate(route))
                .style(move |theme, status| {
                    if selected {
                        button::primary(theme, status)
                    } else {
                        button::secondary(theme, status)
                    }
                })
                .into()
        }))
        .spacing(8);

        let cards = row((0..3).map(|index| {
            let accent = route_color(self.route, index);
            container(
                column![
                    text(format!("0{}", index + 1))
                        .size(13)
                        .color(with_alpha(accent, motion.opacity)),
                    text(card_title(self.route, index))
                        .size(19)
                        .color(Color::from_rgba(1.0, 1.0, 1.0, motion.opacity)),
                    text(card_detail(self.route, index))
                        .size(12)
                        .color(Color::from_rgba(0.68, 0.72, 0.84, motion.opacity))
                ]
                .spacing(9),
            )
            .width(Fill)
            .height(150)
            .padding(18)
            .style(move |_| card_style(accent, motion.opacity))
            .into()
        }))
        .spacing(12);

        let content = container(
            column![
                text(self.route.label()).size(30).color(Color::from_rgba(
                    1.0,
                    1.0,
                    1.0,
                    motion.opacity
                )),
                text(route_detail(self.route))
                    .size(14)
                    .color(Color::from_rgba(0.68, 0.72, 0.84, motion.opacity)),
                cards
            ]
            .spacing(18),
        )
        .width(Fill)
        .padding([26.0, 26.0 + motion.offset.abs()])
        .style(|_| container::Style::default());

        container(
            column![
                text("Route transition").size(32).color(Color::WHITE),
                text("The old route exits, content swaps after completion, then the new route enters.")
                    .size(14)
                    .color(Color::from_rgb8(169, 178, 207)),
                navigation,
                container(content)
                    .width(Fill)
                    .height(300)
                    .style(|_| stage_style()),
                text(format!(
                    "route: {}  |  phase: {}",
                    self.route.label(),
                    if self.pending.is_none() {
                        "idle"
                    } else if self.entering {
                        "entering"
                    } else {
                        "exiting"
                    }
                ))
                .size(13)
                .color(Color::from_rgb8(111, 226, 198))
            ]
            .spacing(18),
        )
        .width(Fill)
        .height(Fill)
        .padding(30)
        .style(|_| page_style())
        .into()
    }
}

fn visible() -> RouteMotion {
    RouteMotion {
        opacity: 1.0,
        offset: 0.0,
    }
}

fn hidden_left() -> RouteMotion {
    RouteMotion {
        opacity: 0.0,
        offset: -24.0,
    }
}

fn hidden_right() -> RouteMotion {
    RouteMotion {
        opacity: 0.0,
        offset: 24.0,
    }
}

fn route_detail(route: Route) -> &'static str {
    match route {
        Route::Overview => "A compact summary of the animation runtime.",
        Route::Activity => "Recent transitions and interrupted motion targets.",
        Route::Settings => "Timing, easing, and lifecycle preferences.",
    }
}

fn card_title(route: Route, index: usize) -> &'static str {
    match (route, index) {
        (Route::Overview, 0) => "Active",
        (Route::Overview, 1) => "Stored",
        (Route::Overview, _) => "Completed",
        (Route::Activity, 0) => "Tween",
        (Route::Activity, 1) => "Spring",
        (Route::Activity, _) => "Timeline",
        (Route::Settings, 0) => "Timing",
        (Route::Settings, 1) => "Easing",
        (Route::Settings, _) => "Retention",
    }
}

fn card_detail(route: Route, index: usize) -> &'static str {
    match (route, index) {
        (Route::Overview, 0) => "2 motions",
        (Route::Overview, 1) => "6 handles",
        (Route::Overview, _) => "24 today",
        (Route::Activity, 0) => "180 ms",
        (Route::Activity, 1) => "damping 24",
        (Route::Activity, _) => "3 phases",
        (Route::Settings, 0) => "60 fps",
        (Route::Settings, 1) => "EaseOut",
        (Route::Settings, _) => "Keep final value",
    }
}

fn route_color(route: Route, index: usize) -> Color {
    match (route, index) {
        (_, 0) => Color::from_rgb8(108, 96, 238),
        (_, 1) => Color::from_rgb8(55, 201, 151),
        _ => Color::from_rgb8(239, 91, 122),
    }
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::from_rgba(color.r, color.g, color.b, alpha)
}

fn theme(_: &RouteExample) -> Theme {
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

fn card_style(accent: Color, opacity: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            accent.r * 0.16,
            accent.g * 0.16,
            accent.b * 0.16,
            opacity,
        ))),
        border: Border::default()
            .rounded(8)
            .width(1)
            .color(Color::from_rgba(
                accent.r,
                accent.g,
                accent.b,
                opacity * 0.45,
            )),
        ..container::Style::default()
    }
}
