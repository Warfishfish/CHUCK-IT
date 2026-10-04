//! The four blob characters you can choose from. They differ in looks only (body shape): speed,
//! hit and catch sizes are the same for everyone, so nobody gets an advantage from their pick.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Character {
    /// The original capsule body.
    Classic,
    /// Round bottom, slimmer shoulders.
    Pear,
    /// Stronger taper, a bit taller.
    Egg,
    /// Wide and low with the biggest round bottom.
    #[default]
    Gumdrop,
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

    /// How far out from the middle the hands rest, so they sit against this body shape.
    /// Matches the numbers in `make_blob.py`. The original blob used 0.47.
    pub fn hand_x(self) -> f32 {
        match self {
            Character::Classic => 0.47,
            Character::Pear => 0.479,
            Character::Egg => 0.4446,
            Character::Gumdrop => 0.487,
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
    fn default_is_the_gumdrop_marcus_picked() {
        assert_eq!(Character::default(), Character::Gumdrop);
    }
}
