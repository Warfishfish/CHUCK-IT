//! The four blob characters you can choose from. They differ in looks only (body shape): speed,
//! hit and catch sizes are the same for everyone, so nobody gets an advantage from their pick.
//!
//! C9.5 (8 Oct 2026): the four shapes are now clearly different: Classic is the straight-sided
//! capsule, Pear has a big round bottom and narrow shoulders, Egg is tall and slim, Gumdrop is
//! short and wide. Only the drawing changes; the game's sizes for hits and catches do not.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Character {
    /// The original capsule body.
    Classic,
    /// A big round bottom and narrow shoulders.
    Pear,
    /// Tall and slim.
    Egg,
    /// Short, wide and squat with a big head.
    #[default]
    Gumdrop,
}

/// Which set of models the game is drawing. Each set has its own proportions (the pop look has
/// bigger heads and hands, the clay look is squatter).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ModelSet {
    /// `blob_<name>.glb`: the browser look and `--style current`.
    #[default]
    Plain,
    /// `blob_<name>_pop.glb`: the polished look.
    Pop,
    /// `blob_<name>_clay.glb`: the clay example.
    Clay,
}

/// Where things sit on one character's model, in metres up from the ground (or out from the
/// middle). `tools/make_blob.py` prints these as "MEASURE" lines whenever it makes the models.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measure {
    /// How far out from the middle the hands rest (the original blob used 0.47).
    pub hand_x: f32,
    /// The top of the head.
    pub head_top: f32,
    /// The top of the body (the shoulders, under the head).
    pub body_top: f32,
    /// Half the body's width at its widest.
    pub body_w: f32,
}

const fn m(hand_x: f32, head_top: f32, body_top: f32, body_w: f32) -> Measure {
    Measure { hand_x, head_top, body_top, body_w }
}

impl Character {
    pub const ALL: [Character; 4] = [
        Character::Classic,
        Character::Pear,
        Character::Egg,
        Character::Gumdrop,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Character::Classic => "Classic",
            Character::Pear => "Pear",
            Character::Egg => "Egg",
            Character::Gumdrop => "Gumdrop",
        }
    }

    /// The model file, relative to the assets folder (made by `tools/make_blob.py`).
    pub fn model(self) -> &'static str {
        match self {
            Character::Classic => "models/blob_classic.glb",
            Character::Pear => "models/blob_pear.glb",
            Character::Egg => "models/blob_egg.glb",
            Character::Gumdrop => "models/blob_gumdrop.glb",
        }
    }

    /// Where the hands, head and shoulders are on this character's model in a model set.
    /// Copied from the "MEASURE" lines `make_blob.py` prints (8 Oct 2026).
    pub fn measure(self, set: ModelSet) -> Measure {
        use Character::*;
        use ModelSet::*;
        match (set, self) {
            (Plain, Classic) => m(0.470, 1.850, 1.415, 0.358),
            (Plain, Pear) => m(0.473, 1.749, 1.410, 0.431),
            (Plain, Egg) => m(0.470, 2.059, 1.690, 0.372),
            (Plain, Gumdrop) => m(0.562, 1.600, 1.190, 0.501),
            (Pop, Classic) => m(0.470, 1.952, 1.415, 0.358),
            (Pop, Pear) => m(0.446, 1.845, 1.410, 0.405),
            (Pop, Egg) => m(0.443, 2.164, 1.690, 0.346),
            (Pop, Gumdrop) => m(0.523, 1.718, 1.190, 0.467),
            (Clay, Classic) => m(0.544, 1.834, 1.487, 0.430),
            (Clay, Pear) => m(0.526, 1.600, 1.276, 0.518),
            (Clay, Egg) => m(0.542, 1.880, 1.528, 0.449),
            (Clay, Gumdrop) => m(0.646, 1.469, 1.078, 0.604),
        }
    }

    pub fn next(self) -> Character {
        let i = Self::ALL.iter().position(|c| *c == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_cycles_through_all_four_and_back() {
        let mut c = Character::Classic;
        let mut seen = vec![c];
        for _ in 0..3 {
            c = c.next();
            seen.push(c);
        }
        assert_eq!(seen, Character::ALL.to_vec());
        assert_eq!(c.next(), Character::Classic);
    }

    #[test]
    fn every_character_has_its_own_model_and_name() {
        let models: std::collections::HashSet<_> =
            Character::ALL.iter().map(|c| c.model()).collect();
        let names: std::collections::HashSet<_> = Character::ALL.iter().map(|c| c.name()).collect();
        assert_eq!(models.len(), 4);
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn measures_make_sense_for_every_model() {
        for set in [ModelSet::Plain, ModelSet::Pop, ModelSet::Clay] {
            for c in Character::ALL {
                let m = c.measure(set);
                assert!(m.hand_x > m.body_w, "{c:?} {set:?}: hands outside the body");
                assert!(m.head_top > m.body_top + 0.2, "{c:?} {set:?}: head on top of the body");
                assert!(m.body_top > 1.0 && m.head_top < 2.3, "{c:?} {set:?}: a sensible height");
            }
        }
        // the original capsule keeps the original hand spot
        assert_eq!(Character::Classic.measure(ModelSet::Plain).hand_x, 0.47);
    }

    #[test]
    fn the_four_shapes_are_clearly_different() {
        // C9.5: you can tell them apart by outline: Egg the tallest, Gumdrop the shortest and
        // widest, Pear wider than Classic and Egg
        for set in [ModelSet::Plain, ModelSet::Pop] {
            let [c, p, e, g] = Character::ALL.map(|ch| ch.measure(set));
            assert!(e.body_top > c.body_top + 0.2 && g.body_top < c.body_top - 0.2);
            assert!(g.body_w > p.body_w && p.body_w > c.body_w && p.body_w > e.body_w);
        }
    }

    #[test]
    fn default_is_the_gumdrop_marcus_picked() {
        assert_eq!(Character::default(), Character::Gumdrop);
    }
}
