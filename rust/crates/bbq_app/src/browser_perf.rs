//! Opt-in local browser frame diagnostics. No telemetry is sent over the game connection.
use bevy::prelude::*;
use std::collections::VecDeque;
use std::time::Instant;

pub fn enabled() -> bool {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .is_some_and(|q| q.trim_start_matches('?').split('&').any(|p| p == "perf=1"))
}

#[derive(Resource)]
struct Samples {
    previous: Option<Instant>,
    frame_ms: VecDeque<(Instant, f64, f64)>,
}
impl Default for Samples {
    fn default() -> Self {
        Self {
            previous: None,
            frame_ms: VecDeque::new(),
        }
    }
}

#[derive(Component)]
struct PerfPanel;

pub struct BrowserPerfPlugin;
impl Plugin for BrowserPerfPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Samples>()
            .add_systems(Startup, spawn_panel)
            .add_systems(Update, (sample_frames, paint_panel).chain());
    }
}

fn spawn_panel(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(12.0),
            top: Val::Px(64.0),
            padding: UiRect::all(Val::Px(10.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.055, 0.075, 0.09, 0.88)),
        Text::new("SMOKO PERF · collecting local frame timings…"),
        TextFont {
            font_size: bevy::text::FontSize::Px(13.0),
            ..default()
        },
        TextColor(Color::WHITE),
        PerfPanel,
    ));
}

fn sample_frames(
    screen: Res<crate::menu::Screen>,
    game: Res<crate::game::Game>,
    fixed: Res<Time<Fixed>>,
    mut samples: ResMut<Samples>,
) {
    if *screen != crate::menu::Screen::Playing || game.rules.mode != bbq_core::GameMode::Smoko {
        samples.previous = None;
        samples.frame_ms.clear();
        return;
    }
    let now = Instant::now();
    if let Some(previous) = samples.previous {
        let dt = now.duration_since(previous).as_secs_f64();
        if dt.is_finite() && dt > 0.0 {
            samples
                .frame_ms
                .push_back((now, dt * 1000.0, fixed.overstep().as_secs_f64() * 1000.0));
            // A bounded five-second rolling window: expire old entries, cap for 240 Hz.
            while samples
                .frame_ms
                .front()
                .is_some_and(|(at, _, _)| now.duration_since(*at).as_secs_f32() > 5.0)
                || samples.frame_ms.len() > 1200
            {
                samples.frame_ms.pop_front();
            }
        }
    }
    samples.previous = Some(now);
}

fn paint_panel(
    time: Res<Time>,
    game: Res<crate::game::Game>,
    online: Res<crate::online::Online>,
    screen: Res<crate::menu::Screen>,
    samples: Res<Samples>,
    panel: Single<(&mut Text, &mut Visibility), With<PerfPanel>>,
    mut refresh: Local<f32>,
) {
    if *refresh > 0.0 {
        *refresh -= time.delta_secs();
        return;
    }
    *refresh = 0.5;
    let (mut text, mut visibility) = panel.into_inner();
    if *screen != crate::menu::Screen::Playing || game.rules.mode != bbq_core::GameMode::Smoko {
        **text = String::new();
        *visibility = Visibility::Hidden;
        return;
    }
    *visibility = Visibility::Inherited;
    if samples.frame_ms.len() < 2 {
        return;
    }
    let mut sorted: Vec<f64> = samples.frame_ms.iter().map(|s| s.1).collect();
    sorted.sort_by(f64::total_cmp);
    let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
    let p95 = sorted[((sorted.len() as f64 * 0.95).ceil() as usize).saturating_sub(1)].min(9999.0);
    let long = sorted.iter().filter(|&&ms| ms >= 33.3).count();
    let fixed_lag = samples.frame_ms.iter().filter(|s| s.2 >= 16.7).count();
    let peer = match online
        .session
        .as_ref()
        .filter(|s| s.joined())
        .map(|s| s.role)
    {
        Some(crate::online::Role::Host) => "HOST",
        Some(crate::online::Role::Guest) => "GUEST",
        None => "SOLO",
    };
    **text = format!(
        "SMOKO PERF · {peer} · local only\n~{:.0} FPS  avg {:.1} ms  p95 {:.1} ms\n≥33 ms frames: {long}  ·  fixed backlog ≥16.7 ms: {fixed_lag}  ·  rolling sample",
        1000.0 / mean.max(0.1),
        mean,
        p95
    );
}
