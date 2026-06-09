//! Interactive showcase for Aura animation primitives in Iced.

use std::time::Instant;

use aura_anim_core::{
    Animatable, Hold, Motion, MotionRuntime, Parallel, Presence, Sequence, Spring, SpringConfig,
    Tween,
    keyframes::Keyframes,
    timing::{Direction, Easing, IterationCount, Timing},
};
use iced::{
    Background, Border, Color, Element, Fill, Shadow, Subscription, Theme, Vector,
    widget::{button, column, container, mouse_area, row, text},
};

#[derive(Clone, Debug, Animatable)]
struct HeroMotion {
    width: f32,
    lift: f32,
    glow: f32,
    accent: Color,
}

#[derive(Clone, Debug, Animatable)]
struct MenuMotion {
    width: f32,
    opacity: f32,
}

#[derive(Clone, Debug, Animatable)]
struct RouteMotion {
    opacity: f32,
    offset: Vector,
}

#[derive(Clone, Debug, Animatable)]
struct CardMotion {
    height: f32,
    glow: f32,
}

#[derive(Clone, Debug, Animatable)]
struct PulseMotion {
    glow: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Route {
    Dashboard,
    Activity,
    Settings,
}

impl Route {
    const ALL: [Self; 3] = [Self::Dashboard, Self::Activity, Self::Settings];

    const fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Activity => "Activity",
            Self::Settings => "Settings",
        }
    }
}

struct Showcase {
    runtime: MotionRuntime,
    hero: Motion<HeroMotion>,
    menu: Presence<MenuMotion>,
    route_motion: Motion<RouteMotion>,
    cards: [Motion<CardMotion>; 3],
    pulse: Motion<PulseMotion>,
    route: Route,
    pending_route: Option<Route>,
    route_swapped: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Frame(Instant),
    HeroEnter,
    HeroExit,
    HeroPress,
    HeroRelease,
    ToggleMenu,
    Navigate(Route),
    ReplayCards,
}

fn main() -> iced::Result {
    iced::application(Showcase::new, Showcase::update, Showcase::view)
        .title("Aura Motion Runtime")
        .theme(theme)
        .subscription(Showcase::subscription)
        .window_size((1080.0, 720.0))
        .run()
}

impl Showcase {
    fn new() -> Self {
        let mut runtime = MotionRuntime::new();
        let hero = runtime.motion_with(
            HeroMotion {
                width: 430.0,
                lift: 0.0,
                glow: 0.18,
                accent: Color::from_rgb(0.39, 0.45, 1.0),
            },
            Timing::new(180.0).with_easing(Easing::EaseOut),
        );
        let menu = Presence::new(
            &mut runtime,
            MenuMotion {
                width: 0.0,
                opacity: 0.0,
            },
            MenuMotion {
                width: 236.0,
                opacity: 1.0,
            },
            Timing::new(180.0).with_easing(Easing::EaseOut),
        );
        let route_motion = runtime.motion(RouteMotion {
            opacity: 1.0,
            offset: Vector::new(0.0, 0.0),
        });
        let cards = std::array::from_fn(|_| {
            runtime.motion(CardMotion {
                height: 138.0,
                glow: 0.08,
            })
        });
        let pulse = runtime.motion(PulseMotion { glow: 0.2 });
        pulse
            .play(
                Keyframes::new(PulseMotion { glow: 0.2 })
                    .push_eased(900.0, PulseMotion { glow: 0.9 }, Easing::EaseInOut)
                    .push_eased(1800.0, PulseMotion { glow: 0.2 }, Easing::EaseInOut)
                    .with_iterations(IterationCount::INFINITE)
                    .with_direction(Direction::Alternate),
                &mut runtime,
            )
            .unwrap();

        Self {
            runtime,
            hero,
            menu,
            route_motion,
            cards,
            pulse,
            route: Route::Dashboard,
            pending_route: None,
            route_swapped: false,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => {
                aura_anim_iced::frame(&mut self.runtime, now);

                self.menu.sync(&self.runtime).unwrap();

                if let Some(pending) = self.pending_route
                    && self.route_motion.is_completed(&self.runtime).unwrap()
                {
                    if self.route_swapped {
                        self.pending_route = None;
                        self.route_swapped = false;
                    } else {
                        self.route = pending;
                        self.route_swapped = true;
                        let hidden = self.route_motion.value(&self.runtime).unwrap();
                        self.route_motion
                            .play(
                                Tween::between(
                                    hidden,
                                    RouteMotion {
                                        opacity: 1.0,
                                        offset: Vector::new(0.0, 0.0),
                                    },
                                    Timing::new(210.0).with_easing(Easing::EaseOut),
                                ),
                                &mut self.runtime,
                            )
                            .unwrap();
                    }
                }
            }
            Message::HeroEnter => {
                self.hero
                    .transition_to(
                        HeroMotion {
                            width: 454.0,
                            lift: 8.0,
                            glow: 0.55,
                            accent: Color::from_rgb(0.51, 0.41, 1.0),
                        },
                        &mut self.runtime,
                    )
                    .unwrap();
            }
            Message::HeroExit => {
                self.hero
                    .transition_to(
                        HeroMotion {
                            width: 430.0,
                            lift: 0.0,
                            glow: 0.18,
                            accent: Color::from_rgb(0.39, 0.45, 1.0),
                        },
                        &mut self.runtime,
                    )
                    .unwrap();
            }
            Message::HeroPress => {
                let current = self.hero.value(&self.runtime).unwrap();
                self.hero
                    .play(
                        Spring::new(
                            current,
                            HeroMotion {
                                width: 414.0,
                                lift: 2.0,
                                glow: 0.8,
                                accent: Color::from_rgb(0.64, 0.36, 1.0),
                            },
                            SpringConfig {
                                stiffness: 240.0,
                                damping: 10.0,
                                ..SpringConfig::default()
                            },
                        ),
                        &mut self.runtime,
                    )
                    .unwrap();
            }
            Message::HeroRelease => {
                self.hero
                    .transition_to(
                        HeroMotion {
                            width: 454.0,
                            lift: 8.0,
                            glow: 0.55,
                            accent: Color::from_rgb(0.51, 0.5, 1.0),
                        },
                        &mut self.runtime,
                    )
                    .unwrap();
            }
            Message::ToggleMenu => {
                let current = self.menu.value(&self.runtime).unwrap().clone();
                let target = if self.menu.is_visible() {
                    MenuMotion {
                        width: 0.0,
                        opacity: 0.0,
                    }
                } else {
                    MenuMotion {
                        width: 236.0,
                        opacity: 1.0,
                    }
                };
                let animation = Spring::new(
                    current,
                    target,
                    SpringConfig {
                        stiffness: 300.0,
                        damping: 31.0,
                        ..SpringConfig::default()
                    },
                );
                if self.menu.is_visible() {
                    self.menu.hide_with(animation, &mut self.runtime).unwrap();
                } else {
                    self.menu.show_with(animation, &mut self.runtime).unwrap();
                }
            }
            Message::Navigate(route) => {
                if route == self.route || self.pending_route == Some(route) {
                    return;
                }

                let current = self.route_motion.value(&self.runtime).unwrap();
                let hidden = RouteMotion {
                    opacity: 0.0,
                    offset: Vector::new(-18.0, 0.0),
                };
                self.route_motion
                    .play(
                        Tween::between(
                            current,
                            hidden,
                            Timing::new(130.0).with_easing(Easing::EaseIn),
                        ),
                        &mut self.runtime,
                    )
                    .unwrap();
                self.pending_route = Some(route);
                self.route_swapped = false;
            }
            Message::ReplayCards => {
                for (index, card) in self.cards.iter().copied().enumerate() {
                    let current = card.value(&self.runtime).unwrap();
                    let delay = std::time::Duration::from_millis(index as u64 * 55);
                    let compressed = CardMotion {
                        height: 112.0,
                        glow: 0.65,
                    };
                    let height = Sequence::new(current.clone())
                        .then(Hold::new(current.clone(), delay))
                        .then(Tween::between(
                            current.clone(),
                            CardMotion {
                                height: compressed.height,
                                glow: current.glow,
                            },
                            Timing::new(120.0).with_easing(Easing::EaseIn),
                        ));
                    let glow = Sequence::new(current.clone())
                        .then(Hold::new(current.clone(), delay))
                        .then(Tween::between(
                            current.clone(),
                            CardMotion {
                                height: current.height,
                                glow: compressed.glow,
                            },
                            Timing::new(120.0).with_easing(Easing::EaseIn),
                        ));
                    let parallel =
                        Parallel::new(current.clone(), |outputs: &[CardMotion]| CardMotion {
                            height: outputs[0].height,
                            glow: outputs[1].glow,
                        })
                        .with(height)
                        .with(glow);
                    card.play(
                        Sequence::new(current).then(parallel).then(Tween::between(
                            compressed,
                            CardMotion {
                                height: 138.0,
                                glow: 0.08,
                            },
                            Timing::new(260.0).with_easing(Easing::EaseOut),
                        )),
                        &mut self.runtime,
                    )
                    .unwrap();
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        aura_anim_iced::subscription_with_policy(&self.runtime, aura_anim_iced::TickPolicy::fps(60))
            .map(Message::Frame)
    }

    #[allow(clippy::too_many_lines)]
    fn view(&self) -> Element<'_, Message> {
        let hero = self.hero.value_ref(&self.runtime).unwrap();
        let pulse = self.pulse.value_ref(&self.runtime).unwrap();
        let route_motion = self.route_motion.value_ref(&self.runtime).unwrap();

        let hero_color = hero.accent;
        let hero_panel = container(
            column![
                row![
                    container(text("AURA").size(13))
                        .padding([5, 9])
                        .style(move |_| panel_style(hero_color, 0.18, 8.0)),
                    text("typed motion runtime").size(14).color(Color::from_rgb8(158, 166, 202))
                ]
                .spacing(10),
                text("Motion that stays\nout of your way.")
                    .size(38)
                    .color(Color::WHITE),
                text("Hover and press this panel rapidly. Tween and Spring retarget from the current value without visual jumps.")
                    .size(15)
                    .color(Color::from_rgb8(178, 184, 214)),
                row![
                    button("Replay cards").on_press(Message::ReplayCards),
                    button(if self.menu.is_visible() { "Close menu" } else { "Open menu" })
                        .on_press(Message::ToggleMenu)
                ]
                .spacing(10)
            ]
            .spacing(18),
        )
        .width(hero.width)
        .padding(28.0 + hero.lift)
        .style(move |_| panel_style(hero_color, hero.glow, 22.0));

        let hero_area = mouse_area(hero_panel)
            .on_enter(Message::HeroEnter)
            .on_exit(Message::HeroExit)
            .on_press(Message::HeroPress)
            .on_release(Message::HeroRelease);

        let nav = row(Route::ALL.map(|route| {
            let selected = self.route == route;
            button(text(route.label()).size(14))
                .on_press(Message::Navigate(route))
                .style(move |theme, status| {
                    if selected {
                        button::Style::default()
                            .with_background(Color::from_rgb8(104, 91, 255))
                            .text_color(Color::WHITE)
                            .border(Border::default().rounded(10))
                    } else {
                        button::secondary(theme, status)
                    }
                })
                .into()
        }))
        .spacing(8);

        let route_content = container(
            column![
                text(self.route.label()).size(26).color(Color::WHITE),
                text(route_description(self.route))
                    .size(14)
                    .color(Color::from_rgba8(188, 194, 225, route_motion.opacity)),
                row(self.cards.iter().enumerate().map(|(index, card)| {
                    let motion = card.value_ref(&self.runtime).unwrap();
                    let tint = match index {
                        0 => Color::from_rgb8(112, 99, 255),
                        1 => Color::from_rgb8(39, 207, 173),
                        _ => Color::from_rgb8(255, 116, 155),
                    };
                    container(
                        column![
                            text(format!("0{}", index + 1))
                                .size(13)
                                .color(Color::from_rgba(
                                    tint.r,
                                    tint.g,
                                    tint.b,
                                    route_motion.opacity
                                )),
                            text(card_title(self.route, index))
                                .size(18)
                                .color(Color::from_rgba(1.0, 1.0, 1.0, route_motion.opacity)),
                            text("Independent typed handle")
                                .size(12)
                                .color(Color::from_rgba(0.66, 0.69, 0.82, route_motion.opacity))
                        ]
                        .spacing(8),
                    )
                    .width(Fill)
                    .height(motion.height)
                    .padding(18)
                    .style(move |_| panel_style(tint, motion.glow, 16.0))
                    .into()
                }))
                .spacing(12)
            ]
            .spacing(18),
        )
        .width(Fill)
        .padding([26.0, 26.0 + route_motion.offset.x.abs()])
        .style(move |_| {
            panel_style(
                Color::from_rgb8(56, 62, 94),
                0.1 * route_motion.opacity,
                20.0,
            )
        });

        let status = container(
            row![
                container("")
                    .width(10.0 + pulse.glow * 4.0)
                    .height(10.0 + pulse.glow * 4.0)
                    .style(move |_| {
                        panel_style(Color::from_rgb8(75, 224, 176), pulse.glow, 20.0)
                    }),
                text(format!("{} active motion(s)", self.runtime.active_count()))
                    .size(13)
                    .color(Color::from_rgb8(164, 171, 204))
            ]
            .spacing(10),
        )
        .padding([8, 12])
        .style(|_| panel_style(Color::from_rgb8(56, 62, 94), 0.08, 12.0));

        let main = column![nav, hero_area, route_content, status]
            .spacing(18)
            .width(Fill);

        let menu: Element<'_, Message> = if self.menu.is_mounted() {
            let motion = self.menu.value(&self.runtime).unwrap();
            container(
                column![
                    text("Quick menu").size(20).color(Color::from_rgba(
                        1.0,
                        1.0,
                        1.0,
                        motion.opacity
                    )),
                    text("Spring-driven presence")
                        .size(13)
                        .color(Color::from_rgba(0.65, 0.69, 0.83, motion.opacity)),
                    text("The widget remains mounted until its exit motion completes.")
                        .size(13)
                        .color(Color::from_rgba(0.78, 0.80, 0.9, motion.opacity))
                ]
                .spacing(14),
            )
            .width(motion.width.max(1.0))
            .height(Fill)
            .padding(22)
            .style(move |_: &Theme| {
                panel_style(
                    Color::from_rgba(0.3, 0.27, 0.62, motion.opacity),
                    motion.opacity * 0.45,
                    20.0,
                )
            })
            .into()
        } else {
            container("").width(0).into()
        };

        container(row![main, menu].spacing(18))
            .width(Fill)
            .height(Fill)
            .padding(28)
            .style(|_| {
                container::Style::default()
                    .background(Color::from_rgb8(18, 20, 34))
                    .color(Color::WHITE)
            })
            .into()
    }
}

fn theme(_: &Showcase) -> Theme {
    Theme::TokyoNight
}

fn panel_style(accent: Color, glow: f32, radius: f32) -> container::Style {
    container::Style {
        text_color: Some(Color::WHITE),
        background: Some(Background::Color(Color::from_rgba(
            accent.r * 0.18,
            accent.g * 0.18,
            accent.b * 0.18,
            0.96,
        ))),
        border: Border::default()
            .rounded(radius)
            .width(1)
            .color(Color::from_rgba(
                accent.r,
                accent.g,
                accent.b,
                0.25 + glow * 0.45,
            )),
        shadow: Shadow {
            color: Color::from_rgba(accent.r, accent.g, accent.b, glow * 0.32),
            offset: Vector::new(0.0, 8.0 + glow * 8.0),
            blur_radius: 18.0 + glow * 26.0,
        },
        ..container::Style::default()
    }
}

fn route_description(route: Route) -> &'static str {
    match route {
        Route::Dashboard => "Coordinated cards driven by separate Motion<CardMotion> handles.",
        Route::Activity => "The route content swaps at the midpoint of a Timeline transition.",
        Route::Settings => {
            "All sources share pause, cancel, seek, finish, and lifecycle semantics."
        }
    }
}

fn card_title(route: Route, index: usize) -> &'static str {
    match (route, index) {
        (Route::Dashboard, 0) => "Runtime",
        (Route::Dashboard, 1) => "Handles",
        (Route::Dashboard, _) => "Sources",
        (Route::Activity, 0) => "Retarget",
        (Route::Activity, 1) => "Timeline",
        (Route::Activity, _) => "Presence",
        (Route::Settings, 0) => "Timing",
        (Route::Settings, 1) => "Direction",
        (Route::Settings, _) => "Iterations",
    }
}

trait ButtonStyleExt {
    fn text_color(self, color: Color) -> Self;
    fn border(self, border: Border) -> Self;
}

impl ButtonStyleExt for button::Style {
    fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }
}
