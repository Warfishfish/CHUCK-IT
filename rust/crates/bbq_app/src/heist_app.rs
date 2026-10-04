//! Phase 7: Teddy Heist in the game. Teddies in each base with a team flag, stealing, sending
//! strays home, banking for 150, dropping a teddy when you're knocked about, strays that leave
//! the yard going home, the bots' objective goals, and the arena and big scoreboard on screen.
//! The rules themselves are in `bbq_core::heist`.

use bbq_core::flight::{ItemId, ItemState};
use bbq_core::hands::{Picker, can_pick_up};
use bbq_core::heist::{self, Bank, Stray, Touch, Who};
use bbq_core::items::ItemKind;
use bbq_core::teams::Team;
use bbq_core::vec::V3;
use bbq_core::yard::Yard;
use bbq_core::{GameMode, PlayerId, YARD_HALF_X, YARD_HALF_Z};
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::game::Game;
use crate::models::{ModelCache, hex};
use crate::player::{PLAYER_ID, Player};
use crate::yard_scene::YardRes;

/// The Heist round in progress.
pub struct HeistState {
    pub teams: usize,
    pub bank: Bank,
}

/// A team's number on a teddy (`Item::team`): Red 0, Blue 1, Green 2, Yellow 3.
pub fn team_index(t: Team) -> Option<u8> {
    heist::TEAM_KEYS
        .iter()
        .position(|k| *k == t)
        .map(|i| i as u8)
}

pub fn team_of_index(i: u8) -> Option<Team> {
    heist::TEAM_KEYS.get(i as usize).copied()
}

/// Put a teddy in every base: 2 to start, more with bigger teams (`teddy_count`).
pub fn start(g: &mut Game, yard: &Yard) {
    let teams = yard.heist.unwrap_or(2);
    let keys = heist::team_keys(teams);
    g.heist = Some(HeistState {
        teams,
        bank: Bank::new(keys),
    });
    let n = heist::teddy_count(g.dummies.len() + 1, teams);
    let layout = heist::layout(teams);
    for (def, team) in layout.into_iter().zip(keys) {
        for i in 0..n {
            let (x, z) = heist::teddy_spot(def, i).unwrap_or_else(|| {
                (
                    def.x + g.rng.range(-1.4, 1.4),
                    def.z + g.rng.range(-1.4, 1.4),
                )
            });
            let id = g.world.spawn(ItemKind::Teddy, x, z, true, &mut g.rng);
            if let Some(it) = g.world.items.get_mut(&id) {
                it.team = team_index(*team);
            }
        }
    }
}

fn base_centre(teams: usize, t: Team) -> Option<V3> {
    heist::base_of(teams, t).map(|b| V3::new(b.x, 0.0, b.z))
}

/// Send a loose teddy back to its base, tossing it in from above.
pub fn return_teddy(g: &mut Game, id: ItemId) {
    let Some(h) = &g.heist else {
        return;
    };
    let teams = h.teams;
    let Some(it) = g.world.items.get(&id) else {
        return;
    };
    let Some(team) = it.team.and_then(team_of_index) else {
        return;
    };
    let Some(c) = base_centre(teams, team) else {
        return;
    };
    // already home: nothing to do
    if it.state == ItemState::Ground && it.pos.horiz_dist(c) < heist::HOME_RADIUS {
        return;
    }
    // whoever had it loses it
    if let Some(holder) = it.holder {
        if holder == PLAYER_ID {
            g.slots.remove(id);
        } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == holder) {
            d.bot.slots.remove(id);
        }
    }
    let (dx, dz) = (g.rng.range(-1.2, 1.2), g.rng.range(-1.2, 1.2));
    let (vx, vz) = (g.rng.range(-1.0, 1.0), g.rng.range(-1.0, 1.0));
    if let Some(it) = g.world.items.get_mut(&id) {
        it.holder = None;
        it.state = ItemState::Flying;
        it.live = false;
        it.thrower = None;
        it.pos = V3::new(c.x + dx, 2.2, c.z + dz);
        it.vel = V3::new(vx, 2.5, vz);
    }
    g.say(format!(
        "A stray {} teddy scurried back to base",
        team.name()
    ));
}

/// What walking over a ground item should do for `who`: pick it up, or (for a Heist teddy)
/// steal it, send your own home, or ignore it. Returns the item to give them, if any.
#[allow(clippy::too_many_arguments)]
pub fn try_pickup(
    g: &mut Game,
    who: PlayerId,
    pos: V3,
    held: usize,
    stunned: bool,
    is_bot: bool,
) -> Option<ItemId> {
    let now = g.now;
    let picker = Picker {
        id: who,
        pos,
        held,
        stunned,
        frozen: false,
        is_bot,
    };
    let mut cands: Vec<(ItemId, f32)> = g
        .world
        .items
        .values()
        .filter(|it| can_pick_up(&picker, it, now))
        .map(|it| (it.id, it.pos.horiz_dist(pos)))
        .collect();
    cands.sort_by(|a, b| a.1.total_cmp(&b.1));
    for (id, _) in cands {
        let team = g.world.items.get(&id).and_then(|it| it.team);
        let Some(ti) = team else {
            return Some(id);
        };
        let (Some(h), Some(mine), Some(theirs)) =
            (g.heist.as_ref(), g.teams.get(who), team_of_index(ti))
        else {
            return Some(id);
        };
        let home = base_centre(h.teams, theirs).unwrap_or(V3::ZERO);
        let d = g.world.items[&id].pos.horiz_dist(home);
        match heist::touch_teddy(mine, theirs, d) {
            Touch::Steal => return Some(id),
            Touch::SendHome => {
                return_teddy(g, id);
                return None;
            }
            Touch::Nothing => continue,
        }
    }
    None
}

/// Make a stray that left the yard come home (call with the ids that were removed).
pub fn teddies_left(g: &mut Game, snapshot: &[(ItemId, u8)], removed: &[ItemId]) {
    let Some(h) = &g.heist else {
        return;
    };
    let teams = h.teams;
    for (id, ti) in snapshot {
        if !removed.contains(id) {
            continue;
        }
        let Some(team) = team_of_index(*ti) else {
            continue;
        };
        let Some(c) = base_centre(teams, team) else {
            continue;
        };
        let new = g.world.spawn(ItemKind::Teddy, c.x, c.z, false, &mut g.rng);
        let (dx, dz) = (g.rng.range(-1.2, 1.2), g.rng.range(-1.2, 1.2));
        if let Some(it) = g.world.items.get_mut(&new) {
            it.team = Some(*ti);
            it.pos = V3::new(c.x + dx, 2.2, c.z + dz);
            it.state = ItemState::Flying;
        }
        g.say(format!(
            "A {} teddy went over the wall and came back",
            team.name()
        ));
    }
}

/// The team teddies now, with their teams (so a removal can be spotted afterwards).
pub fn team_teddies(g: &Game) -> Vec<(ItemId, u8)> {
    g.world
        .items
        .values()
        .filter_map(|it| it.team.map(|t| (it.id, t)))
        .collect()
}

/// Bank stolen teddies, and drop them when knocked about. Every fixed step.
pub fn step(g: &mut Game, p: &Player) {
    let Some(teams) = g.heist.as_ref().map(|h| h.teams) else {
        return;
    };
    let ids: Vec<PlayerId> = std::iter::once(PLAYER_ID)
        .chain(g.dummies.iter().map(|d| d.id))
        .collect();
    for id in ids {
        let (pos, stunned, down) = if id == PLAYER_ID {
            (
                V3::new(p.mover.x, p.mover.y, p.mover.z),
                g.me.body.stun > 0.0,
                g.me.body.is_down(),
            )
        } else if let Some(d) = g.dummies.iter().find(|d| d.id == id) {
            (
                V3::new(d.mover.x, d.mover.y, d.mover.z),
                d.body.stun > 0.0,
                d.body.is_down(),
            )
        } else {
            continue;
        };
        let held: Vec<ItemId> = if id == PLAYER_ID {
            g.slots.ids().to_vec()
        } else {
            g.dummies
                .iter()
                .find(|d| d.id == id)
                .map(|d| d.bot.slots.ids().to_vec())
                .unwrap_or_default()
        };
        let Some(mine) = g.teams.get(id) else {
            continue;
        };
        for item in held {
            let Some(ti) = g.world.items.get(&item).and_then(|it| it.team) else {
                continue;
            };
            let Some(theirs) = team_of_index(ti) else {
                continue;
            };
            // knocked or stunned: it drops where you stand
            if stunned || down {
                drop_held(g, id, item, pos);
                continue;
            }
            let Some(home) = base_centre(teams, mine) else {
                continue;
            };
            if heist::can_bank(mine, theirs, pos.horiz_dist(home), stunned, down) {
                bank(g, id, item, mine);
            }
        }
    }
}

fn drop_held(g: &mut Game, who: PlayerId, item: ItemId, pos: V3) {
    if who == PLAYER_ID {
        g.slots.remove(item);
    } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == who) {
        d.bot.slots.remove(item);
    }
    let (ox, oz) = (g.rng.range(-0.5, 0.5), g.rng.range(-0.5, 0.5));
    let (vx, vz) = (g.rng.range(-1.0, 1.0), g.rng.range(-1.0, 1.0));
    if let Some(it) = g.world.items.get_mut(&item) {
        it.state = ItemState::Flying;
        it.holder = None;
        it.thrower = None;
        it.live = false;
        it.pos = V3::new(pos.x + ox, pos.y + 0.8, pos.z + oz);
        it.vel = V3::new(vx, 2.0, vz);
    }
}

fn bank(g: &mut Game, who: PlayerId, item: ItemId, mine: Team) {
    if who == PLAYER_ID {
        g.slots.remove(item);
    } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == who) {
        d.bot.slots.remove(item);
    }
    g.world.remove(item);
    let count = g
        .heist
        .as_mut()
        .map(|h| h.bank.add(mine))
        .unwrap_or_default();
    let pts = g.board.bank(&g.rules, who);
    let name = crate::round::name_of(g, who);
    g.say(format!(
        "{name} banked a teddy for the {} team! ({count} so far, +{pts})",
        mine.name()
    ));
    if who == PLAYER_ID {
        g.popup(format!("BANKED! +{pts}"), true);
    }
}

/// `Red 2 | Blue 1` with the points.
pub fn tally_text(h: &HeistState) -> String {
    heist::team_keys(h.teams)
        .iter()
        .map(|t| {
            format!(
                "{} {} ({} pts)",
                t.name(),
                h.bank.get(*t),
                h.bank.points(*t)
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

// ------------------------------------------------------------------ the bots' objective

/// The goal for the bot in dummy slot `i`, and whether it is carrying a stolen teddy.
pub fn goal_for_bot(g: &mut Game, i: usize) -> (Option<V3>, bool) {
    let Some(teams) = g.heist.as_ref().map(|h| h.teams) else {
        return (None, false);
    };
    let id = g.dummies[i].id;
    let Some(my_team) = g.teams.get(id) else {
        return (None, false);
    };
    let carrying_of = |g: &Game, held: &[ItemId]| -> Vec<Team> {
        held.iter()
            .filter_map(|h| {
                g.world
                    .items
                    .get(h)
                    .and_then(|it| it.team)
                    .and_then(team_of_index)
            })
            .collect()
    };
    let mut people = Vec::new();
    let me_pos = V3::new(g.dummies[i].mover.x, 0.0, g.dummies[i].mover.z);
    people.push(Who {
        id: PLAYER_ID,
        pos: V3::new(0.0, 0.0, 0.0),
        team: g.teams.get(PLAYER_ID).unwrap_or(Team::Wildcard),
        is_bot: false,
        carrying: carrying_of(g, g.slots.ids()),
    });
    for d in &g.dummies {
        people.push(Who {
            id: d.id,
            pos: V3::new(d.mover.x, 0.0, d.mover.z),
            team: g.teams.get(d.id).unwrap_or(Team::Wildcard),
            is_bot: true,
            carrying: carrying_of(g, d.bot.slots.ids()),
        });
    }
    let strays: Vec<Stray> = g
        .world
        .items
        .values()
        .filter_map(|it| {
            it.team.and_then(team_of_index).map(|t| Stray {
                team: t,
                pos: it.pos,
                on_ground: it.state == ItemState::Ground,
            })
        })
        .collect();
    let me = people[i + 1].clone();
    let carrying_stolen = me.carrying.iter().any(|t| *t != my_team);
    let hands_free = g.dummies[i].bot.slots.len() < 2;
    let mut patrol = g
        .crowd
        .brains
        .get_mut(&id)
        .map_or(0, |b| b.patrol_corner_or_zero());
    let goal = heist::bot_goal(&me, hands_free, teams, &people, &strays, &mut patrol);
    if let Some(b) = g.crowd.brains.get_mut(&id) {
        b.set_patrol_corner(patrol);
    }
    let _ = me_pos;
    (goal, carrying_stolen)
}

// ------------------------------------------------------------------ the arena on screen

pub struct HeistViewPlugin;

impl Plugin for HeistViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (sync_heist_props, update_board));
    }
}

#[derive(Component)]
struct HeistProp;

fn sync_heist_props(
    mut commands: Commands,
    yard: Res<YardRes>,
    mut cache: ResMut<ModelCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    old: Query<Entity, With<HeistProp>>,
    mut shown: Local<Option<Option<usize>>>,
) {
    let want = yard.0.heist;
    if *shown == Some(want) {
        return;
    }
    *shown = Some(want);
    for e in &old {
        commands.entity(e).despawn();
    }
    let Some(teams) = want else {
        return;
    };
    let parts = bbq_core::looks_yard::heist_look(teams);
    let root = cache.spawn_parts(
        &mut commands,
        &parts,
        &mut meshes,
        &mut mats,
        Transform::default(),
    );
    commands.entity(root).insert(HeistProp);
    // the big four-sided scoreboard floating over the middle: a dark cube with the picture of
    // the scores on each side (Text2d is not drawn by the 3D camera, so it is a small texture)
    let image = images.add(blank_board());
    commands.insert_resource(BoardImage(image.clone()));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(7.0, 7.0, 7.0))),
        MeshMaterial3d(mats.add(StandardMaterial {
            base_color: hex(0x14202b),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 11.0, 0.0),
        HeistProp,
    ));
    let face = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(image),
        unlit: true,
        ..default()
    });
    let quad = meshes.add(Rectangle::new(7.0, 7.0));
    for k in 0..4 {
        let yaw = k as f32 * std::f32::consts::FRAC_PI_2;
        let n = Vec3::new(yaw.sin(), 0.0, yaw.cos());
        commands.spawn((
            Mesh3d(quad.clone()),
            MeshMaterial3d(face.clone()),
            Transform::from_translation(n * 3.52 + Vec3::Y * 11.0)
                .with_rotation(Quat::from_rotation_y(yaw)),
            HeistProp,
        ));
    }
    // and the pole it floats on
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.3, 7.5))),
        MeshMaterial3d(mats.add(StandardMaterial {
            base_color: hex(0x9aa0a8),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 3.75, 0.0),
        HeistProp,
    ));
}

#[derive(Resource)]
struct BoardImage(Handle<Image>);

const BOARD_PX: usize = 128;

fn blank_board() -> Image {
    let mut img = Image::new(
        Extent3d {
            width: BOARD_PX as u32,
            height: BOARD_PX as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![0; BOARD_PX * BOARD_PX * 4],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    img.sampler = ImageSampler::nearest();
    img
}

/// 3 by 5 pixel digits.
const DIGITS: [[u8; 5]; 10] = [
    [0b111, 0b101, 0b101, 0b101, 0b111],
    [0b010, 0b110, 0b010, 0b010, 0b111],
    [0b111, 0b001, 0b111, 0b100, 0b111],
    [0b111, 0b001, 0b111, 0b001, 0b111],
    [0b101, 0b101, 0b111, 0b001, 0b001],
    [0b111, 0b100, 0b111, 0b001, 0b111],
    [0b111, 0b100, 0b111, 0b101, 0b111],
    [0b111, 0b001, 0b001, 0b001, 0b001],
    [0b111, 0b101, 0b111, 0b101, 0b111],
    [0b111, 0b101, 0b111, 0b001, 0b111],
];

fn put_px(d: &mut [u8], x: usize, y: usize, c: [u8; 3]) {
    if x < BOARD_PX && y < BOARD_PX {
        let i = (y * BOARD_PX + x) * 4;
        d[i..i + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
    }
}

fn rect(d: &mut [u8], x: usize, y: usize, w: usize, h: usize, c: [u8; 3]) {
    for yy in y..y + h {
        for xx in x..x + w {
            put_px(d, xx, yy, c);
        }
    }
}

/// Draw a number with its left edge at `x` (or its right edge, when `right`).
fn number(d: &mut [u8], n: i32, x: usize, y: usize, scale: usize, right: bool) {
    let text = n.max(0).to_string();
    let w = text.len() * 4 * scale - scale;
    let mut x0 = if right { x.saturating_sub(w) } else { x };
    for ch in text.bytes() {
        let g = DIGITS[(ch - b'0') as usize];
        for (row, bits) in g.iter().enumerate() {
            for col in 0..3 {
                if bits & (0b100 >> col) != 0 {
                    rect(
                        d,
                        x0 + col * scale,
                        y + row * scale,
                        scale,
                        scale,
                        [255, 255, 255],
                    );
                }
            }
        }
        x0 += 4 * scale;
    }
}

fn paint_board(h: &HeistState) -> Vec<u8> {
    let mut d = vec![0u8; BOARD_PX * BOARD_PX * 4];
    rect(&mut d, 0, 0, BOARD_PX, BOARD_PX, [0x14, 0x20, 0x2b]);
    rect(&mut d, 0, 0, BOARD_PX, 3, [0xff, 0xd2, 0x3f]);
    rect(&mut d, 0, BOARD_PX - 3, BOARD_PX, 3, [0xff, 0xd2, 0x3f]);
    let keys = heist::team_keys(h.teams);
    let best = keys.iter().map(|t| h.bank.get(*t)).max().unwrap_or(0);
    let leaders = keys.iter().filter(|t| h.bank.get(**t) == best).count();
    let row_h = (112 / keys.len().max(1)).min(40);
    for (i, t) in keys.iter().enumerate() {
        let y = 8 + i * row_h;
        let c = bbq_core::looks_yard::team_colour(*t);
        let rgb = [(c >> 16) as u8, (c >> 8) as u8, c as u8];
        rect(&mut d, 4, y, BOARD_PX - 8, row_h - 3, rgb);
        let n = h.bank.get(*t) as i32;
        let scale = if row_h >= 30 { 4 } else { 3 };
        number(&mut d, n, 10, y + 3, scale, false);
        number(
            &mut d,
            h.bank.points(*t),
            BOARD_PX - 10,
            y + row_h - 3 - 7 - 2,
            1,
            true,
        );
        // a gold bar under the leader
        if best > 0 && leaders == 1 && n as u32 == best {
            rect(
                &mut d,
                4,
                y + row_h - 5,
                BOARD_PX - 8,
                2,
                [0xff, 0xd2, 0x3f],
            );
        }
    }
    d
}

fn update_board(
    game: Res<Game>,
    board: Option<Res<BoardImage>>,
    mut images: ResMut<Assets<Image>>,
    mut last: Local<Vec<u32>>,
) {
    let (Some(h), Some(board)) = (&game.heist, board) else {
        return;
    };
    let sig: Vec<u32> = heist::team_keys(h.teams)
        .iter()
        .map(|t| h.bank.get(*t))
        .collect();
    if *last == sig
        && images
            .get(&board.0)
            .is_some_and(|i| i.data.as_ref().is_some_and(|d| d[3] != 0))
    {
        return;
    }
    *last = sig;
    if let Some(mut img) = images.get_mut(&board.0) {
        img.data = Some(paint_board(h));
    }
}

#[allow(dead_code)]
fn unused(_: GameMode, _: f32, _: f32) {
    let _ = (YARD_HALF_X, YARD_HALF_Z);
}
