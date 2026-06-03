//! Theme color interpolation comparison for sRGB and palette-backed Oklab.

#[cfg(feature = "palette")]
mod shared;

#[cfg(feature = "palette")]
use std::time::Instant;

#[cfg(feature = "palette")]
use aura_anim_iced::{
    color::{AnimColor, tag},
    iced_ext::{self, EffectSnapshot},
    keyframes::KeyframesBuilder,
    property::{self, PropertySpec},
    runtime::{AnimationRuntime, AnimationTargetId},
    timeline::{Timeline, Track},
    timing::{Easing, Timing},
};
#[cfg(feature = "palette")]
use iced::{
    Background, Border, Color, Element, Length, Subscription, Task, Theme,
    alignment::{Horizontal, Vertical},
    widget::{button, column, container, row, text},
};

#[cfg(not(feature = "palette"))]
fn main() -> iced::Result {
    eprintln!(
        "Run this example with the palette feature: cargo run --features palette --example theme_palette_compare"
    );

    Ok(())
}

#[cfg(feature = "palette")]
fn main() -> iced::Result {
    iced::application(Demo::default, Demo::update, Demo::view)
        .title(title)
        .subscription(Demo::subscription)
        .run()
}

#[cfg(feature = "palette")]
fn title(_: &Demo) -> String {
    String::from("aura-anim-iced theme palette compare")
}

#[cfg(feature = "palette")]
const TRANSITION_MS: f64 = 1_600.0;
#[cfg(feature = "palette")]
const CARD_WIDTH: f32 = 300.0;
#[cfg(feature = "palette")]
const CARD_HEIGHT: f32 = 190.0;

#[cfg(feature = "palette")]
#[derive(Debug, Clone, Copy)]
enum Message {
    NextTheme,
    AnimationTick(Instant),
}

#[cfg(feature = "palette")]
#[derive(Debug)]
struct Demo {
    runtime: AnimationRuntime,
    srgb_target: AnimationTargetId,
    oklab_target: AnimationTargetId,
    srgb_effects: EffectSnapshot,
    oklab_effects: EffectSnapshot,
    theme: ThemeChoice,
}

#[cfg(feature = "palette")]
impl Default for Demo {
    fn default() -> Self {
        let theme = ThemeChoice::Blue;
        let visual = theme.visual();

        Self {
            runtime: AnimationRuntime::new(),
            srgb_target: AnimationTargetId::new(),
            oklab_target: AnimationTargetId::new(),
            srgb_effects: visual.effects(),
            oklab_effects: visual.effects(),
            theme,
        }
    }
}

#[cfg(feature = "palette")]
impl Demo {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NextTheme => self.next_theme(),
            Message::AnimationTick(tick_instant) => {
                let tick = iced_ext::update_tick(&mut self.runtime, tick_instant);
                let srgb = iced_ext::tick_effect_snapshot_for(&tick, self.srgb_target);
                let oklab = iced_ext::tick_effect_snapshot_for(&tick, self.oklab_target);

                if !srgb.is_empty() {
                    self.srgb_effects = shared::merge_effects(&self.srgb_effects, &srgb);
                }

                if !oklab.is_empty() {
                    self.oklab_effects = shared::merge_effects(&self.oklab_effects, &oklab);
                }
            }
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_ext::subscription(&self.runtime, Message::AnimationTick)
    }

    fn view(&self) -> Element<'_, Message> {
        let next = self.theme.next();
        let controls = row![
            text(format!("Theme: {}", self.theme.name()))
                .size(18)
                .color(Color::from_rgb(0.80, 0.86, 0.90)),
            button(text(format!("Switch to {}", next.name()))).on_press(Message::NextTheme),
        ]
        .spacing(18)
        .align_y(Vertical::Center);

        let cards = row![
            theme_card("sRGB alpha", &self.srgb_effects),
            theme_card("Oklab alpha", &self.oklab_effects),
        ]
        .spacing(20)
        .align_y(Vertical::Center);

        container(
            column![controls, cards]
                .spacing(24)
                .align_x(Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(48)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(app_style)
        .into()
    }

    fn next_theme(&mut self) {
        let next = self.theme.next();
        let target = next.visual();

        self.runtime.register_timeline(
            self.srgb_target,
            theme_timeline_srgb(ThemeVisual::from_effects(&self.srgb_effects), target),
        );
        self.runtime.register_timeline(
            self.oklab_target,
            theme_timeline_oklab(ThemeVisual::from_effects(&self.oklab_effects), target),
        );
        self.theme = next;
    }
}

#[cfg(feature = "palette")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeChoice {
    Blue,
    Yellow,
    Red,
    CyanBlue,
    Purple,
    YellowGreen,
}

#[cfg(feature = "palette")]
impl ThemeChoice {
    fn next(self) -> Self {
        match self {
            Self::Blue => Self::Yellow,
            Self::Yellow => Self::Red,
            Self::Red => Self::CyanBlue,
            Self::CyanBlue => Self::Purple,
            Self::Purple => Self::YellowGreen,
            Self::YellowGreen => Self::Blue,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Blue => "Blue",
            Self::Yellow => "Yellow",
            Self::Red => "Red",
            Self::CyanBlue => "CyanBlue",
            Self::Purple => "Purple",
            Self::YellowGreen => "YellowGreen",
        }
    }

    #[allow(clippy::unreadable_literal)]
    fn visual(self) -> ThemeVisual {
        match self {
            Self::Blue => ThemeVisual {
                background: AnimColor::from_hex::<tag::Srgba>(0x0000FFFF).into_iced(),
                border: Color::BLACK,
                text: Color::BLACK,
            },
            Self::Yellow => ThemeVisual {
                background: AnimColor::from_hex::<tag::Srgba>(0xFFFF00FF).into_iced(),
                border: Color::BLACK,
                text: Color::BLACK,
            },
            Self::Red => ThemeVisual {
                background: AnimColor::from_hex::<tag::Srgba>(0xFF0000FF).into_iced(),
                border: Color::WHITE,
                text: Color::WHITE,
            },
            Self::CyanBlue => ThemeVisual {
                background: AnimColor::from_hex::<tag::Srgba>(0x00FFFFFF).into_iced(),
                border: Color::BLACK,
                text: Color::BLACK,
            },
            Self::Purple => ThemeVisual {
                background: AnimColor::from_hex::<tag::Srgba>(0x8000FFFF).into_iced(),
                border: Color::WHITE,
                text: Color::WHITE,
            },
            Self::YellowGreen => ThemeVisual {
                background: AnimColor::from_hex::<tag::Srgba>(0xB8FF00FF).into_iced(),
                border: Color::BLACK,
                text: Color::BLACK,
            },
        }
    }
}

#[cfg(feature = "palette")]
#[derive(Debug, Clone, Copy)]
struct ThemeVisual {
    background: Color,
    border: Color,
    text: Color,
}

#[cfg(feature = "palette")]
impl ThemeVisual {
    fn effects(self) -> EffectSnapshot {
        EffectSnapshot {
            background: Some(self.background),
            border_color: Some(self.border),
            text_color: Some(self.text),
            ..EffectSnapshot::default()
        }
    }

    fn from_effects(effects: &EffectSnapshot) -> Self {
        Self {
            background: effects
                .background
                .unwrap_or(ThemeChoice::Blue.visual().background),
            border: effects
                .border_color
                .unwrap_or(ThemeChoice::Blue.visual().border),
            text: effects
                .text_color
                .unwrap_or(ThemeChoice::Blue.visual().text),
        }
    }
}

#[cfg(feature = "palette")]
fn theme_timeline_srgb(from: ThemeVisual, to: ThemeVisual) -> Timeline {
    theme_timeline(
        AnimColor::from(from.background),
        AnimColor::from(to.background),
        AnimColor::from(from.border),
        AnimColor::from(to.border),
        AnimColor::from(from.text),
        AnimColor::from(to.text),
    )
}

#[cfg(feature = "palette")]
fn theme_timeline_oklab(from: ThemeVisual, to: ThemeVisual) -> Timeline {
    theme_timeline(
        AnimColor::from_color::<tag::Oklaba>(from.background),
        AnimColor::from_color::<tag::Oklaba>(to.background),
        AnimColor::from_color::<tag::Oklaba>(from.border),
        AnimColor::from_color::<tag::Oklaba>(to.border),
        AnimColor::from_color::<tag::Oklaba>(from.text),
        AnimColor::from_color::<tag::Oklaba>(to.text),
    )
}

#[cfg(feature = "palette")]
fn theme_timeline(
    from_background: AnimColor,
    to_background: AnimColor,
    from_border: AnimColor,
    to_border: AnimColor,
    from_text: AnimColor,
    to_text: AnimColor,
) -> Timeline {
    let timing = Timing::new(TRANSITION_MS).with_easing(Easing::EaseInOut);

    Timeline::parallel([
        color_track(property::BACKGROUND, from_background, to_background, timing).into(),
        color_track(property::BORDER_COLOR, from_border, to_border, timing).into(),
        color_track(property::TEXT_COLOR, from_text, to_text, timing).into(),
    ])
}

#[cfg(feature = "palette")]
fn color_track(
    spec: PropertySpec<property::Color>,
    from: AnimColor,
    to: AnimColor,
    timing: Timing,
) -> Track {
    Track::new(
        KeyframesBuilder::new()
            .with_timing(timing)
            .at(0.0, (spec, from))
            .at(1.0, (spec, to))
            .finish(),
    )
}

#[cfg(feature = "palette")]
fn theme_card<'a>(label: &'static str, effects: &EffectSnapshot) -> Element<'a, Message> {
    let background = effects
        .background
        .unwrap_or(ThemeChoice::Blue.visual().background);
    let border_color = effects
        .border_color
        .unwrap_or(ThemeChoice::Blue.visual().border);
    let text_color = effects
        .text_color
        .unwrap_or(ThemeChoice::Blue.visual().text);

    container(
        column![
            text(label).size(16).color(text_color),
            text("Theme surface").size(28).color(text_color),
            text("Background, border, and text move through the same theme endpoints.")
                .size(14)
                .width(Length::Fill)
                .color(text_color),
        ]
        .spacing(14),
    )
    .width(Length::Fixed(CARD_WIDTH))
    .height(Length::Fixed(CARD_HEIGHT))
    .padding(24)
    .align_y(Vertical::Center)
    .style(move |_theme: &Theme| container::Style {
        text_color: Some(text_color),
        background: Some(Background::Color(background)),
        border: Border {
            color: border_color,
            width: 2.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

#[cfg(feature = "palette")]
fn app_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.03, 0.05, 0.07))),
        ..container::Style::default()
    }
}

#[cfg(all(test, feature = "palette"))]
mod tests {
    use super::*;
    use aura_anim_iced::timing::Duration;

    fn background_at_midpoint(timeline: &Timeline) -> Color {
        let snapshot = timeline
            .sample_at(Duration::from_millis(TRANSITION_MS / 2.0))
            .expect("timeline samples midpoint");
        let Some(entry) = snapshot.find_property(&property::BACKGROUND.raw()) else {
            panic!("expected background property");
        };

        match entry.value() {
            aura_anim_iced::property::PropertyValue::Color(color) => color.into_iced(),
            _ => panic!("expected color value"),
        }
    }

    #[test]
    fn theme_palette_example_uses_distinct_midpoint_color_spaces() {
        let from = ThemeChoice::Brand.visual();
        let to = ThemeChoice::Dark.visual();

        let srgb = background_at_midpoint(&theme_timeline_srgb(from, to));
        let oklab = background_at_midpoint(&theme_timeline_oklab(from, to));
        let largest_channel_delta = (srgb.r - oklab.r)
            .abs()
            .max((srgb.g - oklab.g).abs())
            .max((srgb.b - oklab.b).abs());

        assert!(largest_channel_delta > 0.03);
    }

    #[test]
    fn theme_sequence_cycles_through_all_example_themes() {
        assert_eq!(ThemeChoice::Dark.next(), ThemeChoice::Light);
        assert_eq!(ThemeChoice::Light.next(), ThemeChoice::Brand);
        assert_eq!(ThemeChoice::Brand.next(), ThemeChoice::Dark);
    }

    #[test]
    fn theme_timeline_samples_midpoint_snapshot() {
        let timeline =
            theme_timeline_oklab(ThemeChoice::Dark.visual(), ThemeChoice::Light.visual());
        let snapshot = timeline
            .sample_at(Duration::from_millis(TRANSITION_MS / 2.0))
            .expect("timeline samples midpoint");

        assert!(
            snapshot
                .find_property(&property::BACKGROUND.raw())
                .is_some()
        );
        assert!(
            snapshot
                .find_property(&property::BORDER_COLOR.raw())
                .is_some()
        );
        assert!(
            snapshot
                .find_property(&property::TEXT_COLOR.raw())
                .is_some()
        );
    }
}
