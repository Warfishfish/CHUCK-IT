//! Geometry shared by the tunnel floor, railings and runner collision.
use crate::{heist, vec::V3};

pub const HATCH: V3 = V3::new(3.65, 0.0, 3.65);
pub const DEPTH: f32 = 4.5;
pub const WIDTH: f32 = 3.0;
pub const RAMP: f32 = 9.0;

pub fn exits(teams: usize) -> impl Iterator<Item = V3> {
    heist::layout(teams).into_iter().map(heist::gate)
}

/// Closest walkable point and floor elevation on the connected passage arms.
pub fn track(teams: usize, x: f32, z: f32) -> (f32, f32, f32) {
    let mut best = (f32::INFINITY, HATCH.x, HATCH.z, -DEPTH);
    for end in exits(teams) {
        let delta = end - HATCH;
        let length = delta.horiz_len();
        let axis = delta * (1.0 / length);
        let along = ((x - HATCH.x) * axis.x + (z - HATCH.z) * axis.z).clamp(0.0, length);
        let centre = HATCH + axis * along;
        let dx = x - centre.x;
        let dz = z - centre.z;
        let distance = dx.hypot(dz);
        if distance < best.0 {
            let scale = ((WIDTH * 0.5 - 0.45) / distance.max(0.001)).min(1.0);
            let floor = -DEPTH * ((length - along) / RAMP).clamp(0.0, 1.0);
            best = (
                distance,
                centre.x + dx * scale,
                centre.z + dz * scale,
                floor,
            );
        }
    }
    (best.1, best.2, best.3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{movement::{Modifiers,MoveInput,Mover},yard::{Features,Yard}};
    #[test]
    fn a_carrier_can_exit_every_branch_after_the_hatch_shuts() {
        for teams in 2..=4 {
            let mut yard=Yard::heist(Features::default(),0,teams);
            assert!(yard.heist_clear(HATCH.x,HATCH.z,0.5));
            for exit in exits(teams) {
                let mut runner=Mover::new(HATCH.x,HATCH.z);
                let modifiers=Modifiers{carrying:true,..Default::default()};
                yard.hatch_open=true;
                for _ in 0..90 {runner.step(1./60.,MoveInput::default(),&modifiers,&yard);}
                assert!(runner.underground && runner.y < -4.4,"failed to descend");
                yard.hatch_open=false;
                let axis=(exit-HATCH).normalised();
                for _ in 0..3600 {
                    runner.step(1./60.,MoveInput{wish:(axis.x,axis.z),pitch:0.},&modifiers,&yard);
                    if !runner.underground {break;}
                }
                assert!(!runner.underground && (runner.x-exit.x).hypot(runner.z-exit.z)<1.0,
                    "teams {teams}, exit {exit:?}, runner ({},{},{})",runner.x,runner.y,runner.z);
            }
        }
    }
}

#[cfg(test)]
mod vault_tests {
    use crate::{heist::vault_sites,movement::{Mover,Modifiers,MoveInput},yard::{Yard,Features}};
    #[test]
    fn all_six_vaults_work_with_normal_jumps_and_a_teddy() {
        for n in 2..=4 {
            let yard=Yard::heist(Features::default(),0,n);
            for (x,z) in vault_sites(n) {
                for carrying in [false,true] {
                    let mut m=Mover::new(x,z+5.4);
                    let mut highest=0.0_f32;
                    for _ in 0..600 {
                        if m.grounded {m.try_jump(false);}
                        m.step(1./60.,MoveInput{wish:(0.,-1.),pitch:0.},&Modifiers{carrying,..Default::default()},&yard);
                        highest=highest.max(m.y);
                        if m.z<z-1. {break;}
                    }
                    assert!(highest>=3.15 && m.z<z-1.,"{n} flags at {x} {z} carrying {carrying}: reached {} {}",m.z,m.y);
                }
            }
        }
    }
}
