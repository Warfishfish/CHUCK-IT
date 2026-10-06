//! The game's style (GRAPHICS_2C.md, S1 and S2). Marcus chose "pop" with the warm golden light
//! that was tried in "clay"; the other two are kept as switches.
//!
//!  * `--style pop`: "cartoon pop". Thick black outlines on the characters and on the props in
//!    the BBQ corner, punchier colours and contrast, big-headed, big-handed, big-footed blobs.
//!  * `--style clay`: "dusty clay". No outlines, a muted dusty palette with a warm grade and
//!    flatter, softer light, and wide, squat, round toy-like blobs with big soft bellies.
//!
//! Nothing here is on by default: without `--style` the game looks exactly as before. All of it
//! is meant to be thrown away or kept whole, depending on which one Marcus picks.

use std::sync::atomic::{AtomicU8, Ordering};

use bbq_core::character::Character;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    Current,
    Pop,
    Clay,
}

static STYLE: AtomicU8 = AtomicU8::new(0);

/// Read `--style` and tell the core crate about the palette.
pub fn init(args: &[String]) {
    // Marcus chose "pop" with clay's warm light (6 Oct 2026): it is now what the polished look
    // is. `--style current` gets the old polished look back, `--style clay` the clay example,
    // and the browser look (`--look browser`) is never changed.
    let polished = crate::lighting::LookMode::from_args(args) == crate::lighting::LookMode::Polished;
    let s = match args
        .iter()
        .position(|a| a == "--style")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
    {
        Some("pop") => Style::Pop,
        Some("clay") => Style::Clay,
        Some("current") => Style::Current,
        _ if polished => Style::Pop,
        _ => Style::Current,
    };
    STYLE.store(s as u8, Ordering::Relaxed);
    // how far plain colours fade towards grey (negative = more vivid), and the lawn's colour
    let (fade, lawn) = match s {
        Style::Current => (0.14, 1.0),
        Style::Pop => (-0.12, 1.06),
        Style::Clay => (0.34, 0.8),
    };
    bbq_core::looks_yard::set_palette(fade, lawn);
}

pub fn current() -> Style {
    match STYLE.load(Ordering::Relaxed) {
        1 => Style::Pop,
        2 => Style::Clay,
        _ => Style::Current,
    }
}

/// The model file for a character in the current style.
pub fn model_path(c: Character) -> String {
    let suffix = match current() {
        Style::Pop => "_pop",
        Style::Clay => "_clay",
        Style::Current => "",
    };
    format!("models/blob_{}{suffix}.glb", c.name().to_lowercase())
}

/// The black material for outlines: unlit, and only the far side of the shell is drawn (the
/// "inverted hull" trick), so a slightly bigger copy shows as an edge round the real mesh.
pub fn outline_material(mats: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    mats.add(StandardMaterial {
        base_color: Color::linear_rgb(0.02, 0.025, 0.02),
        unlit: true,
        cull_mode: Some(bevy::render::render_resource::Face::Front),
        ..default()
    })
}

/// The thin outline on things you hold: a warm dark brown instead of black, so it reads as
/// shading rather than a drawn line.
pub fn soft_outline_material(mats: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    mats.add(StandardMaterial {
        base_color: Color::linear_rgb(0.075, 0.05, 0.035),
        unlit: true,
        cull_mode: Some(bevy::render::render_resource::Face::Front),
        ..default()
    })
}

/// A character's skin or body material. In the clay style it gets a soft mottled clay surface
/// (fine pores, thumb-print swirls and a bumpy normal map) and a matt finish; otherwise it is the
/// plain colour as before.
pub fn body_material(colour: Color, assets: &AssetServer) -> StandardMaterial {
    if current() != Style::Clay {
        return colour.into();
    }
    use bevy::image::{ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
    let load = |s: &mut ImageLoaderSettings| {
        s.is_srgb = false;
        s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            mipmap_filter: ImageFilterMode::Linear,
            anisotropy_clamp: 8,
            ..default()
        });
    };
    StandardMaterial {
        base_color: colour,
        base_color_texture: Some(assets.load_builder().with_settings(load).load("textures/clay.png")),
        normal_map_texture: Some(assets.load_builder().with_settings(load).load("textures/clay_n.png")),
        uv_transform: bevy::math::Affine2::from_scale(Vec2::new(3.0, 2.0)),
        perceptual_roughness: 0.92,
        reflectance: 0.15,
        ..default()
    }
}
