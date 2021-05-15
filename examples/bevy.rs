//! A minimal bevy example.
//!
//! Performs a velocity-verlet integration of particles in a harmonic trap.

extern crate bevy_bench as lib;
use std::time::Duration;

use bevy_ecs::prelude::*;
use bevy_tasks::{ComputeTaskPool, TaskPoolBuilder};
use lib::{PARTICLE_NUMBER, STEP_NUMBER};

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

const BATCH_SIZE: usize = PARTICLE_NUMBER as usize / 6;

fn integrate_position(
    pool: Res<ComputeTaskPool>,
    mut query: Query<(&Velocity, &Mass, &Force, &mut Position, &mut OldForce)>,
    timestep: Res<Timestep>,
) {
    let dt = timestep.dt;
    query.par_for_each_mut(
        &pool,
        BATCH_SIZE,
        |(vel, mass, force, mut pos, mut old_force)| {
            pos.0 = pos.0 + vel.0 * dt + force.0 / (mass.0) / 2.0 * dt * dt;
            old_force.0 = *force;
        },
    );
}

fn integrate_velocity(
    pool: Res<ComputeTaskPool>,
    mut query: Query<(&mut Velocity, &Force, &OldForce, &Mass)>,
    timestep: Res<Timestep>,
) {
    let dt = timestep.dt;
    query.par_for_each_mut(&pool, BATCH_SIZE, |(mut vel, force, old_force, mass)| {
        vel.0 = vel.0 + (force.0 + old_force.0 .0) / (mass.0) / 2.0 * dt;
    });
}

fn harmonic_trap(pool: Res<ComputeTaskPool>, mut query: Query<(&mut Force, &Position)>) {
    query.par_for_each_mut(&pool, BATCH_SIZE, |(mut force, pos)| {
        force.0 = -pos.0;
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemLabel)]
enum SystemLabels {
    IntegratePosition,
    IntegrateVelocity,
    HarmonicTrap,
}

fn main() {
    let mut world = World::new();
    let mut stage = SystemStage::single_threaded();
    world.insert_resource(Timestep { dt: 1.0 });
    world.insert_resource(ComputeTaskPool(
        TaskPoolBuilder::default()
            .thread_name("ComputeTaskPool".into())
            .build(),
    ));

    world.spawn_batch((0..PARTICLE_NUMBER).map(|_| {
        (
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
        )
    }));

    stage
        .add_system(
            integrate_position
                .system()
                .label(SystemLabels::IntegratePosition),
        )
        .add_system(
            harmonic_trap
                .system()
                .label(SystemLabels::HarmonicTrap)
                .after(SystemLabels::IntegratePosition),
        )
        .add_system(
            integrate_velocity
                .system()
                .label(SystemLabels::IntegrateVelocity)
                .after(SystemLabels::HarmonicTrap),
        );

    let mut do_run = || {
        println!("Starting simulation.");
        let start = std::time::Instant::now();
        for _ in 0..STEP_NUMBER {
            stage.run(&mut world);
        }
        let dur = std::time::Instant::now() - start;
        println!("Finished in {:?}", dur);
        dur
    };

    let total: Duration = (0..5).map(|_| do_run()).sum();
    println!("Total loop time: {:?}", total);
    println!("Avg loop time: {:?}", total / 5)
}
