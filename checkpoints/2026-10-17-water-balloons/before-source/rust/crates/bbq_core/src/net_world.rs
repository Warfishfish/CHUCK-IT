//! Online play, step 9.4: how the host describes the whole yard to everybody else.
//!
//! The host runs the rules, so the host knows where the items, the bots and Dazza are, what the
//! scores are and what is happening in the round. About 15 times a second it sends a
//! `WorldSnap`; the guests draw from it. Numbers are the host's own, and every person is named
//! by their "net id" (the host is 1, guests 2 and up, bots 100 and up); `bbq_app/src/online.rs`
//! turns those into the ids each computer uses.
//!
//! The snapshot holds what is needed to DRAW the yard and show the scoreboard, not what is
//! needed to run it. Everything is checked when it is read, and a snapshot that claims too many
//! of anything is refused.

use crate::net::{DecodeError, Reader, Writer};

/// The most of each kind of thing in one snapshot (more than the game ever has).
pub const MAX_ITEMS: usize = 64;
pub const MAX_BOTS: usize = 16;
pub const MAX_BOARD: usize = 32;
pub const MAX_FEED: usize = 8;
pub const MAX_FX: usize = 32;
pub const LINE_MAX: usize = 120;

/// How a thing is held: not at all, or in somebody's hand, or flying.
pub const STATE_GROUND: u8 = 0;
pub const STATE_HELD: u8 = 1;
pub const STATE_FLYING: u8 = 2;

#[derive(Clone, Debug, PartialEq)]
pub struct ItemSnap {
    pub id: u32,
    /// Index into `ItemKind::ALL`.
    pub kind: u8,
    /// Index into `DildoVariant::ALL`, or 255 for none.
    pub variant: u8,
    pub state: u8,
    /// Net id of whoever holds it (0 = nobody).
    pub holder: u32,
    pub pos: (f32, f32, f32),
    pub vel: (f32, f32, f32),
    pub ground_y: f32,
    /// Heist team index, or 255.
    pub team: u8,
    /// Uses left, or 255 for "not a slapping item".
    pub uses: u8,
}

pub const BOT_GROUNDED: u16 = 1;
pub const BOT_STUNNED: u16 = 2;
pub const BOT_DOWN: u16 = 4;
pub const BOT_FALLEN: u16 = 8;
pub const BOT_DRINKING: u16 = 16;
pub const BOT_SEATED: u16 = 32;
pub const BOT_NAUGHTY: u16 = 64;
pub const BOT_WINDING: u16 = 128;
pub const BOT_CROWN: u16 = 256;
pub const BOT_SMELLY: u16 = 512;

#[derive(Clone, Debug, PartialEq)]
pub struct BotSnap {
    pub id: u32,
    pub pos: (f32, f32, f32),
    pub vel: (f32, f32),
    pub face: f32,
    pub flags: u16,
    /// Wind-up, 0 to 1.
    pub charge: f32,
    /// Smoko seat number, or 255.
    pub seat: u8,
    /// Team index, or 255.
    pub team: u8,
    /// Drunk meter, 0 to 100 (stored in a byte).
    pub drunk: u8,
    /// Beer belly, 0 to 2 (stored in a byte).
    pub belly: u8,
    /// The item it has in its hand (id), or 0.
    pub selected: u32,
    /// Seconds it has left of a swing, in 1/100 s (so the swing is drawn).
    pub swing: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DazzaSnap {
    pub x: f32,
    pub z: f32,
    pub face: f32,
    /// Index into `DazzaState::ALL`.
    pub state: u8,
    /// Go up each time he speaks / swings the spatula / flips the grill, so the picture can
    /// start the matching animation once.
    pub say_seq: u32,
    pub swing_seq: u32,
    pub flip_seq: u32,
    pub say: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoardRow {
    pub id: u32,
    pub score: i32,
    pub hits: u16,
    pub taken: u16,
    pub catches: u16,
    pub throws: u16,
    pub streak: u8,
    pub banked: u16,
}

pub const FEAT_BAR: u8 = 1;
pub const FEAT_BBQ: u8 = 2;
pub const FEAT_CHEST: u8 = 4;
pub const FEAT_SMOKO: u8 = 8;
pub const FEAT_ADULT: u8 = 16;
pub const FEAT_FRIENDLY_FIRE: u8 = 32;
pub const FEAT_TIMED: u8 = 64;

#[derive(Clone, Debug, PartialEq)]
pub struct RoundSnap {
    /// 0 menu, 1 warm-up, 2 countdown, 3 play, 4 results (the `Phase` order).
    pub phase: u8,
    /// 0 free for all, 1 teams, 2 heist.
    pub mode: u8,
    pub features: u8,
    pub time_left: f32,
    pub round_no: u8,
    pub match_len: u8,
    pub chest_spot: u8,
    pub chest_stock: u8,
    pub smoko_at: (f32, f32),
    pub heist_teams: u8,
    /// The big banner on screen, with seconds left.
    pub banner: Option<(String, f32)>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FxSnap {
    /// Index into the game's list of effect kinds.
    pub kind: u8,
    pub at: (f32, f32, f32),
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldSnap {
    /// Counts up, so a late old snapshot is thrown away.
    pub seq: u32,
    /// The host's game clock.
    pub now: f32,
    pub round: RoundSnap,
    pub items: Vec<ItemSnap>,
    pub bots: Vec<BotSnap>,
    pub dazza: DazzaSnap,
    pub board: Vec<BoardRow>,
    /// Who is on which team: (net id, team index).
    pub teams: Vec<(u32, u8)>,
    /// New lines for the feed since the last snapshot.
    pub feed: Vec<String>,
    /// Particle effects that happened since the last snapshot.
    pub fx: Vec<FxSnap>,
}

impl WorldSnap {
    pub(crate) fn write(&self, w: &mut Writer) {
        w.u32(self.seq);
        w.f32(self.now);
        let r = &self.round;
        w.u8(r.phase);
        w.u8(r.mode);
        w.u8(r.features);
        w.f32(r.time_left);
        w.u8(r.round_no);
        w.u8(r.match_len);
        w.u8(r.chest_spot);
        w.u8(r.chest_stock);
        w.pair(r.smoko_at);
        w.u8(r.heist_teams);
        match &r.banner {
            Some((t, secs)) => {
                w.u8(1);
                w.line(t, LINE_MAX);
                w.f32(*secs);
            }
            None => w.u8(0),
        }
        w.u8(self.items.len().min(MAX_ITEMS) as u8);
        for it in self.items.iter().take(MAX_ITEMS) {
            w.u32(it.id);
            w.u8(it.kind);
            w.u8(it.variant);
            w.u8(it.state);
            w.u32(it.holder);
            w.triple(it.pos);
            w.triple(it.vel);
            w.f32(it.ground_y);
            w.u8(it.team);
            w.u8(it.uses);
        }
        w.u8(self.bots.len().min(MAX_BOTS) as u8);
        for b in self.bots.iter().take(MAX_BOTS) {
            w.u32(b.id);
            w.triple(b.pos);
            w.pair(b.vel);
            w.f32(b.face);
            w.u16(b.flags);
            w.u8((b.charge.clamp(0.0, 1.0) * 255.0).round() as u8);
            w.u8(b.seat);
            w.u8(b.team);
            w.u8(b.drunk);
            w.u8(b.belly);
            w.u32(b.selected);
            w.u8(b.swing);
        }
        let d = &self.dazza;
        w.f32(d.x);
        w.f32(d.z);
        w.f32(d.face);
        w.u8(d.state);
        w.u32(d.say_seq);
        w.u32(d.swing_seq);
        w.u32(d.flip_seq);
        w.line(&d.say, LINE_MAX);
        w.u8(self.board.len().min(MAX_BOARD) as u8);
        for row in self.board.iter().take(MAX_BOARD) {
            w.u32(row.id);
            w.i32(row.score);
            w.u16(row.hits);
            w.u16(row.taken);
            w.u16(row.catches);
            w.u16(row.throws);
            w.u8(row.streak);
            w.u16(row.banked);
        }
        w.u8(self.teams.len().min(MAX_BOARD) as u8);
        for (id, t) in self.teams.iter().take(MAX_BOARD) {
            w.u32(*id);
            w.u8(*t);
        }
        w.u8(self.feed.len().min(MAX_FEED) as u8);
        for line in self.feed.iter().take(MAX_FEED) {
            w.line(line, LINE_MAX);
        }
        w.u8(self.fx.len().min(MAX_FX) as u8);
        for f in self.fx.iter().take(MAX_FX) {
            w.u8(f.kind);
            w.triple(f.at);
        }
    }

    pub(crate) fn read(r: &mut Reader) -> Result<WorldSnap, DecodeError> {
        let seq = r.u32()?;
        let now = r.f32()?;
        let round = RoundSnap {
            phase: r.u8()?,
            mode: r.u8()?,
            features: r.u8()?,
            time_left: r.f32()?,
            round_no: r.u8()?,
            match_len: r.u8()?,
            chest_spot: r.u8()?,
            chest_stock: r.u8()?,
            smoko_at: r.pair()?,
            heist_teams: r.u8()?,
            banner: match r.u8()? {
                0 => None,
                1 => Some((r.line(LINE_MAX)?, r.f32()?)),
                _ => return Err(DecodeError::BadText),
            },
        };
        let count = |r: &mut Reader, max: usize| -> Result<usize, DecodeError> {
            let n = r.u8()? as usize;
            if n > max { Err(DecodeError::TooMany) } else { Ok(n) }
        };
        let n = count(r, MAX_ITEMS)?;
        let mut items = Vec::with_capacity(n);
        for _ in 0..n {
            items.push(ItemSnap {
                id: r.u32()?,
                kind: r.u8()?,
                variant: r.u8()?,
                state: r.u8()?,
                holder: r.u32()?,
                pos: r.triple()?,
                vel: r.triple()?,
                ground_y: r.f32()?,
                team: r.u8()?,
                uses: r.u8()?,
            });
        }
        let n = count(r, MAX_BOTS)?;
        let mut bots = Vec::with_capacity(n);
        for _ in 0..n {
            bots.push(BotSnap {
                id: r.u32()?,
                pos: r.triple()?,
                vel: r.pair()?,
                face: r.f32()?,
                flags: r.u16()?,
                charge: r.u8()? as f32 / 255.0,
                seat: r.u8()?,
                team: r.u8()?,
                drunk: r.u8()?,
                belly: r.u8()?,
                selected: r.u32()?,
                swing: r.u8()?,
            });
        }
        let dazza = DazzaSnap {
            x: r.f32()?,
            z: r.f32()?,
            face: r.f32()?,
            state: r.u8()?,
            say_seq: r.u32()?,
            swing_seq: r.u32()?,
            flip_seq: r.u32()?,
            say: r.line(LINE_MAX)?,
        };
        let n = count(r, MAX_BOARD)?;
        let mut board = Vec::with_capacity(n);
        for _ in 0..n {
            board.push(BoardRow {
                id: r.u32()?,
                score: r.i32()?,
                hits: r.u16()?,
                taken: r.u16()?,
                catches: r.u16()?,
                throws: r.u16()?,
                streak: r.u8()?,
                banked: r.u16()?,
            });
        }
        let n = count(r, MAX_BOARD)?;
        let mut teams = Vec::with_capacity(n);
        for _ in 0..n {
            teams.push((r.u32()?, r.u8()?));
        }
        let n = count(r, MAX_FEED)?;
        let mut feed = Vec::with_capacity(n);
        for _ in 0..n {
            feed.push(r.line(LINE_MAX)?);
        }
        let n = count(r, MAX_FX)?;
        let mut fx = Vec::with_capacity(n);
        for _ in 0..n {
            fx.push(FxSnap { kind: r.u8()?, at: r.triple()? });
        }
        Ok(WorldSnap { seq, now, round, items, bots, dazza, board, teams, feed, fx })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::Msg;

    fn sample() -> WorldSnap {
        WorldSnap {
            seq: 42,
            now: 123.5,
            round: RoundSnap {
                phase: 3,
                mode: 1,
                features: FEAT_BAR | FEAT_BBQ | FEAT_SMOKO | FEAT_TIMED,
                time_left: 97.25,
                round_no: 2,
                match_len: 3,
                chest_spot: 4,
                chest_stock: 3,
                smoko_at: (-24.0, -3.0),
                heist_teams: 2,
                banner: Some(("ROUND 2".into(), 1.5)),
            },
            items: (0..10)
                .map(|i| ItemSnap {
                    id: 100 + i,
                    kind: (i % 8) as u8,
                    variant: if i == 3 { 4 } else { 255 },
                    state: (i % 3) as u8,
                    holder: if i % 3 == 1 { 101 } else { 0 },
                    pos: (i as f32, 0.2 * i as f32, -3.0),
                    vel: (1.0, 2.0, -3.0),
                    ground_y: 0.6,
                    team: 255,
                    uses: if i == 3 { 2 } else { 255 },
                })
                .collect(),
            bots: vec![BotSnap {
                id: 100,
                pos: (4.0, 0.0, 5.0),
                vel: (1.0, -1.0),
                face: 2.0,
                flags: BOT_GROUNDED | BOT_WINDING | BOT_CROWN,
                charge: 1.0,
                seat: 255,
                team: 1,
                drunk: 80,
                belly: 130,
                selected: 104,
                swing: 20,
            }],
            dazza: DazzaSnap { x: -6.0, z: -19.3, face: 0.5, state: 2, say_seq: 7, swing_seq: 3, flip_seq: 9, say: "Oi! Get off me snags!".into() },
            board: vec![
                BoardRow { id: 1, score: 350, hits: 3, taken: 1, catches: 2, throws: 9, streak: 2, banked: 0 },
                BoardRow { id: 100, score: -50, hits: 0, taken: 1, catches: 0, throws: 4, streak: 0, banked: 0 },
            ],
            teams: vec![(1, 0), (100, 1)],
            feed: vec!["Marcus hit Kev (+100)".into(), "Dazza: bugger off".into()],
            fx: vec![FxSnap { kind: 2, at: (1.0, 1.5, 2.0) }],
        }
    }

    #[test]
    fn a_world_snapshot_survives_the_trip() {
        let m = Msg::World(Box::new(sample()));
        assert_eq!(Msg::decode(&m.encode()), Ok(m));
    }

    #[test]
    fn a_busy_snapshot_still_fits_in_one_relay_message() {
        let mut s = sample();
        s.items = (0..MAX_ITEMS as u32)
            .map(|i| ItemSnap { id: i, kind: 1, variant: 255, state: 0, holder: 0, pos: (1.0, 1.0, 1.0), vel: (0.0, 0.0, 0.0), ground_y: 0.0, team: 255, uses: 255 })
            .collect();
        s.bots = (0..MAX_BOTS as u32).map(|i| BotSnap { id: 100 + i, ..sample().bots[0].clone() }).collect();
        s.board = (0..MAX_BOARD as u32).map(|i| BoardRow { id: i, ..sample().board[0].clone() }).collect();
        s.feed = vec!["x".repeat(LINE_MAX); MAX_FEED];
        s.fx = vec![sample().fx[0]; MAX_FX];
        let bytes = Msg::World(Box::new(s)).encode();
        // the relay allows 12 288 characters of base64, which is about 9 200 bytes
        assert!(bytes.len() < 9000, "{} bytes", bytes.len());
        assert!(Msg::decode(&bytes).is_ok());
    }

    #[test]
    fn a_typical_snapshot_is_small() {
        let bytes = Msg::World(Box::new(sample())).encode();
        assert!(bytes.len() < 800, "{}", bytes.len());
    }

    #[test]
    fn too_many_things_are_refused() {
        let good = Msg::World(Box::new(sample())).encode();
        // the item count byte follows: 1 kind + 4 seq + 4 now + 3 + 4 + 4 + 2 + 8 + 1 + banner
        let mut s = sample();
        s.round.banner = None;
        let mut bytes = Msg::World(Box::new(s)).encode();
        let at = 1 + 4 + 4 + 3 + 4 + 4 + 8 + 1 + 1; // kind, seq, now, phase/mode/features, time, round/len/spot/stock, smoko, heist, banner flag
        bytes[at] = 200;
        assert!(Msg::decode(&bytes).is_err());
        // every cut-off version is refused, never a panic
        for n in 0..good.len() {
            assert!(Msg::decode(&good[..n]).is_err(), "cut at {n}");
        }
    }

    #[test]
    fn rubbish_never_panics() {
        let good = Msg::World(Box::new(sample())).encode();
        let mut x = 0x9E3779B9u32;
        for _ in 0..3000 {
            let mut bad = good.clone();
            for _ in 0..4 {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                let i = 1 + (x as usize % (bad.len() - 1));
                bad[i] = (x >> 8) as u8;
            }
            let _ = Msg::decode(&bad);
        }
    }

    #[test]
    fn long_lines_are_cut_and_odd_characters_dropped() {
        let mut s = sample();
        s.feed = vec![format!("{}\u{7}!", "y".repeat(500))];
        let Ok(Msg::World(back)) = Msg::decode(&Msg::World(Box::new(s)).encode()) else { panic!() };
        assert_eq!(back.feed[0].chars().count(), LINE_MAX);
        assert!(!back.feed[0].contains('\u{7}'));
    }
}
