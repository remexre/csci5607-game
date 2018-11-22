//! Common systems.

use cgmath::{Deg, Matrix3, Vector3};
pub use crate::gui::{ControlSystem, GuiSystem};
use crate::{
    components::{
        CameraComponent, CollisionComponent, DecalComponent, DoorComponent, GoalComponent,
        KeyComponent, LocationComponent,
    },
    State, System,
};
use smallvec::SmallVec;
use std::mem::replace;

/// A system that lets the user grab keys.
pub struct HoldSystem;

impl System for HoldSystem {
    fn step(&mut self, state: &mut State, _dt: u64) {
        let world = match state {
            State::Playing(ref mut world) | State::Done(ref mut world, _) => world,
            _ => return,
        };

        let camera: LocationComponent = match world.iter().next() {
            Some((_, hlist_pat![CameraComponent, loc])) => *loc,
            None => {
                warn!("No camera?");
                return;
            }
        };

        let held_keys = world
            .iter()
            .filter(|(_, hlist_pat![&KeyComponent {held, ..}])| held)
            .map(|(entity, _)| entity)
            .collect::<SmallVec<[_; 8]>>();

        for entity in held_keys {
            let loc: &mut LocationComponent = world.get_mut(entity).unwrap();
            let forward = Matrix3::from_angle_y(Deg(camera.rotation[1])) * Vector3::unit_z();
            loc.xyz = camera.xyz + 0.3 * forward;
            loc.xyz.y = 0.1;
        }
    }
}

/// A system that makes unlocked doors sink.
pub struct SinkingDoorSystem;

impl System for SinkingDoorSystem {
    fn step(&mut self, state: &mut State, dt: u64) {
        let world = match state {
            State::Playing(ref mut world) | State::Done(ref mut world, _) => world,
            _ => return,
        };

        let to_sink = world
            .iter()
            .filter(|(_, hlist_pat![&DoorComponent(_), &CollisionComponent(c)])| !c)
            .map(|(entity, _)| entity)
            .collect::<SmallVec<[_; 8]>>();

        for entity in to_sink {
            if let Some(LocationComponent { xyz, .. }) = world.get_mut(entity) {
                xyz[1] -= (dt as f32) / 2500.0;
            }
        }
    }
}

/// A system that lets the user grab keys.
pub struct SnagSystem;

impl System for SnagSystem {
    fn step(&mut self, state: &mut State, _dt: u64) {
        let world = match state {
            State::Playing(ref mut world) | State::Done(ref mut world, _) => world,
            _ => return,
        };

        let camera: LocationComponent = match world.iter().next() {
            Some((_, hlist_pat![CameraComponent, loc])) => *loc,
            None => {
                warn!("No camera?");
                return;
            }
        };

        let snagged_keys = world
            .iter()
            .filter(|(_, hlist_pat![loc, &KeyComponent {held, ..}])| !held && camera.collides(loc))
            .map(|(entity, _)| entity)
            .collect::<SmallVec<[_; 2]>>();

        for entity in snagged_keys {
            let KeyComponent { ref mut held, .. } = world.get_mut(entity).unwrap();
            *held = true;
            info!("Snagged {}!", entity);
        }
    }
}

/// A system that makes keys spin.
pub struct SpinningKeySystem;

impl System for SpinningKeySystem {
    fn step(&mut self, state: &mut State, dt: u64) {
        let world = match state {
            State::Playing(ref mut world) | State::Done(ref mut world, _) => world,
            _ => return,
        };

        let to_spin = world
            .iter()
            .map(|(entity, hlist_pat![&KeyComponent {..}])| entity)
            .collect::<SmallVec<[_; 8]>>();

        for entity in to_spin {
            if let Some(LocationComponent { rotation, .. }) = world.get_mut(entity) {
                rotation[1] += dt as f32 / 5.0;
            }
        }
    }
}

/// A system that destroys entities who have positions less than `y=-1`.
pub struct TheFloorIsLavaSystem;

impl System for TheFloorIsLavaSystem {
    fn step(&mut self, state: &mut State, _dt: u64) {
        let world = match state {
            State::Playing(ref mut world) | State::Done(ref mut world, _) => world,
            _ => return,
        };

        let mut to_delete = SmallVec::<[_; 4]>::new();
        for (entity, hlist_pat![LocationComponent{xyz,..}]) in world.iter() {
            if xyz[1] < -1.0 {
                to_delete.push(entity);
            }
        }
        for entity in to_delete {
            world.delete_entity(entity);
        }
    }
}

/// A system that allows for the unlocking of doors with their corresponding keys.
pub struct UnlockSystem;

impl System for UnlockSystem {
    fn step(&mut self, state: &mut State, _dt: u64) {
        let world = match state {
            State::Playing(ref mut world) | State::Done(ref mut world, _) => world,
            _ => return,
        };

        let mut unlocks = SmallVec::<[_; 2]>::new();
        for (door, hlist_pat![&DoorComponent(door_letter), door_loc]) in world.iter() {
            for (key, hlist_pat![&KeyComponent{letter: key_letter,..}, key_loc]) in world.iter() {
                if !LocationComponent::collides(door_loc, key_loc) {
                    continue;
                }
                let diff = (key_letter as u32).wrapping_sub(door_letter as u32);
                if diff != 32 {
                    continue;
                }

                unlocks.push((door, key));
            }
        }

        for (door, key) in unlocks {
            info!("{} was unlocked with {}!", door, key);
            world.delete_entity(key);
            if let Some(CollisionComponent(ref mut active)) = world.get_mut(door) {
                *active = false;
            }
        }
    }
}

/// A system that allows for the user to win the game.
pub struct WinSystem;

impl System for WinSystem {
    fn step(&mut self, state: &mut State, dt: u64) {
        let won = match state {
            State::Playing(ref mut world) => {
                let camera: LocationComponent = match world.iter().next() {
                    Some((_, hlist_pat![CameraComponent, loc])) => *loc,
                    None => {
                        warn!("No camera?");
                        return;
                    }
                };

                let goal = world
                    .iter()
                    .filter(|(_, hlist_pat![GoalComponent, loc])| camera.collides(loc))
                    .map(|(entity, _)| entity)
                    .next();

                if let Some(goal) = goal {
                    info!("Player won the game!");
                    world.delete_entity(goal);

                    let decal = match world.iter().next() {
                        Some((entity, hlist_pat![DecalComponent{..}])) => entity,
                        None => {
                            warn!("No win decal?");
                            return;
                        }
                    };
                    world.get_mut::<DecalComponent>(decal).unwrap().enabled = true;

                    true
                } else {
                    false
                }
            }
            State::Done(_, ref mut t) => {
                *t += dt;
                false
            }
            State::Close => false,
        };

        if won {
            let world = match replace(state, State::Close) {
                State::Playing(world) => world,
                _ => unreachable!(),
            };
            replace(state, State::Done(world, 0));
        }
    }
}
