//! Route transition demo with overlaid horizontal screen switching.

mod shared;

use aura_anim_iced::{
    AnimationHandle, AnimationRuntime, AnimationTargetId, Duration, Easing, EffectSnapshot,
    RouteAnimator, RouteIncomingMotion, RouteScreenTargets, RouteScreenTransition, Timeline,
    Timing, Track, iced_ext, property,
};
use iced::{
    Background, Border, Color, Element, Length, Shadow, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    widget::{Space, button, column, container, row, stack, text},
};
use std::time::Instant;

const CARD_WIDTH: f32 = 440.0;
const CARD_HEIGHT: f32 = 150.0;
const CARD_OFFSET: f32 = 50.0;
const STAGE_WIDTH: f32 = CARD_WIDTH + CARD_OFFSET * 2.0;
const TRANSITION_MS: f64 = 300.0;

fn main() -> iced::Result {
    iced::application(Demo::default, Demo::update, Demo::view)
        .title(title)
        .subscription(Demo::subscription)
        .run()
}

fn title(_: &Demo) -> String {
    String::from("aura-anim-iced route transition")
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Navigate(Route),
    AnimationTick(Instant),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Route {
    Home,
    Reports,
    Settings,
}

#[derive(Debug)]
struct Demo {
    runtime: AnimationRuntime,
    animator: RouteAnimator<Route>,
    outgoing_target: AnimationTargetId,
    incoming_target: AnimationTargetId,
    current: Route,
    leaving: Option<Route>,
    outgoing_effects: EffectSnapshot,
    incoming_effects: EffectSnapshot,
    active_route_handle: Option<AnimationHandle>,
    active_incoming_handle: Option<AnimationHandle>,
}

impl Default for Demo {
    fn default() -> Self {
        let route_target = AnimationTargetId::new();

        Self {
            runtime: AnimationRuntime::new(),
            animator: RouteAnimator::new(route_target, Route::Home),
            outgoing_target: AnimationTargetId::new(),
            incoming_target: AnimationTargetId::new(),
            current: Route::Home,
            leaving: None,
            outgoing_effects: hidden_left_effects(),
            incoming_effects: visible_effects(),
            active_route_handle: None,
            active_incoming_handle: None,
        }
    }
}

impl Demo {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(route) => self.navigate(route),
            Message::AnimationTick(tick_instant) => {
                let tick = iced_ext::update_tick(&mut self.runtime, tick_instant);
                let outgoing =
                    aura_anim_iced::tick_effect_snapshot_for(&tick, self.outgoing_target);
                let incoming =
                    aura_anim_iced::tick_effect_snapshot_for(&tick, self.incoming_target);

                if !outgoing.is_empty() {
                    self.outgoing_effects =
                        shared::merge_effects(&self.outgoing_effects, &outgoing);
                }

                if !incoming.is_empty() {
                    self.incoming_effects =
                        shared::merge_effects(&self.incoming_effects, &incoming);
                }

                if self
                    .active_incoming_handle
                    .is_some_and(|handle| tick.completed().contains(&handle))
                {
                    self.leaving = None;
                    self.outgoing_effects = hidden_left_effects();
                    self.incoming_effects = visible_effects();
                    self.active_incoming_handle = None;
                }

                if self
                    .active_route_handle
                    .is_some_and(|handle| tick.completed().contains(&handle))
                {
                    self.active_route_handle = None;
                    self.animator.handle_completion(&self.runtime);
                }
            }
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_ext::subscription(&self.runtime, Message::AnimationTick)
    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            route_button("Home", Route::Home, self.current),
            route_button("Reports", Route::Reports, self.current),
            route_button("Settings", Route::Settings, self.current),
        ]
        .spacing(12)
        .align_y(Vertical::Center);

        let stage = if let Some(route) = self.leaving {
            transition_stage(
                route,
                self.current,
                &self.outgoing_effects,
                &self.incoming_effects,
            )
        } else {
            translated_card(self.current, &self.incoming_effects, false)
        };

        container(
            column![controls, stage]
                .spacing(28)
                .align_x(Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(48)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    fn navigate(&mut self, to: Route) {
        if self.current == to {
            return;
        }

        let from = self.current;
        let transition = route_screen_transition(from, to);
        let registration = self.animator.transition_screens_with(
            &mut self.runtime,
            &transition,
            RouteScreenTargets::new(self.outgoing_target, self.incoming_target),
        );

        let Some(registration) = registration else {
            return;
        };

        self.leaving = Some(from);
        self.current = to;
        self.outgoing_effects = visible_effects();
        self.incoming_effects = hidden_right_effects();
        self.active_route_handle = Some(registration.route().handle());
        self.active_incoming_handle = Some(registration.incoming().handle());

        if let Some(properties) = registration.outgoing().properties() {
            self.outgoing_effects = shared::merge_effects(
                &self.outgoing_effects,
                &EffectSnapshot::from_properties(properties),
            );
        }

        if let Some(properties) = registration.incoming().properties() {
            self.incoming_effects = shared::merge_effects(
                &self.incoming_effects,
                &EffectSnapshot::from_properties(properties),
            );
        }
    }
}

fn route_button(label: &'static str, route: Route, current: Route) -> Element<'static, Message> {
    button(text(label))
        .on_press_maybe((route != current).then_some(Message::Navigate(route)))
        .into()
}

fn transition_stage<'a>(
    leaving: Route,
    current: Route,
    outgoing_effects: &'a EffectSnapshot,
    incoming_effects: &'a EffectSnapshot,
) -> Element<'a, Message> {
    stack![
        translated_card(leaving, outgoing_effects, true),
        translated_card(current, incoming_effects, false),
    ]
    .width(Length::Fixed(STAGE_WIDTH))
    .height(Length::Fixed(CARD_HEIGHT))
    .into()
}

fn translated_card(route: Route, effects: &EffectSnapshot, leaving: bool) -> Element<'_, Message> {
    let translation = effects.translation.unwrap_or_default();
    let left_offset = (CARD_OFFSET + translation.x).clamp(0.0, CARD_OFFSET * 2.0);

    row![
        Space::new().width(Length::Fixed(left_offset)),
        screen_card(route, effects, leaving),
    ]
    .width(Length::Fixed(STAGE_WIDTH))
    .height(Length::Fixed(CARD_HEIGHT))
    .align_y(Vertical::Center)
    .into()
}

fn screen_card(route: Route, effects: &EffectSnapshot, leaving: bool) -> Element<'_, Message> {
    let visual = screen_visual(route);
    let opacity = effects.opacity.unwrap_or(1.0).clamp(0.0, 1.0);
    let background = Color {
        a: opacity,
        ..visual.background
    };
    let border_color = Color {
        a: opacity,
        ..visual.accent
    };
    let shadow = Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.18 * opacity),
        offset: iced::Vector::new(0.0, 12.0),
        blur_radius: 28.0,
    };

    container(
        column![
            text(if leaving { "Leaving" } else { "Current" })
                .size(13)
                .color(Color::from_rgba(0.84, 0.89, 0.94, opacity)),
            text(visual.title)
                .size(28)
                .color(Color::from_rgba(1.0, 1.0, 1.0, opacity)),
            text(visual.body)
                .size(15)
                .color(Color::from_rgba(0.80, 0.86, 0.92, opacity)),
        ]
        .spacing(10),
    )
    .width(Length::Fixed(CARD_WIDTH))
    .height(Length::Fixed(CARD_HEIGHT))
    .padding(24)
    .align_y(Vertical::Center)
    .style(move |_theme: &Theme| container::Style {
        text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, opacity)),
        background: Some(Background::Color(background)),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 12.0.into(),
        },
        shadow,
        ..container::Style::default()
    })
    .into()
}

fn route_screen_transition(from: Route, to: Route) -> RouteScreenTransition<Route> {
    RouteScreenTransition::with_incoming_motion(
        from,
        to,
        outgoing_timeline(),
        RouteIncomingMotion::new(
            iced::Vector::new(CARD_OFFSET, 0.0),
            Duration::from_millis(TRANSITION_MS),
        ),
    )
}

fn outgoing_timeline() -> Timeline {
    let timing = Timing::new(TRANSITION_MS).with_easing(Easing::EaseOut);

    Timeline::parallel([
        shared::scalar_track(property::OPACITY, 1.0, 0.0, timing).into(),
        shared::scalar_track(property::SCALE, 1.0, 0.98, timing).into(),
        shared::scalar_track(property::RADIUS, 12.0, 12.0, timing).into(),
        Track::from(property::TRANSLATE, iced::Vector::new(0.0, 0.0))
            .to(iced::Vector::new(-CARD_OFFSET, 0.0))
            .duration(Duration::from_millis(TRANSITION_MS))
            .easing(Easing::EaseOut)
            .finish()
            .into(),
        shared::color_track(
            property::BACKGROUND,
            Color::from_rgb(0.11, 0.14, 0.18),
            Color::from_rgb(0.11, 0.14, 0.18),
            timing,
        )
        .into(),
    ])
}

fn visible_effects() -> EffectSnapshot {
    EffectSnapshot {
        opacity: Some(1.0),
        translation: Some(iced::Vector::new(0.0, 0.0)),
        scale: Some(1.0),
        radius: Some(12.0),
        ..EffectSnapshot::default()
    }
}

fn hidden_right_effects() -> EffectSnapshot {
    EffectSnapshot {
        opacity: Some(0.0),
        translation: Some(iced::Vector::new(CARD_OFFSET, 0.0)),
        scale: Some(1.0),
        radius: Some(12.0),
        ..EffectSnapshot::default()
    }
}

fn hidden_left_effects() -> EffectSnapshot {
    EffectSnapshot {
        opacity: Some(0.0),
        translation: Some(iced::Vector::new(-CARD_OFFSET, 0.0)),
        scale: Some(0.98),
        radius: Some(12.0),
        ..EffectSnapshot::default()
    }
}

fn screen_visual(route: Route) -> ScreenVisual {
    match route {
        Route::Home => ScreenVisual {
            title: "Home",
            body: "A compact overview screen used as the default route.",
            background: Color::from_rgb(0.10, 0.15, 0.19),
            accent: Color::from_rgb(0.36, 0.66, 0.82),
        },
        Route::Reports => ScreenVisual {
            title: "Reports",
            body: "A data-heavy route that benefits from smooth repeated navigation.",
            background: Color::from_rgb(0.14, 0.13, 0.20),
            accent: Color::from_rgb(0.70, 0.58, 0.90),
        },
        Route::Settings => ScreenVisual {
            title: "Settings",
            body: "A configuration route with a distinct visual identity.",
            background: Color::from_rgb(0.13, 0.17, 0.13),
            accent: Color::from_rgb(0.54, 0.74, 0.42),
        },
    }
}

#[derive(Debug, Clone, Copy)]
struct ScreenVisual {
    title: &'static str,
    body: &'static str,
    background: Color,
    accent: Color,
}
