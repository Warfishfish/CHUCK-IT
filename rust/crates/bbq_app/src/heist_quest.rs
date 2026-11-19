//! Host-side Dazza drink deliveries in the Heist arena.
use crate::{
    game::Game,
    player::{PLAYER_ID, Player},
};
use bbq_core::{
    PlayerId,
    bots::navigation_waypoint,
    flight::ItemId,
    items::{Bottle, ItemKind},
    passage::HATCH,
    teams::Team,
    vec::V3,
    yard::Yard,
};

const WARMUP: f32 = 30.0;
const TIMEOUT: f32 = 30.0;
const OPEN_TIME: f32 = 12.0;
const COOLDOWN: f32 = 45.0;
const REACH: f32 = 2.8;
const PATROL: [V3; 4] = [
    V3::new(-3.65, 0.0, 3.65),
    V3::new(10.95, 0.0, -3.65),
    V3::new(-10.95, 0.0, -3.65),
    HATCH,
];

#[derive(Default)]
pub struct Quest {
    age: f32,
    until: f32,
    ready: f32,
    used: Vec<Team>,
    request: Option<(PlayerId, Team)>,
    escort: bool,
    deadline: f32,
    best: f32,
    patrol: usize,
    waypoint: Option<V3>,
    replan: f32,
    pub tipsy: f32,
    chatter: f32,
    line: usize,
    drinks: Vec<(ItemId, &'static str)>,
}

pub fn start(g: &mut Game) {
    g.life.quest = Quest::default();
    g.life.dazza.pos = PATROL[0];
    for (i, name) in ["water", "soda", "cider", "beer"].into_iter().enumerate() {
        let p = PATROL[i];
        let id = g
            .world
            .spawn(ItemKind::Stubby, p.x + 0.8, p.z, true, &mut g.rng);
        let alcohol = g.options.adult && i >= 2;
        if let Some(item) = g.world.items.get_mut(&id) {
            item.bottle = Some(if alcohol {
                Bottle::Beer
            } else {
                Bottle::Lemonade
            });
        }
        g.life
            .quest
            .drinks
            .push((id, if i >= 2 && !alcohol { "soda" } else { name }));
    }
}

fn eligible(g: &Game, team: Team) -> bool {
    let q = &g.life.quest;
    q.age >= WARMUP && q.age >= q.ready && !q.used.contains(&team)
}

fn nearby(g: &Game, pos: V3) -> bool {
    g.heist.is_some() && g.rules.in_play() && pos.y >= -0.05
        && g.life.dazza.pos.horiz_dist(pos) < REACH
}

pub fn prompt(g: &Game, p: &Player) -> Option<String> {
    if !nearby(g, V3::new(p.mover.x, p.mover.y, p.mover.z)) {
        return None;
    }
    let q = &g.life.quest;
    let team = g.teams.get(PLAYER_ID)?;
    let held = g
        .slots
        .selected()
        .and_then(|id| g.world.items.get(&id))
        .is_some_and(|i| i.kind == ItemKind::Stubby);
    Some(
        if held && q.request == Some((PLAYER_ID, team)) && !q.escort {
            "R: deliver Dazza’s cold drink"
        } else if held {
            "R: shout Dazza a drink"
        } else if q.request.is_some_and(|(owner, _)| owner != PLAYER_ID) {
            "Dazza is helping another player"
        } else if q.escort {
            "Follow Dazza to the hatch"
        } else if q.used.contains(&team) {
            "Your team has used its hatch opening"
        } else if !g.mirror && (q.age < WARMUP || q.age < q.ready) {
            "Dazza’s on smoko"
        } else {
            "Dazza wants a cold drink from the centre lanes"
        }
        .into(),
    )
}

pub fn interact(g: &mut Game, p: &Player) -> bool {
    let Some(id) = g.slots.selected() else { return false; };
    hand_over(g, PLAYER_ID, V3::new(p.mover.x, p.mover.y, p.mover.z), id)
}

pub fn hand_over(g: &mut Game, who: PlayerId, pos: V3, id: ItemId) -> bool {
    if !nearby(g, pos) { return false; }
    let Some(team) = g.teams.get(who) else { return false; };
    let Some(item) = g
        .world
        .items
        .get(&id)
        .filter(|i| i.kind == ItemKind::Stubby)
    else {
        return false;
    };
    let alcoholic = g.options.adult && item.bottle == Some(Bottle::Beer);
    let q = &mut g.life.quest;
    let name = q
        .drinks
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, name)| *name)
        .unwrap_or(if alcoholic { "beer" } else { "soft drink" });
    let delivery = q.request == Some((who, team)) && !q.escort;
    q.tipsy = (q.tipsy + if alcoholic { 1.0 } else { -1.0 }).clamp(0.0, 4.0);
    let say = if delivery {
        q.escort = true;
        q.deadline = q.age + TIMEOUT;
        q.best = f32::INFINITY;
        q.replan = 0.0;
        "Cold as! Come with me, I know a way underneath."
    } else if q.escort || q.age < q.chatter {
        ""
    } else if !alcoholic {
        "Ah. Infrastructure for tomorrow morning."
    } else if q.tipsy > 1.8 {
        "Left at the esky. Hang on. That’s where I left the esky."
    } else {
        "Cheers, mate. That one barely touched the sides."
    };
    q.chatter = q.age + 9.0;
    if who == PLAYER_ID {
        g.slots.remove(id);
    } else if let Some(guest) = g.dummies.iter_mut().find(|d| d.id == who) {
        guest.bot.slots.remove(id);
    }
    g.world.remove(id);
    if !say.is_empty() {
        g.life.dazza_says(say);
    }
    g.popup(format!("Gave Dazza a {name}"), false);
    true
}

pub fn tick(g: &mut Game, p: &Player, yard: &mut Yard, dt: f32) {
    if yard.heist.is_none() || !g.rules.in_play() {
        yard.hatch_open = false;
        return;
    }
    let pos = g.life.dazza.pos;
    let local_pos = V3::new(p.mover.x, p.mover.y, p.mover.z);
    let humans = std::iter::once((PLAYER_ID, local_pos)).chain(
        g.dummies.iter().filter(|d| d.remote.is_some())
            .map(|d| (d.id, V3::new(d.mover.x, d.mover.y, d.mover.z))),
    );
    let mut others = humans.clone();
    let candidate = others.find_map(|(id, at)| {
        let team = g.teams.get(id)?;
        (eligible(g, team) && nearby(g, at)).then_some((id, team))
    });
    let escort_pos = g.life.quest.request.and_then(|(id, _)| {
        humans.clone().find_map(|(who, at)| (who == id).then_some(at))
    });
    let escort_distance = escort_pos.map_or(f32::INFINITY, |at| (pos - at).len());
    let q = &mut g.life.quest;
    q.age += dt;
    q.tipsy = (q.tipsy - dt / 90.0).max(0.0);
    yard.hatch_open = q.age < q.until;
    if q.request.is_some() && (q.age >= q.deadline || escort_pos.is_none()) {
        q.request = None;
        q.escort = false;
        q.replan = 0.0;
    }
    let mut say = None;
    if q.request.is_none() && candidate.is_some() {
        q.request = candidate;
        q.deadline = q.age + TIMEOUT;
        say = Some("Got a cold one, mate? Try the lanes by the middle scoreboard.");
    }
    let target = if q.escort {
        HATCH
    } else {
        PATROL[q.patrol % 4]
    };
    let distance = pos.horiz_dist(target);
    if q.escort && escort_distance < 9.0 && distance + 0.5 < q.best {
        q.best = distance;
        q.deadline = q.age + TIMEOUT;
    }
    if distance < 0.8 {
        if q.escort && escort_distance < 6.0 {
            if let Some((_, team)) = q.request.take() {
                q.used.push(team);
            }
            q.escort = false;
            q.until = q.age + OPEN_TIME;
            q.ready = q.age + COOLDOWN;
            yard.hatch_open = true;
            say = Some("Hatch is up! Pick your colour down there.");
        }
        q.patrol += 1;
        q.replan = 0.0;
    } else if (q.escort && escort_distance < 9.0) || (!q.escort && escort_distance > 2.0) {
        q.replan -= dt;
        if q.replan <= 0.0 || q.waypoint.is_none_or(|w| w.horiz_dist(pos) < 0.3) {
            q.waypoint = navigation_waypoint(yard, pos, target);
            q.replan = 1.2;
        }
        if let Some(w) = q.waypoint {
            let delta = w - pos;
            let step = (2.3 - q.tipsy * 0.09) * dt;
            let next = pos + delta.normalised() * step.min(delta.horiz_len());
            if yard.heist_clear(next.x, next.z, 0.44) {
                g.life.dazza.pos = next;
                g.life.dazza.face = delta.x.atan2(delta.z);
            } else {
                q.replan = 0.0;
            }
        }
    }
    if say.is_none() && !q.escort && q.tipsy > 0.65 && q.age >= q.chatter && pos.horiz_dist(local_pos) < 10.0 {
        let lines = if q.tipsy > 1.8 {
            [
                "Follow me. If I follow you too, we’ll be here all arvo.",
                "That corner was here a minute ago.",
            ]
        } else {
            [
                "Who put two bends in this straight?",
                "Reckon the esky knows the way home.",
            ]
        };
        say = Some(lines[q.line % 2]);
        q.line += 1;
    }
    if let Some(line) = say {
        q.chatter = q.age + 12.0;
        g.life.dazza_says(line);
    }
}

pub fn snapshot(
    g: &Game,
    heist: &crate::heist_app::HeistState,
    hatch_open: bool,
    net: impl Fn(PlayerId) -> u32,
) -> bbq_core::net_world::HeistSnap {
    let quest = &g.life.quest;
    bbq_core::net_world::HeistSnap {
        banks: bbq_core::heist::TEAM_KEYS.map(|team| heist.bank.get(team)),
        hatch_open,
        tipsy: quest.tipsy,
        requesting: quest.request.map_or(0, |(owner, _)| net(owner)),
        escort: quest.escort,
    }
}

pub fn apply_snapshot(
    g: &mut Game,
    snap: Option<&bbq_core::net_world::HeistSnap>,
    local: impl Fn(u32) -> PlayerId,
) {
    let quest = &mut g.life.quest;
    quest.request = snap.filter(|s| s.requesting != 0).and_then(|s| {
        let owner = local(s.requesting);
        g.teams.get(owner).map(|team| (owner, team))
    });
    quest.tipsy = snap.map_or(0.0, |s| s.tipsy);
    quest.escort = snap.is_some_and(|s| s.escort);
}

#[derive(bevy::prelude::Component)]
pub struct CookHint;
pub fn draw_bubble(
    mut commands: bevy::prelude::Commands,
    game: bevy::prelude::Res<Game>,
    player: bevy::prelude::Res<Player>,
    yard: bevy::prelude::Res<crate::yard_scene::YardRes>,
    camera: bevy::prelude::Query<
        (&bevy::prelude::Camera, &bevy::prelude::GlobalTransform),
        bevy::prelude::With<crate::player::EyeCamera>,
    >,
    mut label: bevy::prelude::Query<
        (&mut bevy::prelude::Text, &mut bevy::prelude::Node),
        bevy::prelude::With<CookHint>,
    >,
    mut last: bevy::prelude::Local<(u32, f32)>,
) {
    use bevy::prelude::*;
    if label.is_empty() {
        commands.spawn((
            CookHint,
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(17.),
                ..default()
            },
            TextColor(Color::BLACK),
            BackgroundColor(Color::srgb(1., 0.96, 0.84)),
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                max_width: px(260),
                padding: UiRect::all(px(6)),
                ..default()
            },
        ));
        return;
    }
    let Ok((mut text, mut node)) = label.single_mut() else {
        return;
    };
    node.display = Display::None;
    if game.heist.is_none() || !game.rules.in_play() || player.mover.underground {
        return;
    }
    if last.0 != game.life.say_seq {
        last.0 = game.life.say_seq;
        last.1 = game.now + 4.0;
    }
    let pos = game.life.dazza.pos;
    let eye = V3::new(player.mover.x, player.mover.y, player.mover.z);
    if game.now > last.1
        || eye.horiz_dist(pos) > 12.
        || !bbq_core::bots::line_of_sight(&yard.0, eye, pos)
    {
        return;
    }
    let Ok((cam, tf)) = camera.single() else {
        return;
    };
    if let Ok(at) = cam.world_to_viewport(tf, Vec3::new(pos.x, 2.6, pos.z)) {
        node.display = Display::Flex;
        node.left = px((at.x - 105.).max(3.));
        node.top = px((at.y - 42.).max(3.));
        text.0 = game.life.say_text.clone();
    }
}
