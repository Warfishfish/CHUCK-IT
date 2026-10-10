//! Dirty clothes (Marcus, 10 Oct 2026): the singlet and shorts carry grubby marks, as many as the
//! Dirty slider on the Customise tab says (a little by default).
//!
//! The marks are painted by the game into a small repeating picture, one per level of dirt:
//! soft muddy blotches, a few darker grease spots and a general dinginess, more and darker the
//! dirtier. The cloth's colour is multiplied by it (the weave still comes from the fabric normal
//! map).

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// How many steps the slider's 0..100 is cut into for the pictures (0 is spotless: no picture).
pub const LEVELS: u8 = 5;
/// The picture's size (it repeats round the cloth).
const SIZE: u32 = 256;

/// Which picture a dirt amount (0..100) uses: 0 for none, else 1..=LEVELS.
pub fn level_of(dirt: u8) -> u8 {
    if dirt < 5 { 0 } else { (((dirt as u32 - 5) * LEVELS as u32 / 96) as u8 + 1).min(LEVELS) }
}

/// The pictures, made when first needed.
#[derive(Resource, Default)]
pub struct DirtKit(Vec<Option<Handle<Image>>>);

impl DirtKit {
    pub fn picture(&mut self, level: u8, images: &mut Assets<Image>) -> Option<Handle<Image>> {
        if level == 0 {
            return None;
        }
        if self.0.len() <= level as usize {
            self.0.resize(level as usize + 1, None);
        }
        Some(self.0[level as usize].get_or_insert_with(|| images.add(dirt_image(level))).clone())
    }
}

/// A smooth random value at (x, y) that repeats every `period` cells (so the picture tiles).
fn noise(x: f32, y: f32, period: u32, seed: u32) -> f32 {
    let hash = |ix: i64, iy: i64| -> f32 {
        let (ix, iy) = (ix.rem_euclid(period as i64) as u32, iy.rem_euclid(period as i64) as u32);
        let mut h = ix.wrapping_mul(374_761_393) ^ iy.wrapping_mul(668_265_263) ^ seed.wrapping_mul(2_246_822_519);
        h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
        ((h ^ (h >> 16)) & 0xffff) as f32 / 65535.0
    };
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (ix, iy) = (x0 as i64, y0 as i64);
    let a = hash(ix, iy) + (hash(ix + 1, iy) - hash(ix, iy)) * s(fx);
    let b = hash(ix, iy + 1) + (hash(ix + 1, iy + 1) - hash(ix, iy + 1)) * s(fx);
    a + (b - a) * s(fy)
}

/// The dirt picture's pixels (RGBA, white where clean) for a level 1..=LEVELS.
pub fn dirt_pixels(level: u8) -> Vec<u8> {
    let k = level as f32 / LEVELS as f32; // 0.2 .. 1.0
    // round grease and mud splotches scattered about (more the dirtier), each a soft round blob
    let mut seed = 0x2545_F491u32;
    let mut rand = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed & 0xffff) as f32 / 65535.0
    };
    let spots: Vec<(f32, f32, f32, f32)> = (0..(4 + 9 * level as usize))
        .map(|_| (rand(), rand(), 0.008 + 0.022 * rand() * rand(), 0.5 + 0.5 * rand()))
        .collect();
    let mut px = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (u, v) = (x as f32 / SIZE as f32, y as f32 / SIZE as f32);
            // big soft blotches and smaller smudges, on top of each other
            let blotch = noise(u * 4.0, v * 4.0, 4, 11) * 0.6 + noise(u * 9.0, v * 9.0, 9, 23) * 0.3 + noise(u * 23.0, v * 23.0, 23, 37) * 0.1;
            // the nearest splotch (the picture wraps, so they wrap too), a little ragged at the edge
            let ragged = 1.0 + 0.35 * (noise(u * 40.0, v * 40.0, 40, 51) - 0.5);
            let mut grease = 0.0f32;
            for &(sx, sy, r, strength) in &spots {
                let wrap = |d: f32| { let d = d.abs(); d.min(1.0 - d) };
                let dist = wrap(u - sx).hypot(wrap(v - sy)) / (r * ragged);
                grease = grease.max(((1.0 - dist) / 0.35).clamp(0.0, 1.0) * strength);
            }
            // the dirtier, the more of the cloth the blotches cover
            let cover = |val: f32, from: f32, soft: f32| ((val - from) / soft).clamp(0.0, 1.0);
            let mud = cover(blotch, 0.78 - 0.32 * k, 0.12) * (0.25 + 0.45 * k);
            let spot = grease * (0.25 + 0.45 * k);
            let dingy = 0.06 * k + noise(u * 31.0, v * 31.0, 31, 71) * 0.05 * k;
            // mud is a dusty brown, grease a dark brown-grey
            let mut c = [1.0f32, 1.0, 1.0];
            for (i, m) in [0.62f32, 0.5, 0.36].iter().enumerate() {
                c[i] *= 1.0 - mud * (1.0 - m);
            }
            for (i, g) in [0.35f32, 0.31, 0.27].iter().enumerate() {
                c[i] *= 1.0 - spot * (1.0 - g);
            }
            for v in &mut c {
                *v *= 1.0 - dingy;
            }
            px.extend(c.iter().map(|v| (v.clamp(0.0, 1.0) * 255.0) as u8));
            px.push(255);
        }
    }
    px
}

fn dirt_image(level: u8) -> Image {
    let mut img = Image::new(
        Extent3d { width: SIZE, height: SIZE, depth_or_array_layers: 1 },
        TextureDimension::D2,
        dirt_pixels(level),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_follow_the_slider() {
        assert_eq!(level_of(0), 0);
        assert_eq!(level_of(4), 0);
        assert_eq!(level_of(30), 2);
        assert_eq!(level_of(100), LEVELS);
        let mut last = 0;
        for d in 0..=100u8 {
            assert!(level_of(d) >= last && level_of(d) <= LEVELS);
            last = level_of(d);
        }
    }

    #[test]
    fn dirtier_pictures_are_darker_and_the_picture_repeats_seamlessly() {
        let mean = |px: &[u8]| px.chunks(4).map(|p| p[0] as u32 + p[1] as u32 + p[2] as u32).sum::<u32>() as f32 / (px.len() / 4) as f32;
        let means: Vec<f32> = (1..=LEVELS).map(|l| mean(&dirt_pixels(l))).collect();
        for w in means.windows(2) {
            assert!(w[1] < w[0], "{means:?}");
        }
        assert!(means[0] > 600.0, "a little dirty is still mostly clean: {means:?}");
        // the left edge carries on from the right edge (it wraps round the body)
        let px = dirt_pixels(3);
        let at = |x: u32, y: u32| px[((y * SIZE + x) * 4) as usize] as i32;
        let jumps = (0..SIZE).filter(|y| (at(0, *y) - at(SIZE - 1, *y)).abs() > 40).count();
        assert!(jumps < 8, "seam: {jumps}");
    }
}
