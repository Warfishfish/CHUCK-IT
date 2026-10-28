//! Reconcile the host against the report it accepted, not today's predicted position.
use std::collections::VecDeque;

pub const POSITION_TOLERANCE: f32 = 0.6;
const HISTORY_LIMIT: usize = 80; // four seconds at the normal 20 Hz send rate

#[derive(Default)]
pub struct MovementHistory {
    next: u32,
    sent: VecDeque<(u32, (f32, f32))>,
}
impl MovementHistory {
    pub fn record(&mut self, pos: (f32,f32)) -> u32 {
        self.next=self.next.wrapping_add(1).max(1);
        self.sent.push_back((self.next,pos));
        if self.sent.len()>HISTORY_LIMIT {self.sent.pop_front();}
        self.next
    }
    pub fn clear(&mut self) {self.sent.clear();}
    pub fn correction(&mut self, ack:u32, accepted:(f32,f32))->(f32,f32) {
        let Some(i)=self.sent.iter().position(|(seq,_)|*seq==ack) else {return (0.,0.);};
        let predicted=self.sent[i].1;
        self.sent.drain(..=i);
        let delta=(accepted.0-predicted.0,accepted.1-predicted.1);
        if !delta.0.is_finite() || !delta.1.is_finite() || delta.0.hypot(delta.1)<=POSITION_TOLERANCE {return (0.,0.);}
        // Later predictions already include the movement since this report. Preserve it,
        // and move their reference frame too so in-flight acknowledgements don't double-correct.
        for (_,pos) in &mut self.sent {pos.0+=delta.0;pos.1+=delta.1;}
        delta
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn delayed_walk_and_sprint_do_not_pull_the_player_back() {
        for speed in [6.2,10.54,4.96] {for lag in [2,4,8,16] {
            let mut h=MovementHistory::default();let mut reports=VecDeque::new();
            let mut current=0.;
            for _ in 0..200 {current+=speed*0.05;let seq=h.record((current,0.));reports.push_back((seq,current));
                if reports.len()>lag {let (ack,past)=reports.pop_front().unwrap();
                    assert_eq!(h.correction(ack,(past,0.)),(0.,0.));
                }
            }
        }}
    }
    #[test] fn blocked_movement_is_corrected_once_while_preserving_newer_prediction() {
        let mut h=MovementHistory::default();let a=h.record((2.,0.));let b=h.record((3.,0.));
        assert_eq!(h.correction(a,(1.,0.)),(-1.,0.));
        assert_eq!(h.correction(a,(1.,0.)),(0.,0.));
        assert_eq!(h.correction(b,(2.,0.)),(0.,0.));
    }
    #[test] fn old_round_unknown_and_expired_reports_cannot_teleport_players() {
        let mut h=MovementHistory::default();let a=h.record((5.,0.));h.clear();
        assert_eq!(h.correction(a,(50.,0.)),(0.,0.));
        let old=h.record((0.,0.));for i in 0..100 {h.record((i as f32,0.));}
        assert_eq!(h.sent.len(),HISTORY_LIMIT);
        assert_eq!(h.correction(old,(50.,0.)),(0.,0.));
    }
}
