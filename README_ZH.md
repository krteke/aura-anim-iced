# aura-anim

为 Rust 桌面界面提供的强类型动画原语及 Iced 集成。

应用程序存储轻量级的 `Motion<T>` 句柄，而 `MotionRuntime` 则拥有并推进实际的动画源。动画值保持为普通的 Rust 结构体，通过 `#[derive(Animatable)]` 生成逐字段的插值。

```text
Application
├── 显式 UI 状态
├── Motion<T> 句柄
└── 事件驱动的 transition_to / play 调用

MotionRuntime
├── 拥有类型擦除的动画槽位
├── 仅对活跃槽位进行 Tick 驱动
├── 针对单个动画及批量的 暂停 / 恢复 / 跳转 / 取消 / 完成 操作
├── 基于代际检查 (Generation-checked) 的句柄重用
└── 完成后的压缩及可选的自动移除

Animation<T>
├── Tween<T>
├── Spring<T>
├── Keyframes<T>
├── Sequence<T>
├── Parallel<T>
└── Hold<T>
```

## 工作区 Crate

- `aura-anim-core`: 运行时、句柄、插值、动画源及时间线组合。
- `aura-anim-iced`: Iced 值集成及帧订阅。
- `aura-anim`: 一个暴露 `core` 和 `iced` 命名空间的小型门面 (facade)。
- `aura-anim-macros`: `Animatable` 派生宏实现。

## 安装

对于 Iced 应用程序：

```toml
[dependencies]
aura-anim = "0.3.0"
iced = "0.14"
```

在不需要 Iced 集成时，可以直接使用 `aura-anim-core`。

## 强类型动画 (Typed Motion)

```rust
use aura_anim::core::{
    macros::Animatable,
    runtime::{AnimationCommand, MotionRuntime},
    target::tween_to,
    timing::Timing,
};

#[derive(Clone, Debug, Animatable)]
struct ButtonMotion {
    opacity: f32,
    scale: f32,
}

let mut runtime = MotionRuntime::new();
let button = runtime.motion_with(
    ButtonMotion {
        opacity: 0.5,
        scale: 0.95,
    },
    Timing::ease_out(160.0),
);

button.transition_to(
    ButtonMotion {
        opacity: 1.0,
        scale: 1.0,
    },
    &mut runtime,
)
.unwrap();

runtime.tick(std::time::Duration::from_millis(80));
let visual = button.value(&runtime).unwrap();
```

`transition_to` 从当前采样值开始重新定向，因此被中断的悬停、按压、菜单和路由动画不会跳回过时的起点。动画的访问和更改返回 `Result<_, MotionError>`，以便区分已移除、过期、越界及类型不匹配的句柄。

`Timing::linear`、`Timing::ease_in`、`Timing::ease_out` 和 `Timing::ease_in_out` 涵盖了常见的时长/缓动组合，同时保持为普通的 `Timing` 值，可以扩展延迟 (delay)、迭代 (iterations) 或方向 (direction)。

在替换当前动画时，可以使用延迟目标工厂：

```rust
button
    .play(
        tween_to(
            ButtonMotion {
                opacity: 0.0,
                scale: 0.9,
            },
            Timing::ease_in(120.0),
        ),
        &mut runtime,
    )
    .expect("motion belongs to this runtime");
```

`tween_to` 和 `spring_to` 在播放开始时采样动画的当前值，因此调用者无需手动读取或克隆它。

运行时范围的命令可用于应用程序生命周期和无障碍策略：

```rust
runtime.command_all(AnimationCommand::Pause);
runtime.command_all(AnimationCommand::Resume);
runtime.command_all(AnimationCommand::Finish);
```

`command_all` 应用于每个存储的动画，包括已暂停和空闲的动画。完成、取消和 `DropWhenSettled` 移除事件使用与通过单个 `Motion<T>` 发送的命令相同的语义。

## 独立的字段动画

`#[derive(Animatable)]` 还会生成强类型的字段描述符。一个结构体可以保持为一个 `Motion<T>`，而每个选定的字段可以使用不同的动画：

```rust
use aura_anim::core::{
    field::fields,
    macros::{field, Animatable},
    runtime::MotionRuntime,
    spring::SpringConfig,
    target::{spring_to, tween_to},
    timing::Timing,
};

#[derive(Clone, Debug, Animatable)]
struct Position {
    x: f32,
    y: f32,
}

let mut runtime = MotionRuntime::new();
let position = runtime.motion(Position { x: 0.0, y: 0.0 });

position
    .play(
        fields()
            .animate(
                field!(Position::x),
                tween_to(100.0, Timing::ease_in(100.0)),
            )
            .animate(
                PositionFields::y,
                spring_to(200.0, SpringConfig::snappy()),
            ),
        &mut runtime,
    )
    .expect("motion belongs to this runtime");
```

目标工厂在调用 `play` 时接收字段的当前采样值。自定义 `|from| ...` 工厂仍然受支持。因此，被中断的字段动画会从可见值继续，而未包含在计划中的字段将保留其当前值。

对于命名的结构体，派生宏会生成 `PositionFields::x`、`PositionFields::y` 以及等效的 `field!(Position::x)` 描述符。元组结构体描述符使用 `_0`、`_1` 等，而 `field!(Offset::0)` 直接使用元组索引。生成的描述符类型在必要时可以重命名：

```rust
#[derive(Clone, Animatable)]
#[animatable(fields = PositionAnimationFields)]
struct Position {
    x: f32,
    y: f32,
}
```

## 动画事件

当播放完成、取消、被中断或离开运行时存储时，`MotionRuntime` 会排队结构化的生命周期事件。事件在状态转换时触发一次，并保持在队列中，直到应用程序获取 (take) 或清除它们：

```rust
use aura_anim::iced::Subscribe;

runtime.frame(now);

for event in runtime.take_events() {
    if event.is_completed_for(motion) {
        // 运行一次性完成逻辑。
    }
}
```

对于简单的清理工作，匹配 `Motion<T>` 就足够了。多阶段流程应追踪精确的播放 ID，以便旧的排队事件不会完成同一动画句柄上较新的动画：

```rust
let exit = motion
    .play_tracked(
        Tween::between(current, hidden, Timing::new(150.0)),
        &mut runtime,
    )
    .expect("motion belongs to this runtime");

// 在稍后的帧更新中：
for event in runtime.take_events() {
    if event.is_completed_for(exit) {
        let _enter = motion
            .play_tracked(
                Tween::between(hidden, visible, Timing::new(220.0)),
                &mut runtime,
            )
            .expect("motion belongs to this runtime");
    }
}
```

`play_tracked` 和 `transition_to_tracked` 返回一个 `PlaybackId`。当不需要播放标识时，现有的 `play` 和 `transition_to` 调用保持不变。

事件类型包括：

- `Completed`
- `Canceled`
- `Interrupted(Replaced | Retargeted | Removed)`
- `Removed(Explicit | Settled)`

`DropWhenSettled` 动画在其移除事件之前发出终端事件，因此在句柄失效后仍可观察到完成。

`Presence::handle_event` 使用当前的退出播放 ID，以避免过时的退出事件卸载已经重新显示的内容：

```rust
for event in runtime.take_events() {
    menu.handle_event(&event);
    toast.handle_event(&event);
}
```

`Presence::sync` 仍可用于基于轮询的集成。

对于布尔类型的应用程序状态，`set_visible` 在请求的目标未改变时避免重启动画：

```rust
let _started = menu
    .set_visible(is_open, &mut runtime)
    .expect("presence motion belongs to this runtime");
let _toggled = menu
    .toggle(&mut runtime)
    .expect("presence motion belongs to this runtime");
```

这两个方法都返回是否开始了新的过渡。当需要有意重新播放过渡时，显式的 `show`/`hide` 和自定义的 `show_with`/`hide_with` 调用仍然可用。

## Iced 集成

在应用程序状态中存储运行时和强类型句柄：

```rust
use std::time::Instant;

use aura_anim::{
    core::{
        macros::Animatable,
        runtime::{Motion, MotionRuntime},
        timing::Timing,
    },
    iced::{Subscribe, TickPolicy},
};
use iced::{Subscription, Vector};

#[derive(Clone, Debug, Animatable)]
struct PanelMotion {
    opacity: f32,
    offset: Vector,
}

struct App {
    runtime: MotionRuntime,
    panel: Motion<PanelMotion>,
}

#[derive(Clone, Debug)]
enum Message {
    Frame(Instant),
    Open,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => self.runtime.frame(now),
            Message::Open => {
                if let Err(error) = self.panel.transition_to(
                    PanelMotion {
                        opacity: 1.0,
                        offset: Vector::ZERO,
                    },
                    &mut self.runtime,
                ) {
                    eprintln!("panel transition failed: {error}");
                }
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        self.runtime
            .subscription_with_policy(TickPolicy::fps(60))
            .map(Message::Frame)
    }
}
```

当没有活跃动画时，订阅返回 `Subscription::none()` 并且不会继续唤醒应用程序。

`TickPolicy` 支持：

```rust
TickPolicy::Frames
TickPolicy::fps(60)
TickPolicy::interval(std::time::Duration::from_millis(32))
```

对于与特定窗口关联的运行时，请使用窗口特定的订阅。`WindowFrame` 保留了目标窗口和时间戳：

```rust
use aura_anim::{
    core::runtime::MotionRuntime,
    iced::{Subscribe, WindowFrame},
};
use iced::Subscription;

fn window_frames(
    runtime: &MotionRuntime,
    window: iced::window::Id,
) -> Subscription<WindowFrame> {
    runtime.subscription_for(window)
}

fn advance(runtime: &mut MotionRuntime, frame: WindowFrame) {
    runtime.frame(frame.at);
}
```

每个运行时使用一个 tick 源。如果一个运行时被多个窗口有意共享，请选择或组合它们的帧消息，以确保运行时不会在同一个更新周期内推进多次。

## 动画绑定 (Motion Binding)

`MotionBinding<S, T>` 将可复用的业务状态映射到视觉目标和过渡工厂。绑定是不可变的配置；每个按钮、菜单项或路由拥有一个小型的 `MotionBindingState<S>`，用于记录其上次成功应用的状态。

```rust
use aura_anim::core::{
    binding::MotionBinding,
    runtime::MotionRuntime,
    spring::SpringConfig,
    timing::Timing,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ButtonState {
    Idle,
    Hovered,
    Pressed,
}

let button_binding = MotionBinding::new(ButtonState::Idle, idle)
    .when(ButtonState::Hovered, hovered)
    .when(ButtonState::Pressed, pressed)
    .transition(ButtonState::Idle, ButtonState::Hovered, |ctx| {
        ctx.tween(Timing::new(120.0))
    })
    .transition(ButtonState::Hovered, ButtonState::Pressed, |ctx| {
        ctx.spring(SpringConfig::snappy())
    })
    .fallback(|ctx| ctx.tween(Timing::new(100.0)));

let motion = runtime.motion(idle);
let mut binding_state = button_binding.state();

let playback = button_binding
    .set_state_tracked(
        &mut binding_state,
        ButtonState::Hovered,
        motion,
        &mut runtime,
    )
    .expect("binding and motion are compatible")
    .expect("state changed");

// 在稍后的更新中，运行时 tick 之后：
for event in runtime.take_events() {
    if event.is_completed_for(playback) {
        // Hovered 过渡完成。
    }
}
```

在每次状态改变时，绑定会解析目标、采样动画的当前值、选择精确的过渡或回退工厂、构建动画、调用 `motion.play(...)`，并且仅在播放成功后记录新的业务状态。工厂可以返回具体的 Tween、Spring、Keyframes、Timeline 或任何自定义的 `Animation<T>`；绑定在内部处理类型擦除。

`set_state` 返回是否开始了过渡。
`set_state_tracked` 对于未改变的状态返回 `None`，对于新开始的过渡返回精确的 `PlaybackId`，从而允许在不轮询动画的情况下匹配完成和中断事件。

一个绑定配置可以被克隆或共享，并与独立的 `MotionBindingState` 值一起复用。

## Iced 可动画类型

在启用核心 `iced` 集成后，这些类型可以作为 `Animatable` 结构体中的字段：

- `iced::Vector<T>`
- `iced::Point<T>`
- `iced::Size<T>`
- `iced::Rectangle<T>`
- `iced::Padding`
- `iced::border::Radius`

激活 `rgba` 或 `oklaba` 颜色特性后，还支持：

- `iced::Color`
- `iced::Shadow`
- `iced::Border`

## 颜色插值

默认启用 RGBA 分量插值：

```toml
aura-anim = "0.3.0"
```

对于具有独立插值 Alpha 通道的 Oklab RGB 插值：

```toml
aura-anim = {
    version = "0.3.0",
    default-features = false,
    features = ["oklaba"]
}
```

`rgba` 和 `oklaba` 是互斥的。Oklaba 转换流程如下：

```text
Iced sRGB
→ palette sRGB
→ Oklab 插值
→ display sRGB
```

## 追踪 (Tracing)

启用可选的 `tracing` 特性以发出运行时诊断信息，无需在库内部安装或配置订阅者：

```toml
aura-anim = {
    version = "0.3.0",
    features = ["tracing"]
}
```

运行时、绑定和呈现组件会报告动画的分配与重用、播放命令、生命周期变化、无效句柄、绑定过渡选择以及呈现挂载。每 tick 驱动的诊断使用 `TRACE` 级别；生命周期和错误诊断使用 `DEBUG`。应用程序仍负责安装兼容的 `tracing` 订阅者。

## 动画源

### Tween

```rust
motion.play(
    Tween::between(current, target, Timing::new(180.0)).rate(2.0),
    &mut runtime,
);
```

Timing 支持延迟、缓动、有限或无限次迭代以及播放方向。`Animation::rate` 直接缩放存储的时长：`2.0` 使时长减半，`0.5` 使时长翻倍。它会递归更新现有的时间线子节点，而 Spring 会忽略速率，因为它的运动是基于物理的。

### Keyframes

```rust
motion.play(
    Keyframes::new(start)
        .push_eased(180.0, overshoot, Easing::EaseOut)
        .push_eased(280.0, settled, Easing::EaseInOut),
    &mut runtime,
);
```

### Spring

```rust
motion.play(
    Spring::new(current, target, SpringConfig::default()),
    &mut runtime,
);
```

弹簧插值可能会产生过冲 (overshoot)，并且可以在活跃时重新定向。

对于字段需要不同物理响应的值，请创建独立的弹簧通道并显式组合它们的输出：

```rust
#[derive(Clone, Debug, Animatable)]
struct PanelMotion {
    offset: f32,
    opacity: f32,
}

let movement = SpringConfig::new(180.0, 20.0);
let fade = SpringConfig::new(420.0, 32.0)
    .with_mass(1.2)
    .with_epsilon(0.001);

let spring = Spring::with_channels(
    PanelMotion {
        offset: 24.0,
        opacity: 0.0,
    },
    PanelMotion {
        offset: 0.0,
        opacity: 1.0,
    },
    [movement, fade],
    |outputs| PanelMotion {
        offset: outputs[0].offset,
        opacity: outputs[1].opacity,
    },
);
```

每个通道拥有自己的位置、速度和 `SpringConfig`。弹簧推进使用解析阻尼振荡器解 (analytic damped-oscillator solution)，因此长帧间隔会被完全消耗而不会被截断。

## 时间线组合

`Sequence`、`Parallel` 和 `Hold` 都实现了 `Animation<T>`，因此组合是递归的：

```text
Sequence(
    Parallel(
        Sequence(Hold, Tween),
        Sequence(Tween, Tween),
    ),
    Tween,
)
```

并行分支产生完整的 `T` 值。组合器显式选择每个分支拥有的字段：

```rust
let parallel = Parallel::new(start.clone(), |outputs: &[Position]| Position {
    x: outputs[0].x,
    y: outputs[1].y,
})
.with(x_sequence)
.with(y_sequence);
```

Sequence 会将未使用的帧时间传播到后续子节点。Parallel 在其最长分支完成时完成。

具体动画可以通过 `AnimationExt` 直接开始一个序列：

```rust
let timeline = Tween::between(hidden, visible, Timing::ease_out(180.0))
    .delay(80.0)
    .then(Hold::new(visible.clone(), 240.0))
    .then(Tween::between(visible, hidden, Timing::ease_in(120.0)));
```

`delay` 在动画之前插入一个 `Hold`。这两个组合器都返回现有的 `Sequence<T>` 类型，因此生命周期、跳转、速率变化和溢出传播保持与手动构建序列相同的行为。

## 生命周期

普通的动画句柄保留其最终值：

```rust
let motion = runtime.motion(initial);
```

已完成的源被压缩为最终值，释放关键帧和时间线树，同时保持句柄有效。

瞬时动画可以自动移除其槽位：

```rust
let transient = runtime.play_once(animation);
```

槽位通过代际计数器进行重用，防止陈旧句柄访问新分配的动画。

## 示例

运行 Iced 展示案例：

```sh
cargo run -p aura-anim-iced --example showcase
```

运行专项视觉示例：

```sh
cargo run -p aura-anim-iced --example tween
cargo run -p aura-anim-iced --example keyframes
cargo run -p aura-anim-iced --example timeline
cargo run -p aura-anim-iced --example spring
```

运行交互式 UI 示例：

```sh
cargo run -p aura-anim-iced --example button
cargo run -p aura-anim-iced --example menu
cargo run -p aura-anim-iced --example notification
cargo run -p aura-anim-iced --example route_transition
```

使用感知颜色插值运行展示案例：

```sh
cargo run -p aura-anim-iced \
    --no-default-features \
    --features oklaba \
    --example showcase
```
