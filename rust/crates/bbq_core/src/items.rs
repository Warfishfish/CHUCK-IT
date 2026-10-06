//! Item and dildo-variant data (spec section 3).

/// Every kind of throwable thing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Teddy,
    Stubby,
    Gnome,
    Dildo,
    Steak,
    Fish,
    Noodle,
    /// Retired: never spawns, kept so old behaviour can be matched.
    Snag,
}

/// What a slap with the item does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Melee {
    /// Can only be thrown.
    None,
    /// Dizzy stun on the victim (steak, fish, noodle).
    Stun,
    /// Knocks the victim flat (dildo).
    Down,
}

/// Tuning numbers for one kind of item.
#[derive(Clone, Copy, Debug)]
pub struct ItemDef {
    /// Launch speed at full charge, metres per second.
    pub speed: f32,
    pub knock: f32,
    /// Seconds to wind up to full charge.
    pub charge: f32,
    pub radius: f32,
    pub bounce: f32,
    /// Multiplier on the item gravity of 18.
    pub grav: f32,
    /// Chance weight in the random spawner (out of 100).
    pub weight: f32,
    /// An explicit stun length; otherwise `0.35 + knock * 0.035`.
    pub stun: Option<f32>,
    pub melee: Melee,
    /// A melee item that can also be thrown (the noodle).
    pub throwable: bool,
    /// Uses before it is used up (noodle).
    pub uses: Option<u32>,
    /// Stun length when slapping (noodle).
    pub slap_stun: Option<f32>,
    /// Points for slapping (noodle).
    pub slap_pts: Option<i32>,
    /// Breaks on hard impact and leaves a puddle (VP can).
    pub smash: bool,
    /// Only exists in Cheeky mode.
    pub adult: bool,
}

const BASE: ItemDef = ItemDef {
    speed: 20.0,
    knock: 3.0,
    charge: 0.6,
    radius: 0.18,
    bounce: 0.1,
    grav: 1.0,
    weight: 0.0,
    stun: None,
    melee: Melee::None,
    throwable: false,
    uses: None,
    slap_stun: None,
    slap_pts: None,
    smash: false,
    adult: false,
};

static TEDDY: ItemDef = ItemDef {
    speed: 24.0,
    knock: 6.0,
    charge: 0.55,
    radius: 0.24,
    bounce: 0.5,
    weight: 45.0,
    ..BASE
};
static STUBBY: ItemDef = ItemDef {
    speed: 27.0,
    knock: 8.0,
    charge: 0.7,
    radius: 0.14,
    bounce: 0.0,
    weight: 35.0,
    smash: true,
    ..BASE
};
static GNOME: ItemDef = ItemDef {
    speed: 19.0,
    knock: 15.0,
    charge: 1.0,
    radius: 0.28,
    bounce: 0.25,
    grav: 1.15,
    weight: 20.0,
    stun: Some(0.65),
    ..BASE
};
static DILDO: ItemDef = ItemDef {
    speed: 23.0,
    knock: 9.0,
    charge: 0.75,
    radius: 0.2,
    bounce: 0.6,
    melee: Melee::Down,
    adult: true,
    ..BASE
};
static STEAK: ItemDef = ItemDef {
    bounce: 0.1,
    melee: Melee::Stun,
    ..BASE
};
static FISH: ItemDef = ItemDef {
    bounce: 0.2,
    melee: Melee::Stun,
    ..BASE
};
static NOODLE: ItemDef = ItemDef {
    speed: 21.0,
    knock: 5.0,
    radius: 0.2,
    bounce: 0.45,
    grav: 0.85,
    melee: Melee::Stun,
    throwable: true,
    uses: Some(6),
    slap_stun: Some(1.2),
    slap_pts: Some(50),
    ..BASE
};
static SNAG: ItemDef = ItemDef {
    adult: true,
    ..BASE
};

impl ItemKind {
    /// In the order the JavaScript game lists them (this order matters for
    /// the random spawner).
    pub const ALL: [ItemKind; 8] = [
        ItemKind::Teddy,
        ItemKind::Stubby,
        ItemKind::Dildo,
        ItemKind::Steak,
        ItemKind::Fish,
        ItemKind::Gnome,
        ItemKind::Noodle,
        ItemKind::Snag,
    ];

    pub fn def(self) -> &'static ItemDef {
        match self {
            ItemKind::Teddy => &TEDDY,
            ItemKind::Stubby => &STUBBY,
            ItemKind::Gnome => &GNOME,
            ItemKind::Dildo => &DILDO,
            ItemKind::Steak => &STEAK,
            ItemKind::Fish => &FISH,
            ItemKind::Noodle => &NOODLE,
            ItemKind::Snag => &SNAG,
        }
    }

    /// How long a thrown hit from this item stuns the victim.
    pub fn hit_stun(self) -> f32 {
        let d = self.def();
        d.stun.unwrap_or(0.35 + d.knock * 0.035)
    }

    /// Uses a freshly made item has (steak, fish, dildo: 3; noodle: 6; others unlimited).
    pub fn starting_uses(self) -> Option<u32> {
        match self {
            ItemKind::Steak | ItemKind::Fish | ItemKind::Dildo => Some(SLAP_USES),
            ItemKind::Noodle => self.def().uses,
            _ => None,
        }
    }
}

/// Uses for steak, fish and dildo.
pub const SLAP_USES: u32 = 3;
/// Dizzy time after a steak or fish slap.
pub const MEAT_STUN: f32 = 2.0;
/// Points for a steak or fish slap.
pub const MEAT_PTS: i32 = 50;

/// Pick a spawnable item from a roll in `[0, 1)`, using the weights.
pub fn random_type(roll: f32) -> ItemKind {
    let r = roll * 100.0;
    let mut acc = 0.0;
    for k in ItemKind::ALL {
        acc += k.def().weight;
        if r < acc {
            return k;
        }
    }
    ItemKind::Teddy
}

/// How many items the spawner aims to keep around.
pub fn spawn_target(players: usize) -> usize {
    (6.0 + players as f32 * 1.6).min(22.0) as usize
}

/// How many items are placed at the start of a round.
pub fn initial_items(players: usize) -> usize {
    8 + (players as f32 * 1.5).floor() as usize
}

/// How often the spawner may add an item, in seconds.
pub const SPAWN_EVERY: f32 = 1.3;
/// Two spawned items must be this far apart.
pub const SPAWN_CLEARANCE: f32 = 2.2;
/// Noodles kept floating in the pool.
pub const NOODLES: usize = 2;

/// The dildo types: the four sizes from the browser game (spec section 3), plus two new ones
/// that only the Rust version has, each with an ability (Marcus, 6 Oct 2026).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DildoVariant {
    Classic,
    Mini,
    Jumbo,
    Gold,
    /// Small and teal: swings fast, so you can slap more often, but it hits light.
    Quickie,
    /// Long and orange: hits from further away, but swings slowly.
    LongJohn,
}

#[derive(Clone, Copy, Debug)]
pub struct VariantDef {
    pub label: &'static str,
    pub scale: f32,
    /// Bonus (or penalty) points on a slap.
    pub points: i32,
    /// Added to the 3.5 s knockdown time.
    pub down: f32,
    pub weight: f32,
    pub rare: bool,
    /// Chance that a slap is a critical (the browser game has 0.15 for all of them).
    pub crit: f32,
    /// How likely each knockdown is, in the order Sent Flying, Cartwheel, Timber (they need not
    /// add up to 1: they are weights).
    pub poses: [f32; 3],
    /// Multiplies the wait between swings and the swing animation (below 1 = faster).
    pub swing: f32,
    /// Multiplies how far away a slap can land.
    pub reach: f32,
}

impl DildoVariant {
    pub const ALL: [DildoVariant; 6] = [
        DildoVariant::Classic,
        DildoVariant::Mini,
        DildoVariant::Jumbo,
        DildoVariant::Gold,
        DildoVariant::Quickie,
        DildoVariant::LongJohn,
    ];

    pub fn def(self) -> VariantDef {
        // `crit` and `poses` are the odds of each type: the browser's types all use 0.15 and an
        // even third of each knockdown; the Rust version gives them each their own character.
        match self {
            DildoVariant::Classic => VariantDef {
                label: "Purple Dildo",
                scale: 1.0,
                points: 0,
                down: 0.0,
                weight: 46.0,
                rare: false,
                crit: 0.15,
                poses: [1.0, 1.0, 1.0],
                swing: 1.0,
                reach: 1.0,
            },
            // light and nippy: crits a bit more often, but never sends anyone flying
            DildoVariant::Mini => VariantDef {
                label: "Pocket Rocket",
                scale: 0.66,
                points: -25,
                down: -1.0,
                weight: 28.0,
                rare: false,
                crit: 0.22,
                poses: [0.0, 1.0, 1.0],
                swing: 1.0,
                reach: 1.0,
            },
            // heavy: rarely crits, but mostly sends people flying
            DildoVariant::Jumbo => VariantDef {
                label: "The Unit",
                scale: 1.38,
                points: 50,
                down: 1.0,
                weight: 18.0,
                rare: false,
                crit: 0.08,
                poses: [1.0, 0.5, 0.5],
                swing: 1.0,
                reach: 1.0,
            },
            DildoVariant::Gold => VariantDef {
                label: "Golden Wonder",
                scale: 1.12,
                points: 120,
                down: 2.0,
                weight: 4.0,
                rare: true,
                crit: 0.25,
                poses: [1.0, 1.0, 1.0],
                swing: 1.0,
                reach: 1.0,
            },
            // fast: swings in 60% of the time (about 0.33 s between slaps, not 0.55)
            DildoVariant::Quickie => VariantDef {
                label: "The Quickie",
                scale: 0.8,
                points: -10,
                down: -0.5,
                weight: 12.0,
                rare: false,
                crit: 0.12,
                poses: [0.5, 1.0, 1.0],
                swing: 0.6,
                reach: 0.9,
            },
            // long: reaches 40% further, but swings in 150% of the time
            DildoVariant::LongJohn => VariantDef {
                label: "Long John",
                scale: 1.5,
                points: 25,
                down: 0.5,
                weight: 10.0,
                rare: false,
                crit: 0.10,
                poses: [1.0, 1.0, 1.0],
                swing: 1.5,
                reach: 1.4,
            },
        }
    }

    /// How long a slap with this variant knocks someone flat (a critical is always 5 s).
    pub fn knockdown_time(self, crit: bool) -> f32 {
        if crit {
            CRIT_DOWN
        } else {
            (DOWN_TIME + self.def().down).max(1.2)
        }
    }
}

/// Default knockdown time.
pub const DOWN_TIME: f32 = 3.5;
/// A critical dildo slap flattens for this long.
pub const CRIT_DOWN: f32 = 5.0;
/// Chance of a critical dildo slap.
pub const CRIT_CHANCE: f32 = 0.15;
/// Extra points on a critical.
pub const CRIT_POINTS: i32 = 60;

/// Pick a variant from a roll in `[0, 1)`.
pub fn pick_dildo_variant(roll: f32) -> DildoVariant {
    let total: f32 = DildoVariant::ALL.iter().map(|v| v.def().weight).sum();
    let mut r = roll * total;
    for v in DildoVariant::ALL {
        r -= v.def().weight;
        if r <= 0.0 {
            return v;
        }
    }
    DildoVariant::Classic
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_stun_lengths_match_the_spec() {
        let near = |a: f32, b: f32| (a - b).abs() < 1e-4;
        assert!(near(ItemKind::Teddy.hit_stun(), 0.56));
        assert!(near(ItemKind::Stubby.hit_stun(), 0.63));
        assert!(near(ItemKind::Dildo.hit_stun(), 0.665));
        assert!(near(ItemKind::Steak.hit_stun(), 0.455));
        assert!(near(ItemKind::Fish.hit_stun(), 0.455));
        assert!(near(ItemKind::Noodle.hit_stun(), 0.525));
        assert!(near(ItemKind::Gnome.hit_stun(), 0.65));
    }

    #[test]
    fn spawn_weights_add_to_100_and_only_three_items_spawn() {
        let total: f32 = ItemKind::ALL.iter().map(|k| k.def().weight).sum();
        assert_eq!(total, 100.0);
        assert_eq!(random_type(0.0), ItemKind::Teddy);
        assert_eq!(random_type(0.449), ItemKind::Teddy);
        assert_eq!(random_type(0.451), ItemKind::Stubby);
        assert_eq!(random_type(0.799), ItemKind::Stubby);
        assert_eq!(random_type(0.801), ItemKind::Gnome);
        assert_eq!(random_type(0.999), ItemKind::Gnome);
    }

    #[test]
    fn spawn_counts() {
        assert_eq!(spawn_target(2), 9); // 6 + 3.2
        assert_eq!(spawn_target(4), 12); // 6 + 6.4
        assert_eq!(spawn_target(16), 22); // capped
        assert_eq!(initial_items(4), 14);
        assert_eq!(initial_items(1), 9);
    }

    #[test]
    fn dildo_variant_knockdown_times() {
        assert_eq!(DildoVariant::Classic.knockdown_time(false), 3.5);
        assert_eq!(DildoVariant::Mini.knockdown_time(false), 2.5);
        assert_eq!(DildoVariant::Jumbo.knockdown_time(false), 4.5);
        assert_eq!(DildoVariant::Gold.knockdown_time(false), 5.5);
        assert_eq!(DildoVariant::Gold.knockdown_time(true), 5.0);
    }

    #[test]
    fn dildo_variant_weights() {
        // Weights 46/28/18/4/12/10 add up to 118, so the cut-offs are 46, 74, 92, 96 and 108
        // out of 118 (0.390, 0.627, 0.780, 0.814, 0.915).
        assert_eq!(pick_dildo_variant(0.0), DildoVariant::Classic);
        assert_eq!(pick_dildo_variant(0.38), DildoVariant::Classic);
        assert_eq!(pick_dildo_variant(0.40), DildoVariant::Mini);
        assert_eq!(pick_dildo_variant(0.62), DildoVariant::Mini);
        assert_eq!(pick_dildo_variant(0.64), DildoVariant::Jumbo);
        assert_eq!(pick_dildo_variant(0.77), DildoVariant::Jumbo);
        assert_eq!(pick_dildo_variant(0.79), DildoVariant::Gold);
        assert_eq!(pick_dildo_variant(0.82), DildoVariant::Quickie);
        assert_eq!(pick_dildo_variant(0.90), DildoVariant::Quickie);
        assert_eq!(pick_dildo_variant(0.93), DildoVariant::LongJohn);
        assert_eq!(pick_dildo_variant(0.999), DildoVariant::LongJohn);
    }

    #[test]
    fn the_new_types_have_their_abilities() {
        let q = DildoVariant::Quickie.def();
        let l = DildoVariant::LongJohn.def();
        assert!(q.swing < 1.0 && l.swing > 1.0, "quick swings faster, long slower");
        assert!(l.reach > 1.0 && q.reach <= 1.0, "long reaches further");
        // every type that existed in the browser game keeps its reach and swing speed
        for v in [DildoVariant::Classic, DildoVariant::Mini, DildoVariant::Jumbo, DildoVariant::Gold] {
            assert_eq!((v.def().swing, v.def().reach), (1.0, 1.0));
        }
        // the odds make sense for every type
        for v in DildoVariant::ALL {
            let d = v.def();
            assert!((0.0..=1.0).contains(&d.crit), "{v:?}");
            assert!(d.poses.iter().all(|p| *p >= 0.0) && d.poses.iter().sum::<f32>() > 0.0, "{v:?}");
        }
    }

    #[test]
    fn uses() {
        assert_eq!(ItemKind::Steak.starting_uses(), Some(3));
        assert_eq!(ItemKind::Noodle.starting_uses(), Some(6));
        assert_eq!(ItemKind::Teddy.starting_uses(), None);
    }
}
