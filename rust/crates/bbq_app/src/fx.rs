//! Particles (step 2c, with `bevy_hanabi`): sparks and a puff where a throw lands, a slap burst,
//! beer foam when a VP can goes off, a splash when someone hits the pool, dust when someone falls
//! or lands hard, and a steady wisp of smoke from the BBQ.
//!
//! The game logic only says "this happened here" by pushing an `FxEvent` onto `Game::fx`; this
//! file turns those into little one-shot effects that remove themselves.

use bbq_core::vec::V3;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::game::Game;
use crate::yard_scene::YardRes;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FxKind {
    /// A thrown item hits someone: stars and a puff.
    Hit,
    /// A VP can bursts: beer foam.
    Smash,
    /// A slap with something in hand.
    Whack,
    /// Into the pool.
    Splash,
    /// Someone lands hard or falls over.
    Dust,
}

#[derive(Clone, Copy, Debug)]
pub struct FxEvent {
    pub kind: FxKind,
    pub at: V3,
}

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .add_systems(Startup, make_effects)
            .add_systems(Update, (spawn_events, expire_effects, bbq_smoke));
    }
}

#[derive(Resource)]
struct Effects {
    stars: Handle<EffectAsset>,
    puff: Handle<EffectAsset>,
    foam: Handle<EffectAsset>,
    splash: Handle<EffectAsset>,
    dust: Handle<EffectAsset>,
    smoke: Handle<EffectAsset>,
    dot: Handle<Image>,
}

/// A one-shot effect that removes itself after this many seconds.
#[derive(Component)]
struct FxLife(f32);

#[derive(Component)]
struct BbqSmoke;

/// What a burst looks like.
struct Burst {
    name: &'static str,
    count: f32,
    radius: f32,
    /// Outwards speed, from and to.
    speed: (f32, f32),
    /// Straight-up push added to the outward speed.
    up: f32,
    life: (f32, f32),
    /// Gravity (negative pulls down) and air drag.
    gravity: f32,
    drag: f32,
    /// Particle size at birth and at death, in metres.
    size: (f32, f32),
    /// Colour at birth, in the middle and gone (rgba). Values above 1 make the bloom glow.
    colours: [Vec4; 3],
}

fn burst_asset(b: &Burst) -> EffectAsset {
    let w = ExprWriter::new();
    let init_pos = SetPositionSphereModifier {
        center: w.lit(Vec3::ZERO).expr(),
        radius: w.lit(b.radius).expr(),
        dimension: ShapeDimension::Volume,
    };
    let dir = (w.rand(VectorType::VEC3F) * w.lit(2.0) - w.lit(1.0)).normalized();
    let speed = w.lit(b.speed.0).uniform(w.lit(b.speed.1));
    let vel = (dir * speed + w.lit(Vec3::Y * b.up)).expr();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel);
    let init_age = SetAttributeModifier::new(Attribute::AGE, w.lit(0.0).expr());
    let init_life = SetAttributeModifier::new(
        Attribute::LIFETIME,
        w.lit(b.life.0).uniform(w.lit(b.life.1)).expr(),
    );
    let accel = AccelModifier::new(w.lit(Vec3::Y * b.gravity).expr());
    let drag = LinearDragModifier::new(w.lit(b.drag).expr());
    let slot = w.lit(0u32).expr();
    let mut module = w.finish();
    module.add_texture_slot("dot");
    let mut grad = bevy_hanabi::Gradient::new();
    grad.add_key(0.0, b.colours[0]);
    grad.add_key(0.4, b.colours[1]);
    grad.add_key(1.0, b.colours[2]);
    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(b.size.0));
    size.add_key(1.0, Vec3::splat(b.size.1));
    EffectAsset::new(256, SpawnerSettings::once(b.count.into()), module)
        .with_name(b.name)
        .with_alpha_mode(bevy_hanabi::AlphaMode::Blend)
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_life)
        .update(accel)
        .update(drag)
        .render(ParticleTextureModifier {
            texture_slot: slot,
            sample_mapping: ImageSampleMapping::Modulate,
        })
        .render(ColorOverLifetimeModifier::new(grad))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
}

/// The BBQ's smoke: a steady trickle of slow grey puffs that rise, spread and fade.
fn smoke_asset() -> EffectAsset {
    let w = ExprWriter::new();
    let init_pos = SetPositionCircleModifier {
        center: w.lit(Vec3::ZERO).expr(),
        axis: w.lit(Vec3::Y).expr(),
        radius: w.lit(0.25).expr(),
        dimension: ShapeDimension::Volume,
    };
    let drift = w.rand(VectorType::VEC3F) * w.lit(0.25) - w.lit(0.125);
    let vel = (drift + w.lit(Vec3::new(0.12, 0.7, 0.05))).expr();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel);
    let init_age = SetAttributeModifier::new(Attribute::AGE, w.lit(0.0).expr());
    let init_life = SetAttributeModifier::new(Attribute::LIFETIME, w.lit(3.0).uniform(w.lit(4.5)).expr());
    let drag = LinearDragModifier::new(w.lit(0.6).expr());
    let slot = w.lit(0u32).expr();
    let mut module = w.finish();
    module.add_texture_slot("dot");
    let mut grad = bevy_hanabi::Gradient::new();
    grad.add_key(0.0, Vec4::new(0.5, 0.5, 0.52, 0.0));
    grad.add_key(0.1, Vec4::new(0.5, 0.5, 0.52, 0.85));
    grad.add_key(1.0, Vec4::new(0.72, 0.72, 0.74, 0.0));
    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.4));
    size.add_key(1.0, Vec3::splat(1.8));
    EffectAsset::new(128, SpawnerSettings::rate(9.0.into()), module)
        .with_name("bbq smoke")
        .with_alpha_mode(bevy_hanabi::AlphaMode::Blend)
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_life)
        .update(drag)
        .render(ParticleTextureModifier {
            texture_slot: slot,
            sample_mapping: ImageSampleMapping::Modulate,
        })
        .render(ColorOverLifetimeModifier::new(grad))
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        })
}

fn make_effects(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>, assets: Res<AssetServer>) {
    let c = |r: f32, g: f32, b: f32, a: f32| Vec4::new(r, g, b, a);
    let stars = effects.add(burst_asset(&Burst {
        name: "hit stars",
        count: 26.0,
        radius: 0.1,
        speed: (2.2, 4.5),
        up: 1.2,
        life: (0.35, 0.6),
        gravity: -7.0,
        drag: 1.5,
        size: (0.2, 0.04),
        colours: [c(2.4, 2.0, 0.6, 1.0), c(1.6, 1.0, 0.3, 0.9), c(1.0, 0.5, 0.1, 0.0)],
    }));
    let puff = effects.add(burst_asset(&Burst {
        name: "hit puff",
        count: 9.0,
        radius: 0.15,
        speed: (0.6, 1.4),
        up: 0.4,
        life: (0.5, 0.8),
        gravity: 0.3,
        drag: 2.0,
        size: (0.35, 0.9),
        colours: [c(0.95, 0.93, 0.88, 0.7), c(0.9, 0.88, 0.82, 0.3), c(0.9, 0.88, 0.82, 0.0)],
    }));
    let foam = effects.add(burst_asset(&Burst {
        name: "beer foam",
        count: 44.0,
        radius: 0.12,
        speed: (1.5, 3.8),
        up: 1.5,
        life: (0.5, 0.9),
        gravity: -8.0,
        drag: 1.0,
        size: (0.15, 0.05),
        colours: [c(1.0, 0.95, 0.7, 1.0), c(0.95, 0.8, 0.3, 0.9), c(0.9, 0.7, 0.2, 0.0)],
    }));
    let splash = effects.add(burst_asset(&Burst {
        name: "splash",
        count: 70.0,
        radius: 0.35,
        speed: (1.0, 3.2),
        up: 3.4,
        life: (0.7, 1.1),
        gravity: -10.0,
        drag: 0.6,
        size: (0.18, 0.06),
        colours: [c(0.85, 0.97, 1.0, 0.95), c(0.5, 0.82, 0.98, 0.8), c(0.4, 0.75, 0.95, 0.0)],
    }));
    let dust = effects.add(burst_asset(&Burst {
        name: "dust",
        count: 12.0,
        radius: 0.3,
        speed: (0.8, 1.9),
        up: 0.3,
        life: (0.6, 1.0),
        gravity: 0.2,
        drag: 2.5,
        size: (0.4, 1.1),
        colours: [c(0.78, 0.68, 0.5, 0.6), c(0.74, 0.65, 0.48, 0.3), c(0.74, 0.65, 0.48, 0.0)],
    }));
    let smoke = effects.add(smoke_asset());
    commands.insert_resource(Effects {
        stars,
        puff,
        foam,
        splash,
        dust,
        smoke,
        dot: assets.load("textures/particle.png"),
    });
}

fn spawn_events(
    mut commands: Commands,
    mut game: ResMut<Game>,
    fx: Option<Res<Effects>>,
    time: Res<Time>,
    mut tested: Local<bool>,
) {
    let Some(fx) = fx else { return };
    // `--fxtest` fires one of every effect in a row, for looking at them
    if !*tested && time.elapsed_secs() > 0.8 && std::env::args().any(|a| a == "--fxtest") {
        *tested = true;
        for (i, k) in [FxKind::Hit, FxKind::Smash, FxKind::Whack, FxKind::Splash, FxKind::Dust].into_iter().enumerate() {
            game.fx(k, V3::new(-2.4 + i as f32 * 1.2, if k == FxKind::Dust { 0.15 } else { 1.2 }, 12.0));
        }
    }
    let events: Vec<FxEvent> = std::mem::take(&mut game.fx);
    for e in events.into_iter().take(12) {
        let (list, life): (Vec<&Handle<EffectAsset>>, f32) = match e.kind {
            FxKind::Hit => (vec![&fx.stars, &fx.puff], 1.2),
            FxKind::Whack => (vec![&fx.stars], 1.0),
            FxKind::Smash => (vec![&fx.foam, &fx.puff], 1.4),
            FxKind::Splash => (vec![&fx.splash], 1.8),
            FxKind::Dust => (vec![&fx.dust], 1.6),
        };
        for h in list {
            commands.spawn((
                ParticleEffect::new(h.clone()),
                EffectMaterial { images: vec![fx.dot.clone()] },
                Transform::from_xyz(e.at.x, e.at.y, e.at.z),
                FxLife(life),
            ));
        }
    }
}

fn expire_effects(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut FxLife)>) {
    for (e, mut l) in &mut q {
        l.0 -= time.delta_secs();
        if l.0 <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

/// The smoke rises from the grill while the BBQ is in the yard.
fn bbq_smoke(
    mut commands: Commands,
    yard: Res<YardRes>,
    fx: Option<Res<Effects>>,
    smoke: Query<Entity, With<BbqSmoke>>,
) {
    let Some(fx) = fx else { return };
    let want = yard.0.features.bbq;
    if want && smoke.is_empty() {
        commands.spawn((
            ParticleEffect::new(fx.smoke.clone()),
            EffectMaterial { images: vec![fx.dot.clone()] },
            Transform::from_xyz(-6.0, 1.3, -18.3),
            BbqSmoke,
        ));
    } else if !want {
        for e in &smoke {
            commands.entity(e).despawn();
        }
    }
}
