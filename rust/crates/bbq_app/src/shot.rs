//! Screenshot mode, for comparing the Rust game with the browser game.
//!
//!     cargo run -p bbq_app -- --shot out.png --gallery
//!     cargo run -p bbq_app -- --shot out.png --cam 0,1.55,10,0,0,85
//!
//! The window is made 800 x 600 at normal pixel size, the camera is put exactly where asked,
//! a picture is saved and the game quits. `--gallery` also lines up every item against the sky,
//! the same way the browser comparison page does (see `rust/tools/compare.md`). `--calib` shows
//! one big white sphere instead, to measure how the lights compare.

use bbq_core::items::{DildoVariant, ItemKind};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use crate::game::Game;
use crate::models::{ModelCache, ModelKey};
use crate::player::{EyeCamera, HudText, update_camera};

/// What the command line asked for.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct ShotConfig {
    pub out: String,
    pub gallery: bool,
    /// One grey ball against the sky, for measuring the lighting.
    pub calib: bool,
    /// Only the yard: no people, items or Dazza.
    pub bare: bool,
    /// Switch the sun's shadows off (to see how much they change things).
    pub no_shadows: bool,
    /// Keep only one of the lights: "ambient", "up" (the sky light from above) or "sun".
    pub only: Option<String>,
    /// x, y, z, yaw degrees, pitch degrees, field of view degrees.
    pub cam: Option<[f32; 6]>,
}

/// The gallery camera: level, 3 m from the line-up, high above the yard so only sky is behind.
pub const GALLERY_CAM: [f32; 6] = [0.0, 30.0, 2.0, 0.0, 0.0, 45.0];

/// The white ball used to measure the lighting.
pub const CALIB_AT: Vec3 = Vec3::new(0.0, 30.0, -1.0);
pub const CALIB_RADIUS: f32 = 0.8;
/// A mid grey, so that nothing is washed out to white while measuring.
pub const CALIB_COLOUR: u32 = 0x666666;

/// Read `--shot`, `--gallery` and `--cam` from the arguments.
pub fn parse(args: &[String]) -> Option<ShotConfig> {
    let at = args.iter().position(|a| a == "--shot")?;
    let out = args.get(at + 1)?.clone();
    let gallery = args.iter().any(|a| a == "--gallery");
    let calib = args.iter().any(|a| a == "--calib");
    let bare = args.iter().any(|a| a == "--bare");
    let no_shadows = args.iter().any(|a| a == "--no-shadows");
    let cam = args
        .iter()
        .position(|a| a == "--cam")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| {
            let v: Vec<f32> = s.split(',').filter_map(|p| p.trim().parse().ok()).collect();
            <[f32; 6]>::try_from(v).ok()
        });
    let only = args
        .iter()
        .position(|a| a == "--only")
        .and_then(|i| args.get(i + 1))
        .cloned();
    Some(ShotConfig {
        only,
        out,
        gallery,
        calib,
        bare,
        no_shadows,
        cam: cam.or((gallery || calib).then_some(GALLERY_CAM)),
    })
}

/// Where the line-up of items stands (the same numbers the browser page uses).
pub fn gallery_places() -> Vec<(ModelKey, Vec3)> {
    let mut v = Vec::new();
    for (i, k) in [
        ItemKind::Teddy,
        ItemKind::Stubby,
        ItemKind::Gnome,
        ItemKind::Steak,
        ItemKind::Fish,
        ItemKind::Noodle,
        ItemKind::Dildo,
    ]
    .into_iter()
    .enumerate()
    {
        v.push((
            ModelKey::item(k, None, 0),
            Vec3::new((i as f32 - 3.0) * 0.55, 30.35, -1.0),
        ));
    }
    for (i, d) in [DildoVariant::Mini, DildoVariant::Jumbo, DildoVariant::Gold]
        .into_iter()
        .enumerate()
    {
        v.push((
            ModelKey::item(ItemKind::Dildo, Some(d), 0),
            Vec3::new((i as f32 - 1.0) * 0.6, 29.7, -1.0),
        ));
    }
    v
}

pub struct ShotPlugin(pub ShotConfig);

impl Plugin for ShotPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.0.clone())
            .add_systems(PreStartup, bare_start)
            .add_systems(Startup, spawn_gallery)
            .add_systems(
                Update,
                (
                    shot_camera.after(update_camera),
                    hide_hud,
                    take_shot,
                    only_light,
                    keep_bare,
                ),
            );
    }
}

fn spawn_gallery(
    mut commands: Commands,
    cfg: Res<ShotConfig>,
    mut cache: ResMut<ModelCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    if cfg.calib {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(CALIB_RADIUS).mesh().uv(64, 48))),
            MeshMaterial3d(mats.add(crate::models::material(
                &bbq_core::looks::Surface::matt(CALIB_COLOUR),
                &cache,
            ))),
            Transform::from_translation(CALIB_AT),
        ));
    }
    if !cfg.gallery {
        return;
    }
    for (key, at) in gallery_places() {
        cache.spawn(
            &mut commands,
            key,
            &mut meshes,
            &mut mats,
            Transform::from_translation(at),
        );
    }
}

fn shot_camera(
    cfg: Res<ShotConfig>,
    game: Res<Game>,
    mut cam: Single<(&mut Transform, &mut Projection), With<EyeCamera>>,
) {
    // `--bot N`: a close-up of bot N from the front (3 m away, `--bot-side` for the profile)
    let args: Vec<String> = std::env::args().collect();
    let bot = args
        .iter()
        .position(|a| a == "--bot")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse::<usize>().ok());
    if let Some(d) = bot.and_then(|n| game.dummies.get(n)) {
        let side = if args.iter().any(|a| a == "--bot-profile") {
            std::f32::consts::FRAC_PI_2
        } else if args.iter().any(|a| a == "--bot-side") {
            1.1
        } else {
            0.0
        };
        let head = args.iter().any(|a| a == "--bot-head");
        let a = d.face + side;
        let (dx, dz) = (a.sin(), a.cos());
        let (dist, y, fov) = if head { (4.2, 1.45 + d.mover.y, 22.0f32) } else { (3.0, 1.05, 42.0) };
        let (tf, proj) = &mut *cam;
        tf.translation = Vec3::new(d.mover.x + dx * dist, y, d.mover.z + dz * dist);
        tf.rotation = Quat::from_euler(EulerRot::YXZ, a, if head { -0.06 } else { -0.05 }, 0.0);
        if let Projection::Perspective(p) = &mut **proj {
            p.fov = fov.to_radians();
        }
        return;
    }
    let Some([mut x, y, mut z, yaw, pitch, fov]) = cfg.cam else {
        return;
    };
    // `--cam-drift`: the camera creeps sideways, like someone walking past (to catch flicker)
    if args.iter().any(|a| a == "--cam-drift") {
        x += game.now * 0.6;
        z += game.now * 0.2;
    }
    let (tf, proj) = &mut *cam;
    tf.translation = Vec3::new(x, y, z);
    tf.rotation = Quat::from_euler(EulerRot::YXZ, yaw.to_radians(), pitch.to_radians(), 0.0);
    if let Projection::Perspective(p) = &mut **proj {
        p.fov = fov.to_radians();
    }
}

/// `--bare`: start with nobody in the yard.
fn bare_start(cfg: Res<ShotConfig>, mut game: ResMut<Game>) {
    if cfg.bare || cfg.gallery || cfg.calib {
        game.dummies.clear();
        game.world.items.clear();
    }
}

/// `--bare`: keep the yard empty (items spawn on their own) and send Dazza away, knocked out.
fn keep_bare(cfg: Res<ShotConfig>, mut game: ResMut<Game>) {
    if cfg.bare || cfg.gallery || cfg.calib {
        game.world.items.clear();
        game.life.dazza.state = bbq_core::dazza::DazzaState::Ko;
        game.life.dazza.pos = bbq_core::vec::V3::new(1000.0, 0.0, 1000.0);
    }
}

/// `--only`: switch off every light but one, to measure each on its own.
fn only_light(
    cfg: Res<ShotConfig>,
    mut ambient: Query<&mut AmbientLight>,
    mut dirs: Query<&mut DirectionalLight>,
) {
    if cfg.no_shadows {
        for mut d in &mut dirs {
            d.shadow_maps_enabled = false;
        }
    }
    let Some(only) = cfg.only.as_deref() else {
        return;
    };
    for mut a in &mut ambient {
        if only != "ambient" {
            a.brightness = 0.0;
        }
    }
    for mut d in &mut dirs {
        let is_sun = d.shadow_maps_enabled;
        if (is_sun && only != "sun") || (!is_sun && only != "up") {
            d.illuminance = 0.0;
        }
    }
}

fn hide_hud(mut q: Query<&mut Visibility, With<HudText>>) {
    for mut v in &mut q {
        *v = Visibility::Hidden;
    }
}

fn take_shot(
    mut commands: Commands,
    cfg: Res<ShotConfig>,
    mut frame: Local<u32>,
    mut exit: MessageWriter<AppExit>,
    meshes: Res<Assets<Mesh>>,
    mats: Res<Assets<StandardMaterial>>,
    drawn: Query<(), With<Mesh3d>>,
) {
    *frame += 1;
    // `--stats`: how many meshes, materials and drawn parts there are (for measuring cuts)
    if *frame == 100 && std::env::args().any(|a| a == "--stats") {
        println!(
            "STATS meshes={} materials={} mesh_entities={}",
            meshes.iter().count(),
            mats.iter().count(),
            drawn.iter().count()
        );
    }
    // `--burst N`: N pictures 4 frames apart once the round is under way (for things that move),
    // saved as out_0.png, out_1.png...
    let args: Vec<String> = std::env::args().collect();
    let burst = args
        .iter()
        .position(|a| a == "--burst")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse::<u32>().ok());
    let every = args
        .iter()
        .position(|a| a == "--burst-step")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(4)
        .max(1);
    if let Some(n) = burst {
        let start = 320;
        if *frame >= start && (*frame - start) % every == 0 && (*frame - start) / every < n {
            let k = (*frame - start) / every;
            let path = cfg.out.replace(".png", &format!("_{k}.png"));
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        }
        if *frame == start + n * every + 60 {
            exit.write(AppExit::Success);
        }
        return;
    }
    // give the textures a moment to load, then take the picture and leave
    if *frame == 60 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(cfg.out.clone()));
    }
    if *frame == 150 {
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn no_shot_flag_means_a_normal_game() {
        assert_eq!(parse(&args("bbq_app")), None);
        assert_eq!(parse(&args("bbq_app --gallery")), None);
    }

    #[test]
    fn gallery_uses_its_own_camera() {
        let c = parse(&args("bbq_app --shot a.png --gallery")).unwrap();
        assert_eq!(c.out, "a.png");
        assert!(c.gallery && !c.calib && !c.bare);
        assert_eq!(c.cam, Some(GALLERY_CAM));
        let k = parse(&args("bbq_app --shot a.png --calib")).unwrap();
        assert!(k.calib && !k.gallery);
        assert_eq!(k.cam, Some(GALLERY_CAM));
    }

    #[test]
    fn camera_flag_reads_six_numbers() {
        let c = parse(&args("bbq_app --shot a.png --cam 1,2,3,4,5,60")).unwrap();
        assert!(!c.gallery);
        assert_eq!(c.cam, Some([1.0, 2.0, 3.0, 4.0, 5.0, 60.0]));
        // a short list is ignored rather than guessed at
        assert_eq!(
            parse(&args("bbq_app --shot a.png --cam 1,2,3"))
                .unwrap()
                .cam,
            None
        );
        assert!(parse(&args("bbq_app --shot a.png --bare")).unwrap().bare);
    }

    #[test]
    fn the_line_up_has_ten_things_in_a_row() {
        let g = gallery_places();
        assert_eq!(g.len(), 10);
        assert!(g.iter().all(|(_, p)| p.z == -1.0 && p.y > 29.0));
    }
}
