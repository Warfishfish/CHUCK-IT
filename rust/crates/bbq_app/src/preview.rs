//! The character preview beside the menu: your blob, turning slowly, drawn by a second camera
//! into a picture that the menu card shows. Picking another blob swaps the model at once.
//!
//! Everything for the preview lives on render layer 1, so the yard behind the menu and the
//! preview never see each other (own light, own camera).

use bbq_core::character::Character;
use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::ClearColorConfig;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::world_serialization::WorldInstanceReady;

use crate::menu::{Screen, Settings};

pub const PREVIEW_W: u32 = 360;
pub const PREVIEW_H: u32 = 400;
const LAYER: usize = 1;

#[derive(Resource, Clone)]
pub struct PreviewImage(pub Handle<Image>);

#[derive(Component)]
struct PreviewCamera;
#[derive(Component)]
struct PreviewModel;
/// The preview's body and singlet: their belly follows the slider.
#[derive(Component)]
struct PreviewBelly;

pub struct PreviewPlugin;

impl Plugin for PreviewPlugin {
    fn build(&self, app: &mut App) {
        let img = Image::new_target_texture(PREVIEW_W, PREVIEW_H, TextureFormat::Rgba8Unorm, None);
        let handle = app.world_mut().resource_mut::<Assets<Image>>().add(img);
        app.insert_resource(PreviewImage(handle))
            .init_resource::<Spin>()
            .add_systems(Startup, spawn_preview_rig)
            .add_systems(Update, (swap_preview_model, drag_preview, spin_preview, preview_on_only_in_menu).chain());
    }
}

fn spawn_preview_rig(
    mut commands: Commands,
    image: Res<PreviewImage>,
    look: Res<crate::lighting::LookMode>,
) {
    let layers = RenderLayers::layer(LAYER);
    let (ambient, _, _) = crate::lighting::hemisphere(*look);
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(crate::models::hex(0xcfe9f7)),
            ..default()
        },
        RenderTarget::Image(image.0.clone().into()),
        Projection::from(PerspectiveProjection {
            fov: 34f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 1.0, 4.4).looking_at(Vec3::new(0.0, 0.82, 0.0), Vec3::Y),
        crate::lighting::camera_style(*look),
        ambient,
        layers.clone(),
        PreviewCamera,
    ));
    let (mut sun, _) = crate::lighting::sun(*look);
    sun.shadow_maps_enabled = false;
    commands.spawn((
        sun,
        Transform::from_xyz(2.5, 4.0, 3.5).looking_at(Vec3::new(0.0, 0.8, 0.0), Vec3::Y),
        layers,
    ));
}

fn swap_preview_model(
    mut commands: Commands,
    assets: Res<AssetServer>,
    settings: Res<Settings>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    old: Query<Entity, With<PreviewModel>>,
    mut shown: Local<Option<Character>>,
) {
    if *shown == Some(settings.character) {
        return;
    }
    *shown = Some(settings.character);
    for e in &old {
        commands.entity(e).despawn();
    }
    let base = crate::models::hex(0xffd23f);
    let (body, head, foot) = (
        mats.add(crate::style::body_material(base, &assets)),
        mats.add(crate::style::body_material(crate::characters::lighter(base), &assets)),
        mats.add(crate::style::body_material(crate::characters::darker(base), &assets)),
    );
    commands
        .spawn((
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(crate::style::model_path(settings.character)))),
            Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(0.5)),
            RenderLayers::layer(LAYER),
            PreviewModel,
        ))
        .observe(
            move |ready: On<WorldInstanceReady>,
                  children: Query<&Children>,
                  names: Query<&Name>,
                  has_mat: Query<(), With<MeshMaterial3d<StandardMaterial>>>,
                  mut commands: Commands| {
                for node in children.iter_descendants(ready.entity) {
                    // everything of the model is on the preview's own layer
                    commands.entity(node).insert(RenderLayers::layer(LAYER));
                    let Ok(name) = names.get(node) else { continue };
                    if matches!(name.as_str(), "Torso" | "Singlet") {
                        commands.entity(node).insert(PreviewBelly);
                    }
                    if name.as_str() == "Head" {
                        commands.entity(node).insert(crate::face::HeadNode { owner: crate::face::FaceOwner::Preview, layer: Some(LAYER) });
                    }
                    let m = match name.as_str() {
                        "Torso" | "Belly" => Some(body.clone()),
                        "Head" | "HandR" | "HandL" => Some(head.clone()),
                        "FootL" | "FootR" => Some(foot.clone()),
                        _ => None,
                    };
                    if let Some(m) = m {
                        let parts = std::iter::once(node).chain(children.iter_descendants(node));
                        for e in parts {
                            if has_mat.contains(e) {
                                commands.entity(e).insert(MeshMaterial3d(m.clone()));
                            }
                        }
                    }
                    // your look's colours
                    let part = match name.as_str() {
                        "Torso" | "Belly" => Some(crate::face::TintPart::Body),
                        "Head" | "HandR" | "HandL" => Some(crate::face::TintPart::Head),
                        "FootL" | "FootR" => Some(crate::face::TintPart::Foot),
                        "Singlet" => Some(crate::face::TintPart::Singlet),
                        "ThongL" | "ThongR" => Some(crate::face::TintPart::Thong),
                        _ => None,
                    };
                    if let Some(part) = part {
                        for e in std::iter::once(node).chain(children.iter_descendants(node)) {
                            if has_mat.contains(e) {
                                commands.entity(e).insert(crate::face::LookTint { owner: crate::face::FaceOwner::Preview, part });
                            }
                        }
                    }
                }
            },
        );
}

/// The picture of the preview in the menu: hold the left button on it and drag to spin the blob.
#[derive(Component)]
pub struct PreviewDrag;

/// How the preview is turning (Marcus, 6 Oct 2026): it turns slowly by itself; hold the left
/// button on it and drag to spin it by hand; let go and a flick carries on, slowing back to the
/// slow turn.
#[derive(Resource, Clone, Copy, Debug)]
pub struct Spin {
    pub angle: f32,
    /// Radians per second right now.
    pub vel: f32,
    pub dragging: bool,
}

/// The slow turn on its own, radians per second.
pub const AUTO_SPIN: f32 = 0.7;
/// Radians per pixel of mouse movement while dragging.
pub const DRAG_SENS: f32 = 0.012;
/// The fastest a flick can spin it.
pub const MAX_SPIN: f32 = 14.0;

impl Default for Spin {
    fn default() -> Self {
        Spin { angle: 0.6, vel: AUTO_SPIN, dragging: false }
    }
}

impl Spin {
    /// One frame. `drag_px` is how far the mouse moved sideways while held (None when the
    /// button is up).
    pub fn tick(&mut self, dt: f32, drag_px: Option<f32>) {
        let dt = dt.clamp(0.0, 0.1);
        match drag_px {
            Some(px) => {
                let turn = px * DRAG_SENS;
                self.angle += turn;
                // the speed of the hand, smoothed a little so one jerky frame is not a flick
                if dt > 0.0 {
                    let v = (turn / dt).clamp(-MAX_SPIN, MAX_SPIN);
                    self.vel += (v - self.vel) * (1.0 - (-18.0 * dt).exp());
                }
                self.dragging = true;
            }
            None => {
                // let go: whatever speed it had eases back to the slow turn
                self.dragging = false;
                self.vel += (AUTO_SPIN - self.vel) * (1.0 - (-1.6 * dt).exp());
                self.angle += self.vel * dt;
            }
        }
        self.angle = self.angle.rem_euclid(std::f32::consts::TAU);
    }
}

/// Holding the left button on the preview picture and moving the mouse spins the blob.
fn drag_preview(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<bevy::input::mouse::AccumulatedMouseMotion>,
    pic: Query<&Interaction, With<PreviewDrag>>,
    screen: Res<Screen>,
    mut spin: ResMut<Spin>,
) {
    if *screen != Screen::Menu {
        return;
    }
    // `--preview-angle A` (screenshots): hold the preview still, A radians round
    if let Some(a) = std::env::args().position(|a| a == "--preview-angle").and_then(|i| std::env::args().nth(i + 1)).and_then(|v| v.parse::<f32>().ok()) {
        spin.angle = a;
        spin.vel = 0.0;
        return;
    }
    // a drag starts on the picture and keeps going until the button comes up, wherever the
    // mouse wanders meanwhile
    let pressed_on_pic = pic.iter().any(|i| *i == Interaction::Pressed);
    let held = mouse.pressed(MouseButton::Left) && (spin.dragging || pressed_on_pic);
    spin.tick(time.delta_secs(), held.then_some(motion.delta.x));
}

/// The turn (by itself or by hand) with a little bounce, so you can see all of it; the belly is
/// the slider's size and jiggles with each hop.
fn spin_preview(
    time: Res<Time>,
    settings: Res<Settings>,
    spin: Res<Spin>,
    mut q: Query<&mut Transform, With<PreviewModel>>,
    mut bellies: Query<&mut bevy::mesh::morph::MorphWeights, With<PreviewBelly>>,
) {
    let t = time.elapsed_secs();
    for mut tf in &mut q {
        tf.rotation = Quat::from_rotation_y(spin.angle);
        tf.translation.y = (t * 2.4).sin().abs() * 0.04;
    }
    let jiggle = (t * 4.8).cos() * 0.25;
    for mut w in &mut bellies {
        let ws = w.weights_mut();
        if ws.len() >= 2 {
            ws[0] = settings.belly.powf(0.8);
            ws[1] = jiggle * settings.belly;
        }
    }
}

/// The preview only draws while the menu is up.
fn preview_on_only_in_menu(screen: Res<Screen>, mut cam: Query<&mut Camera, With<PreviewCamera>>) {
    for mut c in &mut cam {
        c.is_active = *screen == Screen::Menu;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn on_its_own_it_turns_slowly() {
        let mut s = Spin::default();
        let a0 = s.angle;
        for _ in 0..60 {
            s.tick(DT, None);
        }
        let turned = (s.angle - a0).rem_euclid(std::f32::consts::TAU);
        assert!((turned - AUTO_SPIN).abs() < 0.02, "{turned}");
    }

    #[test]
    fn dragging_turns_it_by_hand_and_holding_still_stops_it() {
        let mut s = Spin::default();
        let a0 = s.angle;
        // drag 100 pixels to the right over 10 frames
        for _ in 0..10 {
            s.tick(DT, Some(10.0));
        }
        let turned = (s.angle - a0).rem_euclid(std::f32::consts::TAU);
        assert!((turned - 100.0 * DRAG_SENS).abs() < 1e-3, "follows the mouse exactly ({turned})");
        // holding the button without moving: it stays where it is (no turning by itself)
        let held = s.angle;
        for _ in 0..30 {
            s.tick(DT, Some(0.0));
        }
        assert!((s.angle - held).abs() < 1e-5);
        assert!(s.vel.abs() < 0.05, "and has no speed left to fling it ({})", s.vel);
        // dragging the other way turns it back
        for _ in 0..10 {
            s.tick(DT, Some(-10.0));
        }
        assert!((s.angle - a0).rem_euclid(std::f32::consts::TAU) < 1e-3);
    }

    #[test]
    fn a_flick_carries_on_and_slows_back_to_the_slow_turn() {
        let mut s = Spin::default();
        for _ in 0..6 {
            s.tick(DT, Some(40.0)); // a quick flick
        }
        let after_flick = s.vel;
        assert!(after_flick > 5.0, "fast ({after_flick})");
        s.tick(DT, None);
        assert!(s.vel > 4.0, "carries on when let go");
        for _ in 0..240 {
            s.tick(DT, None);
        }
        assert!((s.vel - AUTO_SPIN).abs() < 0.05, "back to the slow turn ({})", s.vel);
        // a flick to the left spins it left, then it comes round to the slow turn again
        for _ in 0..6 {
            s.tick(DT, Some(-40.0));
        }
        assert!(s.vel < -5.0);
        for _ in 0..300 {
            s.tick(DT, None);
        }
        assert!((s.vel - AUTO_SPIN).abs() < 0.05);
    }

    #[test]
    fn a_wild_flick_has_a_speed_limit_and_the_angle_stays_tidy() {
        let mut s = Spin::default();
        for _ in 0..10 {
            s.tick(DT, Some(5000.0));
        }
        assert!(s.vel <= MAX_SPIN + 1e-3);
        assert!((0.0..std::f32::consts::TAU).contains(&s.angle));
        s.tick(0.0, Some(3.0));
        assert!(s.angle.is_finite() && s.vel.is_finite());
    }
}
