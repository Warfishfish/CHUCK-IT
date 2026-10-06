//! How a player has customised their blob (Marcus, 6 Oct 2026): mouth, eyebrows, hair and the
//! colours of the body, singlet, thongs, hair and brows. Looks only: nothing here changes speed
//! or hit sizes. The body shape itself is `Character`.
//!
//! Colours are picked from small palettes (an index each), not a free colour wheel, so every
//! blob still looks good in the yard. Saved in `settings.txt` as plain words and numbers
//! (`to_lines` / `read_line`), and sent online as 8 bytes (`to_bytes` / `from_bytes`). Anything
//! odd or old falls back to the default instead of failing.

use crate::rng::Rng;

/// One feature with a few kinds, the first always "none".
pub trait Choice: Copy + PartialEq + Sized + 'static {
    const ALL: &'static [Self];
    fn name(self) -> &'static str;
    fn index(self) -> u8 {
        Self::ALL.iter().position(|x| *x == self).unwrap_or(0) as u8
    }
    fn from_index(i: u8) -> Self {
        Self::ALL.get(i as usize).copied().unwrap_or(Self::ALL[0])
    }
    fn from_name(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|x| x.name().eq_ignore_ascii_case(s.trim()))
    }
    /// The next one along (wrapping), for the menu's arrows.
    fn step(self, dir: i32) -> Self {
        let n = Self::ALL.len() as i32;
        let i = (self.index() as i32 + dir.signum()).rem_euclid(n);
        Self::ALL[i as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Mouth {
    None,
    /// A small, happy smile.
    #[default]
    Smile,
    /// A big open grin with teeth.
    Grin,
    /// A lopsided, cheeky smirk.
    Smirk,
}

impl Choice for Mouth {
    const ALL: &'static [Mouth] = &[Mouth::None, Mouth::Smile, Mouth::Grin, Mouth::Smirk];
    fn name(self) -> &'static str {
        match self {
            Mouth::None => "None",
            Mouth::Smile => "Smile",
            Mouth::Grin => "Grin",
            Mouth::Smirk => "Smirk",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Brows {
    None,
    /// Thick and straight.
    #[default]
    Flat,
    /// Thin and arched (surprised-looking).
    Arched,
    /// Big and bushy.
    Bushy,
}

impl Choice for Brows {
    const ALL: &'static [Brows] = &[Brows::None, Brows::Flat, Brows::Arched, Brows::Bushy];
    fn name(self) -> &'static str {
        match self {
            Brows::None => "None",
            Brows::Flat => "Flat",
            Brows::Arched => "Arched",
            Brows::Bushy => "Bushy",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Hair {
    #[default]
    None,
    /// A little tuft sticking up on top.
    Tuft,
    /// Business at the front, party at the back.
    Mullet,
    /// A round bowl cut.
    Bowl,
}

impl Choice for Hair {
    const ALL: &'static [Hair] = &[Hair::None, Hair::Tuft, Hair::Mullet, Hair::Bowl];
    fn name(self) -> &'static str {
        match self {
            Hair::None => "None",
            Hair::Tuft => "Tuft",
            Hair::Mullet => "Mullet",
            Hair::Bowl => "Bowl",
        }
    }
}

/// The colour palettes (0xRRGGBB). Sun-baked but clear, picked to sit well in the yard.
pub const BODY_COLOURS: [u32; 10] = [
    0xffd23f, // sunny yellow (the old fixed preview colour)
    0xff9f1c, // orange
    0xe85d4a, // tomato
    0xf28fb0, // pink
    0xa77bd6, // purple
    0x4f8fe0, // sky blue
    0x2fb5a8, // teal
    0x5fbf5a, // grass green
    0xc89a6a, // tan
    0x9aa3ad, // grey
];
pub const SINGLET_COLOURS: [u32; 8] = [0xf4f1e6, 0x2f3b4c, 0xd63a2f, 0x2f7fe0, 0x2e9e4f, 0xf2c230, 0x6d4c9e, 0x111111];
pub const THONG_COLOURS: [u32; 8] = [0xe8443a, 0x2f8ee8, 0xf4c20d, 0x30c26b, 0xff7ab8, 0x1b1b1b, 0xffffff, 0xff8a3d];
pub const HAIR_COLOURS: [u32; 8] = [0x3b2a1a, 0x6b4423, 0xb87333, 0xe8c26a, 0xd9d9d9, 0x111111, 0xd63a2f, 0x2fb5a8];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Appearance {
    pub mouth: Mouth,
    pub brows: Brows,
    pub hair: Hair,
    /// Indexes into the palettes above.
    pub body: u8,
    pub singlet: u8,
    pub thong: u8,
    pub hair_colour: u8,
    /// The brows' colour (from the hair palette).
    pub brow_colour: u8,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance { mouth: Mouth::Smile, brows: Brows::Flat, hair: Hair::None, body: 0, singlet: 0, thong: 0, hair_colour: 0, brow_colour: 0 }
    }
}

/// Which palette a colour index belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Palette {
    Body,
    Singlet,
    Thong,
    Hair,
    Brow,
}

impl Palette {
    pub fn colours(self) -> &'static [u32] {
        match self {
            Palette::Body => &BODY_COLOURS,
            Palette::Singlet => &SINGLET_COLOURS,
            Palette::Thong => &THONG_COLOURS,
            Palette::Hair | Palette::Brow => &HAIR_COLOURS,
        }
    }
}

impl Appearance {
    pub fn colour_index(&self, p: Palette) -> u8 {
        match p {
            Palette::Body => self.body,
            Palette::Singlet => self.singlet,
            Palette::Thong => self.thong,
            Palette::Hair => self.hair_colour,
            Palette::Brow => self.brow_colour,
        }
    }

    /// The colour itself (an out-of-range index wraps round, so it is always a real colour).
    pub fn colour(&self, p: Palette) -> u32 {
        let c = p.colours();
        c[self.colour_index(p) as usize % c.len()]
    }

    /// Move a colour along its palette (the menu's arrows), wrapping.
    pub fn step_colour(&mut self, p: Palette, dir: i32) {
        let n = p.colours().len() as i32;
        let v = ((self.colour_index(p) as i32 + dir.signum()).rem_euclid(n)) as u8;
        match p {
            Palette::Body => self.body = v,
            Palette::Singlet => self.singlet = v,
            Palette::Thong => self.thong = v,
            Palette::Hair => self.hair_colour = v,
            Palette::Brow => self.brow_colour = v,
        }
    }

    /// Pull every index back inside its palette (after loading or receiving).
    pub fn tidy(mut self) -> Self {
        let fix = |v: u8, p: Palette| v % p.colours().len() as u8;
        self.body = fix(self.body, Palette::Body);
        self.singlet = fix(self.singlet, Palette::Singlet);
        self.thong = fix(self.thong, Palette::Thong);
        self.hair_colour = fix(self.hair_colour, Palette::Hair);
        self.brow_colour = fix(self.brow_colour, Palette::Brow);
        self
    }

    /// A random look (the Randomise button, and the bots).
    pub fn random(rng: &mut Rng) -> Self {
        let pick = |rng: &mut Rng, n: usize| rng.index(n) as u8;
        Appearance {
            mouth: Mouth::from_index(pick(rng, Mouth::ALL.len())),
            brows: Brows::from_index(pick(rng, Brows::ALL.len())),
            hair: Hair::from_index(pick(rng, Hair::ALL.len())),
            body: pick(rng, BODY_COLOURS.len()),
            singlet: pick(rng, SINGLET_COLOURS.len()),
            thong: pick(rng, THONG_COLOURS.len()),
            hair_colour: pick(rng, HAIR_COLOURS.len()),
            brow_colour: pick(rng, HAIR_COLOURS.len()),
        }
    }

    /// `key=value` lines for `settings.txt`.
    pub fn to_lines(&self) -> String {
        format!(
            "mouth={}\nbrows={}\nhair={}\nbody_colour={}\nsinglet_colour={}\nthong_colour={}\nhair_colour={}\nbrow_colour={}\n",
            self.mouth.name(),
            self.brows.name(),
            self.hair.name(),
            self.body,
            self.singlet,
            self.thong,
            self.hair_colour,
            self.brow_colour
        )
    }

    /// Read one `key=value` line from `settings.txt`. Returns false if the key is not ours.
    /// A bad value leaves that part as it was.
    pub fn read_line(&mut self, key: &str, value: &str) -> bool {
        let num = |v: &str| v.trim().parse::<u8>().ok();
        match key {
            "mouth" => self.mouth = Mouth::from_name(value).unwrap_or(self.mouth),
            "brows" => self.brows = Brows::from_name(value).unwrap_or(self.brows),
            "hair" => self.hair = Hair::from_name(value).unwrap_or(self.hair),
            "body_colour" => self.body = num(value).unwrap_or(self.body),
            "singlet_colour" => self.singlet = num(value).unwrap_or(self.singlet),
            "thong_colour" => self.thong = num(value).unwrap_or(self.thong),
            "hair_colour" => self.hair_colour = num(value).unwrap_or(self.hair_colour),
            "brow_colour" => self.brow_colour = num(value).unwrap_or(self.brow_colour),
            _ => return false,
        }
        *self = self.tidy();
        true
    }

    /// The look as 8 bytes, for the online messages.
    pub fn to_bytes(&self) -> [u8; 8] {
        [
            self.mouth.index(),
            self.brows.index(),
            self.hair.index(),
            self.body,
            self.singlet,
            self.thong,
            self.hair_colour,
            self.brow_colour,
        ]
    }

    /// From 8 bytes; anything out of range becomes something sensible.
    pub fn from_bytes(b: [u8; 8]) -> Self {
        Appearance {
            mouth: Mouth::from_index(b[0]),
            brows: Brows::from_index(b[1]),
            hair: Hair::from_index(b[2]),
            body: b[3],
            singlet: b[4],
            thong: b[5],
            hair_colour: b[6],
            brow_colour: b[7],
        }
        .tidy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn odd() -> Appearance {
        Appearance { mouth: Mouth::Smirk, brows: Brows::Bushy, hair: Hair::Mullet, body: 7, singlet: 3, thong: 5, hair_colour: 2, brow_colour: 6 }
    }

    #[test]
    fn each_feature_has_none_and_three_kinds() {
        assert_eq!(Mouth::ALL.len(), 4);
        assert_eq!(Brows::ALL.len(), 4);
        assert_eq!(Hair::ALL.len(), 4);
        assert_eq!(Mouth::ALL[0], Mouth::None);
        assert_eq!(Brows::ALL[0], Brows::None);
        assert_eq!(Hair::ALL[0], Hair::None);
    }

    #[test]
    fn the_arrows_go_round_and_round() {
        assert_eq!(Mouth::Smirk.step(1), Mouth::None);
        assert_eq!(Mouth::None.step(-1), Mouth::Smirk);
        assert_eq!(Hair::Tuft.step(1), Hair::Mullet);
        let mut a = Appearance::default();
        a.step_colour(Palette::Body, -1);
        assert_eq!(a.body as usize, BODY_COLOURS.len() - 1);
        a.step_colour(Palette::Body, 1);
        assert_eq!(a.body, 0);
        a.step_colour(Palette::Brow, 1);
        assert_eq!(a.colour(Palette::Brow), HAIR_COLOURS[1]);
    }

    #[test]
    fn saving_and_loading_gives_the_same_look() {
        let a = odd();
        let mut b = Appearance::default();
        for line in a.to_lines().lines() {
            let (k, v) = line.split_once('=').unwrap();
            assert!(b.read_line(k, v), "{k}");
        }
        assert_eq!(a, b);
    }

    #[test]
    fn bad_or_old_settings_fall_back_quietly() {
        let mut a = Appearance::default();
        assert!(!a.read_line("fov", "85"), "not ours");
        a.read_line("mouth", "Moustache");
        a.read_line("hair", "");
        a.read_line("body_colour", "banana");
        a.read_line("thong_colour", "200");
        assert_eq!(a.mouth, Mouth::Smile, "unknown words keep the default");
        assert_eq!(a.hair, Hair::None);
        assert_eq!(a.body, 0);
        assert!((a.thong as usize) < THONG_COLOURS.len(), "out of range is pulled back in");
        // names are not fussy about case or spaces
        a.read_line("brows", " bushy ");
        assert_eq!(a.brows, Brows::Bushy);
    }

    #[test]
    fn the_look_fits_in_8_bytes_and_junk_is_made_safe() {
        let a = odd();
        assert_eq!(Appearance::from_bytes(a.to_bytes()), a);
        let junk = Appearance::from_bytes([255; 8]);
        assert_eq!(junk.mouth, Mouth::None);
        for p in [Palette::Body, Palette::Singlet, Palette::Thong, Palette::Hair, Palette::Brow] {
            assert!((junk.colour_index(p) as usize) < p.colours().len());
            let _ = junk.colour(p);
        }
    }

    #[test]
    fn random_looks_stay_inside_the_choices_and_vary() {
        let mut rng = Rng::new(7);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..300 {
            let a = Appearance::random(&mut rng);
            assert_eq!(a, a.tidy());
            seen.insert((a.mouth, a.hair, a.body));
        }
        assert!(seen.len() > 50, "lots of different blobs ({})", seen.len());
    }
}
