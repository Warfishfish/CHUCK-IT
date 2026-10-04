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

pub struct PreviewPlugin;

impl Plugin for PreviewPlugin {
    fn build(&self, app: &mut App) {
        let img = Image::new_target_texture(PREVIEW_W, PREVIEW_H, TextureFormat::Rgba8Unorm, None);
        let handle = app.world_mut().resource_mut::<Assets<Image>>().add(img);
        app.insert_resource(PreviewImage(handle))
            .add_systems(Startup, spawn_preview_rig)
            .add_systems(Update, (swap_preview_model, spin_preview, preview_on_only_in_menu));
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
        mats.add(base),
        mats.add(crate::characters::lighter(base)),
        mats.add(crate::characters::darker(base)),
    );
    commands
        .spawn((
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(settings.character.model()))),
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
                }
            },
        );
}

/// A slow turn with a little bounce, so you can see all of it.
fn spin_preview(time: Res<Time>, mut q: Query<&mut Transform, With<PreviewModel>>) {
    let t = time.elapsed_secs();
    for mut tf in &mut q {
        tf.rotation = Quat::from_rotation_y(0.6 + t * 0.7);
        tf.translation.y = (t * 2.4).sin().abs() * 0.04;
    }
}

/// The preview only draws while the menu is up.
fn preview_on_only_in_menu(screen: Res<Screen>, mut cam: Query<&mut Camera, With<PreviewCamera>>) {
    for mut c in &mut cam {
        c.is_active = *screen == Screen::Menu;
    }
}
