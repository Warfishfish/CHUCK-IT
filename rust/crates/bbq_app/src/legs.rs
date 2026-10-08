//! Legs (C9.6, Marcus 8 Oct 2026). They're a customisation: blobs have no legs unless you pick
//! Stumpy or Skinny on the Customise tab. Then your blob stands on two legs that follow its feet
//! as they step.
//!
//! The models have no legs. When a blob's model loads (in the yard, the menu preview or the
//! lobby), its feet, thongs and straps are moved out from under the body so they stay on the
//! ground, and two legs are added beside them. Each frame the body (with the head, eyes and
//! hands) is lifted by the legs' length and each leg is stretched from the hip to its foot.
//! Looks only: the hit and catch sizes do not change.

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

/// A blob's legs: on the model's body node, which is lifted when the legs are on.
#[derive(Component)]
struct LegRig {
    owner: FaceOwner,
    /// The left and right foot (where each leg ends).
    feet: [Entity; 2],
    /// The two legs, and the black line round each (pop look only).
    legs: [Entity; 2],
    lines: [Option<Entity>; 2],
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
    let mut tube = |legs: Legs, grow: f32| {
        let (top, bottom) = legs.radii();
        let mut m = Mesh::from(ConicalFrustum { radius_top: top + grow, radius_bottom: bottom + grow, height: 1.0 });
        let _ = m.generate_tangents(); // the clay look's skin texture needs them
        meshes.add(m)
    };
    let kit = LegKit {
        stubby: (tube(Legs::Stumpy, 0.0), tube(Legs::Stumpy, LINE)),
        skinny: (tube(Legs::Skinny, 0.0), tube(Legs::Skinny, LINE)),
        outline: crate::style::outline_material(&mut mats),
        plain: mats.add(Color::WHITE),
    };
    commands.insert_resource(kit);
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
        let (tube, line) = kit.meshes(Legs::Stumpy).clone();
        let mut legs = [Entity::PLACEHOLDER; 2];
        let mut lines = [None; 2];
        for k in 0..2 {
            let mut leg = commands.spawn((
                Mesh3d(tube.clone()),
                MeshMaterial3d(kit.plain.clone()),
                Transform::default(),
                Visibility::Hidden,
                LegPart,
                ChildOf(root),
            ));
            if let Some(l) = node.layer {
                leg.insert(RenderLayers::layer(l));
            }
            legs[k] = leg.id();
            if pop {
                let mut ol = commands.spawn((
                    Mesh3d(line.clone()),
                    MeshMaterial3d(kit.outline.clone()),
                    Transform::default(),
                    bevy::light::NotShadowCaster,
                    LegLine,
                    ChildOf(legs[k]),
                ));
                if let Some(l) = node.layer {
                    ol.insert(RenderLayers::layer(l));
                }
                lines[k] = Some(ol.id());
            }
        }
        commands.entity(body).try_insert(LegRig { owner: node.owner, feet: [foot_l, foot_r], legs, lines, hand, shown: Legs::Stumpy });
    }
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
        if body.translation.y != lift {
            body.translation.y = lift;
        }
        if kind != Legs::None && rig.shown != kind {
            let (tube, line) = kit.meshes(kind).clone();
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
        for k in 0..2 {
            let Ok((mut tf, mut vis, _, mut mat)) = legs.get_mut(rig.legs[k]) else { continue };
            let want = if kind == Legs::None { Visibility::Hidden } else { Visibility::Inherited };
            if *vis != want {
                *vis = want;
            }
            if kind == Legs::None {
                continue;
            }
            if let Some(m) = &hand_mat
                && mat.0 != *m
            {
                mat.0 = m.clone();
            }
            let Ok(foot) = feet.get(rig.feet[k]) else { continue };
            let side = if k == 0 { -1.0 } else { 1.0 };
            let ankle = foot.translation + ANKLE;
            let hip = Vec3::new(side * HIP_X, HIP_UP + lift, LEG_Z);
            *tf = leg_transform(ankle, hip);
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
    fn the_legs_reach_the_body_at_every_lift() {
        // the hip end is always inside the body (above its bottom) and above the foot
        for kind in Legs::ALL {
            let hip_up = HIP_UP + kind.lift();
            assert!(hip_up > 0.07 + kind.lift(), "{kind:?}: the top of the leg is hidden in the body");
            assert!(hip_up > 0.14 + 0.05, "{kind:?}: the leg goes up from the foot");
        }
    }
}
