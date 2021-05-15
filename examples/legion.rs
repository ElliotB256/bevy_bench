//! A minimal legion example.
//!
//! Performs a velocity-verlet integration of particles in a harmonic trap.

extern crate bevy_bench as lib;
use lib::{PARTICLE_NUMBER, STEP_NUMBER};

extern crate legion;
use legion::*;

extern crate nalgebra;
use nalgebra::Vector3;

pub struct Position(Vector3<f64>);
pub struct Velocity(Vector3<f64>);
#[derive(Copy, Clone)]
pub struct Force(Vector3<f64>);
pub struct OldForce(Force);
pub struct Mass(f64);

pub struct Timestep {
    pub dt: f64,
}

use legion::world::SubWorld;

#[system]
fn integrate_position(
    world: &mut SubWorld,
    query: &mut Query<(&Velocity, &Mass, &Force, &mut Position, &mut OldForce)>,
    #[resource] timestep: &Timestep,
) {
    let dt = timestep.dt;
    query.par_for_each_mut(world, |(vel, mass, force, mut pos, mut old_force)| {
        pos.0 = pos.0 + vel.0 * dt + force.0 / (mass.0) / 2.0 * dt * dt;
        old_force.0 = *force;
    });
}

#[system]
fn integrate_velocity(
    world: &mut SubWorld,
    query: &mut Query<(&mut Velocity, &Force, &OldForce, &Mass)>,
    #[resource] timestep: &Timestep,
) {
    let dt = timestep.dt;
    query.par_for_each_mut(world, |(mut vel, force, old_force, mass)| {
        vel.0 = vel.0 + (force.0 + old_force.0 .0) / (mass.0) / 2.0 * dt;
    });
}

#[system]
fn harmonic_force(world: &mut SubWorld, query: &mut Query<(&mut Force, &Position)>) {
    query.par_for_each_mut(world, |(mut force, pos)| {
        force.0 = -pos.0;
    });
}

fn main() {
    let mut world = World::default();

    let mut schedule = Schedule::builder()
        .add_system(integrate_position_system())
        .add_system(harmonic_force_system())
        .add_system(integrate_velocity_system())
        .build();

    let mut resources = Resources::default();
    resources.insert(Timestep { dt: 1.0 });

    for _ in 0..PARTICLE_NUMBER {
        world.push((
            Position {
                0: Vector3::new(0.0, 0.0, 0.0),
            },
            Velocity {
                0: Vector3::new(0.2, 0.5, 1.0),
            },
            Mass { 0: 1.0 },
            Force {
                0: Vector3::new(0.0, 0.0, 0.0),
            },
            OldForce {
                0: Force {
                    0: Vector3::new(0.0, 0.0, 0.0),
                },
            },
        ));
    }

    println!("Starting simulation.");
    for _ in 0..STEP_NUMBER {
        schedule.execute(&mut world, &mut resources);
    }
    println!("Finished!");
}
