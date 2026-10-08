//! The online lobby's line-up (Marcus, 8 Oct 2026): while everybody waits for a round, their
//! blobs stand in a row on the lawn, each wearing their own look, with their name above. When
//! somebody changes their look you see it at once; when they press Ready they throw their arms
//! up (the Gang Beasts way of showing it) and READY shows under their name. The camera looks at
//! the row from the lawn, with the customise and lobby cards in front.
//!
//! The figures are separate from the game's blobs (those are hidden while the lobby is up), so
//! nothing here touches the rules.

use bbq_core::appearance::Appearance;
use bbq_core::character::Character;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use crate::face::{FaceOwner, HeadNode, LookTint, TintPart};
use crate::menu::Settings;
use crate::online::{Lobby, Online, code_char};
use crate::player::EyeCamera;

/// Where the row stands: at `STAGE_Z`, centred a little right of the middle of the view, two
/// rows of eight at most.
pub const STAGE_Z: f32 = 9.0;
pub const SPACING: f32 = 1.55;
pub const PER_ROW: usize = 8;

/// The middle of the row (x), where the camera aims.
pub const STAGE_MID: f32 = 5.0;

/// The camera while the lobby is up: where it stands and what it looks at. It steps back a
/// little for every extra person so the whole row fits between the two cards.
pub fn stage_camera(n: usize) -> (Vec3, Vec3) {
    let front = n.min(PER_ROW).max(1) as f32;
    let back = 3.0 + 0.9 * (front - 1.0) + if n > PER_ROW { 1.5 } else { 0.0 };
    let look = Vec3::new(STAGE_MID, 1.0, STAGE_Z);
    (Vec3::new(STAGE_MID, 1.45 + back * 0.08, STAGE_Z + back), look)
}

/// One person's blob in the line-up.
#[derive(Component)]
pub struct LobbyFigure {
    pub net_id: u32,
    pub character: Character,
    pub slot: usize,
}

/// A figure's hands (they go up when the owner is ready).
#[derive(Component)]
struct LobbyHand {
    net_id: u32,
    right: bool,
    base: Vec3,
}

/// A figure's body and singlet (the beer belly).
#[derive(Component)]
struct LobbyBelly(u32);

/// The name and the READY sign above a figure.
#[derive(Component)]
struct LobbyTag {
    target: Entity,
    height: f32,
    ready_sign: bool,
    net_id: u32,
}

/// How a person in the lobby looks: their body shape, look and belly. For ourselves this is
/// what the menu says right now (so the row answers at once); for the others it is what the host
/// last told everybody.
pub fn lobby_look(online: &Online, settings: &Settings, net_id: u32) -> Option<(Character, Appearance, f32, bool)> {
    let s = online.session.as_ref()?;
    let m = s.members.iter().find(|m| m.id == net_id)?;
    if net_id == s.my_id {
        return Some((settings.character, settings.look, settings.belly, m.ready));
    }
    Some((code_char(m.character), m.look, m.belly as f32 / 255.0 * 2.0, m.ready))
}

/// Where slot `k` stands (front row first; the back row sits between the front ones).
pub fn slot_pos(k: usize, n: usize) -> Vec3 {
    let row = k / PER_ROW;
    let in_row = k % PER_ROW;
    let count = (n - row * PER_ROW).min(PER_ROW).max(1);
    // centre each row on the stage
    let width = (count - 1) as f32 * SPACING;
    let x0 = STAGE_MID - width * 0.5;
    Vec3::new(x0 + in_row as f32 * SPACING + row as f32 * SPACING * 0.5, 0.0, STAGE_Z - row as f32 * 1.8)
}

pub struct LobbyPlugin;

impl Plugin for LobbyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (sync_figures, animate_figures, place_tags).chain());
    }
}

/// Build a figure for everybody in the lobby, and throw away the ones that left (or changed
/// body shape, which needs another model).
fn sync_figures(
    mut commands: Commands,
    assets: Res<AssetServer>,
    lobby: Res<Lobby>,
    online: Res<Online>,
    settings: Res<Settings>,
    figures: Query<(Entity, &LobbyFigure)>,
    tags: Query<(Entity, &LobbyTag)>,
) {
    let wanted: Vec<(u32, Character)> = if lobby.active {
        let mut ids: Vec<u32> = online.session.as_ref().map(|s| s.members.iter().map(|m| m.id).collect()).unwrap_or_default();
        ids.sort();
        ids.into_iter().filter_map(|id| lobby_look(&online, &settings, id).map(|l| (id, l.0))).collect()
    } else {
        Vec::new()
    };
    let n = wanted.len();
    let mut have: Vec<u32> = Vec::new();
    for (e, f) in &figures {
        let slot = wanted.iter().position(|(id, _)| *id == f.net_id);
        match slot {
            Some(k) if wanted[k].1 == f.character && f.slot == k => have.push(f.net_id),
            _ => {
                commands.entity(e).despawn();
                for (t, tag) in &tags {
                    if tag.target == e {
                        commands.entity(t).despawn();
                    }
                }
            }
        }
    }
    for (k, (id, character)) in wanted.iter().enumerate() {
        if have.contains(id) {
            continue;
        }
        let id = *id;
        let at = slot_pos(k, n);
        let root = commands
            .spawn((
                WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(crate::style::model_path(*character)))),
                Transform::from_translation(at),
                LobbyFigure { net_id: id, character: *character, slot: k },
            ))
            .observe(
                move |ready: On<WorldInstanceReady>,
                      children: Query<&Children>,
                      names: Query<&Name>,
                      tfs: Query<&Transform>,
                      has_mat: Query<(), With<MeshMaterial3d<StandardMaterial>>>,
                      mut commands: Commands| {
                    for node in children.iter_descendants(ready.entity) {
                        let Ok(name) = names.get(node) else { continue };
                        let base = tfs.get(node).map(|t| t.translation).unwrap_or_default();
                        let part = match name.as_str() {
                            "Torso" | "Belly" => Some(TintPart::Body),
                            "Head" | "HandR" | "HandL" => Some(TintPart::Head),
                            "FootL" | "FootR" => Some(TintPart::Foot),
                            "Singlet" => Some(TintPart::Singlet),
                            "ThongL" | "ThongR" => Some(TintPart::Thong),
                            _ => None,
                        };
                        if let Some(part) = part {
                            for e in std::iter::once(node).chain(children.iter_descendants(node)) {
                                if has_mat.contains(e) {
                                    commands.entity(e).try_insert(LookTint { owner: FaceOwner::Lobby(id), part });
                                }
                            }
                        }
                        match name.as_str() {
                            "Head" => {
                                commands.entity(node).try_insert(HeadNode { owner: FaceOwner::Lobby(id), layer: None });
                            }
                            "HandR" | "HandL" => {
                                commands.entity(node).try_insert(LobbyHand { net_id: id, right: name.as_str() == "HandR", base });
                            }
                            "Torso" | "Singlet" => {
                                commands.entity(node).try_insert(LobbyBelly(id));
                            }
                            _ => {}
                        }
                    }
                },
            )
            .id();
        // the name above, and the READY sign under it (screen labels that follow the figure)
        let name = online
            .session
            .as_ref()
            .and_then(|s| s.members.iter().find(|m| m.id == id).map(|m| if m.id == s.my_id { format!("{} (you)", m.name) } else { m.name.clone() }))
            .unwrap_or_default();
        // over this shape's head (the shapes are different heights)
        let top = crate::style::measure(*character).head_top;
        for (ready_sign, label, height) in [(false, name, top + 0.5), (true, "READY".to_string(), top + 0.25)] {
            let (bg, fg) = if ready_sign { (crate::ui::GOOD, crate::ui::INK) } else { (crate::ui::HUD, Color::WHITE) };
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(3.0)),
                    border_radius: BorderRadius::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(bg),
                GlobalZIndex(5),
                Visibility::Hidden,
                LobbyTag { target: root, height, ready_sign, net_id: id },
                children![(crate::ui::text(label, if ready_sign { 13.0 } else { 15.0 }, true, fg))],
            ));
        }
    }
}

/// The figures face the camera and bob; ready ones throw their arms up and hop; bellies follow
/// each person's slider.
fn animate_figures(
    time: Res<Time>,
    online: Res<Online>,
    settings: Res<Settings>,
    mut figures: Query<(&LobbyFigure, &mut Transform), (Without<LobbyHand>, Without<EyeCamera>)>,
    mut hands: Query<(&LobbyHand, &mut Transform), Without<LobbyFigure>>,
    mut bellies: Query<(&LobbyBelly, &mut bevy::mesh::morph::MorphWeights)>,
) {
    let t = time.elapsed_secs();
    let n = figures.iter().count();
    for (f, mut tf) in &mut figures {
        let ready = lobby_look(&online, &settings, f.net_id).is_some_and(|l| l.3);
        let base = slot_pos(f.slot, n.max(f.slot + 1));
        let phase = f.slot as f32 * 1.3;
        let hop = if ready { (t * 7.0 + phase).sin().abs() * 0.18 } else { (t * 2.2 + phase).sin().abs() * 0.025 };
        tf.translation = base + Vec3::Y * hop;
        // face the camera, with a little sway from side to side
        let to_cam = stage_camera(n).0 - base;
        let yaw = to_cam.x.atan2(to_cam.z) + (t * 0.9 + phase).sin() * 0.25;
        tf.rotation = Quat::from_rotation_y(yaw);
    }
    for (h, mut tf) in &mut hands {
        let ready = lobby_look(&online, &settings, h.net_id).is_some_and(|l| l.3);
        if ready {
            // arms up, waving
            let side = if h.right { -1.0 } else { 1.0 };
            let wave = (t * 9.0 + if h.right { 0.0 } else { 1.6 }).sin() * 0.08;
            tf.translation = h.base + Vec3::new(side * 0.12 + wave, 0.95, -0.05);
        } else {
            tf.translation = h.base + Vec3::new(0.0, (t * 2.2 + h.net_id as f32).sin() * 0.02, 0.0);
        }
    }
    for (b, mut w) in &mut bellies {
        let size = lobby_look(&online, &settings, b.0).map_or(1.0, |l| l.2);
        let ws = w.weights_mut();
        if ws.len() >= 2 {
            ws[0] = size.powf(0.8);
            ws[1] = (t * 4.0 + b.0 as f32).cos() * 0.15 * size;
        }
    }
}

/// Names and READY signs float above their figure, as labels on the screen.
fn place_tags(
    online: Res<Online>,
    settings: Res<Settings>,
    cam: Single<(&Camera, &GlobalTransform), With<EyeCamera>>,
    targets: Query<&GlobalTransform, With<LobbyFigure>>,
    mut tags: Query<(&LobbyTag, &mut Node, &ComputedNode, &mut Visibility)>,
) {
    let (camera, cam_gt) = *cam;
    for (tag, mut node, computed, mut vis) in &mut tags {
        let Ok(gt) = targets.get(tag.target) else { continue };
        let look = lobby_look(&online, &settings, tag.net_id);
        // legs stand the figure up taller (C9.6)
        let lift = look.map_or(0.0, |l| l.1.legs.lift());
        let p = gt.translation() + Vec3::Y * (tag.height + lift);
        let shown = !tag.ready_sign || look.is_some_and(|l| l.3);
        let at = camera.world_to_viewport(cam_gt, p).ok();
        let want = if shown && at.is_some() { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
        if let Some(at) = at {
            // centred on the point (the label's own size is in physical pixels)
            let size = computed.size() * computed.inverse_scale_factor();
            node.left = Val::Px(at.x - size.x * 0.5);
            node.top = Val::Px(at.y - size.y * 0.5);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_row_is_tidy() {
        // one to sixteen people: nobody stands on anybody, the front row is in front
        for n in 1..=16 {
            let spots: Vec<Vec3> = (0..n).map(|k| slot_pos(k, n)).collect();
            for (i, a) in spots.iter().enumerate() {
                for b in &spots[i + 1..] {
                    assert!(a.distance(*b) > 1.2, "{n}: {a} and {b} too close");
                }
                assert!(a.z <= STAGE_Z + 1e-4);
            }
        }
        // a pair stands either side of the middle of the view
        let (a, b) = (slot_pos(0, 2), slot_pos(1, 2));
        assert!(((a.x + b.x) * 0.5 - STAGE_MID).abs() < 1e-4);
        // more people: the camera steps back
        assert!(stage_camera(8).0.z > stage_camera(2).0.z);
    }
}
