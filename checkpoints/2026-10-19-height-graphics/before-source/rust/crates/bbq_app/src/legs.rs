//! Legs (C9.6, Marcus 8 Oct 2026). They're a customisation: blobs have no legs unless you pick
//! Stumpy or Skinny on the Customise tab. Then your blob stands on two legs that follow its feet
//! as they step.
//!
//! The models have no legs. When a blob's model loads (in the yard, the menu preview or the
//! lobby), its feet, thongs and straps are moved out from under the body so they stay on the
//! ground, and two legs are added beside them. Each frame the body (with the head, eyes and
//! hands) is lifted by the legs' length and each leg is stretched from the hip to its foot.
//! Looks only: the hit and catch sizes do not change.
//!
//! Walking (10 Oct 2026, "so it looks like they are walking instead of floating"): on legs the
//! feet take longer, higher steps (`characters.rs`), the body dips a little as the feet pass and
//! spread (like a real walk), and Skinny legs have knees: a thigh and a shin that bend forward at
//! a round knee as the foot swings and lifts.

use bbq_core::appearance::Legs;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::face::{FaceOwner, HeadNode};
use crate::game::Game;
use crate::menu::Settings;

/// Where a leg meets the body, in the model's own space: the feet sit 0.2 m either side, and the
/// body's bottom is 0.07 m up (`make_blob.py`). The top of the leg is hidden inside the body.
const HIP_X: f32 = 0.2;
const HIP_UP: f32 = 0.07 + 0.16;
const LEG_Z: f32 = 0.04;
/// From the middle of a foot to its ankle (inside the foot, so the leg's end is hidden).
const ANKLE: Vec3 = Vec3::new(0.0, 0.04, -0.08);
/// The black line round the legs in the pop look (the same as round the hands and feet).
const LINE: f32 = 0.011;
/// How much the body dips for each metre the two feet are apart along the walk.
const BOB: f32 = 0.2;
/// Skinny legs' thigh and shin together are this much longer than the hip-to-ankle gap when
/// standing, so the knee is a touch bent even at rest.
const KNEE_SLACK: f32 = 1.07;

/// A blob's legs: on the model's body node, which is lifted when the legs are on.
#[derive(Component)]
struct LegRig {
    owner: FaceOwner,
    /// The left and right foot (where each leg ends).
    feet: [Entity; 2],
    /// The two legs (the thighs, for legs with knees), and the black line round each (pop look
    /// only).
    legs: [Entity; 2],
    lines: [Option<Entity>; 2],
    /// Skinny legs: the shins and the knees, each with its line.
    shins: [Entity; 2],
    shin_lines: [Option<Entity>; 2],
    knees: [Entity; 2],
    knee_lines: [Option<Entity>; 2],
    /// Where each foot rests (to see how far it has stepped).
    rest: [Vec3; 2],
    /// The right hand: the legs wear its colour.
    hand: Entity,
    /// The kind of legs the meshes are now.
    shown: Legs,
}

/// A leg.
#[derive(Component)]
struct LegPart;
/// The black line round a leg.
#[derive(Component)]
struct LegLine;

/// The leg shapes: one unit-tall tapered tube per kind (stretched to the leg's length), plus a
/// slightly fatter one for the black line.
#[derive(Resource)]
struct LegKit {
    stubby: (Handle<Mesh>, Handle<Mesh>),
    skinny: (Handle<Mesh>, Handle<Mesh>),
    /// Skinny legs with knees: thigh, shin and knee (each with its line).
    thigh: (Handle<Mesh>, Handle<Mesh>),
    shin: (Handle<Mesh>, Handle<Mesh>),
    knee: (Handle<Mesh>, Handle<Mesh>),
    outline: Handle<StandardMaterial>,
    plain: Handle<StandardMaterial>,
}

impl LegKit {
    fn meshes(&self, legs: Legs) -> &(Handle<Mesh>, Handle<Mesh>) {
        match legs {
            Legs::Skinny => &self.skinny,
            _ => &self.stubby,
        }
    }
}

pub struct LegsPlugin;

impl Plugin for LegsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, make_kit).add_systems(
            PostUpdate,
            // after the feet have stepped this frame, before everything is moved into place
            (rig_legs, pose_legs).chain().before(bevy::transform::TransformSystems::Propagate),
        );
    }
}

fn make_kit(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let mut tube = |top: f32, bottom: f32, grow: f32| {
        let mut m = Mesh::from(ConicalFrustum { radius_top: top + grow, radius_bottom: bottom + grow, height: 1.0 });
        let _ = m.generate_tangents(); // the clay look's skin texture needs them
        meshes.add(m)
    };
    let ball = |r: f32| {
        let mut m = Mesh::from(Sphere::new(r).mesh().uv(16, 10));
        let _ = m.generate_tangents();
        m
    };
    let (st, sb) = Legs::Stumpy.radii();
    let (kt, kb) = Legs::Skinny.radii();
    let km = knee_radius();
    let kit = LegKit {
        stubby: (tube(st, sb, 0.0), tube(st, sb, LINE)),
        skinny: (tube(kt, kb, 0.0), tube(kt, kb, LINE)),
        thigh: (tube(kt, km, 0.0), tube(kt, km, LINE)),
        shin: (tube(km * 0.9, kb, 0.0), tube(km * 0.9, kb, LINE)),
        knee: (meshes.add(ball(km * 1.15)), meshes.add(ball(km * 1.15 + LINE))),
        outline: crate::style::outline_material(&mut mats),
        plain: mats.add(Color::WHITE),
    };
    commands.insert_resource(kit);
}

/// How thick a Skinny leg is at the knee.
fn knee_radius() -> f32 {
    let (top, bottom) = Legs::Skinny.radii();
    (top + bottom) * 0.5
}

/// What kind of legs this blob, preview or lobby figure wears now.
fn legs_of(owner: FaceOwner, game: &Game, settings: &Settings, online: Option<&crate::online::Online>) -> Option<Legs> {
    match owner {
        FaceOwner::Preview => Some(settings.look.legs),
        FaceOwner::Lobby(id) => online.and_then(|o| crate::lobby::lobby_look(o, settings, id)).map(|l| l.1.legs),
        FaceOwner::Blob(i) => game.dummies.get(i).map(legs_shown),
    }
}

/// The legs a yard blob has on show: none while it sits at smoko or in the Naughty Corner,
/// because the sitting pose already lowers the whole blob onto the chair (so a seated blob looks
/// just as it does without legs).
pub fn legs_shown(d: &crate::game::Dummy) -> Legs {
    if d.seat.is_some() || d.naughty_t > 0.0 { Legs::None } else { d.look.legs }
}

/// How much a yard blob's legs lift it (0 without legs). Things that sit on a blob (the crown,
/// the stars, the name tag, a held item) go up by this much.
pub fn lift_of(game: &Game, i: usize) -> f32 {
    game.dummies.get(i).map_or(0.0, |d| legs_shown(d).lift())
}

/// A model has just loaded (its head was found): move its feet out from under the body and add
/// two hidden legs beside them.
fn rig_legs(
    mut commands: Commands,
    kit: Res<LegKit>,
    heads: Query<(Entity, &HeadNode), Added<HeadNode>>,
    parents: Query<&ChildOf>,
    kids: Query<&Children>,
    names: Query<&Name>,
    tfs: Query<&Transform>,
) {
    for (head, node) in &heads {
        let Ok(body) = parents.get(head).map(|p| p.parent()) else { continue };
        if names.get(body).map(|n| n.as_str()) != Ok("Body") {
            continue;
        }
        let Ok(root) = parents.get(body).map(|p| p.parent()) else { continue };
        let named = |want: &str| kids.get(body).ok().and_then(|c| c.iter().find(|e| names.get(*e).is_ok_and(|n| n.as_str() == want)));
        let (Some(foot_l), Some(foot_r), Some(hand)) = (named("FootL"), named("FootR"), named("HandR")) else { continue };
        // the feet, thongs and straps hang off the model's root instead of the body, so lifting
        // the body leaves them on the ground (the body sits at the root's middle, so they stay put)
        if let Ok(children) = kids.get(body) {
            for e in children.iter() {
                if names.get(e).is_ok_and(|n| n.starts_with("Foot") || n.starts_with("Thong") || n.starts_with("Strap")) {
                    commands.entity(e).insert(ChildOf(root));
                }
            }
        }
        // a black line like the rest of the body has (only yard blobs have them in the pop look)
        let pop = crate::style::current() == crate::style::Style::Pop && matches!(node.owner, FaceOwner::Blob(_));
        // one leg part (hidden until needed) with its black line
        let mut part = |commands: &mut Commands, (mesh, line): &(Handle<Mesh>, Handle<Mesh>)| -> (Entity, Option<Entity>) {
            let mut leg = commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(kit.plain.clone()),
                Transform::default(),
                Visibility::Hidden,
                LegPart,
                ChildOf(root),
            ));
            if let Some(l) = node.layer {
                leg.insert(RenderLayers::layer(l));
            }
            let id = leg.id();
            let line = pop.then(|| {
                let mut ol = commands.spawn((
                    Mesh3d(line.clone()),
                    MeshMaterial3d(kit.outline.clone()),
                    Transform::default(),
                    bevy::light::NotShadowCaster,
                    LegLine,
                    ChildOf(id),
                ));
                if let Some(l) = node.layer {
                    ol.insert(RenderLayers::layer(l));
                }
                ol.id()
            });
            (id, line)
        };
        let (mut legs, mut lines) = ([Entity::PLACEHOLDER; 2], [None; 2]);
        let (mut shins, mut shin_lines) = ([Entity::PLACEHOLDER; 2], [None; 2]);
        let (mut knees, mut knee_lines) = ([Entity::PLACEHOLDER; 2], [None; 2]);
        for k in 0..2 {
            (legs[k], lines[k]) = part(&mut commands, kit.meshes(Legs::Stumpy));
            (shins[k], shin_lines[k]) = part(&mut commands, &kit.shin);
            (knees[k], knee_lines[k]) = part(&mut commands, &kit.knee);
        }
        let rest = [foot_rest(&tfs, foot_l), foot_rest(&tfs, foot_r)];
        commands.entity(body).try_insert(LegRig {
            owner: node.owner,
            feet: [foot_l, foot_r],
            legs,
            lines,
            shins,
            shin_lines,
            knees,
            knee_lines,
            rest,
            hand,
            shown: Legs::Stumpy,
        });
    }
}

/// Where a foot rests: its spot in the model (the model has just loaded, so it has not stepped).
fn foot_rest(tfs: &Query<&Transform>, foot: Entity) -> Vec3 {
    tfs.get(foot).map(|t| t.translation).unwrap_or_default()
}

/// Where the knee goes for a leg from `hip` to `ankle` with a thigh and shin each `half` long:
/// half way down, pushed forwards (the blobs face +z) as far as the two lengths need.
pub fn knee_at(hip: Vec3, ankle: Vec3, half: f32) -> Vec3 {
    let d = ankle - hip;
    let len = d.length().max(0.001);
    let along = d / len;
    let mid = (hip + ankle) * 0.5;
    let bend = (half * half - (len * 0.5) * (len * 0.5)).max(0.0).sqrt();
    let fwd = Vec3::Z - along * along.dot(Vec3::Z);
    let fwd = if fwd.length_squared() < 1e-6 { Vec3::Z } else { fwd.normalize() };
    mid + fwd * bend
}

/// Lift each body by its legs and stretch each leg from the hip to its foot.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn pose_legs(
    game: Res<Game>,
    settings: Res<Settings>,
    online: Option<Res<crate::online::Online>>,
    kit: Res<LegKit>,
    mut rigs: Query<(&mut LegRig, &mut Transform), (Without<LegPart>, Without<LegLine>)>,
    feet: Query<&Transform, (Without<LegRig>, Without<LegPart>, Without<LegLine>)>,
    kids: Query<&Children>,
    hand_mats: Query<&MeshMaterial3d<StandardMaterial>, (Without<LegPart>, Without<LegLine>)>,
    mut legs: Query<
        (&mut Transform, &mut Visibility, &mut Mesh3d, &mut MeshMaterial3d<StandardMaterial>),
        (With<LegPart>, Without<LegRig>, Without<LegLine>),
    >,
    mut lines: Query<&mut Mesh3d, (With<LegLine>, Without<LegPart>)>,
) {
    for (mut rig, mut body) in &mut rigs {
        let Some(kind) = legs_of(rig.owner, &game, &settings, online.as_deref()) else { continue };
        let lift = kind.lift();
        // the body dips a little as the feet spread apart in a step, and rises as they pass
        let step = |k: usize| feet.get(rig.feet[k]).map(|f| f.translation - rig.rest[k]).unwrap_or_default();
        let spread = (step(0).z - step(1).z).abs();
        let bob = if kind == Legs::None { 0.0 } else { -spread * BOB };
        if body.translation.y != lift + bob {
            body.translation.y = lift + bob;
        }
        let lift = lift + bob;
        if kind != Legs::None && rig.shown != kind {
            let _ = (&rig.shin_lines, &rig.knee_lines);
            // Skinny legs are a thigh (the leg part), a shin and a knee
            let (tube, line) = if kind == Legs::Skinny { kit.thigh.clone() } else { kit.meshes(kind).clone() };
            for k in 0..2 {
                if let Ok((_, _, mut m, _)) = legs.get_mut(rig.legs[k]) {
                    m.0 = tube.clone();
                }
                if let Some(Ok(mut m)) = rig.lines[k].map(|e| lines.get_mut(e)) {
                    m.0 = line.clone();
                }
            }
            rig.shown = kind;
        }
        // the legs wear the hand's colour (it changes with the look)
        let hand_mat = kids
            .get(rig.hand)
            .ok()
            .and_then(|c| c.iter().find_map(|e| hand_mats.get(e).ok()))
            .map(|m| m.0.clone());
        let knees = kind == Legs::Skinny;
        for k in 0..2 {
            let side = if k == 0 { -1.0 } else { 1.0 };
            let ankle = feet.get(rig.feet[k]).map(|f| f.translation + ANKLE).ok();
            let hip = Vec3::new(side * HIP_X, HIP_UP + lift, LEG_Z);
            // the knee: thigh and shin each half the standing leg (a little more), bent forwards
            let rest_ankle = rig.rest[k] + ANKLE;
            let half = (Vec3::new(side * HIP_X, HIP_UP + kind.lift(), LEG_Z) - rest_ankle).length() * 0.5 * KNEE_SLACK;
            let knee = ankle.map(|a| knee_at(hip, a, half));
            let parts = [(rig.legs[k], true), (rig.shins[k], knees), (rig.knees[k], knees)];
            for (n, (e, on)) in parts.into_iter().enumerate() {
                let Ok((mut tf, mut vis, _, mut mat)) = legs.get_mut(e) else { continue };
                let on = on && kind != Legs::None;
                let want = if on { Visibility::Inherited } else { Visibility::Hidden };
                if *vis != want {
                    *vis = want;
                }
                if !on {
                    continue;
                }
                if let Some(m) = &hand_mat
                    && mat.0 != *m
                {
                    mat.0 = m.clone();
                }
                let (Some(ankle), Some(knee)) = (ankle, knee) else { continue };
                *tf = match (n, knees) {
                    (0, false) => leg_transform(ankle, hip), // one straight leg
                    (0, true) => leg_transform(knee, hip),   // the thigh
                    (1, _) => leg_transform(ankle, knee),    // the shin
                    _ => Transform::from_translation(knee),  // the knee
                };
            }
        }
    }
}

/// A unit-tall tube stood from the ankle up to the hip.
fn leg_transform(ankle: Vec3, hip: Vec3) -> Transform {
    let d = hip - ankle;
    let len = d.length().max(0.01);
    Transform {
        translation: (ankle + hip) * 0.5,
        rotation: Quat::from_rotation_arc(Vec3::Y, d / len),
        scale: Vec3::new(1.0, len, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbq_core::appearance::Choice;

    #[test]
    fn a_leg_runs_from_the_ankle_to_the_hip() {
        let ankle = Vec3::new(0.2, 0.14, 0.04);
        let hip = Vec3::new(0.2, 0.53, 0.04);
        let t = leg_transform(ankle, hip);
        // the tube's ends (half a unit up and down, stretched) land on the two points
        let top = t.transform_point(Vec3::new(0.0, 0.5, 0.0));
        let bottom = t.transform_point(Vec3::new(0.0, -0.5, 0.0));
        assert!(top.distance(hip) < 1e-4 && bottom.distance(ankle) < 1e-4);
        // a foot stepping forward tilts the leg, and it still reaches
        let fwd = Vec3::new(0.2, 0.2, 0.25);
        let t = leg_transform(fwd, hip);
        assert!(t.transform_point(Vec3::new(0.0, -0.5, 0.0)).distance(fwd) < 1e-4);
    }

    #[test]
    fn a_knee_bends_forwards_and_the_two_halves_reach() {
        let hip = Vec3::new(0.2, 0.53, 0.04);
        let ankle = Vec3::new(0.2, 0.14, 0.04);
        let half = (hip - ankle).length() * 0.5 * KNEE_SLACK;
        let knee = knee_at(hip, ankle, half);
        assert!(knee.z > hip.z, "standing: a slight forward bend");
        assert!((knee.distance(hip) - half).abs() < 1e-4 && (knee.distance(ankle) - half).abs() < 1e-4);
        // the foot lifts and swings forwards: the knee comes up and out further
        let lifted = Vec3::new(0.2, 0.24, 0.2);
        let k2 = knee_at(hip, lifted, half);
        assert!(k2.z > knee.z && (k2.distance(hip) - half).abs() < 1e-4);
        // stretched further than the leg reaches: straight, no bend
        let far = Vec3::new(0.2, -0.5, 0.04);
        let k3 = knee_at(hip, far, half);
        assert!((k3 - (hip + far) * 0.5).length() < 1e-4);
    }

    #[test]
    fn the_legs_reach_the_body_at_every_lift() {
        // the hip end is always inside the body (above its bottom) and above the foot
        for kind in Legs::ALL {
            let hip_up = HIP_UP + kind.lift();
            assert!(hip_up > 0.07 + kind.lift(), "{kind:?}: the top of the leg is hidden in the body");
            assert!(hip_up > 0.14 + 0.05, "{kind:?}: the leg goes up from the foot");
        }
    }
}
