//! Opt-in `--shot path.png --review`: repeatable visual comparison and wall-clock timing.
//! Simulation advances at 60 Hz; measurement uses Instant, never the artificial game clock.
use std::time::{Duration, Instant};

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::{PresentMode, PrimaryWindow};

pub struct VisualReviewPlugin;

impl Plugin for VisualReviewPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f64(1.0 / 60.0),
        ))
        .init_resource::<ReviewTiming>()
        .add_systems(Startup, configure_window)
        .add_systems(PreUpdate, motion_fixture)
        .add_systems(Update, held_fixture.before(crate::items_view::HeldViewSet).before(crate::player::update_camera))
        .add_systems(Last, capture);
    }
}

fn configure_window(mut window: Single<&mut Window, With<PrimaryWindow>>) {
    window.present_mode = PresentMode::AutoNoVsync;
}

#[derive(Resource, Default)]
struct ReviewTiming {
    frame: u32,
    previous: Option<Instant>,
    milliseconds: Vec<f64>,
}

fn capture(
    mut commands: Commands,
    cfg: Res<crate::shot::ShotConfig>,
    mut timing: ResMut<ReviewTiming>,
    mut exit: MessageWriter<AppExit>,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    images: Res<Assets<Image>>,
    atlases: Res<bevy::text::FontAtlasSet>,
    drawn: Query<(), With<Mesh3d>>,
    game: Res<crate::game::Game>,
) {
    timing.frame += 1;
    if motion_mode().is_some() {
        let water = motion_mode()==Some("water-balloon");
        let frame = timing.frame.saturating_sub(if water {720} else {0});
        if (180..if water {600} else {480}).contains(&frame) && (frame - 180) % 4 == 0 {
            let index = (frame - 180) / 4;
            let path = cfg.out.replace(".png", &format!("-{index:03}.png"));
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
            for id in [900001, 900002, 900003] {
                if let Some(it) = game.world.items.get(&id) {
                    println!("MOTION frame={} id={} state={:?} pos={:.4},{:.4},{:.4} vel={:.4},{:.4},{:.4}", frame, id, it.state, it.pos.x,it.pos.y,it.pos.z,it.vel.x,it.vel.y,it.vel.z);
                }
            }
        }
        if frame == 240 { commands.spawn(Screenshot::primary_window()).observe(save_to_disk(cfg.out.clone())); }
        if water && frame > 0 && frame % 4 == 0 && let Some(d) = game.dummies.first() {
            println!("WATER_REVIEW frame={} wet={:.3} speed={:.3} x={:.3} z={:.3}", frame, d.body.wet_t, d.mover.speed(), d.mover.x, d.mover.z);
        }
        if frame == if water {660} else {540} {
            if motion_mode()==Some("smoko") {
                println!("SMOKO_REVIEW final_score={} carries={}",game.board.score(crate::player::PLAYER_ID),game.smoko.carries.len());
            }
            exit.write(AppExit::Success);
        }
        return;
    }
    let args:Vec<String>=std::env::args().collect();
    let shot_frame=args.iter().position(|a|a=="--capture-frame").and_then(|i|args.get(i+1)).and_then(|v|v.parse::<u32>().ok()).unwrap_or(240);
    let now = Instant::now();
    let previous = timing.previous.replace(now);
    // Capture before timing, then allow readback and PNG writing to finish.
    if timing.frame == shot_frame {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(cfg.out.clone()));
    }
    if (shot_frame+61..=shot_frame+660).contains(&timing.frame)
        && let Some(previous) = previous
    {
        timing
            .milliseconds
            .push(now.duration_since(previous).as_secs_f64() * 1000.0);
    }
    if timing.frame == shot_frame+660 {
        let samples = &mut timing.milliseconds;
        samples.sort_by(f64::total_cmp);
        let mean = samples.iter().sum::<f64>() / samples.len() as f64;
        let p50 = samples[samples.len() / 2];
        let p95 = samples[(samples.len() as f64 * 0.95).ceil() as usize - 1];
        let over = samples.iter().filter(|t| **t > 1000.0 / 60.0).count();
        let image_bytes: usize = images
            .iter()
            .filter_map(|(_, i)| i.data.as_ref().map(Vec::len))
            .sum();
        let report = format!(
            "samples={} mean_ms={mean:.3} median_ms={p50:.3} p95_ms={p95:.3} mean_fps={:.2} frames_over_16_67ms={over}\nmeshes={} materials={} mesh_entities={} image_cpu_bytes={}\nFixed 60 Hz simulation; 300 warmup frames; 600 wall-clock intervals; 1280x800 with --big; AutoNoVsync; not GPU timestamp measurements.\n",
            samples.len(),
            1000.0 / mean,
            meshes.iter().count(),
            materials.iter().count(),
            drawn.iter().count(),
            image_bytes,
        );
        println!("REVIEW {report}");
        if std::env::args().any(|a|a=="--image-costs") {
            println!("FONT_ATLASES count={} bytes={}",atlases.len(),atlases.total_bytes(&images));
            for k in atlases.keys().take(25) {println!("FONT_KEY {k:?}");}
            let mut sizes=std::collections::BTreeMap::new();
            for (_,im) in images.iter(){let n=im.data.as_ref().map_or(0,Vec::len);*sizes.entry((n,format!("{:?} {:?}",im.texture_descriptor.size,im.texture_descriptor.label))).or_insert(0usize)+=1;}
            for ((bytes,desc),count) in sizes {println!("IMAGE_COST count={count} bytes_each={bytes} {desc}");}
        }
        let path = std::path::Path::new(&cfg.out).with_extension("metrics.txt");
        if let Err(error) = std::fs::write(path, report) {
            error!("Could not save review metrics: {error}");
        }
        exit.write(AppExit::Success);
    }
}

/// Staged expression captures only: does not alter gameplay state or saved customization.
pub fn expression() -> Option<crate::face::Mood> {
    static MOOD: std::sync::OnceLock<Option<crate::face::Mood>> = std::sync::OnceLock::new();
    *MOOD.get_or_init(|| {
        let args: Vec<String> = std::env::args().collect();
        if !args.iter().any(|a| a == "--shot") || !args.iter().any(|a| a == "--review") { return None; }
        let value = args.iter().position(|a| a == "--expression").and_then(|i| args.get(i + 1))?;
        use crate::face::Mood;
        match value.as_str() {
            "normal" => Some(Mood::Normal), "grit" => Some(Mood::Grit),
            "ouch" => Some(Mood::Ouch), "sip" => Some(Mood::Sip), "cheer" => Some(Mood::Cheer), _ => None,
        }
    })
}

/// Controlled fixtures are only available inside the opt-in review plugin.
fn motion_mode() -> Option<&'static str> {
    static MODE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    MODE.get_or_init(|| {
        let a: Vec<String> = std::env::args().collect();
        if !a.iter().any(|v|v=="--shot") || !a.iter().any(|v|v=="--review") { return None; }
        a.iter().position(|v| v == "--motion-review").and_then(|i| a.get(i+1)).filter(|v| ["drop", "throw", "held-noodle", "held-rubber", "smoko", "race", "water-balloon"].contains(&v.as_str())).cloned()
    }).as_deref()
}

pub fn is_water_review() -> bool { motion_mode() == Some("water-balloon") }

fn motion_fixture(mut game: ResMut<crate::game::Game>, mut player: ResMut<crate::player::Player>, mut wanted: ResMut<crate::player::Wanted>, mut frame: Local<u32>) {
    let Some(mode) = motion_mode() else { return };
    if mode=="race" {return;}
    if mode=="water-balloon" {
        use bbq_core::{movement::Mover, vec::V3, items::ItemKind};
        *frame+=1;
        let frame_no=frame.saturating_sub(720);
        game.options.bots_on=false;
        if frame_no==120 {
            game.options.bots_on=false;game.options.falls_on=false;
            game.world.clear();game.me.body=Default::default();game.me.drunk.reset();game.life.seated=None;game.rules.phase=bbq_core::scoring::Phase::Play;
            game.round.time_left=180.0;game.round.banner=None;
            game.dummies.truncate(1);game.board.clear();game.feed.clear();game.popups.clear();game.slots.clear();
            game.board.ensure(crate::player::PLAYER_ID);game.board.ensure(100);
            if let Some(d)=game.dummies.first_mut() {d.bot.slots.clear();d.crown=false;d.mover=Mover::new(-16.0,18.0);d.body=Default::default();d.seat=None;d.dragged=None;d.bot.catcher=Default::default();d.face=std::f32::consts::FRAC_PI_2;}
            player.mover=Mover::new(8.0,0.0);
        }
        if frame_no>=120 {
            if let Some(d)=game.dummies.first_mut() {
                d.bot.wish=if frame_no<560 {(1.0,0.0)}else{(0.0,0.0)};
                d.home=(d.mover.x,d.mover.z);
            }
            game.life.dazza.pos=V3::new(1000.0,0.0,1000.0);
        }
        if frame_no==200 {
            let d=&game.dummies[0];let (x,z)=(d.mover.x,d.mover.z);
            let mut rng=game.rng.clone();
            let id=game.world.spawn(ItemKind::WaterBalloon,x,z-4.0,false,&mut rng);
            game.world.give(id,crate::player::PLAYER_ID);
            game.world.throw(id,crate::player::PLAYER_ID,V3::new(x,1.2,z-4.0),V3::new(6.2,1.5,14.0),0.5,true);
        }
        return;
    }
    if mode=="smoko" {
        *frame+=1;
        if *frame==120 {
            game.options.bots_on=false;game.world.clear();game.rules.phase=bbq_core::scoring::Phase::Play;
            game.round.time_left=180.0;game.round.banner=None;
            game.smoko=Default::default();game.board.clear();
            game.board.ensure(crate::player::PLAYER_ID);game.board.ensure(100);
            player.mover=bbq_core::movement::Mover::new(1.5,3.5);
            player.yaw=std::f32::consts::PI;player.pitch=-0.12;
            game.dummies.truncate(1);
            if let Some(d)=game.dummies.first_mut() {
                d.mover=bbq_core::movement::Mover::new(1.5,5.0);d.home=(1.5,5.0);d.body.down_t=8.0;
            }
        }
        if *frame==180 || *frame==220 {wanted.grab_pressed=true;}
        if *frame==181 || *frame==244 {wanted.grab_released=true;}
        return;
    }
    if mode.starts_with("held-") {
        game.dummies.clear();
        game.world.items.retain(|id, _| *id == 900001);
        game.life.dazza.state = bbq_core::dazza::DazzaState::Ko;
        game.life.dazza.pos = bbq_core::vec::V3::new(1000.0,0.0,1000.0);
        return;
    }
    *frame += 1;
    let g = &mut *game;
    g.dummies.clear();
    g.world.items.retain(|id,_| (900001..=900003).contains(id));
    g.life.dazza.state = bbq_core::dazza::DazzaState::Ko;
    g.life.dazza.pos = bbq_core::vec::V3::new(1000.0,0.0,1000.0);
    if *frame == 120 {
        use bbq_core::{flight::Item, items::{ItemKind, DildoVariant}, vec::V3};
        for (n, kind, variant) in [(0,ItemKind::Dildo,Some(DildoVariant::Classic)),(1,ItemKind::Dildo,Some(DildoVariant::Jumbo)),(2,ItemKind::Noodle,None)] {
            let id=900001+n;
            let mut item=Item::new(id,kind,V3::new((n as f32-1.0)*2.8,3.8,8.0));
            item.variant=variant;
            g.world.items.insert(id,item);
        }
    }
    if *frame == 180 {
        for item in g.world.items.values_mut() {
            item.state=bbq_core::flight::ItemState::Flying;
            item.vel=if mode=="throw" { bbq_core::vec::V3::new(0.0,3.0,-6.0) } else { bbq_core::vec::V3::ZERO };
            item.live=mode=="throw";
            item.fly_t=0.0;
        }
    }
}

// Controlled first-person input playback. The camera remains in one location so
// shape and spring behaviour are easy to compare; speed/turn inputs are staged.
fn held_fixture(mut game: ResMut<crate::game::Game>, mut player: ResMut<crate::player::Player>, mut frame: Local<u32>) {
    let Some(mode) = motion_mode().filter(|m| m.starts_with("held-")) else { return; };
    *frame += 1;
    let t = (*frame as f32 - 180.0) / 60.0;
    use bbq_core::{flight::{Item,ItemState},items::ItemKind,vec::V3};
    if *frame == 120 {
        let kind = if mode == "held-noodle" { ItemKind::Noodle } else { ItemKind::Dildo };
        let mut item = Item::new(900001,kind,V3::ZERO);
        item.state = ItemState::Held;
        item.holder = Some(crate::player::PLAYER_ID);
        game.world.items.insert(item.id,item);
        game.slots.clear();
        game.slots.add(900001);
    }
    let speed = if t < 0.5 { 0.0 } else if t < 1.5 { 3.0 } else if t < 3.0 { 8.0 } else { 0.0 };
    player.mover.vx = 0.0;
    player.mover.vz = -speed;
    player.mover.grounded = true;
    player.mover.x = 0.0; player.mover.y = 0.0; player.mover.z = 14.0;
    player.prev = Vec3::new(0.0,0.0,14.0);
    player.walk += speed / 60.0 * 2.4;
    player.yaw = if (1.5..3.0).contains(&t) { ((t-1.5)*std::f32::consts::TAU/1.5).sin()*0.3 } else { 0.0 };
    player.pitch = -0.12;
}
